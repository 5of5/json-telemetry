# Dump analysis — all 535 binaries (local)

**Ran:** 2026-09-02 · `cargo run -p aria-json-telemetry --example dump -- --obsidian "<vault>"`
**Latest scored dump:** `dump/output_260902_2233/` · Obsidian `Aria-Telemetry/output_260902_2233`
**Catalog:** 535 · identify dump **steps=0** · scale **steps=8** · **N=16, d=16** (test 𝒮)
**Raw:** `dump/output_{ts}/*.json` (gitignored). This file is the scored review.
**Plan:** [`plan-3.md`](../../plan-3.md) — P3-0/1/2 landed; P3-3 type-cast next.

Zero Trust held: **0 Trust keys**. Garbage minted **0 Person** nodes. Forget ≠ delete. HOST leak **0**.

---

## 1. Scores (`output_260902_2233`)

| Axis | Before (`7e2e930`) | After surgery | Why |
|---|---|---|---|
| **Completeness (envelope return)** | 100 | **100** | 535/535 envelopes on every case. |
| **Completeness (semantic / 00c)** | 38 | **70** | Mixed lights **exactly 9** research binaries. HOST 0. Role-tag FP 0. Unstructured notes still forgotten until 00c. |
| **Quality (no guess, no Trust)** | 72 | **95** | Garbage → 0 Person. BUYER/COMPETITOR/PARTNER/SELLER/SYNDICATE dark on untagged mixed. 0/401 stress lures-on-node. |
| **Invariants** | 74 | **100** | content_hash present; no Trust; **HOST empty limitation, no Φ**; HASH_STAMP no longer truncates a leaked graph. |
| **Time to scale** | 94 | **94** | 526 ops / 6 ms (~11 µs/op after one Φ). Identify stress 535 ops / 490 ms on 414 nodes (ingest-bound, steps=0). |
| **Kit** | 76 | **88** | Projector surgery + compact wire. Type-cast (M3) and family aggregator (M4) still open. |

---

## 2. Case results

| Case | Payload | ms | no-finding | proposal | limitation | truncation | Envelope B |
|---|---|---|---|---|---|---|---|
| empty | 12 B `nodes:[]` | 3 | 525 | 1 (ARIA) | 9 HOST | 0 | 332 186 |
| garbage | 125 B notes noise | 4 | 525 | 1 (ARIA) | 9 HOST | 0 | 332 233 |
| mixed | 310 B Person×2 + Company + WORKS_AT | 5 | 517 | **9** | 9 HOST | 0 | 332 123 |
| stress | 88 492 B fabricated market map (414 nodes / 206 edges) | 490 | 187 | 338 | 9 HOST | 1 (declared limit) | 362 339 |
| two_cluster | 301 B observation graph | 5 | 525 | 1 (ARIA) | 9 HOST | 0 | 332 395 |
| company_typed | 164 B one Company | 3 | 522 | 4 | 9 HOST | 0 | 332 055 |
| company_notes | 60 B unstructured notes | 3 | 525 | 1 (ARIA) | 9 HOST | 0 | 332 233 |

**Limitation** is all 9 HOST (B6) including DOC_EXTRACT (VERIFY=F). Empty vertical. Correct.

**Truncation** on mixed/empty/garbage is gone. Stress has 1 declared `default_limit` after filter: `BIN.PEOPLE` (family cap, not HASH_STAMP leak).

**Empty/garbage proposal** is TRANSFORM (`BIN.ARIA`) only.

### Mixed proposals (exactly the 9 research binaries)

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

False positive (TAG family matching entity *type*, not tag): **none** after S1.

HOST leak (B6): **none** after S2. HASH_STAMP no longer truncates.

---

## 3. Invariants across systems

| Invariant | Dump | Held? |
|---|---|---|
| B0 closed operator | PEOPLE/COMPANY yes. HOST empty limitation. BUYER/COMPETITOR require role tag. | **Hold** |
| B2 independent hash | Empty no-findings share one empty-vertical hash. Identity via `binary_id`. | Evidence-OK |
| B6 Observe-first / HOST not research | HOST empty limitation, no Φ | **Hold** (PLAN M5) |
| B7 never Judge | 0 Trust keys; garbage did not invent Person | **Hold** |
| B8 no-finding + reason | 517 mixed / 525 empty with reason | **Hold** |
| B11 type-cast | Founder hit because payload already had `tags`. two_cluster / company_notes unlabeled. | **Fail until M3 / P3-3** |
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

Order (after `output_260902_2233`):

1. **M5 HOST out of Φ** — **done** (P3-1).  
2. **Surgical TAG family filters** — **done** (P3-2).  
3. **M3 type-cast (00c)** — lights 327 deep tags from notes/titles without guessing Person. **Next.**  
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
| S1 | BIN.BUYER, BIN.COMPETITOR, BIN.PARTNER, BIN.SELLER, BIN.SYNDICATE | **done** (`output_260902_2233`) | Family TAG fires on `properties.tags` minus `node_types`. Residual TAG.* still matches kind. |
| S2 | 9 HOST except BIN.ARIA | **done** | HOST → `limitation` + empty nodes, no Φ. TRANSFORM still pass_through. |
| S3 | BIN.HASH_STAMP | **done** (follows S2) | No leaked graph to truncate. |
| S4 | BIN.DOC_EXTRACT | **done** | VERIFY=F empty vertical; limitation text kept. |
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
cargo run -p aria-json-telemetry --example dump -- --obsidian "/Users/dylanckawalec/Documents/Obsidian Vault"
# dump/output_{YYMMDD_HHMM}/ + vault Aria-Telemetry/output_{ts}/
```

Garbage collection here means: unstructured input is **classified or forgotten**, never deleted, never Trusted.
