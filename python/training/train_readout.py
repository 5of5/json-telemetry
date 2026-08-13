"""ℒ_NLL — discrete readout training on frozen latents (𝔸5 / spec §6.2).

This module is the only place a reconstruction / discrete-output head is
allowed to train. The predictor I and P are never loaded: gradients cannot
reach Φ because Φ's weights are not in the graph.

Inputs:
  --latents   JSONL of z vectors (one JSON array per line), from
              `aria emit --dump-latents`.
  --targets   JSON array of integer token ids, same length as the latent
              stream (or a JSONL of `{"id": int}` rows).

The head matches crates/aria-backends DiscreteReadout: layer-norm then a
bias-free linear map d → |V_o|. Loss is temperature-scaled cross-entropy.
Exported file is `aria-readout-v1` safetensors, loadable by `aria emit`.
"""

from __future__ import annotations

import argparse
import json
import math
import pathlib
import struct

import torch
import torch.nn.functional as F

READOUT_FORMAT = "aria-readout-v1"
VOCAB_MIN = 256
VOCAB_MAX = 128_000
LN_EPS = 1e-5


class DiscreteHead(torch.nn.Module):
    """LN → linear(d → |V_o|, no bias). Owns every parameter ℒ_NLL may touch."""

    def __init__(self, dim: int, vocab_size: int, temperature: float, seed: int) -> None:
        super().__init__()
        if not VOCAB_MIN <= vocab_size <= VOCAB_MAX:
            raise ValueError(f"vocab_size {vocab_size} outside [{VOCAB_MIN}, {VOCAB_MAX}]")
        if not math.isfinite(temperature) or temperature <= 0.0:
            raise ValueError(f"temperature must be finite and > 0, got {temperature}")
        self.dim = dim
        self.vocab_size = vocab_size
        self.temperature = temperature
        gen = torch.Generator().manual_seed(seed)
        self.ln_weight = torch.nn.Parameter(torch.ones(dim, dtype=torch.float64))
        self.ln_bias = torch.nn.Parameter(torch.zeros(dim, dtype=torch.float64))
        scale = 1.0 / math.sqrt(dim)
        raw = (torch.rand(vocab_size, dim, generator=gen, dtype=torch.float64) - 0.5) * 2.0 * scale
        self.weight = torch.nn.Parameter(raw)

    def logits(self, z: torch.Tensor) -> torch.Tensor:
        y = F.layer_norm(z, (self.dim,), self.ln_weight, self.ln_bias, LN_EPS)
        return y @ self.weight.T

    def nll(self, z: torch.Tensor, targets: torch.Tensor) -> torch.Tensor:
        return F.cross_entropy(self.logits(z) / self.temperature, targets)


def load_latents(path: pathlib.Path) -> torch.Tensor:
    rows: list[list[float]] = []
    for line in path.read_text().splitlines():
        if not line.strip():
            continue
        item = json.loads(line)
        if isinstance(item, dict):
            item = item["z"]
        rows.append(item)
    if not rows:
        raise ValueError(f"{path} contains no latents")
    return torch.tensor(rows, dtype=torch.float64)


def load_targets(path: pathlib.Path) -> torch.Tensor:
    text = path.read_text().strip()
    if text.startswith("["):
        ids = json.loads(text)
    else:
        ids = []
        for line in text.splitlines():
            if not line.strip():
                continue
            item = json.loads(line)
            ids.append(item["id"] if isinstance(item, dict) else item)
    if not ids:
        raise ValueError(f"{path} contains no target ids")
    return torch.tensor(ids, dtype=torch.long)


def _f64_le(tensor: torch.Tensor) -> bytes:
    flat = tensor.detach().cpu().contiguous().to(torch.float64).reshape(-1)
    return struct.pack("<" + "d" * flat.numel(), *flat.tolist())


