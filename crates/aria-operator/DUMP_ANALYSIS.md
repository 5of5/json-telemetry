# Dump analysis — all 535 binaries (local)

**Ran:** 2026-09-02 · `cargo run -p aria-json-telemetry --example dump -- dump`
**Catalog:** 535 · **Φ:** once per payload · **N=16, d=16, steps=8** (test 𝒮)
**Raw:** `dump/*.json` (gitignored). This file is the scored review.

Zero Trust held: **0 Trust keys**. Garbage minted **0 Person** nodes. Forget ≠ delete.

---

## 1. Scores

| Axis | Score | Why |
|---|---|---|
| **Completeness (envelope return)** | **100** | 535/535 envelopes on every case. |
| **Completeness (semantic / 00c)** | **38** | Mixed typed Person+Company+WORKS_AT lights **20** operators. 327 deep tags stay dark unless a tag is already on the record. two_cluster observations never become Company. |
| **Quality (no guess, no Trust)** | **72** | Garbage → 0 Person. Mixed → PEOPLE/COMPANY/NODE/REL.WORKS_AT/TAG.PERSON_FOUNDER correct. **False positives:** BUYER, COMPETITOR, PARTNER, SELLER treat any Person/Company as their tag. |
| **Invariants** | **74** | content_hash present; no Trust. **HOST pass-through leaks the full graph** (9 hosts). HASH_STAMP truncates at default_limit=1. Empty no-findings share one evidence hash (same empty vertical — expected). |
| **Time to scale** | **94** | 1 op ~0 ms · 100 ops 1 ms · **526 ops 6 ms** (~11 µs/op after one Φ). Envelope total ~360 kB dominated by 535 × ~670 B headers, not graph. |
| **Kit (first major work)** | **76** | Gateway + one Φ + lossless forget works. Type-cast, HOST isolation, family aggregator still open (PLAN M3–M5). |

---

## 2. Case results

| Case | Payload | Φ ms | no-finding | proposal | limitation | truncation | Envelope B |
|---|---|---|---|---|---|---|---|
| empty | 12 B `nodes:[]` | 9 | 525 | 9 | 1 | 0 | 359 530 |
| garbage | 125 B notes noise | 5 | 525 | 8 | 1 | 1 | 359 820 |
| mixed | 310 B Person×2 + Company + WORKS_AT | 6 | 513 | 20 | 1 | 1 | 360 308 |
| two_cluster | 301 B observation graph | 6 | 525 | 8 | 1 | 1 | 361 186 |

**Limitation** is always `BIN.DOC_EXTRACT` (VERIFY=F). Correct.

**Truncation** is always `BIN.HASH_STAMP` (HOST, default_limit=1). Pass-through then clipped.

**Empty/garbage proposals** are TRANSFORM + HOST pass-through of Observation nodes — not research hits.

### Mixed proposals (the useful 11 + the leaky 9)

Correct (research):

| Binary | Nodes | Rels | Note |
|---|---|---|---|
| BIN.PEOPLE | 2 Person | 0 | entity filter |
| BIN.COMPANY | 1 Company | 0 | entity filter |
| BIN.NODE.PERSON | 2 Person | 0 | residual |
| BIN.NODE.COMPANY | 1 Company | 0 | residual |
| BIN.REL.WORKS_AT | 3 | 2 | residual rel |
| BIN.TAG.PERSON | 2 Person | 0 | residual tag |
| BIN.TAG.COMPANY | 1 Company | 0 | residual tag |
| BIN.TAG.PERSON_FOUNDER | 1 Person | 0 | deep tag via `tags` on Ada |
| BIN.ARIA | 4 + 2 rels | | pass-through transform |

False positive (TAG family matching entity *type*, not tag):

- BIN.BUYER → both Persons  
- BIN.COMPETITOR / PARTNER / SELLER → Company  

HOST leak (B6): 8 hosts copy the whole mixed graph; DOC_EXTRACT copies it under limitation.

---

## 3. Invariants across systems

| Invariant | Dump | Held? |
|---|---|---|
| B0 closed operator | PEOPLE/COMPANY yes. HOST no. BUYER/COMPETITOR no. | Partial |
| B2 independent hash | Unique hashes **7 of 535** on mixed. 513 empty no-findings share one empty-vertical hash. | Evidence-OK, identity via `binary_id` |
| B6 Observe-first / HOST not research | HOST ran Φ via pass-through | **Fail** (PLAN M5) |
| B7 never Judge | 0 Trust keys; garbage did not invent Person | **Hold** |
| B8 no-finding + reason | 513–525 empty with reason | **Hold** |
| B11 type-cast | Founder hit only because payload already had `tags`. two_cluster unlabeled. | **Fail until M3** |
| Lossless forget | Envelope always returned; prune is view | **Hold** |
| Σ=5 / no decoder | unchanged | **Hold** |

---

## 4. Time to scale

| Ops | ms | µs/op |
|---|---|---|
| 1 | 0 | — |
| 10 | 0 | — |
| 100 | 1 | 10 |
| 526 | 6 | 11.4 |

Bottleneck is **one Φ**, not 535 projectors. Header bytes (~670 B × 535 ≈ 360 kB) dominate tiny graphs. At production N=256 the Φ term will dwarf projection until ingest/Φ is reused across requests (already true inside one `run_many`).

---

## 5. Dozens of optimizations (no binary identity broken)

Gateway / kit (safe for all 535):

