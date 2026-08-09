# Aria — Safety Invariants and Winning Condition

Primary safety properties of the Aria (Ariadne) discrete-state specification.  
These properties are jointly inductive under \(\mathrm{Next}\). Fairness is not required for safety.

Cross-references: [FORMAL_SPEC.md](FORMAL_SPEC.md) for Init, Next, Spec, and named actions; [RATIONALE.md](RATIONALE.md) §4 for why inductive invariants are a defining dimension of Aria; [CONTINUOUS_REFINEMENT.md](CONTINUOUS_REFINEMENT.md) for Level 0–1 continuous prototypes that induce these invariants (energy under unitary flow; residual+\(\varepsilon\); GraphOK edits; TypeOK). Continuous material does **not** change Inv1–4 meanings.

---

## 1. Primary Safety Invariants

### Inv1 — Optical Energy

\[
\|\psi\|_2 = \|\psi_0\|_2
\]

**Inductive argument.**  
OpticalStep is the only action that changes \(\psi\). By axiom **𝔸4**, every admissible optical transformation is (near-)unitary. Under the ideal lossless idealization, unitarity preserves the \(\ell_2\) norm. Predict, Match, and Diffuse leave \(\psi\) unchanged. Init sets \(\psi = \psi_0\), so the equality holds at the initial state and is preserved by every Next step.

### Inv2 — Predictive Contractivity

\[
d\bigl(z,\, P(I(\psi), a)\bigr)
\;\le\;
d\bigl(z_{\mathrm{prev}},\, P(I(\psi_{\mathrm{prev}}), a_{\mathrm{prev}})\bigr)
\;+\; \varepsilon
\]

for a fixed tolerance \(\varepsilon \ge 0\).

**Inductive argument.**  
Follows from postulate **ℙ2** (\(\mathbb{E}[\mathrm{Lip}(P)] \le 1\)) and the fact that Predict is the sole action that updates \(z\) from the optical field via \(I(\psi)\). Diffuse may update \(z\) by \(\mathrm{Diff}_G\), which is conditioned on the graph and does not weaken the average contractivity of \(P\) with respect to optical-field predictions. Match and OpticalStep leave the relevant predictive residual uncontrolled only through \(\varepsilon\)-tolerance accumulation already admitted by the statement.

### Inv3 — Graph Integrity

Every node of \(G\) carries a well-typed embedding in \(\mathcal{Z}\), and every edge is a typed morphism.

**Inductive argument.**  
Match is the only action that alters \(G\). By postulate **ℙ3**, graph edit distance is realized by a finite sequence of elementary operations (add, delete, relabel). Each elementary operation is required to produce only well-typed nodes and typed morphisms. OpticalStep, Predict, and Diffuse leave \(G\) unchanged. Init sets \(G = G_0\) (empty or seed), which is assumed well-typed.

### Inv4 — TypeOK

\[
\square\,\mathrm{TypeOK}
\]

**Inductive argument.**  
Init implies TypeOK. Each named action updates only its declared variables and does so within the types of TypeOK:

