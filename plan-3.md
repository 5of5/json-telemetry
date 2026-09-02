# plan-3 — Surgical binaries, matrix multiplier, Obsidian dump

**Status:** P3-0/P3-1/P3-2 landed. Measured dump `output_260902_2233` (Obsidian `Aria-Telemetry/output_260902_2233`). Next: P3-3 type-cast (semantic 70 → 100).
**Catalog:** `TRACN Binary Repository v1 (1).xlsx` (sheets 00–14). Drive twin: gid `1130561225` (`01_BINARY_CATALOG`).
**Paired:** `crates/aria-operator/PLAN.md` (kit remainder) · `DUMP_ANALYSIS.md` (measured dump) · `WORKER.md`.
**Not `exec/` truth.** AriA tags and weighs. It does not Trust, rewrite, or judge.

Φ stays five actions. Host JSON is never deleted. Prune is a view.

---

## 0. What this plan is for

The kit exists: 535 crates, one JSON telemetry spine, one `work` gateway, one Φ × N projectors.

This plan is how we **surgically** make every operator *correct* (B0/B6/B11), how we **enhance all** of them without touching 535 `src` files by hand, how we **score readiness to 100%** with dump numbers we trust, and how we **verify on original notes** in Obsidian.

The operator’s only job: **determine the means to tag the data**. Not change it. Not manipulate it. The work command names the binary. The payload is the original anchor. The return is a found/likely display for that binary, or `no-finding`.

Example: `work --binary BIN.COMPANY` on a body of company-ish text. COMPANY (and its residuals) categorize **company** structure found in that body. PEOPLE does not borrow those scores. COMPETITOR may later *tag* the same company node. It does not rewrite COMPANY.

---

## 1. Contract (100% confidence only from dump numbers)

A measurement is trusted only if all of these are in the dump folder:

| Must record | Why |
|---|---|
| git SHA | code under test |
| catalog SHA256 of `operators.json` | 535 identities |
| workbook path + sheet count | xlsx is the grammar |
| payload SHA256 | original bytes |
| `output_{YYMMDD_HHMM}/` | this run, not “latest” |
| per-binary: `coverage_state`, node/rel counts, kinds, `content_hash`, envelope bytes | B0/B2/B8 |
| Φ ms and ops | scale |
| Trust-key scan = 0 | B7 |
| Person-from-garbage = 0 | no guess |

If a score cannot be recomputed from those files, it is not a score. It is a vibe. We do not ship vibes.

**Readiness / completion** (target 100 after this plan):

| Gate | Dump `7e2e930` (before surgery) | Dump `output_260902_2233` | 100% means |
|---|---|---|---|
| Envelope return | 535/535 | **535/535** | same |
| No Trust keys | 0 | **0** | same |
| Garbage ≠ Person | 0 | **0** | same |
| HOST leak | 9 hosts copy G | **0** | 0 hosts emit research nodes |
| TAG false-positive | BUYER/COMPETITOR/PARTNER/SELLER | **0** on mixed; 0/401 lure-on-node | 0 entity-type matches without tag |
| Type-cast (00c) | founder only if `tags` pre-set | still open (`company_notes` forgotten) | notes/titles/columns cast or `uncast_token` |
| Semantic hit on mixed Person+Company+WORKS_AT | 11 research + 9 leak + 4 false tag | **9/9** research, 0 leak, 0 false tag | all and only PEOPLE/COMPANY/NODE/REL.WORKS_AT/TAG.* that the payload actually supports |
| Scale | 526 ops / 6 ms | **526 / 6 ms**; identify dump steps=0 | still one Φ; production N=256 measured separately |

---

## 2. Matrix multiplier (the method)

```text
original anchor (JSON body, sheet rows×cols, or Obsidian note)
        │
        ▼
ingest once  →  G₀ + records + source (lossless)
        │
        ▼
optional one Φ
        │
        ▼
535 projectors  (work defines which BIN.*; default = all research)
        │
        ▼
collapse: which binaries found / tagged / forgot
        │
        ▼
dump/output_{ts}/  +  Obsidian copy for human review
```

