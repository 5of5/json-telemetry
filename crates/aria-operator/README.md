# aria-json-telemetry

Published crate for Aria JSON telemetry (`aria_operator` rustc name).

```toml
aria-json-telemetry = "0.2.0"
```

```bash
cargo add aria-json-telemetry
cargo install aria-json-telemetry --bin work
```

Shared library for the Binary Repository v1 operator crates.

Every catalog binary (`crates/operators/*`) is a distinct crate. It owns a
frozen `spec.json` (unique `binary_id`, operator, declared node/rel/tag
types). It links this library, which:

1. Runs `aria_engine_backends::telemetry::transform` — the Aria transformer,
   Init + Φ + projection. No sixth action. No Trust field.
2. Projects a **closed operator JSON** (sheet `09_JSON_OPERATOR_SHAPE`).
3. Nests the full `aria-telemetry-query-v1` document under `telemetry`.

B0: the operator body contains only the types this crate declared. B1: crates
do not share mutable calculation state. B7: Aria is never a judge.

A worker is pointed at one crate. See [WORKER.md](WORKER.md).
What remains: [PLAN.md](PLAN.md).

Dump (garbage collection + scores):

```bash
cargo run -p aria-json-telemetry --example dump -- dump
```

See [DUMP_ANALYSIS.md](DUMP_ANALYSIS.md).

Regenerate crates from the sheet dump:

```bash
python3 crates/aria-operator/generate.py
```
