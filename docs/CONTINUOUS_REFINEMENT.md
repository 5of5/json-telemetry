# Aria — Continuous Refinement (Level 0–1)

**Status:** Level 0 geometric core + Level 1 continuous prototypes and discretization  
**Does not enlarge discrete Spec.** Named actions remain OpticalStep, Predict, Match, Diffuse + Stutter.  
**Inv1–4 meanings unchanged:** energy, contractivity, graph integrity, TypeOK ([SAFETY.md](SAFETY.md)).  
**Discrete authority:** [FORMAL_SPEC.md](FORMAL_SPEC.md), `spec/Aria.tla`.

### Epistemic labels (mandatory)

Every mathematical claim in this annex is marked:

| Label | Meaning |
|-------|---------|
| **theorem** | Follows from stated axioms/postulates by a short argument given here or in SAFETY |
| **conjecture** | Plausible, not proven in-repo; may be softened later |
| **design axiom** | Chosen modeling commitment of Aria; not derived from external orthodoxy |

---

## 0. Purpose and non-claims

### 0.1 Purpose

State the continuous geometry and continuous-time prototypes that **induce** the discrete Spec, and how discrete trajectories **refine** or **sample** those prototypes.

### 0.2 Non-claims

| Non-claim | Why |
|-----------|-----|
| Aria is a residual-stream Softmax GPT | Different family ([RATIONALE.md](RATIONALE.md)) |
| This annex replaces TLA+ | Level 2 remains machine Spec |
| A fifth named action is required | All continuous interfaces map to the existing five |
| Physical photonic hardware is specified | Ideal lossless optical model only |
| Full reverse SDE is an atomic Spec step | Diffuse is one **atomic sample** (§4) |

---

## 1. Level 0 — Geometric core

### 1.1 Hilbert optical field

**Design axiom (𝔸1).** Optical modes form a finite-dimensional complex Hilbert space

\[
\mathcal{H} = \mathbb{C}^N,\qquad
\langle u,v\rangle_{\mathcal{H}} = \sum_{j=1}^{N} \overline{u_j}\, v_j,\qquad
\|u\|_2 = \sqrt{\langle u,u\rangle_{\mathcal{H}}}.
\]

**Design axiom (energy).** Optical energy of a field is \(E(\psi) := \|\psi\|_2\). Inv1 asserts \(E(\psi)=E(\psi_0)\) along discrete behaviors under the ideal lossless idealization ([SAFETY.md](SAFETY.md)).

**Design axiom (𝔸4).** Every admissible optical map \(U:\mathcal{H}\to\mathcal{H}\) is *near-unitary*. Idealization used for Inv1: \(U\in\mathrm{U}(N)\) (exactly unitary), so \(E(U\psi)=E(\psi)\).

**Theorem (energy under ideal unitary group).** If \(t\mapsto U(t)\) is a strongly continuous one-parameter subgroup of \(\mathrm{U}(N)\), then for all \(t\) and \(\psi\),

\[
E\bigl(U(t)\psi\bigr) = E(\psi).
\]

*Proof sketch.* Unitarity \(\Leftrightarrow\) \(U(t)^\*U(t)=I\), hence \(\|U(t)\psi\|_2=\|\psi\|_2\). ∎

**Design axiom (optical multiport).** Discrete OpticalStep applies one member \(U_t\) of a unitary family indexed by the discrete clock (or schedule index), not necessarily the full continuous flow at every real time.

### 1.2 Latent space and isometry

**Design axiom.** There is a real or complex latent space \(\mathcal{Z}\) equipped with a metric \(d:\mathcal{Z}\times\mathcal{Z}\to[0,\infty)\) (implementation: Euclidean or cosine distance lifted to a metric or semi-metric; TLA uses Nat-valued Dist with zero diagonal).

**Design axiom (𝔸2).** Fixed isometry (or isometric embedding up to scaling convention)

\[
I:\mathcal{H}\hookrightarrow\mathcal{Z}.
\]

In the ideal model, \(I\) preserves norms in the sense relevant to residual comparison (electronic sim may use learned \(I\) with energy-aware normalization — PRD operator table).

**Design axiom (residual geometry of \(I\)).** After OpticalStep, \(\psi\) may change while \(z\) is frozen, so \(\mathrm{Res}\) can jump purely because \(I(\psi)\) moved under \(d\) relative to fixed \(z\). That jump is admitted by Inv2’s \(\varepsilon\)-slack (and by subsequent Predict). The ideal model treats \(I\) as Lipschitz with \(\mathrm{Lip}(I)\le L_I\); then

