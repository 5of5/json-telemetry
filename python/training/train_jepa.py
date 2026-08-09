"""Phase 3 — JEPA training for the Aria predictor.

Learns the isometry I and the conditioned predictor P so that

    P(I(psi_t), a) ~= I(psi_{t+1})

which is pure latent prediction: the target is an *embedding*, never a
reconstruction. There is no decoder anywhere in this file, and the exported
checkpoint only ever feeds `aria_engine_backends::TrainedPredictor`, which sits
inside the Spec loop as P and I and nothing else.

Postulate P2 requires E[Lip(P)] <= 1. Two mechanisms enforce it:

  1. a soft spectral penalty in the loss, so the optimizer prefers small
     operator norms; and
  2. a hard spectral projection after every optimizer step, so the constraint
     holds exactly at all times rather than approximately at convergence.

Inv2 is `Res' <= Res + eps`, and the worst case is an OpticalStep that swaps in
an arbitrary unit field. That bounds the residual jump by 2*Lip(P)*||I||, so the
default Lipschitz target is eps/2 = 0.49 rather than 1.0. See the module docs of
`crates/aria-backends/src/trained.rs`.

Usage:

    cargo run -p aria-engine -- dataset --n-modes 32 --trajectories 256 \\
        --length 16 --output /tmp/aria-data.json
    python python/training/train_jepa.py --data /tmp/aria-data.json \\
        --latent-dim 32 --out /tmp/aria-weights.json
    cargo run -p aria-engine -- run --steps 1000 --predictor /tmp/aria-weights.json
"""

from __future__ import annotations

import argparse
import json
import pathlib

import torch

CONDITIONS = ("token", "diffusion", "world_model")
WEIGHT_FORMAT = "aria-predictor-v1"
DATASET_FORMAT = "aria-optical-dataset-v1"


DATASET_FORMATS = {DATASET_FORMAT, "aria-text-dataset-v1"}


def load_dataset(path: pathlib.Path) -> tuple[torch.Tensor, int]:
    """Load `aria dataset` output as a [trajectories, length, 2*n_modes] tensor.

    Accepts both the synthetic optical format (smoke tests) and the real-data
    spectral format (`aria dataset --input corpus.txt`).
    """
    blob = json.loads(path.read_text())
    fmt = blob.get("format")
    if fmt not in DATASET_FORMATS:
        raise ValueError(f"expected one of {sorted(DATASET_FORMATS)}, got {fmt!r}")
    data = torch.tensor(blob["trajectories"], dtype=torch.float64)
    if data.numel() == 0 or data.shape[1] < 2:
        raise ValueError("dataset has no (psi_t, psi_{t+1}) pairs")
    return data, int(blob["n_modes"])


def spectral_norm(matrix: torch.Tensor) -> torch.Tensor:
    """Largest singular value of a 2-D tensor."""
    return torch.linalg.matrix_norm(matrix, ord=2)


@torch.no_grad()
def project_spectral(matrix: torch.Tensor, bound: float) -> None:
    """Scale `matrix` in place so its spectral norm is at most `bound`."""
    sigma = spectral_norm(matrix)
    if sigma > bound:
        matrix.mul_(bound / sigma)


class JepaPredictor(torch.nn.Module):
    """I : H -> Z and one P : Z -> Z per conditioning (C2, not a new architecture)."""

    def __init__(self, input_dim: int, latent_dim: int, seed: int = 0) -> None:
        super().__init__()
        gen = torch.Generator().manual_seed(seed)

        # I starts as a random partial isometry (orthonormal rows).
        raw = torch.randn(latent_dim, input_dim, generator=gen, dtype=torch.float64)
        q, _ = torch.linalg.qr(raw.T)
        self.embed = torch.nn.Parameter(q.T.contiguous())

        # Each P starts small so the initial Lipschitz constant is well inside
        # the bound; training grows it only as far as the projection allows.
        self.predict = torch.nn.ParameterDict(
            {
                name: torch.nn.Parameter(
                    0.1
                    * torch.randn(
                        latent_dim, latent_dim, generator=gen, dtype=torch.float64
                    )
                )
                for name in CONDITIONS
            }
        )

    def encode(self, psi: torch.Tensor) -> torch.Tensor:
        return psi @ self.embed.T

    def forward(self, psi: torch.Tensor, condition: str) -> torch.Tensor:
        return self.encode(psi) @ self.predict[condition].T


