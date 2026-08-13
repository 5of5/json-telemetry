# plan_v0.2.0_remaining_tasks.md — Aria v0.2.0 remainder

**Parent plan:** `plan_v0.2.0.md` (Rev 2, 2026-08-12). This file does not replace it.
It is the execution remainder after WS0–WS3 closed with measured evidence.
**Executor:** the `aria` skill (`.devin/skills/aria/SKILL.md`).
**Authority:** spec PDF ≻ FORMAL_SPEC/SAFETY/Aria.tla ≻ PRD/OPERATING_SCHEDULE/TRACES ≻
parent plan ≻ this remainder.
**Program counter:** `docs/CHANGELOG.log`. Last closed goal: `G-v0.2.0-P3` (2026-08-13 04:28).
**Next session:** WS4.

**Verdict: `plan_v0.2.0.md` is NOT complete.** Four required workstreams and one stretch
workstream remain. Spec §8 predicates 1, 2, and 5 are unearned. Predicate 3 has a
merge-policy measurement that saturates (`β = 0` under `SimPredictor`) and must be
re-measured under a non-collapsing latent (Q-2026-08-13-7). Predicate 4 is earned
(WS2 optical + WS3 retrieval).

Do not re-open WS0–WS3. Do not re-implement anything in §1.

---

## 0. How to use this file

Same contract as the parent plan:

- one workstream per session
- CHANGELOG TASK before the first source edit
- relevant battery green before a GOAL-close
- no fake / dummy / placeholder code
- no closing a goal without measured evidence
- no sixth action, no decoder in `aria-core`, no `catch_unwind`, no silent config clamp
- `FftPlannerScalar` only; `libm` for state-affecting log/trig; seeded LCG only
- do not edit `assets/aria.mmd` unless the user explicitly asks

Session start (parent plan + skill): read `SKILL.md`, this remainder, the CHANGELOG tail,
`assets/aria.mmd`, and the reference library for the active WS.

---

## 1. Closed — do not redo

Evidence lives in `docs/CHANGELOG.log` on 2026-08-13. Workspace is still version **0.1.0**;
the bump is WS7.

| WS | Goal | Closed | What landed | Evidence (do not re-measure unless a later WS regresses it) |
|---|---|---|---|---|
| WS0 | preflight 𝒮 + decoder gate | 00:58 | `AriaConfig::validate()`, workspace deps, `check_no_decoder_in_core.sh` | invalid configs reject; G2 + decoder gates green |
| WS1 | G-v0.2.0-P1 | 01:21 | `spectral.rs` power-iteration + `project_spectral`; trained loader + Python agree | σ_max ≤ 1.0 on T ≥ 10⁴ trained run; vs-torch < 1e-6 |
| WS2 | G-v0.2.0-P2 | 02:18 | `FftOptical` (`FftPlannerScalar` + `libm` mask trig), `eps_energy`, goldens | unitarity 1.110e-16 @ N=4096; 10⁴-step drift 2.132e-14; CLI≡Python bytes; WASM \|Δ\| ≤ 6e-15 |
| WS3 | G-v0.2.0-P3 | 04:28 | Graph v2 + journal + in-repo HNSW + merge + growth fit + SoA + prefetch | Inv3 tests; p99 = **237.7 µs** @ \|V\|=10⁶; β = 0.0000 (caveat below); identity golden byte-stable |

WS3 §0 HNSW decision is final: in-repo `HnswIndex` at `crates/aria-backends/src/index.rs`.
`instant-distance` and `hnsw_rs` both fail `wasm32` on `getrandom`. `usearch` stays a
native-only optional feature. Do not reopen the crate audit.

WS3 β caveat (carried as Q-2026-08-13-7): 1024 merge-policy Φ-cycles under
`SimPredictor` keep `|V| = 1` at every log checkpoint (`β = 0`, `R² = 1`). The
predicate holds. The sphere-packing story is unstressed. WS5/WS6 re-measure.

Battery at WS3 close (do not treat as a substitute for per-WS batteries later):
163 Rust tests / 0 fail · clippy `-D warnings` clean · G2 + decoder OK · 1000-step
OPMD Inv-green · WASM \|Δ\| = 5.662e-15 · Python 10/10.

