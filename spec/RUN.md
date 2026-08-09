# Running the Aria TLA+ specification

## Modules

| File | Role |
|------|------|
| `Aria.tla` | Discrete-state Spec (documentation-faithful) |
| `AriaMC.tla` | Finite carriers / operators for ASSUME |
| `AriaInstance.tla` | TLC entry (`INSTANCE Aria WITH …`) |
| `AriaInstance.cfg` | Spec + Safety invariants |

Docs: [FORMAL_SPEC.md](../docs/FORMAL_SPEC.md), [SAFETY.md](../docs/SAFETY.md), [CONTINUOUS_REFINEMENT.md](../docs/CONTINUOUS_REFINEMENT.md), [VALIDATION.md](../docs/VALIDATION.md).  
Index: [docs/README.md](../docs/README.md).

## Model check

```bash
cd spec
java -XX:+UseParallelGC -cp /path/to/tla2tools.jar tlc2.TLC \
  -config AriaInstance.cfg AriaInstance
```

**Verified (2026-08-09):** no error — 2616 distinct states, Inv1–Inv4 / Safety / TypeOK held under `t ≤ 3` MC bound.

## What TLC checks / does not check

| Checks | Does not check |
|--------|----------------|
| `Spec ⇒ □ Safety` (Inv1–4) | JEPA limit \(d\to 0\) |
| Stutter-safe (deadlock off) | Optical \(O(1)\) (𝐋1) |
| | Asymptotics 𝐂 / 𝐋 big-O |

## Fidelity

**MUST NOT** edit `Aria.tla` for training, hardware, or a fifth named action.  
Cite `docs/` when changing the machine Spec.  
The OPGROK harnesses are optional and external: a sibling checkout, not part of this repository (see [VALIDATION.md](../docs/VALIDATION.md) §2).