This is the multiplier: **one entry × 535 independent weighs**. Spreadsheet-shaped data is first **sorted as rows and columns** (tabular ingest already exists). That first sort is a structural bias of tables, not a Trust bias. Then each binary calculates only its own neighborhood.

Anchors **collapse together** because they share the catalog grammar (xlsx), not because they share scores (B2). COMPANY does not inherit COMPETITOR. BUYER does not inherit PEOPLE’s completeness.

Worker JSON (any body):

```json
{
  "ops": ["BIN.COMPANY", "BIN.NODE.COMPANY", "BIN.TAG.COMPANY"],
  "in": { "...unstructured or rows..." }
}
```

or the full hosted list (`work --commands`) compiled by the gateway (`run_many`). Assume nothing beyond: tag, identify, or forget.

---

## 3. Obsidian verification loop

**Vault (this machine):** `/Users/dylanckawalec/Documents/Obsidian Vault`  
(empty of notes today except `.obsidian` — that is the landing vault, not Holy Primes.)

**Repo dump (gitignored):** `dump/output_{YYMMDD_HHMM}/`

**Connect:**

```text
dump/output_{YYMMDD_HHMM}/
    analysis.json
    empty.json  garbage.json  mixed.json  two_cluster.json
    obsidian_payload.json          ← notes ingested from the vault
    SCORE.md                       ← numbers from analysis.json only
        │
        ▼ copy (not Aria Trust write)
/Users/dylanckawalec/Documents/Obsidian Vault/Aria-Telemetry/output_{YYMMDD_HHMM}/
    SCORE.md
    found.md                       ← binaries that proposed
    forgot.md                      ← no-finding (original text still in source)
    original.md                    ← the note bytes used as payload
```

Rules:

- Aria does **not** write accepted workspace context. The copy into Obsidian is a **human verification folder**.
- Original note text is the payload `source`. Operators only add tags/kinds in the *view*.
- If the vault later holds company notes, the default example op set is COMPANY + NODE.COMPANY + TAG.COMPANY + residuals the sheet binds to COMPANY (01 parent).
- Re-run is a new `output_{ts}`. Never overwrite. Confidence requires the timestamp.

**Dump command (to implement in this plan, not a sixth Φ action):**

```bash
cargo run -p aria-json-telemetry --example dump -- dump/output_$(date +%y%m%d_%H%M)
# then copy that folder into the vault Aria-Telemetry/
```

Extend the example: `--obsidian "/Users/dylanckawalec/Documents/Obsidian Vault"` copies SCORE + original + found/forgot notes.

---

## 4. Surgical binaries (from dump + xlsx)

Do **not** hand-edit 535 `src/lib.rs`. Surgery is **spec row + projector**, regenerate only if the xlsx row changes.

### S1 — TAG family must require the tag (xlsx 01)

Sheet: COMPETITOR “Does not name a new entity. Tags an existing COMPANY… it does not rewrite COMPANY.” BUYER “Role-tag operator on PERSON / ACCOUNT.” Same for SELLER, PARTNER.

**Dump defect:** they matched Person/Company *type*, so BUYER lit both Persons without `BUYER_TAG`.

**Fix:** projector for `layer=TAG` or `class=TAG`: **only `tag_hits`**. Never `matches_kind` on Person/Company. `node_types_emitted` “Company (tagged)” means tagged company, not any company.

**Accept:** mixed Person+Company without role tags → BUYER/COMPETITOR/PARTNER/SELLER = `no-finding`. With `tags:["BUYER_TAG"]` on Ada → BUYER proposal on Ada only.

### S2 — HOST out of Φ (xlsx 01 HOST / B6)

Sheet: HASH_STAMP “Not an AriA operator.” Obscura/feed/neo4j are host toolkit.

