# Completion plan — one JSON telemetry operator, 560 identities

**Status:** `aria-json-telemetry` **0.2.1** on `5of5/json-telemetry` `v3.0.0`.
Harness lane, hosted shell, scratch container, and workspace-versioned 560
operator crates ship in this cut. Measured notes: [OPTIMIZATION.md](OPTIMIZATION.md).
**Catalog:** [Binary Repository v1](https://docs.google.com/spreadsheets/d/1GkFBE1_ZFclDma3DznJV_ONKXNEmNVx2eKbPl9Vf8OI/edit?gid=1130561225#gid=1130561225) (`01_BINARY_CATALOG`). Local copy: `TRACN Binary Repository v1 (1).xlsx` (not a crate).
**Not `exec/` truth.** Product authority stays with tracn-api. AriA stays telemetry.
**Paired:** [WORKER.md](WORKER.md) · [README.md](README.md) · [DUMP_ANALYSIS.md](DUMP_ANALYSIS.md) · [`plan-3.md`](../../plan-3.md) (surgical + Obsidian dump).

Φ stays five actions. Inv1–4 stay. Readout stays outside Φ. This file does not add a sixth action.

---

## Synopsis (this is clear)

The large schema is **already listed**. Eighteen family operators, nine host tools, one transformer, 180 residuals, 327 deep tags, **25 sealed map mixers** (`BIN.REF.*`, sheet 05): **560 named binaries** on one closed JSON spine (sheet 09). The mixers remix already-tagged telemetry into one of the 25 registry maps. They do not invent a 26th type.

What remains is not a bigger catalog. It is a **contained kit**: modular, cheap, easy to extend, hard to bias.

One method (sheet 09): quickly sort and return **only** that binary’s operator type as a structured JSON Aria transformation.

Three honest outcomes, always:

| Outcome | Meaning | `coverage_state` |
|---|---|---|
| **Structure** | Label, tag, and cluster what the payload already contains, for this binary’s purpose | `proposal` |
| **Identify** | This row *is* that binary (a Person, a WORKS_AT, a PERSON_FOUNDER, …) | `proposal` |
| **Forget** | It cannot be found or known as this binary. Empty `nodes[]` plus a reason. Not a guess. | `no-finding` |

**Forget is not delete.** The host payload always returns in `source` (and in optional `telemetry`). Prune is a **view**: the projector keeps the declared tag neighborhood and drops other *kinds from this envelope* (Voss / IAC A1). The bytes the host sent are still there.

The system has **no bias**. It does not learn. It does not influence. It does not judge. It refines the data structure it is given. It maxes on correctly clustering to **what that tag is for**. AriA may prune as telemetry *after* each independent return (B2). It does not fill one binary from another’s score.

Genius is not more specification. Genius is the **faster modular method**: one ingest, one optional Φ, 535 thin identities, graphical JSON out.

**More code is OK.** PCVC keeps a folder per feed (`pcvc/feeds/a16z`, `accel`, …) so each feed can be tweaked without jamming 284 feeds into one file. This repo does the same for binaries: each `BIN.*` is its own crate and `src` program, even when the files look like repeats. Exact compilation and team knowledge beat a tiny shared target. Tweaks later still ride **one JSON telemetry base**. All crates share **one gateway** — `work` — which workers pass any required type of work. The gateway can expand. The 535 crates do not merge.

---

## 1. Why the most modular way

| Temptation | Why it fails the workbook |
|---|---|
| One mega-crate that “knows the market” | B2: cross-binary score leakage. PEOPLE would contaminate COMPANY. |
| 535 copies of Φ | B1: shared *calculation identity* is required; shared *mutable state* is forbidden. Copying the engine is waste and drift. |
| LLM / Mode 3 as the operator | B7: AriA is never JUDGE. Uncast tokens return `limitation: uncast_token` (B11), not a guessed person. |
| Over-specify every tag as a TLA+ action | 00b step 15 / Voss: a spreadsheet row is not `Next`. Mode 4 is still Observe → Orient → Decide → Act → Review. |
| Treat 535 GitHub forks as the first deliverable | Independence is a crate + `content_hash` + declared types. In-tree packages already isolate. Forks can wait. |

Modular form that **does** match the sheet:

```text
one transformer (BIN.ARIA / json-telemetry)
    │  shared spine: aria-telemetry-query-v1
    ▼
535 identities (spec.json + 3-line main)
    │  each returns ONLY its operator type
    ▼
workers / Neo4j templates / family aggregators
```

**Ease of enhancement:** a new Neo4j label, rel, property, or tag already on an eligible projection or a sealed plan becomes one new row + one new crate (00b step 12). Append. Do not redesign. Do not invent a map type to mint a binary (B10).

Five logic crates. 535 identity crates. Repeats are fine. If a change needs more than a spec row + projector, it does not belong in an operator crate.

| Crate | Role |
|---|---|
| `aria-engine-core` | Φ. Do not touch. |
| `aria-engine-backends` | ingest, transform, IPO |
| `aria-operator` | JSON telemetry base |
| `aria-json-telemetry --bin work` | **Nervous-system JSON-CLI.** `work --commands` is the hosted list. `{work\|ops, in}` is the API. One Φ, N crates. Expand here. |
| `crates/operators/<pkg>` | Separate `src` program per binary. Tweak in place. |

---

## 2. How this merges into workers and Neo4j

### Workers (Spawning S6)

A spawn is the Coordinator creating **one worker bound to one requirement, one allowed host capability**. That capability is a catalog row (`01`, `11`, or `14`).

```text
sealed Observation Plan
  requirementId + resultDefinitionRef + subjectIds     (01 HOW IT WORKS)
        │
        ▼
work --binary BIN.PEOPLE --in payload.json            (aria-json-telemetry gateway)
        │  still valid: cargo run -p aria-telemetry-people
        ▼
closed operator JSON   ← vertical (cheap)
  telemetry?           ← optional spine (sheet 09: not the API contract)
```

The worker does not pick the next binary. Control returns to the Judge (S3). Unknown fields fail (B8). Empty + reason is success when evidence is insufficient.

### Neo4j (B3, B6, 00b step 9)

The binary grammar **is** the projection grammar (00b step 1):

| Residual class | Neo4j object | Pass (availability, never Trust) |
|---|---|---|
| NODE | label | `MATCH (n:Kind) WHERE id in subject_ids` |
| REL | typed relationship | `MATCH (a)-[r:TYPE]->(b)` |
| TAG | organizer / `TAGGED_AS` | neighborhood of a declared anchor tag |
| PROP | property key | field read into JSON; does not mint accepted facts |

**Observe first (B6, 01 HOW IT WORKS):** Coordinator reads eligible Neo4j Aura + `existingCoverage`. If required `anchor_tags` are present, `neo4j-aura` may return a projection. The binary crate maps that projection into the closed envelope and stamps `content_hash`. `neo4j_hit` is availability only. It is never copied into Trust (R2).

AriA does not write Neo4j. AriA does not run freeform Cypher. Templates live with the host (`pcvc_neo4j_aura`). This kit reports `neo4j_hit` and shapes JSON.

HOST binaries (feed, obscura, neo4j-aura, …) are **not** research-operator binaries (B6). They must not run Φ as their job.

---

## 3. The large schema is complete (sheets 0–14)

Workbook v1.0 · 2026-09-02. Initial reduced set. Not an exhaustive product contract (B9).

| Sheet | gid / role | What it defines |
|---|---|---|
| `00_RULES` | 326862220 | B0–B11. No-bias. Closed envelope. Observe-first. AriA never Judge. |
| `00b_RESIDUAL_PLAN` | | Family = aggregator. Residual = independent telemetry node. Growth by append. |
| `00c_TYPECAST_SEMANTICS` | | Incoming data is determined as a closed tag. Uncast → `limitation: uncast_token`. |
| **`01_BINARY_CATALOG`** | **1130561225** | One row = one operator. Families + host + `BIN.ARIA`. HOW IT WORKS. |
| `02_ANCHOR_TAGS` | | Tag universe. Structural organizers of Neo4j, not entity tables. |
| `03_NODE_REL_TYPES` | | 24 node kinds, 46 rel kinds. Residual NODE/REL source. |
| `04_PROPERTY_DESCRIPTORS` | | 50 top properties. Residual PROP source. Not Trust. |
| `05_MAP_COVERAGE_25` | | Which family binaries serve which of the 25 sealed maps. |
| `06_ATLAS_57` | | IAC renderings. Not automatic `map_type_registry` rows. |
| `07_CRATE_TELEMETRY` | | Crate name, fork, `writes: telemetry only` for AriA. |
| `08_VERIFY_MATRIX` | | SAFE iff every gate T. `document-extract` is F. |
| `09_JSON_OPERATOR_SHAPE` | | **The single method.** Required fields. `telemetry` optional. |
| `10_PRIOR_SHEET_TRACE` | | Nothing discarded from the stub. Virality is not an operator. Fork root: json-telemetry. |
| `11_RESIDUAL_BINARIES` | | 180 NODE/REL/TAG/PROP. `crate_status` listed T, working later. |
| `12_SEMANTIC_TAXONOMY` | | Type-cast blocks (person, persona, company, industry, ecosystem, category). |
| `13_MAP_LANGUAGE_25` | | 25 maps’ own words (`LANG_*`). Hardcoded tokens, not new map types. |
| `14_DEEP_TAG_BINARIES` | | 327 type-cast TAG crates. Same grammar as residual TAG. |

**Shared calculation identity (00_RULES):** every operator emits the same Aria spine: `binary_id`, `operator`, `plan_hash`, `requirement_id`, `subject_ids`, `resultDefinitionRef`, `anchor_tags[]`, `neo4j_hit`, `nodes[]`, `relationships[]`, `properties{}`, `verify`, `crate`, `schema_version`, `content_hash`. AriA **consumes** that spine. It does not author Trust.

Inventory (formulas, do not hardcode): 18 operators + 9 host + 1 transform + 180 residuals + 327 deep tags = **535 listed**. Grand total on `00` without deep tags = 208.

The kit is listed. The remainder is **modularize and optimize** so the whole set cannot break host JSON telemetry rules.

---

## 4. Input / process / output (efficient graphical JSON)

```text
INPUT     host JSON (notes, facts, tags, graph, sheet). Nothing dropped.
PROCESS   ingest once → optional Φ once → project per BIN.*
OUTPUT    vertical for that binary  +  full source always
          prune/tag/cluster only in the view
          no-finding if this binary cannot know it
```

| Stage | Optimize for | Rule |
|---|---|---|
| **Input** | Lossless | `source` equals parsed payload. `source_sha256` over exact bytes. Unknown host keys live in `records.properties`. |
| **Process** | Once | One G₀. One Φ if Match is needed. 535 projectors, not 535 engines. Small 𝒮 in tests. |
| **Output** | Vertical | Coordinator reads declared types only. Embeddings off unless asked (G10). `telemetry` optional (sheet 09). |

AriA prune (B2) happens **after** independent return. It is graphical telemetry, not a second Judge.

---

## 5. Locks (the set must not break host JSON telemetry)

| Lock | Meaning |
|---|---|
| B0 | Envelope contains only this operator’s types. No Views. |
| B1 | New crate per working binary. No shared mutable calc. Same spine. |
| B2 | No borrowed scores. Independent `content_hash`. |
| B3 | Neo4j pass = availability. Never Trust. |
| B6 | Observe-first. Host tools are not research operators. |
| B7 | Internal transform only. Never Judge. Never invent `resultDefinitionRef`. |
| B8 | Unknown fields fail. Guessing is F. `no-finding` + reason is success. |
| B10 | Family aggregates residuals. Does not compute them. |
| B11 | Type-cast tags. Uncast token → limitation. Not a new map type. |
| Spawning S6 | Worker uses one capability already on the plan. |
| Lossless | Host data always returns. Prune ≠ delete. |

`BIN.DOC_EXTRACT` stays VERIFY=F until Product accepts a document class.

---

## 6. Done (do not rebuild)

| Piece | Proof |
|---|---|
| Shared transform | `telemetry::transform` / `aria node` |
| 535 identity crates | `crates/operators/*`; workspace build 3m19s |
| Closed operator JSON | `OperatorEnvelope` |
| Dispatch | `endpoint_by_binary_id` |
| Tiny efficiency | PEOPLE vertical **96 B** vs nested telemetry **2616 B**; embeddings omitted; 8 ms / 3 operators |
| Σ=5, no decoder in core, no Trust field | G2, decoder script, envelope test |

A worker can already: JSON in → one crate → vertical + (today) full telemetry out.

---

## 7. Remainder (ranked) — optimize the kit, not the catalog

### P1 — Default wire is too fat

Sheet 09: `telemetry` is **optional**, not the API contract. Coordinator needs the vertical.

**Done (M0 + production callback):** default omit telemetry; `--telemetry` opt-in. `execute_work` / `work` CLI return **working verticals only** (`asked` vs `ops`). Empty no-finding is absence, not a skeleton. Dump still scores 535 internally and writes `{case}.callback.json` as the PCVC/Neo4j export.

### P2 — Envelope vs sheet 09

**Missing:** `evidence` on proposal; tracked operator JSON Schema; output unknown-keys fail; `neo4j_hit` still hardcoded false (honest until the host passes a hit).

### P3 — Type-cast (00c / B11) does not run

TAG crates filter tags already on the record. Incoming titles, industries, map phrases are not determined. Most of 392 TAG identities will `no-finding` on real notes until M3.

Uncast → `limitation: uncast_token`. Never invent a Person.

### P4 — Families recompute Φ (B10)

PEOPLE should **request** `NODE.PERSON` + `REL.WORKS_AT` + … and merge verticals. One process of the payload, not 535 Φ.

### P5 — HOST crates still run Φ

**Closed P3-1 (`output_260902_2233`).** Nine host rows return `limitation` + empty vertical, no Φ. TRANSFORM (`BIN.ARIA`) still runs it.

### P6 — 535 compile; 535 are not tested as operators

One catalog loop. Not 535 test files. B0, B8, lossless source, no Trust, vertical ⊆ declared kinds.

### P7 — Large-payload efficiency unproven

3-node gate only. G10 backend WIP is uncommitted on json-telemetry. 1k-row vertical must grow with declared types, not with d × all columns.

### P8 — PCVC spawn table

This repo ships `dispatch.json`. The Judge stays in `pcvc`.

### P9 — Public README still describes the engine

Lead with the worker method. Link this file.

---

## 8. Modules (one at a time, easy to enhance)

Each module is a small PR. Acceptance is a command and, where it matters, a byte or millisecond number.

### M0 — Wire efficiency (P1)

`RunOpts.include_telemetry` default false. CLI `--telemetry`.

**Accept:** default PEOPLE envelope < 768 B on the mixed fixture (hashes dominate; vertical 96 B). `--telemetry` restores the spine.

### M1 — Envelope = sheet 09 (P2)

`evidence`; `schemas/aria-operator-envelope-v1.json`; validate before write.

**Accept:** CI validates a PEOPLE fixture. `evidence.content_hash` = vertical hash.

### M2 — Catalog matrix (P6)

`tests/matrix.rs` loops 535 specs × {empty, mixed}. Prefer ingest-only / `steps=0` when Match is unused.

**Accept:** 535/535; zero guesses; ≤ 60 s debug.

### M3 — Type-cast (P3)

Deterministic projection from fields already in the payload (title, ntype, notes, properties). No LLM. Tags land on `records.properties.tags` in the **operator view**, not in Φ.

**Accept:** `{label:"CTO"}` into the matching `BIN.TAG.PERSON_*` is a tag hit or honest no-finding/uncast. No new Person node.

### M4 — Family aggregator (P4)

Family spec lists residual ids. One ingest/Φ; merge residual verticals.

**Accept:** `BIN.PEOPLE` = union of its residuals on the same payload. One Φ (or zero).

### M5 — Host out of Φ (P5)

HOST: no transform. TRANSFORM (`BIN.ARIA`) still runs it.

**Accept:** `BIN.OBSCURA` ≪ `BIN.ARIA` elapsed. No telemetry unless asked.

### M6 — Scale (P7)

Land or revert uncommitted G10 diffs. 1k-row fixture.

**Accept:** PEOPLE default envelope O(Person rows). No embeddings on the default wire.

### M7 — `dispatch.json` (P8)

Generated from the catalog: `binary_id`, `operator`, `package`, `resultDefinitionRef`, `layer`, `verify`.

**Accept:** 535 rows. PCVC can spawn without compiling Rust.

### M8 — Docs (P9)

json-telemetry README: worker lead. Wave A (`00b` step 14) as `WAVE_A.md`.

### M9 — CI

PR: `cargo test -p aria-operator`. Nightly: M2 + M6. `default-members` stay engine-sized.

---

## 9. Test strategy (all crates, max efficiency)

**Project 535 times. Run Φ as little as possible.**

Shared fixtures: empty graph, mixed Person/Company, untyped notes (00c), typed residual graph, existing tabular sheet.

For every spec: valid envelope, `binary_id` match, kinds ⊆ declared, no Trust keys, empty ⇒ `no-finding` + reason, `source` still holds the host payload.

| Gate | Now | After M0 |
|---|---|---|
| PEOPLE vertical, 3-node mix | 96 B | ≤ 96 B |
| PEOPLE default envelope | 3214 B (telem on) | ≤ 768 B (telem off; hashes dominate) |
| Three operators, 3-node | 8 ms | ≤ 8 ms |
| 535-spec matrix | not run | ≤ 60 s debug |
| 1k-row PEOPLE default | not run | record; fail if embeddings appear |

Do not Φ-loop 535 × 256-mode in CI.

---

## 10. DAG

```text
M0 wire efficiency ─┬─► M1 envelope/schema ─► M2 catalog matrix
                    └─► M5 host out of Φ
M3 type-cast          ─► M4 family aggregator
M6 large payload      (parallel; backend-WIP decision)
M7 dispatch.json      (parallel; generate.py)
M8 docs               after M0+M1
M9 CI                 after M2
```

**First session: M0.** Small. Matches sheet 09. Makes every later test cheaper.

---

## 11. Execute (today)

```bash
# gateway (preferred for workers)
printf '%s' '{"nodes":[{"id":1,"type":"Person","label":"Ada"}]}' \
  | cargo run -q --bin work -- --binary BIN.PEOPLE --seed 1 --steps 8

# still valid: one crate, one program
cargo run -q -p aria-telemetry-people -- --seed 1 --steps 8 --in payload.json

cargo test -p aria-operator
cargo run -q --bin work -- --list | wc -l   # 535
```

`--telemetry` only when the Supervisor asked for the spine.

---

## 12. Open questions (do not silently resolve)

1. Default telemetry **off** — this plan says yes (sheet 09 optional). Confirm no host already requires always-on telemetry from `f895e4b`.
2. Family path: one ingest and no Φ when Match is unused, vs one transform. Prefer ingest-only for pure identify/filter.
3. Uncommitted `aria-backends` G10/compaction: land on json-telemetry or revert before M6.
4. PCVC owns the Judge; this repo owns `dispatch.json`.

Until (1) is confirmed, M0 must keep `--telemetry` as a bit-identical restore of today’s envelope.