\[
\bigl|\,d\bigl(z,P(I(U\psi),a)\bigr) - d\bigl(z,P(I(\psi),a)\bigr)\,\bigr|
\]

is controlled by \(L_I\cdot\mathrm{Lip}(P)\) and \(\|U\psi-\psi\|\) when \(d\) is norm-induced — **conjecture** under pathwise Lip; Spec does not require pathwise control beyond \(\varepsilon\). Residual is therefore a **JEPA predictive score**, not an automatic Lyapunov function of optical flow alone.

**Design axiom (Init).** \(z_0 = I(\psi_0)\).

### 1.3 Predictor and residual

**Design axiom (ℙ2).** Predictor \(P:\mathcal{Z}\times\mathcal{A}\to\mathcal{Z}\) is contractive on average:

\[
\mathbb{E}\bigl[\mathrm{Lip}(P(\cdot,a))\bigr] \le 1
\]

over the training/conditioning measure on \(a\in\mathcal{A}\).

**Design axiom (residual).** For state \((\psi,z,t)\) and conditioning schedule \(a(\cdot)\),

\[
\mathrm{Res}(\psi,z,t)
\;:=\;
d\bigl(z,\, P\bigl(I(\psi),\, a(t)\bigr)\bigr).
\]

**Design axiom (Inv2 slack).** Discrete safety uses

\[
\mathrm{Res}(\psi,z,t) \le \mathit{prevRes} + \varepsilon,\qquad \varepsilon\ge 0,
\]

not pathwise Lip \(\le 1\) on every sample. Average contractivity alone does **not** imply pathwise residual non-increase — **conjecture** if \(\varepsilon=0\) and stronger assumptions; Spec retains \(\varepsilon\).