def write_readout_v1(path: pathlib.Path, head: DiscreteHead) -> None:
    tensors = [
        ("ln_weight", [head.dim], _f64_le(head.ln_weight)),
        ("ln_bias", [head.dim], _f64_le(head.ln_bias)),
        ("weight", [head.vocab_size, head.dim], _f64_le(head.weight)),
    ]
    meta = {
        "format": READOUT_FORMAT,
        "kind": "discrete",
        "dim": str(head.dim),
        "vocab_size": str(head.vocab_size),
        "temperature": repr(float(head.temperature)),
    }
    header: dict = {"__metadata__": meta}
    offset = 0
    blobs: list[bytes] = []
    for name, shape, data in tensors:
        header[name] = {"dtype": "F64", "shape": shape, "data_offsets": [offset, offset + len(data)]}
        blobs.append(data)
        offset += len(data)
    raw = json.dumps(header, separators=(",", ":")).encode("utf-8")
    pad = (8 - (len(raw) % 8)) % 8
    raw = raw + (b" " * pad)
    path.write_bytes(struct.pack("<Q", len(raw)) + raw + b"".join(blobs))


def train_readout(
    z: torch.Tensor,
    targets: torch.Tensor,
    vocab_size: int,
    epochs: int,
    lr: float,
    temperature: float,
    seed: int,
    quiet: bool = False,
) -> tuple[DiscreteHead, list[dict]]:
    if z.requires_grad:
        raise ValueError("latents must be frozen (no grad) — 𝔸5")
    z = z.detach()
    if z.shape[0] != targets.shape[0]:
        raise ValueError(f"latents {z.shape[0]} vs targets {targets.shape[0]}")
    if int(targets.min()) < 0 or int(targets.max()) >= vocab_size:
        raise ValueError("target id outside [0, vocab_size)")

    head = DiscreteHead(z.shape[1], vocab_size, temperature, seed)
    opt = torch.optim.Adam(head.parameters(), lr=lr)
    history: list[dict] = []
    best: tuple[float, dict[str, torch.Tensor]] | None = None
    for epoch in range(epochs + 1):
        if epoch > 0:
            opt.zero_grad()
            loss = head.nll(z, targets)
            if not torch.isfinite(loss):
                raise FloatingPointError(f"non-finite NLL at epoch {epoch}")
            loss.backward()
            opt.step()
        with torch.no_grad():
            nll = float(head.nll(z, targets))
            pred = head.logits(z).argmax(dim=-1)
            acc = float((pred == targets).to(torch.float64).mean())
        history.append({"epoch": epoch, "nll": nll, "acc": acc})
        if best is None or nll < best[0]:
            best = (nll, {k: v.clone() for k, v in head.state_dict().items()})
        if not quiet and (epoch % max(1, epochs // 10) == 0 or epoch == epochs):
            print(f"epoch {epoch:4d}  nll {nll:.6f}  acc {acc:.4f}")
    assert best is not None
    head.load_state_dict(best[1])
    return head, history


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--latents", type=pathlib.Path, required=True)
    p.add_argument("--targets", type=pathlib.Path, required=True)
    p.add_argument("--out", type=pathlib.Path, required=True)
    p.add_argument("--vocab-size", type=int, default=256)
    p.add_argument("--epochs", type=int, default=100)
    p.add_argument("--lr", type=float, default=1e-2)
    p.add_argument("--temperature", type=float, default=1.0)
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--quiet", action="store_true")
    args = p.parse_args()

    z = load_latents(args.latents)
    targets = load_targets(args.targets)
    head, history = train_readout(
        z,
        targets,
        vocab_size=args.vocab_size,
        epochs=args.epochs,
        lr=args.lr,
        temperature=args.temperature,
        seed=args.seed,
        quiet=args.quiet,
    )
    write_readout_v1(args.out, head)
    first, last = history[0], min(history, key=lambda r: r["nll"])
    print(f"NLL {first['nll']:.6f} -> {last['nll']:.6f}; wrote {args.out}")
    return 0 if last["nll"] < first["nll"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
