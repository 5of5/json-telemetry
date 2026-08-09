# Aria — Performance notes

**Phase 4 deliverable.** Measured numbers for the reference (simulated) backends,
plus the scaling that follows from the Spec's own structure.

Reproduce with:

```bash
cargo build --release -p aria-engine
aria bench --n-modes 16,64,256,512,1024 --steps 1000
aria bench --n-modes 64,256 --steps 1000 --with-gates
```

---

## Measurements

Apple M2 Max, `rustc 1.97.1`, `--release`, single thread, `dim(Z) = 64`,
schedule `opmd`, 1000 steps (= 250 Φ-cycles).

| N | setup (ms) | run (ms) | steps/s |
|---:|---:|---:|---:|
| 16 | 0.4 | 15.0 | 66,657 |
| 64 | 1.5 | 38.1 | 26,225 |
| 256 | 29.3 | 154.0 | 6,493 |
| 512 | 230.4 | 427.3 | 2,340 |
| 1024 | 2421.8 | 1364.8 | 733 |

Setup and run are timed separately on purpose: they scale differently, and
reporting one total hides which one matters.

### Setup — O(N³), one-off

Setup builds the reference unitary `U` as a product of `N` Householder
reflections. Each reflection is applied as a rank-1 update, `H·U = U − 2v(v†U)`,
which is O(N²); the product is therefore O(N³) overall.

An earlier revision formed `H` explicitly and multiplied, making setup O(N⁴):
N = 256 took **8.4 s** instead of the 29 ms above, a 288× penalty that was
invisible while setup and run were reported as a single number. Measured
setup ratios now track the cubic prediction: 64→256 is 4× in N and 19× in time,
256→512 is 2× and 7.9× (cache effects account for the excess over 8×).

`SimOptical` builds this matrix **once** at construction and reuses it for every
`OpticalStep`, so the cost is paid per engine, not per step.

### Run — O(N²) per Φ-cycle

Per-step cost is dominated by the `OpticalStep` mat-vec, O(N²), plus the
predictor's O(dim(Z)·N) embed. Doubling N quadruples the run time (256→512:
154 ms → 427 ms ≈ 2.8×; 512→1024: 427 → 1365 ≈ 3.2×), consistent with O(N²)
plus growing cache pressure.

### Operating gates — free

| N | run without gates | run with all 7 gates | overhead |
|---:|---:|---:|---:|
| 64 | 38.1 ms | 38.8 ms | 1.8% |
| 256 | 154.0 ms | 157.0 ms | 1.9% |

Enabling every Inv5–Inv11 gate costs about 2%. The monitor is O(1) per step
except Inv7, which runs a topological sort on Match steps only.

---

## What the Spec predicts

These are the asymptotic corollaries recorded in [ASYMPTOTICS.md](ASYMPTOTICS.md)
and `spec/Aria.tla` §11, restated against what the simulation actually does.

| Quantity | Spec corollary | Simulated reality |
|---|---|---|
| Φ-step depth | `O(log N + polylog M)` from ℙ1 | O(N²) — an electronic mat-vec, not interference |
| Energy / MAC | `O(N⁻¹)` from Inv1, 𝔸1, 𝔸4 | not modelled; simulation has no energy model |
| \|G\| after T trajectories | `O(T^β)`, β ≤ 1 | β = 1 under `match_policy = identity` (one node per Match) |
| Ranking latency | `O(1)` optical from 𝐋1 → 𝐂3 | O(N²) electronic |

**The O(log N) depth claim is a property of the optical substrate, not of this
code.** The simulated backend is an electronic stand-in; it reproduces `U`'s
*semantics* (exact unitarity, hence Inv1) and none of its *complexity*. Reading
the table above as evidence for or against ℙ1 would be a category error.

---

## Known costs, not yet optimised

| Cost | Where | Note |
|---|---|---|
| Graph clone per Match | `Engine::apply`, `SimGraphBackend::edit` | O(\|G\|·dim(Z)) per Match, so O(T²) over a run. Visible only for long runs with a growing graph; a persistent or copy-on-write graph store is the Phase 4+ fix. |
| Dense `Vec<Vec<f64>>` matrices | `SimPredictor`, `TrainedPredictor` | Row-of-rows layout costs a pointer chase per row. A flat buffer would help; not yet a bottleneck relative to `OpticalStep`. |
| Single-threaded | everywhere | The Φ-cycle is inherently sequential, but the `OpticalStep` mat-vec parallelises trivially if it ever dominates. |

---

## Backend swap (Exit₄)

Swapping the predictor requires no Spec change and no engine change:

```bash
# stub predictor
aria run --steps 1000 --schedule opmd

# learned predictor — same schedule, same invariants, same trace shape
aria run --steps 1000 --schedule opmd --predictor weights.json
```

The seam is `runner::engine_with(config, predictor)`. `crates/aria-backends/tests/trained.rs::swapping_the_predictor_needs_no_spec_change`
asserts that both backends produce the same action sequence, the same `t`, and
the same `|G|` — only the numbers inside the latent change.

An optical hardware backend would enter the same way, by implementing
`OpticalBackend`. Simulation remains the default.