**Theorem (Predict zeros self-residual under exact \(P\)).** After a discrete Predict step \(z'=P(I(\psi),a(t))\) with \(\psi,t\) unchanged,

\[
\mathrm{Res}(\psi,z',t) = d\bigl(P(I(\psi),a(t)),\, P(I(\psi),a(t))\bigr) = 0
\]

when \(d(x,x)=0\). *Matches TLA comment on Predict.* ∎

### 1.4 Optical kernel vs Softmax (𝐂1)

Mainstream attention uses a Softmax kernel on electronic query–key scores. Aria’s substrate claim is **phase-sensitive optical interference** among modes in \(\mathcal{H}\).

**Design axiom (optical kernel prototype).** For mode amplitudes assembled into query/key-like optical fields \(q,k\in\mathcal{H}\), a phase-sensitive similarity is of the schematic form

\[
K_{\mathrm{opt}}(q,k)
\;=\;
\bigl|\langle q, k\rangle_{\mathcal{H}}\bigr|^2
\quad\text{or a fixed linear-optical multiport polynomial in }(q,k),
\]

realizable by interference (cf. ℙ1 depth \(O(\log N)\) for finite-mode inner-product similarities).

**Conjecture (𝐂1 expressive power — softened).** For finite \(N\) and a fixed class of ranking / soft-selection tasks expressible via inner-product similarities, there exists an optical multiport of depth \(O(\log N)\) whose induced ranking agrees with Softmax attention up to a continuous strictly increasing reparameterization of scores on compact sets where scores are separated by a margin. **Not proven in-repo.**

**Honest softening (required).** The slogan “without loss of expressive power” in FORMAL_SPEC **𝐂1** is retained as a **corollary label** pointing at this conjecture, not as a completed theorem. Spec does **not** depend on full Softmax equivalence; it depends only on OpticalStep being (near-)unitary and Inv1.

**Non-claim.** Classical residual-stream token geometry is not asserted unless derived as a special conditioning of \(a_t\) (**𝐂2**).

### 1.5 Experience graph

**Design axiom (𝔸3).** Experience is a finite directed typed graph \(G=(V,E)\) with node embeddings \(e:V\to\mathcal{Z}\) and typed morphisms on edges.

**Design axiom (ℙ3).** Graph edit distance is realized by finite sequences of elementary operations: add node, delete node, relabel embedding, add/delete edge (and finite rebuild to a target \(G^*\) as a composite).

**Design axiom.** Match is the only Spec action that mutates \(G\); morphisms remain well-typed (Inv3).

**Theorem (Inv3 under elementary edits).** If \(G\) is GraphOK and \(G'\) is obtained by one elementary edit of the forms admitted in `spec/Aria.tla` (identity, add/delete node, relabel, add/delete edge, rebuild to GraphOK \(G^*\)), then \(G'\) is GraphOK. *By construction of ElementaryEdit.* ∎

### 1.6 Level 0 summary diagram

```text
  ψ ∈ H = C^N  --I-->  Z  ∋ z
       |                |
       U_t           P(·,a), Diff_G
       |                |
       v                v
  OpticalStep        Predict / Diffuse
                         |
                      Match ↔ G (typed, embeddings in Z)
```

---

## 2. Level 1 — Continuous prototypes and discrete interfaces

Each named Spec action is the **discrete interface** of a continuous prototype. Continuous objects are not additional Spec actions.

| Action | Continuous prototype | Discrete interface | Primary Inv |
|--------|----------------------|--------------------|-------------|
| OpticalStep | Unitary ODE / multiport flow \(U(\tau)\) on \(\mathcal{H}\) | \(\psi'=U_t(\psi)\); UNCHANGED \(z,G,t\) | Inv1 |
| Predict | JEPA predictor map / short flow in \(\mathcal{Z}\) | \(z'=P(I(\psi),a_t)\); UNCHANGED \(\psi,G,t\) | Inv2 |
| Match | Graph morphism / edit-distance path in graph space | \(G'=\mathrm{ED}(G\cup\{z\},G^*)\); UNCHANGED \(\psi,z,t\) | Inv3 |
| Diffuse | Score-based / diffusion step conditioned on \(G\) | \(z'=\mathrm{Diff}_G(z)\); \(t'=t+1\); UNCHANGED \(\psi,G\) | Inv2, clock |
| Stutter | Idle or unobserved micro-evolution | UNCHANGED all vars | all (vacuously) |

### 2.1 OpticalStep — unitary flow

**Design axiom (continuous optical dynamics).** Prefer a Schrödinger-like or multiport generator: there exists a skew-Hermitian (or \(i\) times Hermitian) generator \(H(\tau)\) such that

\[
\frac{d}{d\tau}\psi(\tau) = -i H(\tau)\,\psi(\tau)
\quad\text{(or the multiport ODE with unitary fundamental solution)}.
\]

**Theorem (energy along continuous optical flow).** Under ideal unitary evolution, \(E(\psi(\tau))\) is constant in \(\tau\). *Same proof as §1.1.* ∎

**Discretization.** OpticalStep samples one unitary increment \(U_t=\mathcal{T}\exp\bigl(-i\int_{\tau_t}^{\tau_{t+1}}H\bigr)\) (time-ordered), or a designed multiport for discrete address \(t\). Grain: one complete optical address/transform step, not an infinitesimal generator in Spec.

### 2.2 Predict — JEPA continuous view

**Design axiom.** Core prediction never decodes to tokens/pixels inside the loop ([RATIONALE.md](RATIONALE.md) §3). \(P\) acts only in \(\mathcal{Z}\).

**Design axiom (predictor flow prototype).** Either:

1. **Map form (primary):** \(z^+ = P(I(\psi),a)\) as a single Lipschitz map (matches Spec Predict), or  
2. **Flow form (optional refinement):** \(\dot\zeta = f_P(\zeta; I(\psi), a)\) on a short interval with \(\zeta(0)=z\), \(\zeta(1)=P(\ldots)\).

**Conjecture (residual non-increase under Lip bound).** If \(\mathrm{Lip}(P(\cdot,a))\le 1\) pathwise and \(d\) is the metric induced by a norm with \(P\) non-expansive, then distances between latent trajectories do not expand. Spec does **not** assume pathwise Lip; it assumes ℙ2 average Lip plus \(\varepsilon\) (**design axiom** of Inv2).

**Conditioning (𝐂2).** Changing \(a\in\{\texttt{token},\texttt{diffusion},\texttt{world\_model},\ldots\}\) reuses the same \(P\) and \(\mathcal{Z}\) — **design axiom** of unification by conditioning, not three architectures.

### 2.3 Match — graph morphism path

**Design axiom (two-layer continuous object).** Match’s continuous prototype has two layers:

1. **Topology jumps:** discrete elementary edits (add/delete/relabel/edge) — Spec grain.  
2. **Embedding flow (optional continuous fill):** on a *fixed* topology \((V,E)\), node embeddings \(e_v(\tau)\in\mathcal{Z}\) may evolve by a Lipschitz ODE \(\dot e_v = f_v(e; G, z)\) (e.g. attraction toward current latent \(z\) or expert embeddings). Topology changes remain **atomic discrete** events, not continuous.

**Design axiom.** A Match step is either identity, one elementary topology/embedding edit, or finite rebuild to \(G^*\) (composite of elementary edits). Continuous embedding flow between Matches is *not* a separate Spec action; if observed only at event times it is absorbed into the post-state embeddings of \(G'\).

**Discretization.** Spec Match takes **one** elementary-edit step (or identity / rebuild) from \(G\oplus z\) toward \(G^*\) ([FORMAL_SPEC.md](FORMAL_SPEC.md) §6.3; TLA `OneStepEdits`). A long edit path is a **sequence** of Match (and possibly Diffuse/Stutter under L3 merge policy) — not a new action.

**Theorem (Inv3).** GraphOK is preserved (§1.5). ∎

### 2.4 Diffuse — continuous diffusion, atomic sample

**Design axiom (continuous diffusion prototype).** On latent space, a reverse-time or score-based process conditioned on \(G\), schematically

\[
\mathrm{d}Z_s = b(Z_s, s; G)\,\mathrm{d}s + \sigma(s)\,\mathrm{d}W_s
\]

(or a deterministic probability-flow ODE with the same marginals), where \(b\) may depend on graph message-passing features of \(G\).

**Design axiom (atomic sample — critical).** The Spec action **Diffuse** is **not** the entire continuous path \((Z_s)_{s\in[0,1]}\). It is **one atomic sample** of a chosen discretization operator:

\[
z' = \mathrm{Diff}_G(z) \;:=\; \Psi_G(z; \Delta s, \xi),
\]

where \(\Psi_G\) is a measurable one-step kernel (Euler–Maruyama, probability-flow step, message-passing residual block, etc.) and \(\xi\) is optional randomness. The discrete clock advances \(t'=t+1\) exactly on this sample ([FORMAL_SPEC.md](FORMAL_SPEC.md) §6.4).

**Conjecture.** Under standard score-matching training and Lipschitz score assumptions, a small step \(\mathrm{Diff}_G\) decreases a variational free energy conditioned on \(G\). **Not required for Inv1–4.**

**Design axiom (Diffuse vs Inv2).** \(\mathrm{Diff}_G\) may increase \(\mathrm{Res}\) relative to the pre-step residual; safety requires only

\[
\mathrm{Res}(\psi',z',t') \le \mathrm{Res}(\psi,z,t) + \varepsilon
\]

on the post-state (TLA action obligation). There is **no** continuous Lyapunov guarantee that Diffuse alone contracts residual — that is Predict’s role under ℙ2. Implementations that need tighter residual after Diffuse should shrink the step \(\Delta s\) or raise \(\varepsilon\) in operating policy, not invent a new Spec action.

### 2.5 Stutter

**Design axiom.** Stutter is either (i) true idle, or (ii) a modeling of unobserved continuous micro-evolution that does not change the discrete observation \(v\). Safety invariants are preserved because no variable changes ([SAFETY.md](SAFETY.md)). Operating preference 𝐂5 bounds consecutive Stutters; that is **not** a Spec removal of Stutter.

---

## 3. Refine vs sample

### 3.1 Definitions

| Relation | Meaning for Aria |
|----------|------------------|
| **Refinement** | Every discrete behavior is the observation of some continuous trajectory through a projection \(\pi\) (field/latent/graph snapshot at event times). |
| **Sample** | A discrete action applies a fixed discretization operator to the continuous prototype (one step of a numerical scheme), not necessarily dense in continuous time. |

**Design axiom (projection \(\pi\)).** Let continuous micro-state be \(X(\tau)=(\psi(\tau),z(\tau),G(\tau),\alpha(\tau))\) with optional continuous clocks/noise \(\alpha\). At discrete event times \(\tau_k\),

\[
\pi\bigl(X(\tau_k)\bigr)
\;=\;
\bigl(\psi(\tau_k),\, z(\tau_k),\, G(\tau_k),\, t_k\bigr)
\]

with \(t_k\) the Spec counter (advanced only on Diffuse samples). Auxiliary \(\mathit{prevRes}\) is history of \(\mathrm{Res}\circ\pi\), not part of \(\pi\)’s geometric core. TLC’s finite carriers are further abstractions of \(\pi(X)\), not of full \(X(\tau)\).

**Design axiom (Aria stance).** Discrete Spec behaviors **sample** the continuous prototypes at the grain of named actions. They **refine** continuous dynamics only in the weak sense that each named action is consistent with *some* continuous segment (existence of a continuous interpolant), not that every continuous optical/diffusion path is recoverable from Spec alone.

**Theorem (interpolant existence — optical).** Given \(\psi\) and unitary \(U_t\), there exists a continuous unitary path from \(\psi\) to \(U_t(\psi)\) (geodesic in \(\mathrm{U}(N)\) applied to \(\psi\)). ∎

**Conjecture (interpolant — diffusion).** For reasonable \(\mathrm{Diff}_G\), there exists a continuous latent path from \(z\) to \(\mathrm{Diff}_G(z)\) consistent with a score/ODE model conditioned on \(G\). Implementation-dependent.

**Non-claim.** Spec is not a dense time-stepping scheme; TLC finite models use abstract carriers, not \(\mathbb{C}^N\).

### 3.2 Composition \(\Phi\)

The logical composition

\[
\Phi = \mathrm{Diff}\circ\mathrm{Match}\circ P\circ U
\]

is a **design axiom** for one complete discrete macro-step (𝐂4 preferred order). Continuous-time residual flow may interleave micro-dynamics differently; runtime interleavings admitted by Next may differ from 𝐂4 ([THEORIES.md](THEORIES.md), 𝐂7). Spec Next remains a disjunction — **no fifth action**.

---

## 4. Energy, residual, and Diffuse-as-sample (labeled)

| Claim | Label |
|-------|-------|
| \(E(U\psi)=E(\psi)\) for ideal unitary \(U\) | **theorem** |
| Inv1 holds under OpticalStep in the idealization | **theorem** (SAFETY) |
| Pathwise residual non-increase without \(\varepsilon\) from ℙ2 alone | **conjecture** (false in general); Spec uses \(\varepsilon\) |
| After Predict, self-residual is zero when \(d(x,x)=0\) | **theorem** |
| Discrete trajectory is a sample of continuous prototypes at action grain | **design axiom** |
| Diffuse = one atomic kernel sample, not full reverse SDE | **design axiom** |
| 𝐂1 full Softmax equivalence | **conjecture** (softened §1.4) |
| JEPA \(d\to 0\) along winning infinite paths | **winning condition** (liveness), not Inv |

---

## 5. Map back to discrete Spec (LOCK)

| Continuous object | Discrete home | Forbidden |
|-------------------|---------------|-----------|
| \(U(\tau)\), multiports | OpticalStep | New “EvolveH” action |
| \(P\), predictor flow | Predict | Decoder-in-core as Spec action |
| Edit paths | Match (+ policies) | Free-form graph API outside ED |
| Score SDE / flow | Diffuse atomic sample | “RunDiffusionEpisode” macro-action in Spec |
| Idle / hidden micro | Stutter | Removing Stutter from Next |

**Invariant lock:** Inv1 energy, Inv2 contractivity (residual+\(\varepsilon\)), Inv3 graph integrity, Inv4 TypeOK — meanings fixed in [SAFETY.md](SAFETY.md).

**Action lock:** \(\Sigma=\{\mathrm{OpticalStep},\mathrm{Predict},\mathrm{Match},\mathrm{Diffuse},\mathrm{Stutter}\}\) only.

---

## 6. Open mathematical problems

| ID | Problem | Label |
|----|---------|-------|
| OP1 | Prove or refute 𝐂1 margin-equivalence to Softmax on a precise task class | conjecture |
| OP2 | Pathwise residual Lyapunov function under average-Lip \(P\) + Diffuse | open |
| OP3 | Continuous graph gradient-flow whose Euler step is elementary ED | open / design |
| OP4 | Quantitative free-energy decrease for \(\mathrm{Diff}_G\) under graph conditioning | conjecture |
| OP5 | Tight optical depth constants in ℙ1 for concrete multiport libraries | open |
| OP6 | When does sampled Spec dense-refine a given continuous Aria model? | open |

---

## 7. Document control

| Field | Value |
|-------|--------|
| Path | `docs/CONTINUOUS_REFINEMENT.md` |
| Levels covered | 0 (geometry), 1 (prototypes + discretization) |
| Spec enlargement | **None** |
| Depends on | FORMAL_SPEC, SAFETY, RATIONALE, ASYMPTOTICS |
| Feeds | PRD Phase 0; FORMAL_SPEC §9 hooks; implementer design commentary |