---

## 2. Remaining workstreams

```
WS4 (readouts) ── WS5 (ℒ_total + stop-grad) ──┐
                                              ├── WS6 (verify 10⁵) ── WS7 (package 0.2.0)
WS3 (closed) ─────────────────────────────────┘
                                              WS8 (stretch, after WS3; TRACN-gated)
```

WS4 is unblocked now. WS5 needs WS4 (readout to train) and WS1 (already closed).
WS6 needs WS2+WS3 (closed) and prefers WS5's trained backend when it exists.
WS7 is last. WS8 can start after WS3 (now) for the ingest core; market-map
alignment stays blocked on Q-2026-08-12-3.

### WS4 — Spec Phase 4a: decoupled readout heads (𝔸5, 𝕃5)

**Status:** not started. No `readout.rs`, no `aria emit`, no `train_readout.py`.
CLI commands today: `run | step | check | bench | dataset`.
**Spec:** 𝔸5, 𝕃5, ℂ7, §5.1. **Goal:** half of G-v0.2.0-P4 (WS5 closes the rest).
**Closes:** Q-2026-08-12-6 (candle disposition — plan default: defer, hand-rolled f64).

**Observe (2026-08-13, verified in tree):**

- Decoder gate is live: `scripts/check_no_decoder_in_core.sh` greps
  `readout|decoder|tokeniz|vocab|bpe|softmax|logits` over `crates/aria-core/src`.
  Readout code **must** live in `aria-backends` / CLI / WASM / `python/training`.
  The one sanctioned core escape is `vocab_size` in `AriaConfig::validate()`.
- `safetensors 0.8.0` and `tokenizers 0.23.1` are already pinned in
  `[workspace.dependencies]` (WS0). They are not consumed yet.
- Candle stays out of Φ (Q-2026-08-12-6). Do not pull it for this WS unless the
  session explicitly re-opens that question with evidence.

**Do:**

1. `crates/aria-backends/src/readout.rs` (never aria-core):
   - `DiscreteReadout`: layer-norm → linear(d → |V_o|, no bias) → temperature softmax
     — spec §5.1, hand-rolled f64.
   - `ContinuousReadout`: linear(d → d_a), validate `1 ≤ d_a ≤ d`.
   - Weights: safetensors + metadata `format = "aria-readout-v1"` (dims, vocab_size,
     temperature). Loader rejects bad shapes / non-finite values the way `trained.rs` does.
   - Tokenizer: `tokenizers` BPE trained on the corpus (`aria dataset --input`),
     vocab in [256, 128000]; JSON stored beside the weights.
2. CLI `aria emit --trace trace.jsonl --readout readout.safetensors [--tokenizer tok.json]`:
   decode the z-sequence of a **completed** run. Reads traces only. Must be
   structurally incapable of writing back into Φ (𝕃5: ∂Φ/∂y = 0 as an I/O boundary).
3. Optional WASM `emit` export — same I/O boundary.
4. Tests:
   - decoder gate stays green
   - a run with and without emission produces **byte-identical** traces
   - shapes / vocab bounds reject-with-detail (no silent clamp)
   - `aria emit` on a real 1000-step docs-corpus trace produces tokens (discrete path)

**Acceptance:** real 1000-step trace decodes to tokens; traces byte-identical
with/without emission; bounds enforced; zero readout symbols in aria-core.

**CHANGELOG minimum:** TASK (readout.rs + aria emit) · QUESTION-close (Q-2026-08-12-6).

---

### WS5 — Spec Phase 4b: 4-term ℒ_total on Δ³ (training upgrade)

**Status:** not started. `python/training/train_jepa.py:180` is still
`target = model.encode(psi_next)` — **no stop-gradient**. Loss is still 2-term
(JEPA + spectral). No RankMe. No safetensors export. No `from_safetensors`.
**Spec:** §6.1–6.4, ℙ2, ℙ6, 𝕋3. **Goal:** closes G-v0.2.0-P4 with WS4.
**Closes:** Q-2026-08-12-7 (Wilcoxon), Q-2026-08-12-8 (RankMe), Q-2026-08-13-5 (dataset/FFT).

