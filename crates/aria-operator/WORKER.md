# Worker → crate endpoint

A Mode 4 worker is **one catalog edge in flight** (Spawning Specification S6).
The Judge/Coordinator points it at exactly one work definition. That definition
is a row in Binary Repository v1 and a crate under `crates/operators/`.

```text
sealed Observation Plan
    requirement.resultDefinitionRef  +  allowed capability
            │
            ▼
aria_operator::endpoint_by_binary_id("BIN.PEOPLE")
            │
            ▼
work --binary BIN.PEOPLE --in payload.json
# or still: cargo run -p aria-telemetry-people -- --in payload.json
            │
            ▼
closed operator JSON  (vertical: only PEOPLE types)
    telemetry?  aria-telemetry-query-v1   ← optional spine (`--telemetry`)
```

Unstructured JSON (notes, facts, tags, a typed graph, a spreadsheet) goes in.
A structured query comes back. The worker does not pick the next binary.

## Pointing

| Plan field | Dispatch |
|---|---|
| `BIN.PEOPLE` | `endpoint_by_binary_id` |
| operator `PEOPLE` | `endpoint_by_operator` |
| cargo package | `endpoint_by_package("aria-telemetry-people")` |

560 endpoints (535 research/host + 25 `BIN.REF.*` map mixers). Same envelope schema. Unique `binary_id` / declared types. Map mixers ingest a dump callback or tagged graph and return only that map's neighborhood. Source unchanged. See [maps/MAPS.md](maps/MAPS.md).

AriA is linked into every binary. AriA is not the Judge (Spawning §9).

## Wire shape and the pruning contract

Every operator — all 560 — serializes **one** closed key list in **one** order
(`aria_operator::ENVELOPE_KEYS`; test-locked in `tests/wire_shape.rs`). Optional
members are omitted when empty, never filled with skeletons.

| key | always | worker keeps |
|---|---|---|
| `schema`, `schema_version` | yes | no (envelope tag) |
| `binary_id`, `operator`, `crate` | yes | `binary_id` |
| `plan_hash` | yes | when binding to an Observation Plan |
| `requirement_id` | when bound | when bound |
| `resultDefinitionRef` | yes | when routing by result type |
| `anchor_tags` | when declared | no (catalog data) |
| `subject_ids` | when non-empty | optional |
| `nodes` / `relationships` / `properties` | when non-empty | **yes** — the vertical |
| `verify`, `coverage_state` | yes | `coverage_state` |
| `no_finding_reason`, `limitations` | when present | `limitations` (uncast_token = vocabulary gap) |
| `content_hash` | yes | **yes** — independent identity of the vertical |
| `graph` | yes | **yes** — class/layer/weight/height/shape/anchors |
| `telemetry` | `--telemetry` only | no |

Pruning is therefore one operation for every binary:

```text
keep = binary_id, coverage_state, nodes, relationships, properties, content_hash, graph
```

The node is stateless: input → process → output. Nothing is stored here; Neo4j
is memory. The callback (`aria-work-v1`) carries **working verticals only**
(`asked` vs `ops` is the audit). A worker that asks 560 and gets 9 back has been
told, exactly, what the payload supports — the other 551 are absence, not bias.

Remainder and test/efficiency modules: [PLAN.md](PLAN.md).

## Real-time funnel (the only expand point)

The node is **stateless**: `payload bytes + spec + RunOpts → envelope`. Neo4j holds long-term memory. The worker brings the data, gets a structured graphical result, and **prunes on its side**.

```bash
# Identify (ingest + project, no Φ Match) — the lightweight path
printf '{"nodes":[{"id":1,"type":"Person","label":"Ada"}]}' \
  | work --binary BIN.PEOPLE --steps 0

# Batch: one Φ, N verticals. Working data only (asked ≠ ops).
echo '{"ops":["BIN.PEOPLE","BIN.COMPANY","BIN.REF.COMPETITIVE_RADAR"],"in":{"nodes":[...]}}' \
  | work --json

# Hosted command list (what Aria compiles against)
work --commands
```

`--steps 0` is the production identify funnel. Default `--steps 32` still runs Φ when a worker needs Match. Telemetry spine is **off** unless `--telemetry`.

## Canonical envelope keys (one shape for all 560)

Every operator serializes this list in this order. `?` members omit when empty/false/none.

| key | required |
|---|---|
| `schema` | yes |
| `binary_id` | yes |
| `operator` | yes |
| `schema_version` | yes |
| `crate` | yes |
| `plan_hash` | yes |
| `requirement_id` | ? |
| `subject_ids` | ? |
| `resultDefinitionRef` | yes |
| `anchor_tags` | ? |
| `neo4j_hit` | ? |
| `nodes` | ? |
| `relationships` | ? |
| `properties` | ? |
| `verify` | yes |
| `coverage_state` | yes |
| `no_finding_reason` | ? |
| `limitations` | ? |
| `content_hash` | yes |
| `graph` | ? |
| `telemetry` | ? |

**Worker prune (one operation, every operator):** keep `binary_id`, `coverage_state`, `nodes`, `relationships`, `properties`, `content_hash`, `graph`. Drop the rest. Do not prune per-binary.

Production callback (`aria-work-v1`): `{schema, phi_once, asked, ops, organize, results[]}`. `results` holds working verticals only. Absence is not a skeleton.

`organize` is the slop report (observer, not a judge): `tokens` scanned, `hits` (listed tags that fired), `uncast` (vocabulary gaps), `binaries` (catalog identities that will structure those hits), `nodes` / `edges` / `kinds`. The worker already knows the query depth (`steps`) and which tokens it pushed; this is what the node determined from them.

Map mixers (`BIN.REF.*`) re-feed that callback. Source bytes never change. See [maps/MAPS.md](maps/MAPS.md).

## PCVC Mode 4 harness lane

`work` speaks the `mode4/binaries/driver.py` contract directly (tracked as
[5of5/pcvc#70](https://github.com/5of5/pcvc/issues/70)): canonical JSON on
stdin → one JSON document on stdout → **stderr always empty** → exit 0 →
output ≤ 64 KiB → bindings echoed.

```bash
work --harness < request.json  # (bare `work <` auto-detects too; in the container CMD is --serve, so pass --harness)
work --dispatch                # aria-dispatch-v1: capability, executable sha256, 560 binaries
work --serve 0.0.0.0:8080      # hosted shell: /health /commands /dispatch /work /harness
```

| | |
|---|---|
| request | `pcvc-aria-telemetry-request-v1` — `capability: aria.telemetry.project`, `runId`, `planHash`, `attemptId`, `fencingToken`, `requirementId`, `ops[]`, `payload`, `steps` (0), `seed` (1), `outputLimitBytes` (≤ 65536) |
| result | `pcvc-aria-telemetry-result-v1` — bindings echoed + `status ∈ {result, no-finding, truncation, limitation}` + `callback` (aria-work-v1) |
| exit 0 | every bound result (engine rejections are bound `limitation`s) |
| exit 2 | unbound protocol error; stdout carries `{"status":"failure","error":…}` |
| budget | largest verticals dropped first, deterministically; `droppedVerticals` reported |

One executable, one capability, 560 identities named inside the request.
Container: `Dockerfile` target `work` (scratch, static, non-root) ·
`compose.telemetry.yaml`. Registry descriptor: `work --dispatch`.
