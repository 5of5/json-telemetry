# Aria documentation

How to read this tree when **specifying and building** Aria.  
Runtime code is not in this repository yet; these docs *are* the build contract.

## Layers (authority)

| Layer | What | Docs |
|-------|------|------|
| **0–1** | Continuous geometry + prototypes (induce Spec; do not enlarge it) | [CONTINUOUS_REFINEMENT.md](CONTINUOUS_REFINEMENT.md) |
| **2** | Discrete admissible behaviors + Inv1–4 | [FORMAL_SPEC.md](FORMAL_SPEC.md), [SAFETY.md](SAFETY.md), [`../spec/`](../spec/) |

**Locks:** OpticalStep · Predict · Match · Diffuse · Stutter only.  
Inv1–4 meanings fixed: energy · contractivity · graph integrity · TypeOK.

## Reading order for implementers

1. **[FORMAL_SPEC.md](FORMAL_SPEC.md)** — axioms, actions, Next, Spec  
2. **[SAFETY.md](SAFETY.md)** — Inv1–4 + winning condition  
3. **[CONTINUOUS_REFINEMENT.md](CONTINUOUS_REFINEMENT.md)** — what \(U\), \(P\), Match, Diff *mean* continuously  
4. **[TRACES.md](TRACES.md)** — accept / reject trajectory families  
5. **[RATIONALE.md](RATIONALE.md)** — why not Softmax residual-stream GPT  
6. **[ASYMPTOTICS.md](ASYMPTOTICS.md)** · **[THEORIES.md](THEORIES.md)** — depth, ranking, empirical structure  
7. **[VALIDATION.md](VALIDATION.md)** + **[evidence/](evidence/)** — TLC + OPGROK receipts  
8. **[`../spec/RUN.md`](../spec/RUN.md)** — model-check commands  

## File map

| Path | Role |
|------|------|
| [FORMAL_SPEC.md](FORMAL_SPEC.md) | Discrete Spec: 𝔸…𝐂, TypeOK, Init, actions, Next, Spec |
| [SAFETY.md](SAFETY.md) | Primary safety Inv1–4 |
| [CONTINUOUS_REFINEMENT.md](CONTINUOUS_REFINEMENT.md) | Level 0–1 geometry, prototypes, refine/sample |
| [RATIONALE.md](RATIONALE.md) | Differentiation vs frontier models |
| [TRACES.md](TRACES.md) | Winning / rejected path shapes |
| [ASYMPTOTICS.md](ASYMPTOTICS.md) | Asymptotic corollaries |
| [THEORIES.md](THEORIES.md) | Theories from validation searches |
| [VALIDATION.md](VALIDATION.md) | TLC + OPGROK validation narrative |
| [evidence/](evidence/) | Slim JSON receipts only |
| [`../spec/`](../spec/) | TLA+ machine Spec |

## Build stack

| Layer | Language |
|-------|----------|
| Core Spec engine | **Rust** (native + WASM + C/Python ABI) |
| Training \(P\), \(I\) | **Python** (PyTorch/JAX) |
| Formal source of truth | **TLA+** (`spec/`) |

## Out of scope for this tree

Trained weights · photonic fab · product UI · OPGROK monorepo source (external) · campaign process notes.