**Observe:**

- `python/tests/test_training.py` **source is absent** (Q-2026-08-13-3). Only
  `__pycache__/test_training.cpython-312-pytest-9.1.1.pyc` remains.
  `python/tests/test_parity.py` is likewise absent; `test_parity_ws2.py` and
  `test_spectral_agreement.py` exist. Do **not** reconstruct tests from pyc.
  Author `test_training.py` from the spec/plan acceptance list, not from
  disassembled bytecode.
- `aria dataset` still generates trajectories with Householder while inference
  defaults to FFT at N ≥ 256 (Q-2026-08-13-5a). Fix or document in this WS.
- Stub predictor breaches Inv8 under FFT (Q-2026-08-13-5b). Trained P must be
  re-validated on FFT-generated trajectories.
- Docs corpus (153 KB real text) is the quality-gate data. No synthetic data
  in the persistence-beating gate.

**Do:**

1. **Stop-gradient (spec §6.1):** `target = model.encode(psi_next).detach()`.
2. **RankMe collapse gate** (Garrido et al., ICML 2023, arXiv:2210.02885;
   bibliography `jepa-and-world-models.md`):
   RankMe(Z) = exp(−Σₖ pₖ log pₖ), pₖ = σₖ(Z)/Σⱼσⱼ(Z) over held-out latent
   singular values. Abort if RankMe(Z) < `min_rankme_frac · d`, default 0.3
   (CLI-overridable). Log the chosen value and the measured curve as INSIGHT
   (closes Q-2026-08-12-8).
3. **ℒ_total = λ_JEPA·ℒ_JEPA + λ_NLL·ℒ_NLL + λ_Spectral·ℒ_Spectral + λ_Graph·ℒ_Graph**,
   λ ∈ Δ³ (Σλ = 1, λ ≥ 0 — ℙ6, already validated by `AriaConfig::validate()`):
   - ℒ_JEPA: batch-mean squared latent error + stop-gradient
   - ℒ_NLL: trains the **readout head only** on frozen z-trajectories from
     `aria run --output trace.jsonl`. Core predictor gradients never see ℒ_NLL
     (𝔸5 — optimizer param-group split + a test)
   - ℒ_Spectral: Σ max(0, σ_max(W_m) − 1)² + existing hard projection (WS1)
   - ℒ_Graph: Σ_{(u,v)∈E} max(0, d(ℳ(u), ℳ(v)) − γ_uv)² over graphs from
     merge-policy runs. Need `aria run --export-graph graph.json` (does not
     exist today). γ_uv default = merge τ
4. **Weights v2:** safetensors export `aria-predictor-v2`;
   `TrainedPredictor::from_safetensors`; keep JSON v1 loader; test both.
5. **Statistical certification:** ≥ 30 held-out trajectories; paired Wilcoxon
   signed-rank on per-trajectory mean residual vs persistence; require p < 0.01
   AND median improvement > 0; paired bootstrap 99% CI as robustness.
   `scipy` in training extras only, not the wheel.
6. Q-2026-08-13-5: switch `aria dataset` to the FFT path at N ≥ 256, or write
   the Householder/FFT mismatch down as an explicit documented deviation.

**Acceptance (Phase 4 gate = G-v0.2.0-P4):** docs-corpus training beats
persistence with p < 0.01; RankMe gate green; Rust loads v2 weights; 10⁴-step
trained run Inv-green with σ-audit ≤ 1.0; decoder-in-training test allows
readout training **only** in `train_readout.py` against frozen latents.

**CHANGELOG minimum:** TASK (ℒ_total + stop-grad + v2) · INSIGHT (RankMe curve,
closes Q-8) · INSIGHT (Wilcoxon p + effect size, closes Q-7) · GOAL-close P4.

---

### WS6 — Spec Phase 5: long-horizon verification harness (T ≥ 10⁵)

**Status:** not started. No `aria verify`. `Trace` is in-memory. No
`docs/evidence/v0.2.0_longrun_receipt.json`.
**Spec:** §8 predicates 1, 3, 4; TRACES X1–X5. **Goal:** G-v0.2.0-P5.
**Also carries:** Q-2026-08-12-1 (cheapest 10⁵ harness — this WS *is* the answer),
Q-2026-08-13-6 (remaining host-libm trig sites), Q-2026-08-13-7 (β re-measure).

