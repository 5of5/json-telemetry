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
cargo run -p aria-telemetry-people -- --in payload.json
            │
            ▼
closed operator JSON  (vertical: only PEOPLE types)
    telemetry: aria-telemetry-query-v1   ← shared spine under every binary
```

Unstructured JSON (notes, facts, tags, a typed graph, a spreadsheet) goes in.
A structured query comes back. The worker does not pick the next binary.

## Pointing

| Plan field | Dispatch |
|---|---|
| `BIN.PEOPLE` | `endpoint_by_binary_id` |
| operator `PEOPLE` | `endpoint_by_operator` |
| cargo package | `endpoint_by_package("aria-telemetry-people")` |

535 endpoints. Same envelope schema. Unique `binary_id` / declared types.

AriA is linked into every binary. AriA is not the Judge (Spawning §9).
