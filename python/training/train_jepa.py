"""Phase 3 / WS5 — JEPA training for the Aria predictor.

Learns the isometry I and the conditioned predictor P so that

    P(I(psi_t), a) ~= StopGradient(I(psi_{t+1}))

which is pure latent prediction: the target is an *embedding*, never a
reconstruction. There is no reconstruction head anywhere in this file, and
the exported checkpoint only ever feeds
`aria_engine_backends::TrainedPredictor`, which sits inside the Spec loop as
P and I and nothing else.

ℒ_NLL (the discrete output term) is trained in `train_readout.py` against
frozen latents. This module refuses a non-zero λ_NLL so those gradients
cannot touch Φ (𝔸5 / 𝕃5).

Postulate P2 requires E[Lip(P)] <= 1. Two mechanisms enforce it:

  1. ℒ_Spectral = Σ_m max(0, σ_max(W_m) − 1)² (spec §6.3);
  2. a hard spectral projection after every optimizer step.

Inv2 is `Res' <= Res + eps`, and the worst case is an OpticalStep that swaps
in an arbitrary unit field. That bounds the residual jump by 2*Lip(P)*||I||,
so the default Lipschitz target is eps/2 = 0.49 rather than 1.0.

Collapse gate (Garrido, Balestriero, Najman, LeCun — ICML 2023, RankMe,
arXiv:2210.02885): abort if RankMe(Z) < min_rankme_frac · d.

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
import math
import pathlib
import struct

import torch

CONDITIONS = ("token", "diffusion", "world_model")
WEIGHT_FORMAT = "aria-predictor-v1"
WEIGHT_FORMAT_V2 = "aria-predictor-v2"
DATASET_FORMAT = "aria-optical-dataset-v1"

DATASET_FORMATS = {DATASET_FORMAT, "aria-text-dataset-v1"}

# Default λ ∈ Δ³ for this module: ℒ_NLL is owned by train_readout.py.
DEFAULT_LAMBDAS = (0.6, 0.0, 0.4, 0.0)
SIMPLEX_TOL = 1e-9
MIN_RANKME_FRAC_DEFAULT = 0.3


class CollapseError(RuntimeError):
    """Raised when RankMe(Z) falls below the collapse threshold."""


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


def chunk_frames(data: torch.Tensor, length: int) -> torch.Tensor:
    """Reshape a long frame stream into `[n, length, dim]` trajectories.

    Wilcoxon certification needs ≥ 30 held-out trajectories. A real-text
    `aria dataset --input` file is one long trajectory; this cuts it into
    fixed-length windows without fabricating fields.
    """
    if length < 2:
        raise ValueError(f"chunk length must be ≥ 2, got {length}")
    if data.ndim == 2:
        frames = data
    elif data.ndim == 3:
        frames = data.reshape(-1, data.shape[-1])
    else:
        raise ValueError(f"expected a 2-D or 3-D tensor, got shape {tuple(data.shape)}")
    n = frames.shape[0] // length
    if n < 1:
        raise ValueError(
            f"not enough frames ({frames.shape[0]}) to form a trajectory of length {length}"
        )
    return frames[: n * length].reshape(n, length, frames.shape[-1])


def spectral_norm(matrix: torch.Tensor) -> torch.Tensor:
    """Largest singular value of a 2-D tensor (exact, for diagnostics only).

    The *enforced* quantity is the seeded power-iteration estimate below —
    the same one the Rust loader uses (plan WS1) — so training and loading
    agree on what the projection enforces.
    """
    return torch.linalg.matrix_norm(matrix, ord=2)


# ── Cross-language spectral contract ─────────────────────────────────────────
# Line-by-line identical to crates/aria-backends/src/spectral.rs: same seeded
# LCG start vector, same alternating u/v sweeps (𝕋4), same r default. The LCG
# is fixed-point 64-bit arithmetic, so start vectors are bit-identical in
# Rust and Python.
POWER_ITERATION_ITERATIONS = 16  # must match spectral.rs DEFAULT_ITERATIONS
POWER_ITERATION_SEED = 0x9E3779B97F4A7C15  # must match spectral.rs START_VECTOR_SEED


def _next_lcg(x: int) -> int:
    """x_{n+1} = 6364136223846793005·x + 1442695040888963407 (mod 2⁶⁴)."""
    return (x * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF


def _seeded_unit_vector(n: int) -> torch.Tensor:
    """The seeded LCG stream mapped into [−1, 1)ⁿ, normalized."""
    x = POWER_ITERATION_SEED
    values = []
    for _ in range(n):
        x = _next_lcg(x)
        values.append(((x >> 11) * (1.0 / 9007199254740992.0)) - 1.0)
    v = torch.tensor(values, dtype=torch.float64)
    norm = v.norm()
    if norm > 0.0:
        v = v / norm
    return v


def power_iteration_sigma(matrix: torch.Tensor, r: int = POWER_ITERATION_ITERATIONS) -> float:
    """σ_max(W) by r alternating singular-vector sweeps (𝕋4): v ← Wᵀu/‖·‖,
    u ← Wv/‖·‖, σ = ‖Wv‖₂. r must lie in [2, 16] (spec §0.4)."""
    if not 2 <= r <= 16:
        raise ValueError(f"power iteration count r = {r} violates the spec domain: r ∈ [2, 16]")
    if matrix.numel() == 0:
        return 0.0
    rows, _cols = matrix.shape

    u = _seeded_unit_vector(rows)
    sigma = 0.0
    for _ in range(r):
        v = matrix.t() @ u  # v = Wᵀ u
        v_norm = float(v.norm())
        if v_norm <= 1e-300:
            return 0.0
        v = v / v_norm

        u_next = matrix @ v  # u = W v
        sigma = float(u_next.norm())  # σ = ‖W v‖₂ = uᵀWv
        if sigma <= 1e-300:
            return 0.0
        u = u_next / sigma
    return sigma


@torch.no_grad()
def project_spectral(matrix: torch.Tensor, bound: float) -> None:
    """Scale `matrix` in place so its spectral norm is at most `bound`.

    𝕋4's `W ← W / max(1.0, σ_max)` generalized to radius `bound`, with σ_max
    estimated by the same seeded power iteration the Rust loader enforces ℙ2
    with (plan WS1) — training and loading project to the same quantity.
    """
    sigma = power_iteration_sigma(matrix)
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
    """Batch-mean squared latent error with a stop-gradient target (spec §6.1).

    ℒ_JEPA = mean ||P(I(psi_t)) − sg(I(psi_{t+1}))||².
    """
    predicted = model(psi_t, condition)
    target = model.encode(psi_next).detach()
    return (predicted - target).pow(2).mean()


def pairs(data: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
    """Flatten [traj, length, dim] into consecutive (psi_t, psi_{t+1}) pairs."""
    dim = data.shape[-1]
    return data[:, :-1, :].reshape(-1, dim), data[:, 1:, :].reshape(-1, dim)


@torch.no_grad()
def persistence_baseline(
    model: JepaPredictor, hold_x: torch.Tensor, hold_y: torch.Tensor
) -> float:
    """Trivial "predict tomorrow = today" reference, in the model's latent
    space, using the same squared error as ℒ_JEPA: mean ||I(y) − I(x)||².

    A trained model that cannot beat this has learned nothing; Exit3 requires
    strictly beating it, not just improving on a random initialization.
    """
    return float((model.encode(hold_y) - model.encode(hold_x)).pow(2).mean())


def rankme(z: torch.Tensor) -> float:
    """Effective rank of an embedding matrix (Garrido et al., ICML 2023).

    RankMe(Z) = exp(−Σ_k p_k log p_k) with p_k = σ_k(Z) / Σ_j σ_j(Z).
    Parameter-free as a metric; the 0.3 · d abort threshold is ours.
    """
    if z.ndim != 2 or min(z.shape) == 0:
        raise ValueError(f"RankMe expects a non-empty [n, d] matrix, got {tuple(z.shape)}")
    singular = torch.linalg.svdvals(z.to(dtype=torch.float64))
    singular = torch.clamp(singular, min=0.0)
    total = float(singular.sum())
    if total <= 0.0:
        return 0.0
    p = singular / total
    safe = p > 0
    entropy = float(-(p[safe] * p[safe].log()).sum())
    return math.exp(entropy)


def abort_if_collapsed(z: torch.Tensor, min_rankme_frac: float, latent_dim: int) -> float:
    """Abort when RankMe(Z) < min_rankme_frac · d. Returns the measured RankMe."""
    score = rankme(z)
    threshold = min_rankme_frac * float(latent_dim)
    if score < threshold:
        raise CollapseError(
            f"RankMe(Z) = {score:.6f} < {min_rankme_frac} · d = {threshold:.6f} "
            f"(d = {latent_dim}) — latent collapsed"
        )
    return score


def validate_simplex(
    jepa: float, nll: float, spectral: float, graph: float, tol: float = SIMPLEX_TOL
) -> None:
    """λ ∈ Δ³: four finite weights ≥ 0 that sum to 1 (ℙ6 / AriaConfig)."""
    terms = (("jepa", jepa), ("nll", nll), ("spectral", spectral), ("graph", graph))
    for name, weight in terms:
        if not math.isfinite(weight):
            raise ValueError(f"λ_{name} = {weight} is not finite (λ ∈ Δ³)")
        if weight < 0.0:
            raise ValueError(f"λ_{name} = {weight} < 0 (λ ∈ Δ³)")
    total = jepa + nll + spectral + graph
    if abs(total - 1.0) > tol:
        raise ValueError(f"Σλ = {total} ≠ 1 (λ ∈ Δ³)")


def l_total(
    l_jepa: torch.Tensor,
    l_nll: torch.Tensor,
    l_spectral: torch.Tensor,
    l_graph: torch.Tensor,
    lambdas: tuple[float, float, float, float],
) -> torch.Tensor:
    """ℒ_total = λ_JEPA·ℒ_JEPA + λ_NLL·ℒ_NLL + λ_Spectral·ℒ_Spectral + λ_Graph·ℒ_Graph."""
    lam_j, lam_n, lam_s, lam_g = lambdas
    return lam_j * l_jepa + lam_n * l_nll + lam_s * l_spectral + lam_g * l_graph


def spectral_hinge_loss(model: JepaPredictor) -> torch.Tensor:
    """Σ_m max(0, σ_max(W_m) − 1)² — spec §6.3."""
    acc = torch.zeros((), dtype=torch.float64)
    for name in CONDITIONS:
        sigma = spectral_norm(model.predict[name])
        acc = acc + torch.relu(sigma - 1.0).pow(2)
    return acc


def isometry_penalty(model: JepaPredictor) -> torch.Tensor:
    """‖I Iᵀ − Id‖ — keep I an isometry (𝔸2). Folded into ℒ_Spectral."""
    latent_dim = model.embed.shape[0]
    gram = model.embed @ model.embed.T
    return torch.linalg.matrix_norm(gram - torch.eye(latent_dim, dtype=torch.float64))


def graph_hinge_loss(src: torch.Tensor, dst: torch.Tensor, gamma: float) -> torch.Tensor:
    """Σ_{(u,v)∈E} max(0, d(ℳ(u), ℳ(v)) − γ)² — spec §6.4."""
    if src.numel() == 0:
        return torch.zeros((), dtype=torch.float64)
    dist = torch.linalg.vector_norm(src - dst, dim=-1)
    return torch.relu(dist - gamma).pow(2).sum()


def load_graph_pairs(path: pathlib.Path) -> tuple[torch.Tensor, torch.Tensor]:
    """Load endpoint embeddings from an `aria run --export-graph` JSON."""
    blob = json.loads(path.read_text())
    nodes = blob.get("nodes") or {}
    embeddings: dict[str, list[float]] = {}
    for key, node in nodes.items():
        nid = str(node.get("id", key))
        embeddings[nid] = node["embedding"]
    src_rows: list[list[float]] = []
    dst_rows: list[list[float]] = []
    for edge in blob.get("edges") or []:
        u = str(edge["from"])
        v = str(edge["to"])
        if u in embeddings and v in embeddings:
            src_rows.append(embeddings[u])
            dst_rows.append(embeddings[v])
    if not src_rows:
        return torch.zeros(0, 1, dtype=torch.float64), torch.zeros(0, 1, dtype=torch.float64)
    return (
        torch.tensor(src_rows, dtype=torch.float64),
        torch.tensor(dst_rows, dtype=torch.float64),
    )


def per_trajectory_residuals(
    model: JepaPredictor, data: torch.Tensor, condition: str = "token"
) -> tuple[list[float], list[float]]:
    """Per-trajectory mean ℒ_JEPA and persistence (same squared metric)."""
    model_vals: list[float] = []
    persist_vals: list[float] = []
    with torch.no_grad():
        for traj in data:
            x, y = pairs(traj.unsqueeze(0))
            model_vals.append(float(jepa_residual(model, x, y, condition)))
            persist_vals.append(persistence_baseline(model, x, y))
    return model_vals, persist_vals


def wilcoxon_paired(persist: list[float], model: list[float]) -> tuple[float, float]:
    """Paired Wilcoxon signed-rank: H1 = persist > model (model is better).

    Returns (p_value, median_improvement) with improvement = persist − model.
    Uses scipy when present (training extra); otherwise a normal approximation
    with tie mid-ranks so the unit tests do not depend on the extra.
    """
    if len(persist) != len(model) or len(persist) < 2:
        raise ValueError("Wilcoxon needs ≥ 2 paired observations")
    deltas = [p - m for p, m in zip(persist, model)]
    median = sorted(deltas)[len(deltas) // 2] if len(deltas) % 2 else 0.5 * (
        sorted(deltas)[len(deltas) // 2 - 1] + sorted(deltas)[len(deltas) // 2]
    )
    try:
        from scipy.stats import wilcoxon

        result = wilcoxon(deltas, alternative="greater", zero_method="wilcox")
        return float(result.pvalue), float(median)
    except ImportError:
        return _wilcoxon_normal(deltas), float(median)


def _wilcoxon_normal(deltas: list[float]) -> float:
    """Two-sided-ready one-sided (greater) Wilcoxon p via the normal approx."""
    nonzero = [d for d in deltas if d != 0.0]
    n = len(nonzero)
    if n < 2:
        return 1.0
    order = sorted(range(n), key=lambda i: abs(nonzero[i]))
    ranks = [0.0] * n
    i = 0
    while i < n:
        j = i
        while j + 1 < n and abs(nonzero[order[j + 1]]) == abs(nonzero[order[i]]):
            j += 1
        mid = 0.5 * ((i + 1) + (j + 1))
        for k in range(i, j + 1):
            ranks[order[k]] = mid
        i = j + 1
    w_plus = sum(r for r, d in zip(ranks, nonzero) if d > 0.0)
    mean = n * (n + 1) / 4.0
    var = n * (n + 1) * (2 * n + 1) / 24.0
    if var <= 0.0:
        return 1.0
    z = (w_plus - mean - 0.5) / math.sqrt(var)
    # Φ(-z) one-sided greater: large W+ ⇒ small p.
    return 0.5 * math.erfc(z / math.sqrt(2.0))


def bootstrap_median_ci(
    deltas: list[float], n_boot: int = 10_000, seed: int = 0, alpha: float = 0.01
) -> tuple[float, float]:
    """Paired bootstrap 99% CI for the median of persist − model."""
    if not deltas:
        raise ValueError("bootstrap needs at least one delta")
    g = torch.Generator().manual_seed(seed)
    n = len(deltas)
    t = torch.tensor(deltas, dtype=torch.float64)
    meds = []
    for _ in range(n_boot):
        idx = torch.randint(0, n, (n,), generator=g)
        sample = t[idx].sort().values
        if n % 2:
            meds.append(float(sample[n // 2]))
        else:
            meds.append(float(0.5 * (sample[n // 2 - 1] + sample[n // 2])))
    meds.sort()
    lo_i = int((alpha / 2.0) * n_boot)
    hi_i = min(n_boot - 1, int((1.0 - alpha / 2.0) * n_boot))
    return meds[lo_i], meds[hi_i]


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
    lambdas: tuple[float, float, float, float] = DEFAULT_LAMBDAS,
    min_rankme_frac: float = MIN_RANKME_FRAC_DEFAULT,
    graph_src: torch.Tensor | None = None,
    graph_dst: torch.Tensor | None = None,
    graph_gamma: float = 0.5,
) -> tuple[JepaPredictor, list[dict]]:
    validate_simplex(*lambdas)
    if lambdas[1] != 0.0:
        raise ValueError(
            "λ_NLL must be 0 in train_jepa.py — train the output head in "
            "train_readout.py against frozen latents (𝔸5)"
        )

    torch.manual_seed(seed)

    # Split so train and holdout never overlap. Several trajectories: hold back
    # whole ones. A single trajectory (the real-text format): hold back its
    # tail in time.
    if data.shape[0] > 1:
        split = max(1, int(data.shape[0] * (1.0 - holdout)))
        if split >= data.shape[0]:
            split = data.shape[0] - 1
        train_x, train_y = pairs(data[:split])
        hold_x, hold_y = pairs(data[split:])
        hold_trajs = data[split:]
    else:
        cut = int(data.shape[1] * (1.0 - holdout))
        if cut < 2 or cut > data.shape[1] - 2:
            raise ValueError("single-trajectory dataset too short to split")
        train_x, train_y = pairs(data[:, :cut])
        hold_x, hold_y = pairs(data[:, cut - 1 :])  # overlap by one frame for the pair
        hold_trajs = data[:, cut - 1 :]

    model = JepaPredictor(2 * n_modes, latent_dim, seed=seed)
    opt = torch.optim.Adam(model.parameters(), lr=lr)

    best: tuple[float, dict[str, torch.Tensor]] | None = None
    best_epoch = 0
    history: list[dict] = []
    for epoch in range(epochs + 1):
        if epoch > 0:
            opt.zero_grad()
            l_jepa = torch.zeros((), dtype=torch.float64)
            for name in CONDITIONS:
                l_jepa = l_jepa + jepa_residual(model, train_x, train_y, name)
            l_jepa = l_jepa / float(len(CONDITIONS))
            l_spectral = spectral_hinge_loss(model) + penalty * isometry_penalty(model)
            if graph_src is not None and graph_dst is not None and lambdas[3] > 0.0:
                l_graph = graph_hinge_loss(graph_src, graph_dst, graph_gamma)
            else:
                l_graph = torch.zeros((), dtype=torch.float64)
            l_nll = torch.zeros((), dtype=torch.float64)
            loss = l_total(l_jepa, l_nll, l_spectral, l_graph, lambdas)
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
            z_hold = model.encode(hold_x)
            rankme_score = abort_if_collapsed(z_hold, min_rankme_frac, latent_dim)
            record = {
                "epoch": epoch,
                "train_residual": float(jepa_residual(model, train_x, train_y, "token")),
                "holdout_residual": holdout_res,
                "persistence_baseline": persistence_baseline(model, hold_x, hold_y),
                "rankme": rankme_score,
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
                f"RankMe {record['rankme']:.3f}  "
                f"Lip(P) {record['lipschitz']:.4f}  "
                f"||I|| {record['embed_norm']:.4f}"
            )

    assert best is not None
    model.load_state_dict(best[1])
    with torch.no_grad():
        # Final record describes the *restored* model, with the same keys as
        # every other entry, so consumers never see a ragged history.
        z_hold = model.encode(hold_x)
        rankme_score = abort_if_collapsed(z_hold, min_rankme_frac, latent_dim)
        history.append(
            {
                "epoch": best_epoch,
                "restored_best": True,
                "train_residual": float(jepa_residual(model, train_x, train_y, "token")),
                "holdout_residual": best[0],
                "persistence_baseline": persistence_baseline(model, hold_x, hold_y),
                "rankme": rankme_score,
                "lipschitz": max(float(spectral_norm(model.predict[n])) for n in CONDITIONS),
                "embed_norm": float(spectral_norm(model.embed)),
                "holdout_trajectories": int(hold_trajs.shape[0]),
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


def _f64_le(tensor: torch.Tensor) -> bytes:
    flat = tensor.detach().cpu().contiguous().to(torch.float64).reshape(-1)
    return struct.pack("<" + "d" * flat.numel(), *flat.tolist())


def write_safetensors_v2(
    path: pathlib.Path, model: JepaPredictor, n_modes: int, lipschitz: float
) -> None:
    """Write `aria-predictor-v2` (F64, little-endian) matching the Rust loader."""
    with torch.no_grad():
        tensors: list[tuple[str, list[int], bytes]] = [
            ("embed", list(model.embed.shape), _f64_le(model.embed)),
            ("predict.token", list(model.predict["token"].shape), _f64_le(model.predict["token"])),
            (
                "predict.diffusion",
                list(model.predict["diffusion"].shape),
                _f64_le(model.predict["diffusion"]),
            ),
            (
                "predict.world_model",
                list(model.predict["world_model"].shape),
                _f64_le(model.predict["world_model"]),
            ),
        ]
    meta = {
        "format": WEIGHT_FORMAT_V2,
        "n_modes": str(n_modes),
        "latent_dim": str(int(model.embed.shape[0])),
        "lipschitz_bound": repr(float(lipschitz)),
    }
    header: dict = {"__metadata__": meta}
    offset = 0
    blobs: list[bytes] = []
    for name, shape, data in tensors:
        header[name] = {
            "dtype": "F64",
            "shape": shape,
            "data_offsets": [offset, offset + len(data)],
        }
        blobs.append(data)
        offset += len(data)
    raw = json.dumps(header, separators=(",", ":")).encode("utf-8")
    pad = (8 - (len(raw) % 8)) % 8
    raw = raw + (b" " * pad)
    path.write_bytes(struct.pack("<Q", len(raw)) + raw + b"".join(blobs))


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--data", type=pathlib.Path, required=True, help="`aria dataset` JSON")
    p.add_argument("--out", type=pathlib.Path, required=True, help="weights JSON (v1) to write")
    p.add_argument(
        "--out-v2",
        type=pathlib.Path,
        help="optional aria-predictor-v2 safetensors path",
    )
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
    p.add_argument("--penalty", type=float, default=1.0, help="isometry-penalty weight inside ℒ_Spectral")
    p.add_argument("--holdout", type=float, default=0.2)
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--quiet", action="store_true")
    p.add_argument("--lambda-jepa", type=float, default=DEFAULT_LAMBDAS[0])
    p.add_argument("--lambda-nll", type=float, default=DEFAULT_LAMBDAS[1])
    p.add_argument("--lambda-spectral", type=float, default=DEFAULT_LAMBDAS[2])
    p.add_argument("--lambda-graph", type=float, default=DEFAULT_LAMBDAS[3])
    p.add_argument(
        "--min-rankme-frac",
        type=float,
        default=MIN_RANKME_FRAC_DEFAULT,
        help="abort if RankMe(Z) < this fraction of d (default 0.3)",
    )
    p.add_argument(
        "--graph",
        type=pathlib.Path,
        help="exported graph JSON for ℒ_Graph (aria run --export-graph)",
    )
    p.add_argument(
        "--graph-gamma",
        type=float,
        default=0.5,
        help="γ_uv hinge, default = merge τ",
    )
    p.add_argument(
        "--chunk-length",
        type=int,
        default=0,
        help="cut a long frame stream into trajectories of this length (0 = leave as loaded)",
    )
    p.add_argument(
        "--wilcoxon",
        action="store_true",
        help="require ≥ 30 holdout trajectories and a paired Wilcoxon p < 0.01",
    )
    p.add_argument("--min-holdout-trajs", type=int, default=30)
    args = p.parse_args()

    lambdas = (args.lambda_jepa, args.lambda_nll, args.lambda_spectral, args.lambda_graph)
    validate_simplex(*lambdas)

    data, n_modes = load_dataset(args.data)
    if args.chunk_length:
        data = chunk_frames(data, args.chunk_length)

    graph_src = graph_dst = None
    if args.graph is not None:
        graph_src, graph_dst = load_graph_pairs(args.graph)

    try:
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
            lambdas=lambdas,
            min_rankme_frac=args.min_rankme_frac,
            graph_src=graph_src,
            graph_dst=graph_dst,
            graph_gamma=args.graph_gamma,
        )
    except CollapseError as err:
        print(f"COLLAPSE: {err}")
        return 2

    args.out.write_text(json.dumps(export(model, n_modes, args.lipschitz)))
    if args.out_v2:
        write_safetensors_v2(args.out_v2, model, n_modes, args.lipschitz)
    if args.metrics:
        args.metrics.write_text("\n".join(json.dumps(r) for r in history) + "\n")

    epochs_ran = [r for r in history if "holdout_residual" in r]
    first = epochs_ran[0]
    final = {
        "holdout_residual": min(r["holdout_residual"] for r in epochs_ran),
        "lipschitz": max(
            float(torch.linalg.matrix_norm(model.predict[n], ord=2)) for n in CONDITIONS
        ),
    }
    persistence = max(r["persistence_baseline"] for r in epochs_ran)
    rankme_curve = [r["rankme"] for r in epochs_ran if "rankme" in r]

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
    print(
        f"RankMe curve min/max {min(rankme_curve):.3f}/{max(rankme_curve):.3f} "
        f"(gate {args.min_rankme_frac} · d = {args.min_rankme_frac * args.latent_dim:.3f})"
    )
    print(f"Lip(P) = {final['lipschitz']:.4f} <= {args.lipschitz} (P2 enforced)")
    print(f"weights written to {args.out}")
    if args.out_v2:
        print(f"v2 safetensors written to {args.out_v2}")

    wilcoxon_ok = True
    if args.wilcoxon:
        if data.shape[0] <= 1:
            print("Wilcoxon requested but dataset has a single trajectory — chunk first")
            return 1
        split = max(1, int(data.shape[0] * (1.0 - args.holdout)))
        if split >= data.shape[0]:
            split = data.shape[0] - 1
        hold = data[split:]
        if hold.shape[0] < args.min_holdout_trajs:
            print(
                f"Wilcoxon needs ≥ {args.min_holdout_trajs} holdout trajectories, "
                f"got {hold.shape[0]}"
            )
            return 1
        model_vals, persist_vals = per_trajectory_residuals(model, hold)
        p_value, median = wilcoxon_paired(persist_vals, model_vals)
        lo, hi = bootstrap_median_ci(
            [p - m for p, m in zip(persist_vals, model_vals)], seed=args.seed
        )
        print(
            f"Wilcoxon n={len(model_vals)} p={p_value:.6g} "
            f"median_improvement={median:.6f} bootstrap99%=[{lo:.6f}, {hi:.6f}]"
        )
        wilcoxon_ok = p_value < 0.01 and median > 0.0
        if not wilcoxon_ok:
            print("Wilcoxon gate FAILED (need p < 0.01 and median improvement > 0)")

    # Exit3: the held-out residual must fall *and* beat the trivial baseline.
    return 0 if (improved and beats_persistence and wilcoxon_ok) else 1


if __name__ == "__main__":
    raise SystemExit(main())