**Do:**

1. `aria verify --steps 100000 [--gates all]`:
   - streaming JSONL sink — memory O(1) in T (no full-trace retention)
   - keep the in-memory `Trace` API for the 10³ suites
   - per-10⁴-step checkpoint: max energy drift, residual stats, |V|, σ_max, gate breaches
   - final receipt at `docs/evidence/v0.2.0_longrun_receipt.json`
2. Receipt schema is **`aria-verify-receipt-v1`** (do not copy the OPGROK
   `v3_live_receipt.json`):

   ```json
   {
     "format": "aria-verify-receipt-v1",
     "git_rev": "...", "config_hash": "...", "config": { },
     "steps": 100000, "schedule": "opmd", "condition": "token",
     "inv1_max_drift": 0.0, "inv2_violations": 0, "inv3_violations": 0, "inv4_violations": 0,
     "sigma_max_audit": { "token": 1.0, "diffusion": 1.0, "world_model": 1.0 },
     "graph": { "final_nodes": 0, "final_edges": 0, "measured_beta": 0.0, "beta_r2": 0.0 },
     "trace_audit": { "x1": 0, "x2": 0, "x3": 0, "x4": 0, "x5": 0, "family": "W2" },
     "wall_clock_s": 0.0, "steps_per_s": 0.0,
     "gates": { }
   }
   ```

3. X1–X5 finite-window audit (must NOT occur):

   | ID | Rule |
   |----|---|
   | X1 | any run of consecutive `S` longer than K (default 2) |
   | X2 | any window of W=64 steps with zero `O` |
   | X3 | any run of consecutive `D` longer than D-cap (default 8) |
   | X4 | an `M` while Res > ε with no `P` since the last residual-cold point (𝐂8) |
   | X5 | same check as X1, separate counter (TRACES parity) |

   Default scheduler must emit only W1 `(OPMD)^ω` / W2 `(OPMD S^{0..2})^ω`.
4. Nightly (local script, not GitHub Actions — CI is local-only per WS0):
   10⁵-step verify at N = 256 with **merge** policy. Per-PR stays at 10³.
5. Run once per conditioning (`token`, `diffusion`, `world_model`) for
   spec §8 predicate 5 (cross-modal invariance).
6. Re-measure β on this long merge-policy run (Q-2026-08-13-7). Prefer the
   WS5 trained predictor if it has landed; if WS6 runs first, record the
   SimPredictor saturation explicitly and do not call it the final β receipt.
7. Q-2026-08-13-6: audit remaining host-libm trig (`canonical_psi0`,
   Householder `make_unitary`, `data.rs` encoder, rustfft twiddle planning)
   and route state-affecting sites through `libm` so parity is seed-general,
   not just the exercised configs.

**Acceptance:** T ≥ 10⁵, Inv1–4 inside §0.2 tolerances, receipt archived,
X1–X5 counters zero (or explained), nightly script exists and has been run
once, PERFORMANCE.md updated with the long-run numbers.

**CHANGELOG minimum:** TASK (aria verify + receipt) · GOAL-close P5 ·
DOCS (evidence + PERFORMANCE).

---

### WS7 — Packaging, docs, version 0.2.0

**Status:** not started. Workspace version is still `0.1.0`.
`docs/BUILD_STATUS.md` v0.2.0 section still says "WS0 preflight (in progress)"
and does not record WS3. `docs/PERFORMANCE.md` has no FFT-scaling table, no
index-latency table, no 10⁵ numbers. `assets/aria.mmd` is still the v0.1.0
master (skill §5: recommend a refresh, do not edit).

**Do:**

1. Bump 0.1.0 → 0.2.0 in every `Cargo.toml` + `python/pyproject.toml`.
2. Re-verify wheel + WASM builds.
3. BUILD_STATUS: v0.2.0 phase table with exit evidence per WS (replace the
   stale "WS0 in progress" heading).
4. PERFORMANCE: FFT 256→1024 ratio, HNSW p50/p99 @ 10⁴/10⁵/10⁶ (WS3 table),
   long-run WS6 numbers.
