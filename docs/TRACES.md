# Aria — Trace Catalog

**Source:** `aria-math-v3` live artifact `n08_aria-math-v3-trace-catalog.md` + extensive search  
**LOCK:** alphabet \(\Sigma = \{O,P,M,D,S\}\) = OpticalStep, Predict, Match, Diffuse, Stutter only  
**Primary safety:** \(\square(\mathrm{Inv1}\land\mathrm{Inv2}\land\mathrm{Inv3}\land\mathrm{Inv4})\) with correct meanings (energy, contractivity, graph, TypeOK)

Full live artifact (in the OPGROK checkout, external monorepo — see [VALIDATION.md](VALIDATION.md) §2):  
`core/binaries/aria-math-v3/artifacts/n08_aria-math-v3-trace-catalog.md`

---

## 1. Alphabet

| Symbol | Action |
|--------|--------|
| \(O\) | OpticalStep |
| \(P\) | Predict |
| \(M\) | Match |
| \(D\) | Diffuse |
| \(S\) | Stutter |

---

## 2. Winning families (accepted)

| ID | Shape | Inv1–4 | Notes |
|----|-------|:------:|-------|
| **W1** | \((O\,P\,M\,D)^\omega\) | HOLD | Canonical pure Φ — **primary** with 𝐂4 |
| **W2** | \((O\,P\,M\,D\,S^{\{0,2\}})^\omega\) | HOLD | Stutter under 𝐂5 |
| **W3** | Match-heavy, residual-gated | HOLD if gated | Extra \(M\) only when Res policy allows |
| **W4** | Diffuse-heavy, energy/graph capped | HOLD if capped | Reject uncapped \(D^+\) |
| **W5** | Fair round-robin \(O,P,M,D\) + \(S\le K\) | HOLD + fairness | Optional liveness |
| **W6** | Φ-cycles with \(a_t\) flips + GF(\(O\)) | HOLD | 𝐂2 |
| **W7** | Soft ED + degree-bounded \(D\) | HOLD | 𝐋3 |

**Ranking key (lex):** TLC-safe ≻ Inv1–4 ≻ JEPA trend ≻ optical \(O(1)\) ≻ 𝐂5 fairness ≻ doc size.

---

## 3. Rejected infinite shapes

| ID | Shape | Failure |
|----|-------|---------|
| **X1** | \(S^\omega\) | ¬GF(\(O\)); JEPA fail |
| **X2** | \((P\,M\,D\,S^+)^\omega\) | Optical starve |
| **X3** | Uncapped \((O\,P\,M\,D^+)^\omega\) | Inv1/Inv3 risk |
| **X4** | Cold \(M\) before \(P\) forever | Inv2 risk (**𝐂8**) |
| **X5** | Perpetual \(S^{K+1}\) | 𝐂5 / Inv5 candidate |

---

## 4. Safe vs risky pure Φ permutations (π ∈ S₄)

**Prefer (P before M when residual cold):** e.g. \(OPMD\), \(OPDM\), \(POMD\), …  
**Risky without warm residual:** e.g. \(MOPD\), \(MDOP\), \(DMOP\) (Match/Diffuse before Predict).

Extensive search still found many **OpticalStep-first** winners with various tails (**𝐂7**); logical complete-step preference remains **OPMD** (**𝐂4**).

---

## 5. Admissibility predicate (documentation)

\[
\mathrm{Adm}(\sigma)
\;\triangleq\;
\sigma\models\square(\mathrm{Inv1}\land\mathrm{Inv2}\land\mathrm{Inv3}\land\mathrm{Inv4})
\;\land\;
\text{optical }O(1)
\;\land\;
\text{𝐂5 windows OK}
\]

Optional liveness: \(\mathrm{GF}(O)\) and fair productivity (**Inv11** candidate).

---

## 6. Compact regex emit (from v3)

```
W1_CANONICAL_PURE_PHI   = (O P M D)^ω
W2_STUTTER_INJECT_C5    = (O P M D S{0,2})^ω
X1_UNFAIR_STUTTER       = S^ω                         # REJECT
X2_O_STARVE             = (P M D S+)^ω                # REJECT
X4_COLD_M_BEFORE_P      = (M ... P ...)^ω cold res    # REJECT
X5_C5_BREAK             = (... S{K+1} ...)^ω          # REJECT
```
