"""WS5 training tests — authored from plan_v0.2.0.md / last_tasks.

The pre-WS5 `test_training.py` source is gone (pyc only). This file is a
fresh implementation of the plan's contract. The pyc was not decompiled
(Q-2026-08-13-3).
"""

from __future__ import annotations

import pathlib
import sys

import pytest
import torch

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "training"))

import train_jepa
import train_readout


def _fields(trajs: int, length: int, n_modes: int, seed: int = 0) -> torch.Tensor:
    g = torch.Generator().manual_seed(seed)
    x = torch.randn(trajs, length, 2 * n_modes, generator=g, dtype=torch.float64)
    return x / x.norm(dim=-1, keepdim=True).clamp_min(1e-12)


def test_stop_gradient_blocks_target_side_embed_grad() -> None:
    model = train_jepa.JepaPredictor(8, 4, seed=0)
    g = torch.Generator().manual_seed(1)
    psi_t = torch.randn(5, 8, generator=g, dtype=torch.float64)
    psi_next = torch.randn(5, 8, generator=g, dtype=torch.float64)

    loss = train_jepa.jepa_residual(model, psi_t, psi_next, "token")
    loss.backward()
    grad_shipped = model.embed.grad.detach().clone()

    model.zero_grad()
    predicted = model(psi_t, "token")
    target = model.encode(psi_next).detach()
    ((predicted - target).pow(2).mean()).backward()
    grad_manual_detach = model.embed.grad.detach().clone()
    assert torch.allclose(grad_shipped, grad_manual_detach)

    model.zero_grad()
    predicted = model(psi_t, "token")
    target_live = model.encode(psi_next)
    ((predicted - target_live).pow(2).mean()).backward()
    grad_both = model.embed.grad.detach().clone()
    assert not torch.allclose(grad_shipped, grad_both), (
        "stop-gradient must change the embed grad versus a live target"
    )


def test_jepa_residual_uses_squared_error() -> None:
    model = train_jepa.JepaPredictor(6, 3, seed=2)
    x = torch.zeros(2, 6, dtype=torch.float64)
    y = torch.zeros(2, 6, dtype=torch.float64)
    x[0, 0] = 1.0
    y[0, 1] = 1.0
    val = float(train_jepa.jepa_residual(model, x, y, "token"))
    pred = model(x, "token")
    tgt = model.encode(y).detach()
    want = float((pred - tgt).pow(2).mean())
    assert abs(val - want) < 1e-15


def test_rankme_of_identity_is_dimension() -> None:
    z = torch.eye(8, dtype=torch.float64)
    assert abs(train_jepa.rankme(z) - 8.0) < 1e-8


def test_rankme_of_collapsed_is_near_one() -> None:
    z = torch.ones(32, 8, dtype=torch.float64)
    assert train_jepa.rankme(z) < 1.1


def test_rankme_of_full_rank_noise_is_high() -> None:
    g = torch.Generator().manual_seed(3)
    z = torch.randn(80, 8, generator=g, dtype=torch.float64)
    assert train_jepa.rankme(z) > 6.0


def test_collapse_gate_aborts_on_rank_one() -> None:
    z = torch.ones(20, 8, dtype=torch.float64)
    with pytest.raises(train_jepa.CollapseError, match="collapsed"):
        train_jepa.abort_if_collapsed(z, min_rankme_frac=0.3, latent_dim=8)


def test_collapse_gate_passes_full_rank() -> None:
    z = torch.eye(8, dtype=torch.float64)
    score = train_jepa.abort_if_collapsed(z, min_rankme_frac=0.3, latent_dim=8)
    assert score >= 0.3 * 8


def test_simplex_accepts_the_default_and_uniform() -> None:
    train_jepa.validate_simplex(0.6, 0.0, 0.4, 0.0)
    train_jepa.validate_simplex(0.25, 0.25, 0.25, 0.25)


def test_simplex_rejects_negative_and_unnormalized() -> None:
    with pytest.raises(ValueError, match="Δ"):
        train_jepa.validate_simplex(-0.1, 0.4, 0.4, 0.3)
    with pytest.raises(ValueError, match="Δ"):
        train_jepa.validate_simplex(0.5, 0.5, 0.5, 0.5)


def test_l_total_is_the_weighted_sum() -> None:
    j = torch.tensor(2.0, dtype=torch.float64)
    n = torch.tensor(4.0, dtype=torch.float64)
    s = torch.tensor(0.0, dtype=torch.float64)
    g = torch.tensor(8.0, dtype=torch.float64)
    out = float(train_jepa.l_total(j, n, s, g, (0.5, 0.5, 0.0, 0.0)))
    assert abs(out - 3.0) < 1e-15


def test_graph_hinge_is_zero_inside_gamma_and_squared_outside() -> None:
    src = torch.tensor([[0.0, 0.0], [0.0, 0.0]], dtype=torch.float64)
    dst = torch.tensor([[0.5, 0.0], [2.0, 0.0]], dtype=torch.float64)
    loss = float(train_jepa.graph_hinge_loss(src, dst, gamma=1.0))
    assert abs(loss - 1.0) < 1e-12


