# Aria — Theories, Trajectories & Structure Lattice

**Source:** OPGROK `aria-math-v3` extensive offline search + a live high-reasoning-effort session (xAI Grok 4.5), external monorepo (see [VALIDATION.md](VALIDATION.md) §2)  
**LOCK:** named actions = {OpticalStep, Predict, Match, Diffuse} + Stutter only  
**Refuses Spec enlargement beyond documentation.**

Authoritative Inv1–4 remain ([SAFETY.md](SAFETY.md)):

| Inv | Meaning |
|-----|---------|
| **Inv1** | Optical energy \(\|\psi\|_2=\|\psi_0\|_2\) |
| **Inv2** | Predictive contractivity (Res ≤ prev + ε) |
| **Inv3** | Graph integrity (GraphOK) |
| **Inv4** | TypeOK |

---

## 1. Extensive search receipt (offline)

| Metric | Value |
|--------|------:|
| Configs evaluated | 8000 (sampled from full lattice) |
| Core-sound (Inv1–4) | (see ranking) |
| Extended-sound (Inv1–10 gates) | **5612** |
| **Winning** trajectories | **1339** |
| TLC on AriaInstance | PASS (shared gate) |
| Artifact | `aria-math-v3/artifacts/extensive_ranking.json` within the OPGROK checkout |

**Dimensions:** Φ order (4!), conditioning {token,diffusion,world_model}, ε, Match/Diff policies, residual model, stutter budget K∈{0..3}, structure ∈ {single_phi, double_phi, match_heavy, diffuse_heavy}.

---

## 2. Winning trajectories (top patterns)

A trajectory is **winning** if extended-sound ∧ residual_final=0 ∧ jepa_trend≥0 ∧ TLC≠false.

### Frequent runtime interleavings among top-20 winners

| Rank pattern (runtime Next order) | Count in top-20 | Note |
|-----------------------------------|----------------:|------|
| OpticalStep → Match → Diffuse → Predict | 7 | OMDP family |
| OpticalStep → Diffuse → Match → Predict | 6 | ODMP family |
| OpticalStep → Diffuse → Predict → Match | 5 | ODPM family |
| OpticalStep → Predict → Diffuse → Match | 2 | OPDM family |

**Observation:** **OpticalStep-first** dominates winning *runtime* schedules in this finite search.  
**Does not overturn 𝐂4:** 𝐂4 prefers the *logical composition* \(\Phi=\mathrm{Diff}\circ\mathrm{Match}\circ P\circ U\) as one complete Φ-step. Next remains a **disjunction**; many interleavings are admissible. v3 adds **𝐂7** (optical-first bias among winning traces).

### Example winning trajectory (greatest score 1650)

```
OpticalStep → Diffuse → Match → Predict → (repeat)
conditioning: diffusion | ε=2 | Match=identity | Diff=flip | K=0 | single_phi
```

Other high-score winners use `world_model` conditioning and `double_phi` structure — consistent with **𝐂2** (conditioning alone switches task family).

### Conditioning among top-20 winners

| \(a_t\) mode | Count |
|--------------|------:|
| world_model | 9 |
| diffusion | 7 |
| token | 4 |

**Theory T-C2-RICH:** under JEPA residual scoring, diffusion/world_model conditionings appear at least as often as token in top winners — supports 𝐂2 expressive breadth.

---

## 3. Ranked theories