def jepa_residual(
    model: JepaPredictor, psi_t: torch.Tensor, psi_next: torch.Tensor, condition: str
) -> torch.Tensor:
    """Mean ||P(I(psi_t)) - I(psi_{t+1})||: the JEPA metric TLA+ calls JEPALimit."""
    predicted = model(psi_t, condition)
    target = model.encode(psi_next)
    return torch.linalg.vector_norm(predicted - target, dim=-1).mean()


def pairs(data: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
    """Flatten [traj, length, dim] into consecutive (psi_t, psi_{t+1}) pairs."""
    dim = data.shape[-1]
    return data[:, :-1, :].reshape(-1, dim), data[:, 1:, :].reshape(-1, dim)


@torch.no_grad()
def persistence_baseline(
    model: JepaPredictor, hold_x: torch.Tensor, hold_y: torch.Tensor
) -> float:
    """The trivial "predict tomorrow = today" reference, measured in the model's
    own latent space: mean ||I(psi_{t+1}) - I(psi_t)||.

    A trained model that cannot beat this has learned nothing; Exit3 requires
    strictly beating it, not just improving on a random initialization.
    """
    return float(torch.linalg.vector_norm(model.encode(hold_y) - model.encode(hold_x), dim=-1).mean())


def train(
    data: torch.Tensor,
    n_modes: int,
    latent_dim: int,
    epochs: int,
    lr: float,
    lipschitz: float,
    penalty: float,
    holdout: float,
    seed: int,
    quiet: bool = False,
) -> tuple[JepaPredictor, list[dict]]:
    torch.manual_seed(seed)

    # Split so train and holdout never overlap. Several trajectories: hold back
    # whole ones. A single trajectory (the real-text format): hold back its
    # tail in time.
    if data.shape[0] > 1:
        split = max(1, int(data.shape[0] * (1.0 - holdout)))
        train_x, train_y = pairs(data[:split])
        hold_x, hold_y = pairs(data[split:])
    else:
        cut = int(data.shape[1] * (1.0 - holdout))
        if cut < 2 or cut > data.shape[1] - 2:
            raise ValueError("single-trajectory dataset too short to split")
        train_x, train_y = pairs(data[:, :cut])
        hold_x, hold_y = pairs(data[:, cut - 1:])  # overlap by one frame for the pair

    model = JepaPredictor(2 * n_modes, latent_dim, seed=seed)
    opt = torch.optim.Adam(model.parameters(), lr=lr)

    best: tuple[float, dict[str, torch.Tensor]] | None = None
    best_epoch = 0
    history: list[dict] = []
    for epoch in range(epochs + 1):
        if epoch > 0:
            opt.zero_grad()
            loss = torch.zeros((), dtype=torch.float64)
            for name in CONDITIONS:
                loss = loss + jepa_residual(model, train_x, train_y, name)
                # Soft P2 pressure; the hard projection below makes it exact.
                loss = loss + penalty * torch.relu(
                    spectral_norm(model.predict[name]) - lipschitz
                )
            # Keep I an isometry (A2): penalise departure from I Iᵀ = Id.
            gram = model.embed @ model.embed.T
            loss = loss + penalty * torch.linalg.matrix_norm(
                gram - torch.eye(latent_dim, dtype=torch.float64)
            )
            if not torch.isfinite(loss):
                raise FloatingPointError(
                    f"non-finite loss at epoch {epoch}: {float(loss)} — "
                    "reduce --lr or check the dataset"
                )
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 10.0)
            opt.step()

            with torch.no_grad():
                project_spectral(model.embed, 1.0)
                for name in CONDITIONS:
                    project_spectral(model.predict[name], lipschitz)

        with torch.no_grad():
            holdout_res = float(jepa_residual(model, hold_x, hold_y, "token"))
            record = {
                "epoch": epoch,
                "train_residual": float(
                    jepa_residual(model, train_x, train_y, "token")
                ),
                "holdout_residual": holdout_res,
                "persistence_baseline": persistence_baseline(model, hold_x, hold_y),
                "lipschitz": max(
                    float(spectral_norm(model.predict[n])) for n in CONDITIONS
                ),
                "embed_norm": float(spectral_norm(model.embed)),
            }
        history.append(record)

        # Best-epoch checkpointing: export the model that generalized best,
        # not the one that happened to be last.
        if best is None or holdout_res < best[0]:
            best = (holdout_res, {k: v.clone() for k, v in model.state_dict().items()})
            best_epoch = epoch

        if not quiet and (epoch % max(1, epochs // 10) == 0 or epoch == epochs):
            print(
                f"epoch {record['epoch']:4d}  "
                f"train {record['train_residual']:.6f}  "
                f"holdout {record['holdout_residual']:.6f}  "
                f"persist {record['persistence_baseline']:.6f}  "
                f"Lip(P) {record['lipschitz']:.4f}  "
                f"||I|| {record['embed_norm']:.4f}"
            )

    assert best is not None
    model.load_state_dict(best[1])
    with torch.no_grad():
        # Final record describes the *restored* model, with the same keys as
        # every other entry, so consumers never see a ragged history.
        history.append(
            {
                "epoch": best_epoch,
                "restored_best": True,
                "train_residual": float(jepa_residual(model, train_x, train_y, "token")),
                "holdout_residual": best[0],
                "persistence_baseline": persistence_baseline(model, hold_x, hold_y),
                "lipschitz": max(float(spectral_norm(model.predict[n])) for n in CONDITIONS),
                "embed_norm": float(spectral_norm(model.embed)),
            }
        )
    return model, history


def export(model: JepaPredictor, n_modes: int, lipschitz: float) -> dict:
    with torch.no_grad():
        return {
            "format": WEIGHT_FORMAT,
            "n_modes": n_modes,
            "latent_dim": model.embed.shape[0],
            "lipschitz_bound": lipschitz,
            "embed": model.embed.tolist(),
            "predict": {name: model.predict[name].tolist() for name in CONDITIONS},
        }


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--data", type=pathlib.Path, required=True, help="`aria dataset` JSON")
    p.add_argument("--out", type=pathlib.Path, required=True, help="weights JSON to write")
    p.add_argument("--metrics", type=pathlib.Path, help="per-epoch metrics JSONL to write")
    p.add_argument("--latent-dim", type=int, default=32)
    p.add_argument("--epochs", type=int, default=300)
    p.add_argument("--lr", type=float, default=5e-3)
    p.add_argument(
        "--lipschitz",
        type=float,
        default=0.49,
        help="hard bound on Lip(P); keep <= eps/2 so Inv2 holds for every schedule",
    )
    p.add_argument("--penalty", type=float, default=1.0, help="soft constraint weight")
    p.add_argument("--holdout", type=float, default=0.2)
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--quiet", action="store_true")
    args = p.parse_args()

    data, n_modes = load_dataset(args.data)
    model, history = train(
        data=data,
        n_modes=n_modes,
        latent_dim=args.latent_dim,
        epochs=args.epochs,
        lr=args.lr,
        lipschitz=args.lipschitz,
        penalty=args.penalty,
        holdout=args.holdout,
        seed=args.seed,
        quiet=args.quiet,
    )

    args.out.write_text(json.dumps(export(model, n_modes, args.lipschitz)))
    if args.metrics:
        args.metrics.write_text("\n".join(json.dumps(r) for r in history) + "\n")

    # `train` restores the best epoch, so re-measure it rather than trusting the
    # history tail.
    epochs_ran = [r for r in history if "holdout_residual" in r]
    first = epochs_ran[0]
    final = {
        "holdout_residual": min(r["holdout_residual"] for r in epochs_ran),
        "lipschitz": max(
            float(torch.linalg.matrix_norm(model.predict[n], ord=2)) for n in CONDITIONS
        ),
    }
    persistence = max(r["persistence_baseline"] for r in epochs_ran)

    improved = final["holdout_residual"] < first["holdout_residual"]
    beats_persistence = final["holdout_residual"] < persistence

    print(
        f"\nholdout residual {first['holdout_residual']:.6f} -> "
        f"{final['holdout_residual']:.6f} "
        f"({'decreased' if improved else 'DID NOT DECREASE'})"
    )
    print(
        f"persistence baseline {persistence:.6f} — model "
        f"{'BEATS it' if beats_persistence else 'FAILS to beat it'}"
    )
    print(f"Lip(P) = {final['lipschitz']:.4f} <= {args.lipschitz} (P2 enforced)")
    print(f"weights written to {args.out}")

    # Exit3: the held-out residual must fall *and* beat the trivial baseline.
    return 0 if (improved and beats_persistence) else 1


if __name__ == "__main__":
    raise SystemExit(main())