**Dump defect:** 9 HOST crates pass_through the whole G. HASH_STAMP `default_limit=1` then truncates.

**Fix:** `pass_through` only `BIN.ARIA`. HOST named alone → `limitation` + empty vertical, **no Φ**. HOST in `run_many` skipped unless explicitly in `ops`.

**Accept:** mixed dump HOST node_count = 0. HASH_STAMP no truncation. Research bins unchanged.

### S3 — DOC_EXTRACT VERIFY=F (08)

**Fix:** `verify=false` ⇒ empty vertical, keep limitation text. No leaked Person/Company.

### S4 — Type-cast 00c (xlsx 00c / 12 / 14)

Incoming data is **determined as a closed tag**. Families stay families. Uncast → `limitation: uncast_token`.

**Dump defect:** two_cluster `type: observation` never becomes Company. Deep tags dark unless `tags` already present.

**Fix:** projector (not Φ) maps title/notes/column/ntype tokens → `TAG.*` using 00c blocks (PERSON_TYPE, COMPANY_TYPE, INDUSTRY_TYPE, …). No new nodes. No Trust.

**Accept:** a note “Stripe / payments infrastructure” into COMPANY + IND_FINTECH_INFRASTRUCTURE path yields company-kind or uncast_token, never a guessed Person. Founder tag from “founder” in notes is a **listed token**, not an LLM.

### S5 — Spreadsheet first sort (xlsx 04 + tabular ingest)

Row × column ingest already exists. First pass: treat columns as property/tag candidates, rows as observations. That is the “biased first sort” of sheets — structural, not Trust.

**Fix:** dump a real sheet-shaped payload (backend `tabular_market_sheet.json`) as a fifth dump case. Each column name may type-cast; each binary still only emits its kinds.

**Accept:** PROP residuals light when the column exists; others `no-finding`. Source still has every cell.

### S6 — COMPANY example (control set)

Work defines the binary. Data is assumed to need **company** categorization when `BIN.COMPANY` (and bound residuals) are in `ops`.

```json
{"ops":["BIN.COMPANY","BIN.NODE.COMPANY","BIN.TAG.COMPANY"],
 "in": { "notes": ["Acme builds payments infrastructure in fintech"] }}
```

After S4: COMPANY/NODE.COMPANY/TAG.COMPANY may propose; PEOPLE stays no-finding unless a person token casts. Original note unchanged in `source`.

### S7 — content_hash identity

513 empty residuals sharing one hash is **correct** (same empty vertical). Optional later: `content_hash = H(binary_id ‖ vertical)` if a host wants per-id uniqueness. Not required for B2 (independence is no borrowed *scores*).

### S8 — REL/PROP dark is not a bug

No `FOUNDED` edge → `BIN.REL.FOUNDED` no-finding. No `headcount` property → PROP no-finding. Do not invent.

---

## 5. Enhance-all (no per-crate surgery)

These apply to every binary through the gateway:

| # | Enhancement | Ready when |
|---|---|---|
| E1 | `work --dump dump/output_{ts}` writes analysis + SCORE.md | dump example already; wire CLI flag |
| E2 | `--obsidian <vault>` copies output_{ts} into `Aria-Telemetry/` | vault path above |
| E3 | Compact empty JSON fields (skip empty rels/props) | byte drop on 535 empties |
| E4 | Intern catalog `&'static` ids | less alloc |
| E5 | Fast-reject TAG projectors if payload has no tags **and** 00c produced none | keep after S4 |
| E6 | `steps=0` ingest-only when ops are identify/filter | dump N=16 already Φ-light |
| E7 | Production dump at N=256 recorded separately | scale gate |
| E8 | Nightly dump; fail CI if Trust>0 or garbage Person>0 | 100% confidence floor |
| E9 | `dispatch.json` (PLAN M7) | PCVC spawn |
| E10 | Family aggregator (PLAN M4) after S1/S4 | PEOPLE = union of residuals |

