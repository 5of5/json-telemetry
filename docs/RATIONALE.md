# Aria — Differentiation and Rationale

**Additional specification and design rationale.**

Aria differs from every mainstream AI model in four fundamental dimensions: **substrate**, **state representation**, **prediction principle**, and **formal guarantees**.

This document is part of the Aria documentation set. It does not replace the discrete-state Spec in [FORMAL_SPEC.md](FORMAL_SPEC.md); it states *why* that Spec is shaped as it is and how Aria is a different mathematical object from frontier electronic models. Continuous Level 0–1 geometry and prototypes that induce the discrete actions are in [CONTINUOUS_REFINEMENT.md](CONTINUOUS_REFINEMENT.md).

---

## Thesis

Aria is not “a better transformer.” It is a different mathematical object — a formally specified dynamical system whose fundamental operations are:

1. **optical interference**,
2. **latent contractive prediction**, and
3. **graph morphisms**,

designed so that language modeling, diffusion, and world modeling become different conditionings of the **same** state machine rather than separate architectures.

---

## 1. Substrate — Light instead of electrons

### Typical frontier models

Standard transformers (GPT-class, Claude, Gemini, Llama, etc.), diffusion models, and most JEPA implementations execute matrix multiplications and attention on electronic hardware (GPUs / TPUs / NPUs). Softmax attention is realized as electronic \(QK^\top\) (or an approximation thereof).

### Aria

Aria’s core step \(\Phi\) is defined over optical mode amplitudes \(\psi \in \mathbb{C}^N\). Attention is realized by **coherent interference** — the phase-sensitive optical kernel that replaces Softmax (corollary **𝐂1**) — not by an electronic \(QK^\top\) matrix.

### Consequences (specification-level)

| Consequence | Anchor in Spec |
|---|---|
| Dominant operation is a near-unitary optical transformation | **𝔸4**, OpticalStep, Inv1 |
| Ideal energy cost scales as \(O(N^{-1})\) per effective MAC | [ASYMPTOTICS.md](ASYMPTOTICS.md) |
| Ranking over an arbitrary number of keys remains \(O(1)\) optical transit time | **𝐋1**, **𝐂3** |
| Optical depth of one \(\Phi\)-step is \(O(\log N + \mathrm{polylog}\, M)\) | **ℙ1**, asymptotics |

No other production model is specified this way: the substrate of the core dynamics is optical amplitude, not electronic dense arithmetic.

---

## 2. State — A single triple that unifies field, latent, and experience graph

### Typical frontier models

Most models maintain three separate things:

- an activation / residual stream,
- a KV cache or external retrieval store,
- (optionally) a separate world-model or diffusion process.

Memory is external or cache-like. Experience is not a first-class component of the core dynamics.

### Aria

Aria’s state is the single typed object

\[
v = (\psi,\, z,\, G)
\]

(with discrete clock \(t\) in the full Spec), where:

| Component | Role |
|---|---|
| \(\psi\) | Optical field amplitudes |
| \(z = I(\psi)\) (initially; then evolved by Predict / Diffuse) | JEPA embedding of the field |
| \(G\) | Persistent experience / thought graph |

The graph is **not** an external memory. It is part of the state machine; only **Match** mutates \(G\) (via elementary edits). **Diffuse** is conditioned on \(G\) and updates the latent \(z\) (and clock \(t\)), not the graph topology (see [FORMAL_SPEC.md](FORMAL_SPEC.md) §6).

### Consequences (specification-level)

- One-shot **structural correction** — edit-path matching of failed versus expert trajectories — is a native transition (Match via \(\mathrm{ED}\)), not an iterative prompting loop or a fine-tuning stage.
- Graph integrity is an inductive safety invariant (**Inv3**), not an informal memory policy.
- Optical addressability of experience remains \(O(1)\) transit time even as \(\lvert G \rvert\) grows sub-linearly (**𝐋1**, **𝐋3**, **𝐂3**).

No current frontier model treats experience as a first-class, optically addressable graph **inside** the core dynamics.

---

## 3. Prediction principle — Pure joint-embedding prediction, never reconstruction

### Typical frontier models

| Family | Core prediction target |
|---|---|
| Classical language models | Next token in the original (token) space |
| Diffusion models | Noise or clean data in the original (pixel / token) space |
| Hybrid systems | Separate heads / modules glued post-hoc |

