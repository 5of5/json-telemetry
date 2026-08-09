# aria-engine (Python)

Python bindings for the [Aria](https://github.com/aria-ai/aria) transformer runtime.

The extension is a façade over `aria_engine_backends::runner` — the same code
path the `aria` CLI and the WASM module use. Notebook results equal CLI results
byte for byte for the same config.

## Build

```bash
pip install maturin
cd python
maturin develop          # editable install into the active virtualenv
# or: maturin build --release
```

## Use

```python
import aria

engine = aria.AriaEngine(aria.Config(n_modes=64, latent_dim=32, eps=1.0, seed=42))

state = engine.init()
state = engine.step_phi(state)          # OpticalStep -> Predict -> Match -> Diffuse
assert engine.check(state).all_ok

summary = aria.run(steps=1000, config=aria.Config(schedule="opmd"))
# {'steps': 1000, 't': 250, 'graph_size': 250, 'energy': 1.0, 'invariants_ok': True, ...}

jsonl = aria.run_trace_jsonl(steps=1000)  # identical to `aria run --output trace.jsonl`
```

## API

| Symbol | Purpose |
|---|---|
| `aria.Config(...)` | Runtime config; `Config.from_toml(src)` reads the CLI format |
| `aria.AriaEngine(config)` | `init()`, `apply(state, action)`, `step_phi(state)`, `check(state)` |
| `aria.State` | `t`, `energy`, `prev_res`, `graph_size`, `z`, `psi`, `to_json()` |
| `aria.InvariantReport` | `inv1`–`inv4`, `all_ok`, `failures` |
| `aria.run(steps, config)` | Summary dict for a full run |
| `aria.run_trace_jsonl(steps, config)` | JSONL trace string |
| `aria.actions()` | `["OpticalStep", "Predict", "Match", "Diffuse", "Stutter"]` |

## Test

```bash
cargo build -p aria-engine   # the CLI, for the differential parity test
pytest python/tests
```