| Action | Type discipline |
|---|---|
| OpticalStep | \(\psi' = U_t(\psi) \in \mathbb{C}^N\) |
| Predict | \(z' = P(I(\psi), a_t) \in \mathcal{Z}\) |
| Match | \(G'\) is a finite directed typed graph with embeddings in \(\mathcal{Z}\) |
| Diffuse | \(z' = \mathrm{Diff}_G(z) \in \mathcal{Z}\), \(t' = t+1 \in \mathbb{N}\) |

Hence TypeOK is an invariant of Spec.

---

## 2. Joint Inductiveness

The conjunction

\[
\mathrm{Inv1} \land \mathrm{Inv2} \land \mathrm{Inv3} \land \mathrm{Inv4}
\]

is inductive under Next:

1. It holds under Init.
2. If it holds in a state \(s\) and \(s \xrightarrow{\mathrm{Next}} s'\), then it holds in \(s'\).

Therefore every behavior of Spec satisfies

\[
\square\,(\mathrm{Inv1} \land \mathrm{Inv2} \land \mathrm{Inv3} \land \mathrm{Inv4}).
\]

This is the safety content of theorem **𝐓2** (optical energy and predictive contractivity along infinite behaviors), together with graph integrity and typing.

---

## 3. Winning Condition

A behavior \(\sigma\) is **winning** if and only if

\[
\sigma \models
\mathrm{Spec}
\land
\square\,(\mathrm{Inv1} \land \mathrm{Inv2} \land \mathrm{Inv3} \land \mathrm{Inv4})
\]

and, in addition:

1. **Joint embedding predictive property.**  
   The observable trajectory of latents and graph nodes realizes the intended joint embedding predictive property: the distance between predicted and true future embeddings tends to zero along every infinite path.

2. **Optical addressability.**  
   The experience graph remains optically addressable in constant transit time (justified by lemma **𝐋1** and corollary **𝐂3**).

Formally, the second clause is a liveness / asymptotic requirement on the embedding trajectory and is not implied by safety alone. Fairness conditions may be added later if a fully formal liveness proof is required.

---

## 4. What Safety Does Not Claim

| Claim | Status |
|---|---|
| Energy conservation under physical loss | Outside the ideal lossless idealization |
| Strict contractivity on every finite sample | Only average contractivity (**ℙ2**) plus \(\varepsilon\) |
| Optimality of graph merges | Only well-typed elementary edits (**ℙ3**) |
| Fair scheduling of actions | Not required for Inv1–Inv4 |
| Full reverse-SDE as one Spec step | Diffuse is one **atomic sample** of continuous diffusion ([CONTINUOUS_REFINEMENT.md](CONTINUOUS_REFINEMENT.md) §2.4) |
| 𝐂1 full Softmax equivalence as theorem | **Conjecture** / softened corollary — see continuous annex §1.4 |

---

## 5. Summary

| Invariant | Preserved quantity / property | Witness action |
|---|---|---|
| Inv1 | \(\|\psi\|_2\) | OpticalStep (unitary) |
| Inv2 | Predictive residual (up to \(\varepsilon\)) | Predict (+ ℙ2) |
| Inv3 | Well-typed graph structure | Match (+ ℙ3) |
| Inv4 | TypeOK | All actions |

Winning behaviors are those of Spec that maintain all four invariants forever and realize the JEPA-style predictive limit with constant-time optical addressability of experience.

**Operating note (does not change Inv1–Inv4):** combinatorial validation (`aria-math-v1`) prefers the full-Φ schedule OpticalStep→Predict→Match→Diffuse with \(\varepsilon=1\) and idle Match/Diff = identity. Live `aria-math-v2` adds stutter budget **𝐂5** (\(K=2\)), residual score \(\mathrm{score}_r=-\mathrm{Res}\), and Match/Diffuse/Stutter merge policy under **𝐋3**. `aria-math-v3` adds **𝐂7/𝐂8**, residual-adaptive Match gate, winning/rejected trajectory shapes, and **Inv5–Inv11** as documentation candidates only. **Refuses Spec enlargement beyond documentation.** See [THEORIES.md](THEORIES.md), [TRACES.md](TRACES.md), [VALIDATION.md](VALIDATION.md).

### Math-audit gates (from aria-math-v3; enforce on docs & schedules)

1. Refuse any fifth named action beyond OpticalStep, Predict, Match, Diffuse, Stutter.  
2. Enforce **Inv1** energy, **Inv2** contractivity, **Inv3** graph integrity, **Inv4** TypeOK on every step of a claimed winning trajectory.  
3. Optical addressability remains \(O(1)\) (**𝐋1**, **𝐂3**); forbid linear scans over \(|V|\) in the optical idealization.  
4. Stutter windows respect **𝐂5** (\(K\le 2\) default).  
5. Cold residual: prefer Predict before Match (**𝐂8**).
