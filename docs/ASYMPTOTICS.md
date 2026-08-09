# Aria — Asymptotic Corollaries

Asymptotic claims justified by the formal structure of Aria (axioms, postulates, lemmas, and the discrete-state Spec).  
These are consequences of the specification, not independent engineering claims.

Cross-references: [FORMAL_SPEC.md](FORMAL_SPEC.md), [SAFETY.md](SAFETY.md), [RATIONALE.md](RATIONALE.md) §1 (substrate consequences).

---

## 1. Asymptotic Corollaries

| Quantity | Bound | Justification |
|---|---|---|
| Optical depth of one complete \(\Phi\)-step | \(O(\log N + \mathrm{polylog}\, M)\) | **ℙ1** (inner-product similarity at depth \(O(\log N)\)); graph-conditioned steps scale with polylog of graph size \(M\) |
| Optical energy per effective MAC | \(O(N^{-1})\) | Ideal lossless regime; energy spread over \(N\) modes under unitary evolution (**𝔸1**, **𝔸4**, Inv1) |
| Experience-graph size after \(T\) trajectories | \(O(T^\beta)\) with \(\beta \le 1\) | **𝐋3** (aggressive subgraph merging) |
| Ranking latency | \(O(1)\) optical | **𝐋1** (broadcast-and-weight; independent of \(M\)) → **𝐂3** |

Here \(N\) is the number of optical modes, \(M\) is the current number of keys / addressable graph nodes, and \(T\) is the number of trajectories absorbed into \(G\).

---

## 2. Derivation Notes

### 2.1 Optical depth of \(\Phi\)

A complete \(\Phi\)-step is the composition

\[
\Phi = \mathrm{Diff} \circ \mathrm{Match} \circ P \circ U.
\]

- \(U\) (OpticalStep): interference circuits realizing similarities among \(N\) modes require depth \(O(\log N)\) by **ℙ1**.
- \(P\) (Predict): isometry embedding \(I\) and the predictor act in the latent space; any optical realization of the relevant similarities inherits the same logarithmic depth bound in \(N\).
- Match / Diff: operations that touch the experience graph contribute factors polynomial in \(\log M\) under standard hierarchical or broadcast optical addressing, consistent with **𝐋1** for the ranking substep.

Hence one complete \(\Phi\)-step has optical depth

\[
O\bigl(\log N + \mathrm{polylog}\, M\bigr).
\]

### 2.2 Energy per effective MAC

Under Inv1 and the ideal lossless idealization, \(\|\psi\|_2\) is conserved. Spreading work across \(N\) modes yields optical energy per effective multiply–accumulate of order \(O(N^{-1})\) relative to a single-mode baseline. This is an idealization; physical loss is outside Spec.

### 2.3 Graph size

Lemma **𝐋3** states that under aggressive subgraph merging,

\[
\lvert G \rvert = O(T^\beta), \qquad \beta \le 1,
\]

after \(T\) trajectories. Sub-linear growth (\(\beta < 1\)) is permitted; linear growth is the worst case admitted by the lemma. Combined with **𝐂3**, retrieval remains \(O(1)\) optical even as experience accumulates.

### 2.4 Ranking latency

Lemma **𝐋1**: broadcast-and-weight ranking of a query against \(M\) keys requires only \(O(1)\) optical transit time, independent of \(M\). Corollary **𝐂3** restates this for retrieval against the growing experience graph.

---

## 3. Expressive-Power Corollaries (from 𝐂)

These are qualitative consequences of the axiomatization, not big-O statements.

| Corollary | Content |
|---|---|
| **𝐂1** | Softmax attention is replaced by a phase-sensitive optical kernel without loss of expressive power. |
| **𝐂2** | The same latent space supports discrete next-token prediction, continuous diffusion score estimation, and world-model roll-outs by change of conditioning alone. |
| **𝐂3** | Retrieval latency remains \(O(1)\) optical even as experience grows sub-linearly. |
| **𝐂4** | Preferred discrete schedule OpticalStep→Predict→Match→Diffuse among \(4!\) orders (preference only). |
| **𝐂5** | Stutter fairness budget \(K=2\) consecutive (preference; optional liveness). |
| **𝐂6** | Idle Match/Diff = identity when residual is 0 (churn-minimal; validated). |
| **𝐂7** | Optical-first bias among winning *runtime* traces (complements 𝐂4 logical Φ; see [THEORIES.md](THEORIES.md)). |
| **𝐂8** | Prefer Predict before Match when residual is cold (Inv2; see [TRACES.md](TRACES.md)). |

### Optical cycle complexity (v3 audit)

Under **𝐋1** \(O(1)\) address resolve and degree-bounded graph policies, a preferred Φ-cycle is \(O(1 + \mathrm{cost}(P)+\mathrm{cost}(M)+\mathrm{cost}(D))\) w.r.t. \(|V|\) when \(m,\Delta\) are policy-capped — aligns with **𝐂3**.

### Operating efficiency (from combinatorial validation)

Under the finite search of `aria-math-v1` (up to 10368 configs, TLC PASS):

- **No cheaper reordering** of the four named actions beat the documented \(\Phi\) order on soundness+score.
- **Idle Match = identity** minimises graph edit cost while remaining sound.
- **Idle Diff = identity** minimises latent churn when residual is already 0.
- **\(\varepsilon=1\)** is the preferred contractivity budget vs \(\varepsilon=0\) in top ranks.

These are operating defaults, not asymptotic big-O changes.

---

## 4. Error and Layer Bounds

| Lemma | Content |
|---|---|
| **𝐋2** | Residual optical paths keep analog error bounded after \(L = O(N^\alpha)\) layers for some \(\alpha > 0\). |

𝐋2 bounds the analog accumulation of residual path error as a function of depth in \(N\). It supports the claim that the discrete Spec’s ideal unitary steps remain a faithful abstraction over polynomially many optical layers.

---

## 5. Scope

These asymptotics apply only to the Aria discrete-state model and the optical abstractions named in the formal specification. They do not specify wall-clock latency on any particular photonic device, nor do they bound software implementations of the same state machine.
