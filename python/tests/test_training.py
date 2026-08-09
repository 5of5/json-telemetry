"""Exit3 — the held-out JEPA residual must fall, with no decoder in the loop.

Runs the whole Phase 3 loop end to end:

    aria dataset  ->  train_jepa.py  ->  aria run --predictor

Skipped when PyTorch is unavailable; the Spec-admissibility half of Phase 3 is
covered without torch by `crates/aria-backends/tests/trained.rs`.
"""

import json
import pathlib
import subprocess
import sys

import pytest

torch = pytest.importorskip("torch", reason="Phase 3 training needs PyTorch")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "python" / "training"))

import train_jepa  # noqa: E402

N_MODES = 16
LATENT_DIM = 16
EPOCHS = 120


def cli_binary():
    for profile in ("release", "debug"):
        path = REPO_ROOT / "target" / profile / "aria"
        if path.exists():
            return path
    pytest.skip("run `cargo build -p aria-engine` first")


@pytest.fixture(scope="module")
def dataset(tmp_path_factory):
    out = tmp_path_factory.mktemp("aria") / "data.json"
    subprocess.run(
        [
            str(cli_binary()), "dataset",
            "--n-modes", str(N_MODES),
            "--trajectories", "64",
            "--length", "10",
            "--output", str(out),
        ],
        check=True,
        capture_output=True,
    )
    return out


@pytest.fixture(scope="module")
def trained(dataset, tmp_path_factory):
    data, n_modes = train_jepa.load_dataset(dataset)
    model, history = train_jepa.train(
        data=data,
        n_modes=n_modes,
        latent_dim=LATENT_DIM,
        epochs=EPOCHS,
        lr=5e-3,
        lipschitz=0.49,
        penalty=1.0,
        holdout=0.2,
        seed=0,
        quiet=True,
    )
    out = tmp_path_factory.mktemp("aria") / "weights.json"
    out.write_text(json.dumps(train_jepa.export(model, n_modes, 0.49)))
    return out, history


def test_dataset_is_unitary(dataset):
    blob = json.loads(dataset.read_text())
    assert blob["format"] == "aria-optical-dataset-v1"
    for traj in blob["trajectories"]:
        for snapshot in traj:
            norm = sum(v * v for v in snapshot) ** 0.5
            assert abs(norm - 1.0) < 1e-9, "optical steps must preserve energy"


def test_holdout_residual_decreases(trained):
    _, history = trained
    first, last = history[0]["holdout_residual"], history[-1]["holdout_residual"]
    assert last < first, f"held-out residual did not decrease: {first} -> {last}"
    # A real learning signal, not numerical noise.
    assert last < 0.5 * first, f"held-out residual barely moved: {first} -> {last}"


def test_train_residual_decreases(trained):
    _, history = trained
    assert history[-1]["train_residual"] < history[0]["train_residual"]


def test_lipschitz_bound_holds_at_every_epoch(trained):
    # P2 is a hard constraint, not a convergence property.
    _, history = trained
    for record in history[1:]:
        assert record["lipschitz"] <= 0.49 + 1e-9, record
        assert record["embed_norm"] <= 1.0 + 1e-9, record


def test_no_decoder_in_the_training_module():
    """The JEPA axiom: targets are embeddings, never reconstructions.

    Comments and docstrings may discuss the absence of a decoder; executable
    code may not contain one. Tokenizing with `ast` keeps prose out of scope.
    """
    import ast

    src = (REPO_ROOT / "python" / "training" / "train_jepa.py").read_text()
    tree = ast.parse(src)

    # Strip docstrings so only real identifiers and literals remain.
    for node in ast.walk(tree):
        if isinstance(
            node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)
        ):
            body = node.body
            if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant):
                if isinstance(body[0].value.value, str):
                    node.body = body[1:]

    names = {n.id.lower() for n in ast.walk(tree) if isinstance(n, ast.Name)}
    names |= {n.attr.lower() for n in ast.walk(tree) if isinstance(n, ast.Attribute)}
    names |= {
        n.arg.lower() for n in ast.walk(tree) if isinstance(n, ast.arg)
    }
    strings = {
        n.value.lower()
        for n in ast.walk(tree)
        if isinstance(n, ast.Constant) and isinstance(n.value, str)
    }

    for banned in ("decoder", "decode", "reconstruct", "convtranspose"):
        offenders = [s for s in names | strings if banned in s]
        assert not offenders, f"'{banned}' must not appear in the core loop: {offenders}"


