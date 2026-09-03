# aria-json-telemetry

Published crate for Aria JSON telemetry (`aria_operator` rustc name). **0.2.1.**

```toml
aria-json-telemetry = "0.2.1"
```

```bash
cargo add aria-json-telemetry
cargo install aria-json-telemetry --bin work
```

The node is a **stateless IPO**: workers bring organized JSON, one ingest +
optional Φ projects 560 closed verticals, the callback returns **working data
only**. Neo4j is memory. Nothing is stored here.

| Deployed crate | Version | Registry |
|---|---|---|
| `aria-json-telemetry` (`work`) | **0.2.1** | crates.io |
| `aria-engine-core` | 0.2.1 | crates.io (already) |
| `aria-engine-backends` | 0.2.1 | crates.io (already) |
| `aria-engine` / `aria-engine-wasm` | 0.2.1 | crates.io (already) |
| 560 `crates/operators/*` | workspace 0.2.1 | `publish = false` |

Engine crates stay at 0.2.1; this cut does not republish Φ. Workspace will
not move to 0.3.0 until winning-condition W6.

## Call it

```bash
# Identify funnel (production): one binary, stdin JSON, working vertical or nothing
printf '{"nodes":[{"id":1,"type":"Person","label":"Ada","notes":"founder"}]}' \
  | work --binary BIN.PEOPLE --steps 0

# PCVC Mode 4 harness lane (stderr empty, bindings echoed, ≤ 64 KiB)
work --harness < request.json          # or auto-detected by schemaVersion
work --dispatch                        # aria-dispatch-v1: 560 binaries + exe sha256
work --serve 0.0.0.0:8080              # /health /commands /dispatch /work /harness
```

Container (scratch, static MUSL, non-root 65534):

```bash
docker build --target work -t aria-work:0.2.1 .           # 2.04 MB image
docker run -i --network=none aria-work:0.2.1 --steps 0 < payload.json
docker run -p 8080:8080 aria-work:0.2.1                    # hosted shell
docker compose -f compose.telemetry.yaml up --build --scale telemetry=4
```

## Hosted shell under load

`--serve` is a fixed worker pool (4× cores) over a bounded queue (1024). Past
the queue the node answers `503 Retry-After: 1` at once instead of adding
latency; sockets carry 10 s read/write deadlines so a stalled client cannot
pin a worker; `/health /commands /dispatch` are serialized once per process.
There is no shared mutable state — nothing to lock, nothing to deadlock.
Lock: `tests/serve_load.rs` (32 clients × 8 harness calls → one distinct
body, 0 errors; ~950 ops/s debug build on 12 cores).

## What it is

Every catalog binary (`crates/operators/*`) is a distinct crate. It owns a
frozen `spec.json` (unique `binary_id`, operator, declared node/rel/tag
types). It links this library, which:

1. Runs `aria_engine_backends::telemetry::transform` — the Aria transformer,
   Init + Φ + projection. No sixth action. No Trust field.
2. Projects a **closed operator JSON** (sheet `09_JSON_OPERATOR_SHAPE`).
3. Nests the full `aria-telemetry-query-v1` document under `telemetry` only
   when `--telemetry` is set.

B0: the operator body contains only the types this crate declared. B1: crates
do not share mutable calculation state. B7: Aria is never a judge.

A worker is pointed at one crate. See [WORKER.md](WORKER.md).
What remains: [PLAN.md](PLAN.md).
Measured speed / virality / wire shape: [OPTIMIZATION.md](OPTIMIZATION.md).
Every identity: [catalog/INDEX.md](catalog/INDEX.md).
The 25 sealed map mixers: [maps/MAPS.md](maps/MAPS.md).

Dump (garbage collection + scores):

```bash
cargo run -p aria-json-telemetry --example dump -- dump
```

Regenerate crates from the sheet dump:

```bash
python3 crates/aria-operator/generate.py
```
