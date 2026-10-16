# mode4/json-telemetry — Entity.field Selection Layer

## Goal

Provide a closed, selectable catalog of typed JSON operators so PCVC Mode 4 workers return only the data that matches the requested requirement, instead of the full set of feeds a worker touched.

## Exact gap this fills in PCVC Mode 4

Current path (observed in graph_exports.pt style behavior):

1. Requirement arrives (e.g. `company.acme.revenue` or `entity-intelligence.organization.001.legal-name-and-legal-form`).
2. Worker fans out across multiple feeds that might contain related data.
3. Dispatch response contains every feed the worker worked on, including unrelated content.
4. TRACN API must filter the junk.

Mode 4 is currently registry-only (one Cypher template + generated registry from feed declarations). There is no structured selection step that prunes to the exact definition(s) required.

## Recommended selection unit

**Entity.field (definition ID + L0 identity_key), not feed.**

- Resolve entity to stable `identity_key` (domain / website / LinkedIn / LEI only for identity).
- Map requirement to exact definition ID(s) from the 240-field catalog.
- Dispatch only the operators that materialize those definitions for that key.
- Return only the matching nodes / relations / properties.
- Include related entities only when the definition is a relationship type and the requirement explicitly needs it.

Feeds remain the underlying source. The selection and return surface is no longer feed-wise.

## Binary / operator approach

- Do not alter the existing ~500–560 operator identities solely to reduce count.
- Keep them as reusable primitives.
- Add thin, definition-scoped selection (or composable operators) so a requirement resolves to a precise subset.
- Goal: correct data only, not a full feed dump. Reduce count only if a smaller set of more generic, composable operators still covers the graph cleanly.

## Copy target in PCVC

This folder is intended to be copied or linked into `5of5/pcvc` under `mode4/json-telemetry/` so the Coordinator / Worker can use the selection before dispatching the native binaries.

## Status

Design stub. Implementation of the selector and operator mapping follows.
