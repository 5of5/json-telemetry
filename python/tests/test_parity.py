"""Differential parity tests: the Python facade must equal the Rust CLI.

Build the extension first:

    cd python && maturin develop

Then:

    pytest python/tests
"""

import json
import pathlib
import subprocess

import pytest

aria = pytest.importorskip("aria", reason="build the extension with `maturin develop`")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]

# Small config so the CLI subprocess stays fast.
CONFIG_KWARGS = dict(n_modes=8, latent_dim=16, eps=1.0, schedule="opmd", seed=42)
STEPS = 100


def cli_binary():
    """Path to the debug `aria` binary, or skip if it has not been built."""
    path = REPO_ROOT / "target" / "debug" / "aria"
    if not path.exists():
        pytest.skip("run `cargo build -p aria-engine` first")
    return path


def test_actions_alphabet_is_exactly_five():
    # Sigma lock (Spec §1.1): equality, not inclusion.
    assert aria.actions() == [
        "OpticalStep",
        "Predict",
        "Match",
        "Diffuse",
        "Stutter",
    ]


def test_step_phi_advances_t_once_per_cycle():
    engine = aria.AriaEngine(aria.Config(**CONFIG_KWARGS))
    state = engine.init()
    assert state.t == 0
    for i in range(1, 6):
        state = engine.step_phi(state)
        assert state.t == i
        assert engine.check(state).all_ok


def test_invariants_hold_after_every_action():
    engine = aria.AriaEngine(aria.Config(**CONFIG_KWARGS))
    state = engine.init()
    for action in ["OpticalStep", "Predict", "Match", "Diffuse", "Stutter"]:
        state = engine.apply(state, action)
        report = engine.check(state)
        assert report.all_ok, f"{action}: {report.failures}"


def test_unchanged_clauses():
    engine = aria.AriaEngine(aria.Config(**CONFIG_KWARGS))
    base = engine.init()

    # OpticalStep: UNCHANGED z, t
    s = engine.apply(base, "OpticalStep")
    assert s.z == base.z
    assert s.t == base.t

    # Predict: UNCHANGED psi, t
    s = engine.apply(base, "Predict")
    assert s.psi == base.psi
    assert s.t == base.t

    # Diffuse: t advances, psi unchanged
    s = engine.apply(base, "Diffuse")
    assert s.t == base.t + 1
    assert s.psi == base.psi

    # Stutter: UNCHANGED everything
    s = engine.apply(base, "Stutter")
    assert s.t == base.t
    assert s.z == base.z
    assert s.psi == base.psi


def test_unknown_action_raises():
    engine = aria.AriaEngine(aria.Config(**CONFIG_KWARGS))
    with pytest.raises(ValueError):
        engine.apply(engine.init(), "EvolveH")


def test_conditioning_switches_without_a_second_architecture():
    # A4: token | diffusion | world_model all run the same engine.
    for cond in ["token", "diffusion", "world_model"]:
        cfg = aria.Config(condition=cond, **CONFIG_KWARGS)
        summary = aria.run(steps=40, config=cfg)
        assert summary["invariants_ok"], summary["failures"]
        assert summary["action_sequence"] == "OPMD" * 10


def test_gates_are_off_by_default():
    summary = aria.run(steps=40, config=aria.Config(**CONFIG_KWARGS))
    assert summary["gates"]["enabled"] == []
    assert summary["gates"]["ok"]


def test_gates_can_be_enabled_without_changing_behavior():
    off = aria.run(steps=200, config=aria.Config(**CONFIG_KWARGS))
    on = aria.run(steps=200, config=aria.Config(gates="all", **CONFIG_KWARGS))

    assert len(on["gates"]["enabled"]) == 7
    assert on["gates"]["ok"], on["gates"]["breaches"]
    # A gate observes; it must never steer.
    assert off["action_sequence"] == on["action_sequence"]
    assert off["t"] == on["t"]
    assert off["graph_size"] == on["graph_size"]


def test_unknown_gate_raises():
    with pytest.raises(ValueError):
        aria.Config(gates="inv12", **CONFIG_KWARGS)


def test_python_run_matches_cli_run():
    """Differential test: the Python trace equals the CLI trace byte for byte."""
    binary = cli_binary()

    cfg = aria.Config(**CONFIG_KWARGS)
    py_jsonl = aria.run_trace_jsonl(steps=STEPS, config=cfg)

    proc = subprocess.run(
        [
            str(binary),
            "run",
            "--steps", str(STEPS),
            "--schedule", CONFIG_KWARGS["schedule"],
            "--n-modes", str(CONFIG_KWARGS["n_modes"]),
            "--latent-dim", str(CONFIG_KWARGS["latent_dim"]),
            "--eps", str(CONFIG_KWARGS["eps"]),
            "--seed", str(CONFIG_KWARGS["seed"]),
        ],
        capture_output=True,
        text=True,
        check=True,
    )

    assert py_jsonl == proc.stdout, "Python and CLI traces diverged"

    # And the parsed summary agrees too.
    py_summary = aria.run(steps=STEPS, config=cfg)
    last = json.loads(proc.stdout.strip().splitlines()[-1])
    assert py_summary["t"] == last["t"] + 1  # t advances on the final Diffuse
    assert py_summary["graph_size"] == last["graph_size"]