5. CHANGELOG entries already exist per WS; do not rewrite them.
6. Recommend (CHANGELOG + user note) a v0.2.0 `aria.mmd` refresh. Do not
   touch the file.

**Acceptance:** full battery green on CLI, Python, and WASM; docs match
measured reality; version coherent everywhere.

**CHANGELOG minimum:** TASK (0.2.0 bump + builds) · DOCS (README /
BUILD_STATUS / PERFORMANCE) · MILESTONE (M-v0.2.0 reached).

---

### WS8 (stretch, gated) — Knowledge-graph ingestion toward TRACN

**Status:** not started. Implementable core is unblocked (WS3 typed graph exists).
Market-map output topologies stay blocked on Q-2026-08-12-3 (TRACN repo access).

**Do (core, now):**

1. `aria ingest --format json-nodelink|cypher-create --input kg.file --output seed_graph.json`
   - JSON node-link and a restricted Cypher subset
     (`CREATE (n:Type {props})`, `CREATE (a)-[:REL]->(b)`)
   - embeddings via the spectral encoder over node text
   - types map to §5.3 enums + `Custom`
   - result is Inv3-valid G₀ and `Engine::init` accepts it
2. Round-trip test on a **real public** KG sample. Never fabricate data.
3. Defer market-map topologies until `tracn.md` extraction queue items 1–4
   are filled. Record follow-up QUESTIONs, do not speculate.

**Acceptance:** ingested G₀ passes GraphOK + engine init; public-KG round-trip green.

---

## 3. Open questions — the forward queue

Closed this program: Q-2026-08-13-2 (HNSW crate — in-repo), Q-2026-08-13-4
(CLI≡Python FFT bytes). Everything below is still open.

| ID | Owns | Content |
|----|------|---------|
| Q-2026-08-12-1 | WS6 | Cheapest T ≥ 10⁵ harness. Answer = `aria verify`. Close when the receipt exists. |
| Q-2026-08-12-2 | WS8 / later | METIS-style partition of ingested KGs → TRACN market-map patches. Measurement: accuracy / brevity / clarity of emitted maps. |
| Q-2026-08-12-3 | WS8 blocker | Authenticated sync of `github.com/5of5/*` so `tracn.md` items 1–4 can be verified extracts. |
| Q-2026-08-12-4 | closed in practice (WS2) | `wasm_simd` OFF. Formal QUESTION-close still missing — stamp it in WS7. |
| Q-2026-08-12-5 | closed in practice (WS3) | Index is policy-layer; in-repo HNSW default; usearch native-optional. Formal QUESTION-close in WS7. |
| Q-2026-08-12-6 | WS4 | Candle deferred. Confirm in the WS4 TASK and QUESTION-close. |
| Q-2026-08-12-7 | WS5 | Wilcoxon signed-rank + bootstrap. Close with p and effect size. |
| Q-2026-08-12-8 | WS5 | RankMe collapse gate. Close with curve + chosen `min_rankme_frac`. |
| Q-2026-08-13-1 | WS7 | `.gitignore` untracks all of `docs/`, including CHANGELOG.log and `docs/evidence/`. Recommend negation rules so history and receipts survive clones. Owner decision. |
| Q-2026-08-13-3 | WS5 | `python/tests/test_parity.py` and `test_training.py` sources are gone (pyc only). Author new tests from the plan; do not decompile pyc. |
| Q-2026-08-13-5 | WS5 | (a) dataset generator still Householder; (b) stub P breaches Inv8 on FFT; (c) O(N) mask-gen dilutes the 256→1024 ratio. |
| Q-2026-08-13-6 | WS6 | Remaining host-libm trig sites. Route state-affecting ones through `libm`. |
| Q-2026-08-13-7 | WS5 / WS6 | Re-measure β under a non-collapsing latent. SimPredictor+τ=0.5 saturates \|V\|=1. |

---

## 4. Carry-forward defects (not new WS, not optional)

These are real gaps that a later WS must not paper over:

1. **β receipt is saturated.** WS3's OLS fit is honest and in-bound, but it is
   not the 𝕃3 stress test. WS6's merge-policy 10⁵ run (trained P if available)
   is the ship receipt for spec §8 predicate 3.