### Aria

Aria obeys the **JEPA axiom**: the predictor \(P\) only ever acts in latent space \(\mathcal{Z}\), and the objective never requires a decoder back to tokens or pixels as part of the **core loop**.

Formally (Predict action):

\[
z' = P\bigl(I(\psi),\, a_t\bigr)
\]

with average contractivity \(\mathbb{E}[\mathrm{Lip}(P)] \le 1\) (**ℙ2** → Inv2).

### Unification by conditioning alone

The same latent \(z\) can be conditioned for:

- discrete next-token prediction,
- continuous diffusion score estimation, or
- multi-step world-model roll-out,

simply by changing the conditioning variable \(a_t\) (corollary **𝐂2**).

This unification is **architectural** — one state machine, one Spec — not a post-hoc combination of three separate systems.

### Winning condition (predictive part)

Behaviors are winning only if, beyond safety, the distance between predicted and true future embeddings tends to zero along every infinite path ([SAFETY.md](SAFETY.md) §3). That is a joint-embedding predictive property, not a reconstruction loss.

---

## 4. Formal guarantees — Explicit inductive invariants

### Typical frontier models

Virtually all commercial models are defined by:

- a training objective, and
- an architectural diagram (layers, heads, residual paths).

Public definitions do not supply jointly inductive safety invariants over energy, contractivity, and memory integrity.

### Aria

Aria is defined by a discrete-state specification containing:

| Element | Role |
|---|---|
| TypeOK | Typing of \((\psi, z, G, t)\) |
| Init | Initial state \(\psi_0\), \(z = I(\psi_0)\), \(G_0\), \(t = 0\) |
| Four named actions | Realize \(\Phi = \mathrm{Diff} \circ \mathrm{Match} \circ P \circ U\) |
| Four primary safety invariants | Optical energy, predictive contractivity, graph integrity, TypeOK — jointly inductive under Next |
| Spec | \(\mathrm{Init} \land \square[\mathrm{Next}]_{\langle\psi,z,G,t\rangle}\) |
| Winning condition | Spec + \(\square(\mathrm{Inv1}\land\mathrm{Inv2}\land\mathrm{Inv3}\land\mathrm{Inv4})\) + JEPA limit + \(O(1)\) optical addressability |

A behavior is **winning** only if it satisfies the specification and the invariants forever ([SAFETY.md](SAFETY.md)).

This level of explicit inductive reasoning about energy, contractivity, and memory integrity does not exist in the public definitions of GPT-class, Claude, Gemini, or pure JEPA systems.

---

## 5. Summary table of decisive differences

| Dimension | Typical frontier models | Aria |
|---|---|---|
| Compute primitive | Electronic matrix multiply / attention | Optical unitary interference |
| Core state | Activations + KV cache | \((\psi, z, G)\) triple |
| Memory | External or cache | First-class experience graph |
| Prediction | Reconstruct tokens / pixels / noise | Pure latent prediction (no reconstruction in the core loop) |
| Error correction | Iterative reflection or fine-tuning | One-shot graph edit paths |
| Attention cost | Quadratic or linear-electronic | \(O(1)\) optical ranking |
| Formal status | Training objective + architecture diagram | Discrete-state Spec + inductive invariants |

---

## 6. Mapping dimensions → formal anchors

| Dimension | Primary Spec anchors |
|---|---|
| Substrate | **𝔸1**, **𝔸4**, OpticalStep, **ℙ1**, **𝐋1**, Inv1, asymptotics |
| State | Variables \(v\), TypeOK, Match, Diffuse, Inv3, **𝔸3**, **𝐋3** |
| Prediction | **𝔸2**, Predict, **ℙ2**, Inv2, **𝐂2**, winning JEPA clause |
| Formal guarantees | Spec, Inv1–Inv4, **𝐓1**, **𝐓2**, winning condition |

---

## 7. What this document is not

- Not a claim that any particular photonic chip has been built.
- Not a training recipe or loss schedule.
- Not a comparison of benchmark scores.
- Not an extension of Spec to systems other than Aria.

It is additional **specification of intent and distinction**, and **rationale** for the shape of the formal model. The admissible behaviors remain exactly those of Spec in [FORMAL_SPEC.md](FORMAL_SPEC.md).