---

## 6. Execution DAG (delivery)

```text
P3-0  dump CLI + Obsidian copy + SCORE.md from numbers only     ✓ 2233
P3-1  S2 HOST out of Φ + S3 DOC_EXTRACT empty                   ✓ HOST=0
P3-2  S1 TAG family tag_hits only                               ✓ mixed FP=0, lures 0/401
P3-3  S4 type-cast 00c + S5 sheet case in dump          ✓ dump 2317 semantic 90
P3-4  S6 COMPANY fixture + Obsidian original note               ✓ typed; notes wait on P3-3
P3-5  E3 compact + E6 steps=0 + production callback (working-or-nothing)  ✓
P3-6  E7 production 𝒮 dump + E8 nightly floor
P3-7  E9 dispatch.json + E10 family aggregator
```

**One workstream per session.** Re-run dump after each P3-n. Readiness score must **not fall** on Trust, garbage-Person, or envelope count. Semantic score must **rise** after P3-3.

---

## 7. Acceptance — standard readiness 100

A run is 100% ready when `dump/output_{ts}/SCORE.md` shows:

- catalog 535, envelopes 535  
- Trust hits 0  
- garbage Person 0  
- HOST research nodes 0  
- BUYER/COMPETITOR/PARTNER/SELLER false-positive 0 on untagged mixed  
- COMPANY example: COMPANY proposes; PEOPLE no-finding on company-only notes  
- type-cast: at least one 00c tag from notes **or** explicit `uncast_token` (never silent guess)  
- Obsidian folder `Aria-Telemetry/output_{ts}` contains original.md + found.md + forgot.md + SCORE.md  
- Φ once per payload; 526-op time recorded  

Until then, completeness of *envelopes* can be 100 while semantic completeness is not. Say both numbers. Never collapse them.

---

## 8. What we will not do

- Relabel Observation → Company inside Φ  
- Merge 535 crates  
- Write Trust, Use, or Goal from a dump  
- Overwrite a previous `output_{ts}`  
- Use Holy Primes (iCloud) as the telemetry vault  
- Treat 513 shared empty hashes as a defect  
- Invent REL/PROP hits without those fields in the payload  

---

## 9. Landed (P3-0 + P3-1 + P3-2) — dump `output_260902_2233`

Projector surgery (not 535 src edits):

- **Enhance-all:** HOST never pass_through / never Φ; VERIFY=F empty vertical; compact empty envelope fields; identify dump `steps=0`.
- **Fine-tune five family TAG crates:** BUYER / COMPETITOR / PARTNER / SELLER / SYNDICATE fire on `properties.tags` minus `node_types`. Residual TAG.* still matches kind.

Measured (SCORE.md only):

| Axis | 2233 |
|---|---|
| Envelope / Trust / garbage-Person / HOST leak | 535 / 0 / 0 / **0** |
| Mixed research | **9/9** (PEOPLE, COMPANY, NODE.PERSON, NODE.COMPANY, REL.WORKS_AT, TAG.PERSON, TAG.COMPANY, TAG.PERSON_FOUNDER, ARIA) |
| Mixed role-tag FP | **0** |
| Stress grammar | 319/319 expected, lures-on-node **0/401** |
| COMPANY typed | COMPANY proposal, PEOPLE no-finding |
| COMPANY unstructured notes | both no-finding (P3-3) |
| Scale | 526 ops / 6 ms |
| Envelope bytes (mixed) | 332 123 (was 360 308) |
| Obsidian | `Aria-Telemetry/output_260902_2233` (SCORE, found, forgot, original, graph, entities, anchors) |

**Next session:** P3-3 type-cast 00c so unstructured notes (`company_notes`, `two_cluster` observations) either tag or return `uncast_token`. Do not relabel Observation → Company inside Φ. Re-dump; semantic must rise; Trust/garbage-Person/HOST/FP must stay 0.