2. **Python test sources.** `test_parity_ws2.py` + `test_spectral_agreement.py`
   are the only live Python tests. The pre-WS0 20/20 suite is gone. WS5 authors
   `test_training.py` from scratch. Do not invent a `test_parity.py` from pyc.
3. **BUILD_STATUS / PERFORMANCE / version** lag the code. That is WS7, except
   each WS still writes its own BUILD_STATUS row as it lands (parent plan §3 table).
4. **`aria.mmd` is a v0.1.0 picture** of a v0.2.0-in-progress engine (FFT optical,
   typed graph, in-repo HNSW, journaled Match). Recommend a refresh at WS7.
   Do not edit it.
5. **ℙ5 margin is 12 µs** (p99 = 237.7 vs 250). Do not "optimize" by lowering
   `ef_search`. If a later change regresses p99 over 250, that is a WS3
   regression — re-run `aria bench --graph 1000000` and fix the constant, not
   the spec parameter.

---

## 5. Spec §8 — what is still unearned

v0.2.0 ships when all five hold with receipts. Current score:

| # | Predicate | Status |
|---|---|---|
| 1 | Inv1–4 across T ≥ 10⁵ | **OPEN** — WS6 |
| 2 | readout / latent prediction beats persistence, p < 0.01 | **OPEN** — WS5 |
| 3 | \|V\| = O(T^β), β ≤ 1 under merge | **PARTIAL** — WS3 number exists (β=0, saturated); WS6 is the real receipt |
| 4 | O(N log N) optical + < 250 µs @ 10⁶ retrieval | **EARNED** — WS2 + WS3 |
| 5 | token / diffusion / world_model, zero arch change, zero Inv faults | **PARTIAL** — 𝐂2 machinery exists; WS6 must run all three |

---

## 6. Verification matrix (remaining)

| WS | Commands (minimum) | New tests |
|----|---|---|
| WS4 | `cargo test`, decoder gate, `aria emit` on a real 1000-step trace | trace byte-equality with/without emission; shape/vocab bounds |
| WS5 | `python -m pytest python/tests -q` (venv) + 10⁴-step trained run | stop-grad collapse gate; Δ³ validation; Wilcoxon p < 0.01; v1+v2 loader |
| WS6 | `aria verify --steps 100000` (and once per conditioning) | receipt schema; X1–X5 scan; memory bound |
| WS7 | full battery, all surfaces, wheel + WASM | none (release checks) |
| WS8 | ingest round-trip + `Engine::init` | GraphOK on a real public KG |

Battery environment (unchanged from the parent plan, re-proven 2026-08-13):
rustc 1.97.1 · maturin 1.14.1 · wasm-bindgen 0.2.127 · node v24 · `.venv` at
repo root. Python: `source .venv/bin/activate && maturin develop --manifest-path
python/aria-py/Cargo.toml && python -m pytest python/tests -q` — always
`python -m pytest`.

---

## 7. Hard locks (copied, still inviolable)

1. Five actions only: OpticalStep, Predict, Match, Diffuse, Stutter.
2. No decoder in `aria-core`. Readouts live outside Φ.
3. Inv1–4 meanings fixed. Inv5–11 stay observers.
4. No `catch_unwind`. Rollback is journal replay.
5. No fake / dummy / toy / placeholder code.
6. No silent config clamping.
7. No skipping failing tests.
8. No GOAL-close without measured evidence.
9. No unsolicited `assets/aria.mmd` edits.
10. No photonic-hardware complexity claims from electronic sim.
11. `FftPlannerScalar`, never `FftPlanner::new()`.
12. Determinism: seeded LCG, `libm` for state-affecting log/trig, total order
    on candidates, single-threaded insert.
13. ID allocation is monotone and does not recycle after rollback.
14. CHANGELOG is the program counter. OODA on every increment.

---

## 8. First session on this remainder

WS4. Read this file + the WS4 section of `plan_v0.2.0.md` +
`.devin/skills/aria/references/rust-wasm-python.md` (safetensors / tokenizers)
+ the CHANGELOG tail. Add a TASK entry. Then `readout.rs`.
