# Optimization notes — aria-json-telemetry 0.2.2

Measured facts for the production node. Φ is five actions. Inv1–4 hold.
Readout stays outside Φ. The node is **stateless**: request bytes in,
callback bytes out. Neo4j is memory.

## What 0.2.2 ships

| Surface | Contract |
|---|---|
| Library | `run_binary` / `run_many` / `execute_work` — one ingest, N independent projectors |
| CLI / API | `work` — `--binary`, `--json`, `--commands`, `--harness`, `--dispatch`, `--serve` |
| Callback | `aria-work-v1`: `{schema, phi_once, asked, ops, organize, results[]}` — **working verticals only** plus the slop report |
| Harness | `pcvc-aria-telemetry-request/result-v1`, capability `aria.telemetry.project`, stderr empty, ≤ 64 KiB |
| Container | `Dockerfile` target `work`: scratch, static MUSL (asserted with `ldd`), UID 65534. Default CMD is `--serve`. PCVC stdin lane is `docker run -i … --harness`. Measured image **2.04 MB** (`aria-work:0.2.2`) |
| Hosted shell | fixed pool 4× cores · bounded queue 1024 · `503 Retry-After` past the queue · 10 s socket deadlines · static routes cached · zero shared mutable state |

560 catalog identities (535 research/host + 25 `BIN.REF.*` mixers). Operators
are workspace `0.2.1` and `publish = false`. The published crate *is* all 560
via `work --commands` / `run_many`.

Surgery S1–S8 is projector-side (not 560 src edits): family TAG requires the
tag; HOST empty limitation no Φ; VERIFY=F empty vertical; 00c type-cast;
sheet first-sort; COMPANY control; empty `content_hash` sharing is correct;
REL/PROP dark is not a bug. E4 interned catalog lookups. E5 skips family
TAG / DEEP_TAG when the graph has no tags. E9 `catalog/dispatch.json` (560
rows). E10 `BIN.PEOPLE` = union of PEOPLE residuals / DEEP_TAGs (one ingest;
REL residuals not unioned, so Company does not leak).

## Projector cost (how the node stays cheap)

1. **One `GraphIndex` pass, O(N+E).** Kind, kind-like, tag (explicit ∪ 00c
   cast), relationship, first-property, and id→idx are built once. Every
   projector is a lookup, not a rescan.
2. **00c type-cast is an n-gram lexicon** over 327 closed-vocabulary rows
   (PERSON_TYPE 62, COMPANY_TYPE 40, INDUSTRY_TYPE 40, CATEGORY_TAG 28,
   ECOSYSTEM_TAG 28, MAP_LANGUAGE 119, PERSONA_ARCHETYPE 10). Whole-word /
   phrase only. No LLM. Unlisted tokens become `uncast_token` limitations
   on ENTITY envelopes, never new nodes.
3. **Skip-if-empty wire.** `ENVELOPE_KEYS` is one serde order for all 560.
   Empty members omit. Production callback drops skeletons (`asked` vs `ops`
   is the audit). Absence is not bias.
4. **HOST stays out of Φ.** Empty HOST is a limitation, not a graph.

Dump referee (T1/T2, `scripts/dump_referee.py`): **36/36 files byte-identical**.
Identify timings vs the pre-index dump:

| Case | Before | After | Factor |
|---|---|---|---|
| stress (414 nodes) | 1226 ms | 297 ms | 4.1× |
| limit_huge | 14505 ms | 3191 ms | 4.5× |
| tags_storm | 112 ms | 23 ms | 4.9× |

## Virality (measured, not theoretical)

One callback is reusable without re-asking Neo4j.

| Gate | Number | Meaning |
|---|---|---|
| `K_mix` | 0.71 | 25 mixers lighting / 35 working verticals on the mixed dump |
| `K_reuse` | 35 | binaries that light on one payload |
| depth-2 | = depth-1 | re-feeding the callback does not invent kinds (closed grammar) |
| fleet | 64 workers, sequential vs `std::thread::scope` | **byte-identical** callbacks; 25 → 89 ops/s (2.6 s → 0.7 s wall) |
| hosted shell | 32 clients × 8 `/harness` calls over TCP (`tests/serve_load.rs`) | **1 distinct body**, 0 errors, 0 shed, ~950 ops/s (debug, 12 cores) |

Per-request constants removed on this path: `spec_by_id` is a hash lookup
(was a 560-row scan per op), `/dispatch` hashes the executable once per
process (was a file read + sha256 per call).

Dump `output_260902_2317` (P3-3): Trust **0**, garbage Person **0**, HOST **0**,
missing `content_hash` **0**, mixed role-tag FP **0**, semantic **90**, quality
**95**, invariants **100**, completeness **100**.

Mixed production callback: **35 envelopes / 30 202 B** vs 395 725 B full wire.

## How to report a synthesis

A synthesis is a **callback**, not a narrative.

1. Bind the Observation Plan (`planHash`, `requirementId`, `fencingToken`).
2. Send the original anchor as `payload` with `ops` (or `"*"`).
3. Keep from each working envelope: `binary_id`, `coverage_state`, `nodes`,
   `relationships`, `properties`, `content_hash`, `graph`.
4. Re-feed that callback into `BIN.REF.*` for a map view. Source bytes never
   change.
5. Quote dump numbers (git SHA, catalog hash, payload hash, Trust=0). Do not
   relabel Observation→Company inside Φ.

Container health is `work --commands` (catalog load), not a shell.

## What this cut does not do

- No sixth Φ action. No Trust writes. No decoder in Φ.
- Does not publish the 560 operator crates.
- Does not bump the workspace to 0.3.0.
- Does not republish `aria-engine-*` (already 0.2.1 on crates.io).
- Does not mix in-progress backends G10 work into this package.