1. Default telemetry off — **done**.  
2. One Φ, N projectors (`run_many`) — **done**.  
3. Skip `serde_json::to_value` of spine unless `--telemetry` — **done**.  
4. Compact envelope: omit empty `relationships`/`properties`/`limitations` (serde skip). Saves ~360 kB × empty fields.  
5. Hex hashes as raw 32 bytes in a binary sibling format; keep hex on JSON wire.  
6. `content_hash` over vertical only already — stop repeating 64-char hex in 513 identical empties by optional `content_hash_ref`.  
7. Don’t allocate `OperatorEnvelope` strings from spec on every empty: intern `binary_id`/`crate` from catalog `&'static`.  
8. `run_many` skip HOST unless named (B6) — behavior change for hosts only, not research bins.  
9. `steps=0` ingest-only path when Match unused (identify/filter).  
10. Reuse one `TelemetryEnvelope` across HTTP keep-alive (gateway process).  
11. Parallel project in `run_many` (rayon) — only after measuring; 11 µs/op may not pay.  
12. `--dump` on `work` writing `dump/` (example exists).  
13. `dispatch.json` (M7) so PCVC does not parse 535 crates.  
14. Feature `cli` already splits clap from the lib.  
15. Production 𝒮 (N=256) behind a profile; dump stays small 𝒮.  
16. Omit `anchor_tags` copy when empty.  
17. `schema`/`schema_version` as `&'static str` interned.  
18. Streaming JSON array of results instead of one 360 kB object.  
19. HTTP/2 or unix socket later — not in spawn spec; keep `work --json` first.  
20. Cache `commands_json()` (already OnceLock catalog).  
21. Don’t project TAG residuals that cannot hit without 00c when payload has no `tags` key (fast reject).  
22. Compact `no_finding_reason` to an enum code (`empty_types`) plus optional string.  
23. Land G10 embeddings-off / records compaction (uncommitted backends) for 1k-row sheets.  
24. Family aggregator (M4): PEOPLE = union of residuals, still one Φ.  
25. Batch `ops` already JSON-CLI — document as the remote/serverless call.  
26. Avoid cloning `OperatorSpec` in dump summarize (use catalog `&`).  
27. `allow_sub_spec_dims` never in production `work` default.  
28. Truncation only on declared `default_limit` after filter, not on HOST pass-through.  
29. Garbage fixture in CI (`--example dump` nightly).  
30. Score card regenerated into `DUMP_ANALYSIS.md` on nightly.

None of these rename or delete a `BIN.*`.

---

## 6. Next steps (after first major work)

First major work **is** complete: catalog, crates, gateway, JSON-CLI, publish `aria-json-telemetry 0.2.0`, dump score.

Order:

1. **M5 HOST out of Φ** — stops 9 leaks and HASH_STAMP truncation. Highest invariant ROI.  
2. **Surgical TAG family filters** — BUYER/COMPETITOR/PARTNER/SELLER (below).  
3. **M3 type-cast (00c)** — lights 327 deep tags from notes/titles without guessing Person.  
4. **M4 family aggregator** — PEOPLE requests residuals.  
5. **M1 evidence + operator JSON Schema**.  
6. **M7 dispatch.json**.  
7. **M6 1k-row** after backend G10 land/revert.  
8. **Nightly dump** in CI.

---

## 7. Surgical operations (cover every operator correctly)

Do **not** hand-edit 535 `src/lib.rs` files. Surgery is **spec + projector**, then regenerate if the spec row changes.

| # | Binaries | Defect | Surgery |
|---|---|---|---|
| S1 | BIN.BUYER, BIN.COMPETITOR, BIN.PARTNER, BIN.SELLER | TAG family matches entity *type* (Person/Company), not the tag | Projector: TAG/DEEP_TAG only `tag_hits`, never `matches_kind` on Person/Company. Spec `node_types` stay empty; `anchor_tags` do the work. |
| S2 | 9 HOST except we keep BIN.ARIA | pass_through copies G | `pass_through` only TRANSFORM. HOST → `limitation` + empty nodes, no Φ when run alone. |
| S3 | BIN.HASH_STAMP | default_limit=1 truncates pass-through | After S2, limit never applies to a leaked graph. Confirm sheet limit is for hash *outputs*, not Observation nodes. |
| S4 | BIN.DOC_EXTRACT | VERIFY=F still emits nodes | On `verify=false`, force empty vertical; keep limitation text. |
| S5 | BIN.TAG.PERSON / COMPANY vs BIN.PEOPLE / COMPANY | Duplicate verticals | Allowed (residual vs family). Later M4: family *aggregates* residuals rather than re-filter. No delete. |
| S6 | 327 DEEP_TAG | Dark without pre-tagged payload | M3 type-cast from `label`/`notes`/`ntype`/`properties`. Uncast → `limitation: uncast_token`. |
| S7 | two_cluster observations | Never become Company | Ingest `type: observation` is honest. Type-cast or host `type: Company`. Do not relabel inside Φ. |
| S8 | BIN.REL.* other than WORKS_AT | Dark on mixed | Need edges of that type in payload. Not a bug. |
| S9 | BIN.PROP.* | Dark | Need property keys on records. Not a bug. |
| S10 | 513 shared empty `content_hash` | Same empty vertical | Keep. Identity is `binary_id`. Optional later: hash `binary_id ‖ vertical`. |

No surgery that merges crates or adds a sixth action.

---

## 8. How to re-run

```bash
cargo run -p aria-json-telemetry --example dump -- dump
# dump/analysis.json  dump/{empty,garbage,mixed,two_cluster}.json
```

Garbage collection here means: unstructured input is **classified or forgotten**, never deleted, never Trusted.