def test_train_jepa_refuses_nonzero_lambda_nll() -> None:
    data = _fields(4, 4, 4)
    with pytest.raises(ValueError, match="train_readout"):
        train_jepa.train(
            data=data,
            n_modes=4,
            latent_dim=4,
            epochs=1,
            lr=1e-2,
            lipschitz=0.49,
            penalty=1.0,
            holdout=0.25,
            seed=0,
            quiet=True,
            lambdas=(0.5, 0.5, 0.0, 0.0),
        )


def test_short_train_decreases_holdout_and_beats_persistence() -> None:
    # Structured temporal map: next frame is a fixed rotation of the current
    # one, so a linear P can beat persistence. No synthetic "quality-gate"
    # labels — the fields themselves carry the dynamics.
    n_modes, dim, trajs, length = 4, 4, 12, 8
    g = torch.Generator().manual_seed(4)
    q, _ = torch.linalg.qr(torch.randn(2 * n_modes, 2 * n_modes, generator=g, dtype=torch.float64))
    frames = []
    for k in range(trajs):
        x0 = torch.randn(2 * n_modes, generator=g, dtype=torch.float64)
        x0 = x0 / x0.norm()
        seq = [x0]
        x = x0
        for _ in range(length - 1):
            x = q @ x
            seq.append(x)
        frames.append(torch.stack(seq))
    data = torch.stack(frames)

    model, history = train_jepa.train(
        data=data,
        n_modes=n_modes,
        latent_dim=dim,
        epochs=40,
        lr=5e-3,
        lipschitz=0.49,
        penalty=1.0,
        holdout=0.25,
        seed=0,
        quiet=True,
    )
    first = history[0]["holdout_residual"]
    best = min(r["holdout_residual"] for r in history)
    persist = history[-1]["persistence_baseline"]
    assert best < first, f"holdout did not fall: {first} -> {best}"
    assert best < persist, f"did not beat persistence: {best} vs {persist}"
    assert history[-1]["rankme"] >= 0.3 * dim
    assert all(r["rankme"] >= 0.3 * dim for r in history if "rankme" in r)
    # Restored weights stay inside the hard Lip bound.
    # The hard projection uses the seeded power-iteration estimator, not the
    # exact SVD — agree with the Rust loader (plan WS1), not torch.linalg.
    for name in train_jepa.CONDITIONS:
        sigma = train_jepa.power_iteration_sigma(model.predict[name].detach())
        assert sigma <= 0.49 + 1e-6, f"{name} σ={sigma}"


def test_wilcoxon_on_known_pairs() -> None:
    model = [0.10] * 30
    persist = [0.50] * 30
    p_value, median = train_jepa.wilcoxon_paired(persist, model)
    assert p_value < 0.01
    assert median > 0.0
    lo, hi = train_jepa.bootstrap_median_ci([p - m for p, m in zip(persist, model)], n_boot=200)
    assert lo > 0.0
    assert hi > 0.0


def test_chunk_frames_makes_integer_trajectories() -> None:
    frames = torch.arange(40 * 6, dtype=torch.float64).reshape(1, 40, 6)
    chunked = train_jepa.chunk_frames(frames, 8)
    assert chunked.shape == (5, 8, 6)
    assert torch.equal(chunked[0, 0], frames[0, 0])


def test_safetensors_v2_header_is_self_describing(tmp_path: pathlib.Path) -> None:
    model = train_jepa.JepaPredictor(8, 4, seed=5)
    path = tmp_path / "w.safetensors"
    train_jepa.write_safetensors_v2(path, model, n_modes=4, lipschitz=0.49)
    raw = path.read_bytes()
    n = int.from_bytes(raw[:8], "little")
    header = raw[8 : 8 + n].decode("utf-8")
    assert "aria-predictor-v2" in header
    assert "predict.token" in header
    assert "F64" in header


def test_no_decoder_in_the_training_module() -> None:
    src = (ROOT / "training" / "train_jepa.py").read_text()
    lowered = src.lower()
    for pat in ("decoder", "softmax", "logits", "cross_entropy"):
        assert pat not in lowered, f"train_jepa.py must not contain {pat!r} (𝔸5)"


def test_readout_training_is_isolated_and_never_touches_jepa() -> None:
    src = (ROOT / "training" / "train_readout.py").read_text()
    assert "JepaPredictor" not in src
    assert "cross_entropy" in src
    g = torch.Generator().manual_seed(6)
    z = torch.randn(64, 8, generator=g, dtype=torch.float64)
    # Frozen: a caller who forgot detach still gets a hard error if they pass
    # a leaf with grad. The public path detaches.
    z_frozen = z.detach()
    targets = torch.randint(0, 256, (64,), generator=g)
    head, history = train_readout.train_readout(
        z_frozen, targets, vocab_size=256, epochs=8, lr=5e-2, temperature=1.0, seed=0, quiet=True
    )
    assert history[-1]["nll"] <= history[0]["nll"] + 1e-9
    assert z_frozen.requires_grad is False
    # Every trained parameter lives on the head.
    names = set(head.state_dict())
    assert names == {"ln_weight", "ln_bias", "weight"}


def test_frozen_latents_reject_a_live_graph() -> None:
    z = torch.randn(4, 8, dtype=torch.float64, requires_grad=True)
    targets = torch.zeros(4, dtype=torch.long)
    with pytest.raises(ValueError, match="frozen"):
        train_readout.train_readout(
            z, targets, vocab_size=256, epochs=1, lr=1e-2, temperature=1.0, seed=0, quiet=True
        )
