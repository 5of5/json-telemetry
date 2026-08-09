# Aria — Validation & evidence

**Refuses Spec enlargement.** Primary safety = Inv1–Inv4 only.

Consolidates TLC and OPGROK validation receipts (v1–v3).

## 1. TLA+ / TLC (machine Spec)

| Item | Value |
|------|--------|
| Modules | `spec/Aria.tla`, `AriaMC.tla`, `AriaInstance.tla` + `.cfg` |
| Check | `INVARIANT Safety` (Inv1 ∧ Inv2 ∧ Inv3 ∧ Inv4) |
| Result (2026-08-09) | **No error** — 36049 generated, 2616 distinct, depth 10 |
| Bound | `StateConstraint`: \(t \le 3\) (MC only; Spec has \(t \in \mathbb{N}\)) |

```bash
cd spec
java -XX:+UseParallelGC -cp /path/to/tla2tools.jar tlc2.TLC \
  -config AriaInstance.cfg AriaInstance
```

See [spec/RUN.md](../spec/RUN.md). TLC does **not** check JEPA \(d \to 0\), optical \(O(1)\) (**𝐋1**), or asymptotics.

## 2. OPGROK validation lineage (external monorepo)

Root: a sibling checkout, not part of this repository (referred to below as `$OPGROK_HOME`)  
Binaries: `core/binaries/aria-math-v{1,2,3}/`

| Version | Mode | Key result | Spec impact |
|---------|------|------------|-------------|
| **v1** | combinatorial + TLC | OPMD preferred; \(\varepsilon=1\); idle identity; score 1150 | **𝐂4**, operating defaults |
| **v2** | live Grok 4.5 | `dry_run=false`; 53414 tokens; win PASS | **𝐂5**, **𝐂6**, merge policy, \(\mathrm{score}_r\) |
| **v3** | extensive + live HIGH | 8000 configs; 1339 winning; 411k tokens | **𝐂7**, **𝐂8**, Inv5–11 candidates, traces |

Live v3 harness JSON `win=FAIL` = node contract parse gate only; API was live with high reasoning.

## 3. Receipt files (`docs/evidence/`)

| File | Meaning |
|------|---------|
| `evidence/v1_selected_best.json` | v1 crowned config |
| `evidence/v2_live_receipt.json` | v2 live run metadata |
| `evidence/v3_live_receipt.json` | v3 live run metadata |
| `evidence/v3_greatest_extensive.json` | v3 extensive greatest config |

Raw node dumps stay in OPGROK `aria-math-v*/artifacts/` (not duplicated here).

## 4. Continuous annex (Level 0–1)

| Item | Value |
|------|--------|
| Path | [CONTINUOUS_REFINEMENT.md](CONTINUOUS_REFINEMENT.md) |
| Spec impact | Documentation only; no new named actions; Inv1–4 meanings unchanged |
| TLA | Semantics unchanged; TLC Safety green on finite instance (§1) |

## 5. Fidelity rules

1. No fifth named action.  
2. No Spec enlargement beyond documented admissible behaviors.  
3. Inv1–4 meanings fixed in [SAFETY.md](SAFETY.md).  
4. `prevRes` is auxiliary history for Inv2 only ([FORMAL_SPEC.md](FORMAL_SPEC.md)).  
5. Edit `Aria.tla` only with matching anchors in `docs/`.

## 6. Reproduce OPGROK (optional — external harness, not required to build Aria)

```bash
export OPGROK_HOME=/path/to/opgrok   # sibling checkout, not part of this repository

"$OPGROK_HOME/core/binaries/aria-math-v1/bin/opgrok-aria-math-v1" --test
OPGROK_REQUIRE_LIVE=1 "$OPGROK_HOME/core/binaries/aria-math-v2/bin/opgrok-aria-math-v2"
OPGROK_REQUIRE_LIVE=1 OPGROK_REASONING_EFFORT=high OPGROK_MODEL=grok-4.5 \
  "$OPGROK_HOME/core/binaries/aria-math-v3/bin/opgrok-aria-math-v3"
```