def test_trained_weights_run_green_in_the_spec_loop(trained):
    weights, _ = trained
    proc = subprocess.run(
        [
            str(cli_binary()), "run",
            "--steps", "400",
            "--schedule", "opmd",
            "--predictor", str(weights),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    assert "invariants=OK" in proc.stderr, proc.stderr
    rows = [json.loads(line) for line in proc.stdout.strip().splitlines()[1:]]
    assert len(rows) == 400
    assert "".join(r["action"] for r in rows) == "OPMD" * 100


def test_trained_model_beats_persistence(trained):
    """Exit3 quality gate: the model must beat 'predict tomorrow = today'."""
    _, history = trained
    epochs_ran = [r for r in history if "holdout_residual" in r]
    best_residual = min(r["holdout_residual"] for r in epochs_ran)
    persistence = max(r["persistence_baseline"] for r in epochs_ran)
    assert best_residual < persistence, (
        f"model residual {best_residual:.4f} failed to beat "
        f"persistence baseline {persistence:.4f}"
    )


# --- The real-data path: actual text, not synthetic phase ramps -------------


@pytest.fixture(scope="module")
def real_corpus(tmp_path_factory):
    """The repository's own documentation as a real byte stream."""
    docs = sorted(p for p in (REPO_ROOT / "docs").glob("*.md") if p.is_file())
    docs.append(REPO_ROOT / "README.md")
    blob = b"\n\n".join(p.read_bytes() for p in docs)
    out = tmp_path_factory.mktemp("corpus") / "corpus.txt"
    out.write_bytes(blob)
    return out


@pytest.fixture(scope="module")
def real_dataset(real_corpus, tmp_path_factory):
    out = tmp_path_factory.mktemp("data") / "text.json"
    subprocess.run(
        [
            str(cli_binary()), "dataset",
            "--input", str(real_corpus),
            "--n-modes", "32",
            "--output", str(out),
        ],
        check=True,
        capture_output=True,
    )
    return out


def test_real_dataset_is_real_data(real_dataset):
    blob = json.loads(real_dataset.read_text())
    assert blob["format"] == "aria-text-dataset-v1"
    assert blob["encoding"] == "spectral-dft"
    assert blob["source_bytes"] > 100_000, "the corpus is the full docs tree"
    frames = blob["trajectories"][0]
    assert len(frames) > 1000
    for frame in frames[:50]:
        norm = sum(v * v for v in frame) ** 0.5
        assert abs(norm - 1.0) < 1e-9, "every encoded field is unit norm"


@pytest.fixture(scope="module")
def trained_on_text(real_dataset, tmp_path_factory):
    data, n_modes = train_jepa.load_dataset(real_dataset)
    model, history = train_jepa.train(
        data=data,
        n_modes=n_modes,
        latent_dim=32,
        epochs=60,
        lr=5e-3,
        lipschitz=0.49,
        penalty=1.0,
        holdout=0.2,
        seed=0,
        quiet=True,
    )
    out = tmp_path_factory.mktemp("textweights") / "weights.json"
    out.write_text(json.dumps(train_jepa.export(model, n_modes, 0.49)))
    return out, history


def test_real_text_training_beats_persistence(trained_on_text):
    """The headline result: Aria learns structure in real text."""
    _, history = trained_on_text
    epochs_ran = [r for r in history if "holdout_residual" in r]
    first, last = epochs_ran[0], epochs_ran[-1]

    assert last["holdout_residual"] < first["holdout_residual"]
    best = min(r["holdout_residual"] for r in epochs_ran)
    persistence = max(r["persistence_baseline"] for r in epochs_ran)
    assert best < persistence, (
        f"on real text: model {best:.4f} must beat persistence {persistence:.4f}"
    )
    for record in epochs_ran[1:]:
        assert record["lipschitz"] <= 0.49 + 1e-9, record
        assert record["embed_norm"] <= 1.0 + 1e-9, record


def test_text_trained_weights_run_green(trained_on_text):
    weights, _ = trained_on_text
    proc = subprocess.run(
        [
            str(cli_binary()), "run",
            "--steps", "400",
            "--schedule", "opmd",
            "--predictor", str(weights),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    assert "invariants=OK" in proc.stderr, proc.stderr