| ID | Statement | Support (search) | Falsifier |
|----|-----------|------------------|-----------|
| **T-OPTICAL-FIRST** | Winning finite traces disproportionately begin with OpticalStep. | Top-20 all O-first | A large winning set with non-optical prefix under same gates |
| **T-OPMD-LOGICAL** | Logical Φ composition O→P→M→D remains preferred complete-step schedule (𝐂4); not the only winning *runtime* interleaving. | Spec + v1 + 95 OPMD wins in full winner pool | Logical composition fails Inv1–4 |
| **T-EPS-BAND** | ε∈{1,2} yields robust winning sets; ε=0 is stricter. | eps counts in winners | ε=0 dominates wins |
| **T-IDLE-MATCH** | Match=`identity` is common among winners (low graph churn). | top patterns | rebuild_gstar idle always wins |
| **T-C2-RICH** | diffusion/world_model conditionings appear often in top winners. | 16/20 non-token | only token wins |
| **T-STUTTER-BUDGET** | Extended-sound mass under K∈{1,2} is large (𝐂5 related). | support 2834 extended | K free always better |
| **T-STRUCTURE-FLEX** | single_phi and double_phi both admit many winners. | 9 vs 11 in top-20 | only one structure wins |
| **T-NO-FIFTH** | No winning config requires a fifth named action. | entire space | win only with new action |
| **T-P-BEFORE-M** | Prefer Predict before Match when residual is cold (Inv2). | v3 trace catalog safe-π filter | Cold \(M\prec P\) wins without residual warm-start |
| **T-RESIDUAL-ADAPTIVE** | If Res>\(\varepsilon\), Match-first is demoted; Predict/Diffuse first. | v3 math-audit / patches | Match with Res>\(\varepsilon\) always improves score |

---

## 4. Candidate extended invariants (documentation; optional)

These are **not** yet primary Inv1–4. They are candidate gates used in v3 extensive scoring.

| ID | Statement | Status |
|----|-----------|--------|
| **Inv5** | Stutter budget consecutive ≤ \(K\) (**𝐂5**) | operating preference |
| **Inv6** | Residual productivity if Res>\(\varepsilon\) | derived Inv2 / 𝐂6 |
| **Inv7** | Merge acyclicity under Match policy | Inv3 / 𝐋3 |
| **Inv8** | JEPA window trend on winners | winning condition |
| **Inv9** | Energy on all Next (same as Inv1, explicit) | Inv1 |
| **Inv10** | Match ED ⇒ TypeOK/GraphOK | Inv3, Inv4, ℙ3 |
| **Inv11** | Fair productivity (optional liveness) | optional WF |

**Inductive primary set remains Inv1–Inv4 only.** Full table: [FORMAL_SPEC.md](FORMAL_SPEC.md) §2.6.

---

## 5. Structure lattice (transformer schedules — not new architecture)

```
                    Next = O ∨ P ∨ M ∨ D ∨ Stutter
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
        single_phi      double_phi      heavy variants
         (1×Φ)           (2×Φ)        (match/diffuse-heavy)
              │               │               │
              └────────── optical-first bias ─┘
                              │
                    conditioning a_t ∈ C2
                    ε, Match, Diff, K
```

All lattice points stay inside LOCK. “Structure” = schedule/policy choice, not a new model family.

---

## 6. 𝐂7 (new corollary)

**𝐂7 — Optical-first bias among winning traces.**  
In extensive finite searches of Next-schedules, winning trajectories frequently begin with OpticalStep. This is a **runtime bias**, complementary to 𝐂4’s logical Φ composition preference. Falsifier: winning set with non-optical prefixes dominating under identical Inv gates.

---

## 7. Relation to v1 / v2

| Version | Contribution |
|---------|----------------|
| v1 | Combinatorial; crowned logical OPMD + ε=1 + identity idle |
| v2 | Live validation run; 𝐂5/𝐂6, merge policy, residual score |
| **v3** | Extensive lattice; theories; winning trajectory catalog; 𝐂7; Inv5–10 scorecard; live high-reasoning-effort session |

---

## 8. Reproduce (optional — external harness, not required to build Aria)

Requires a separate checkout of the OPGROK monorepo (not part of this repository).

```bash
export OPGROK_HOME=/path/to/opgrok   # sibling checkout, not part of this repository

# offline extensive
python3 "$OPGROK_HOME/core/binaries/aria-math-v3/tools/extensive_search.py" --tlc --eps-max 2 --max-configs 8000

# live, high reasoning effort
OPGROK_REQUIRE_LIVE=1 OPGROK_REASONING_EFFORT=high OPGROK_MODEL=grok-4.5 \
  python3 "$OPGROK_HOME/core/tools/run_harness.py" aria-math-v3 --repo "$OPGROK_HOME" --max-tokens 8192
```
