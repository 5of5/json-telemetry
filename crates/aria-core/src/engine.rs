//! Aria Engine — Spec-faithful state machine.
//!
//! Implements Init, Next, Spec per FORMAL_SPEC.md.
//! Checks Inv1–4 after every apply.
//! The scheduler is policy, not Spec.

use std::fmt::Debug;

use crate::action::Action;
use crate::condition::Condition;
use crate::config::AriaConfig;
use crate::error::AriaError;
use crate::gates::{GateMonitor, GateReport};
use crate::graph::Graph;
use crate::invariants;
use crate::invariants::InvariantReport;
use crate::policy::{DiffPolicy, MatchPolicy};
use crate::scheduler::Scheduler;
use crate::state::State;
use crate::trace::Trace;

/// Optical backend trait — PRD §5.3
pub trait OpticalBackend: Debug + Send + Sync {
    /// Apply unitary step: ψ' = U_t(ψ)
    fn unitary_step(&self, t: u64, psi: &[num_complex::Complex64]) -> Vec<num_complex::Complex64>;
    /// Field energy: ‖ψ‖₂
    fn energy(&self, psi: &[num_complex::Complex64]) -> f64;
}

/// Predictor backend trait — PRD §5.3
pub trait Predictor: Debug + Send + Sync {
    /// Isometry I: H → Z
    fn embed(&self, psi: &[num_complex::Complex64]) -> Vec<f64>;
    /// Predictor P: Z × Condition → Z
    fn predict(&self, z: &[f64], a: Condition) -> Vec<f64>;
    /// Distance in latent space
    fn dist(&self, a: &[f64], b: &[f64]) -> f64;
}

/// Graph backend trait — PRD §5.3
pub trait GraphBackend: Debug + Send + Sync {
    /// Edit graph: G' = ED(G ⊕ z, policy, G*)
    fn edit(
        &self,
        g: &Graph,
        z: &[f64],
        policy: MatchPolicy,
        target: Option<&Graph>,
    ) -> Graph;
    /// GraphOK check
    fn ok(&self, g: &Graph) -> bool;
}

/// Diffuser backend trait — PRD §5.3
pub trait Diffuser: Debug + Send + Sync {
    /// Diffusion step: z' = Diff_G(z)
    fn diffuse(&self, g: &Graph, z: &[f64], policy: DiffPolicy) -> Vec<f64>;
}

/// Aria Engine — the Spec state machine with pluggable backends.
#[derive(Debug)]
pub struct Engine<O, P, G, D>
where
    O: OpticalBackend,
    P: Predictor,
    G: GraphBackend,
    D: Diffuser,
{
    config: AriaConfig,
    optical: O,
    predictor: P,
    graph_backend: G,
    diffuser: D,
}

impl<O, P, GB, D> Engine<O, P, GB, D>
where
    O: OpticalBackend,
    P: Predictor,
    GB: GraphBackend,
    D: Diffuser,
{
    /// Create a new engine with backends and config.
    pub fn new(config: AriaConfig, optical: O, predictor: P, graph_backend: GB, diffuser: D) -> Self {
        Engine {
            config,
            optical,
            predictor,
            graph_backend,
            diffuser,
        }
    }

    /// Init: create the initial state per FORMAL_SPEC §5.
    ///
    /// ψ = ψ₀, z = I(ψ₀), G = G₀, t = 0,
    /// prevRes = d(I(ψ₀), P(I(ψ₀), a(0)))
    ///
    /// Validates shapes up front: a field of the wrong length is a config
    /// error, not a downstream panic or a silently truncated mat-vec.
    pub fn init(
        &self,
        psi0: Vec<num_complex::Complex64>,
        g0: Graph,
        a0: Condition,
    ) -> Result<State, AriaError> {
        if self.config.n_modes == 0 {
            return Err(AriaError::Config("n_modes must be ≥ 1".into()));
        }
        if self.config.latent_dim == 0 {
            return Err(AriaError::Config("latent_dim must be ≥ 1".into()));
        }
        if psi0.len() != self.config.n_modes {
            return Err(AriaError::Config(format!(
                "ψ₀ has {} modes but config.n_modes = {}",
                psi0.len(),
                self.config.n_modes
            )));
        }
        for (i, c) in psi0.iter().enumerate() {
            if !c.re.is_finite() || !c.im.is_finite() {
                return Err(AriaError::Config(format!(
                    "ψ₀[{i}] = {c} is not finite"
                )));
            }
        }

        let energy_0 = self.optical.energy(&psi0);
        let z0 = self.predictor.embed(&psi0);
        if z0.len() != self.config.latent_dim {
            return Err(AriaError::Config(format!(
                "embed(ψ₀) has dim {} but config.latent_dim = {} — backend/config mismatch",
                z0.len(),
                self.config.latent_dim
            )));
        }
        let p0 = self.predictor.predict(&z0, a0);
        if p0.len() != self.config.latent_dim {
            return Err(AriaError::Config(format!(
                "P(z) has dim {} but config.latent_dim = {} — backend/config mismatch",
                p0.len(),
                self.config.latent_dim
            )));
        }
        let prev_res = self.predictor.dist(&z0, &p0);

        Ok(State {
            psi: psi0,
            z: z0,
            g: g0,
            t: 0,
            prev_res,
            energy_0,
        })
    }

    /// Apply a single named action to the state.
    ///
    /// Returns the new state on success, or an invariant violation.
    /// This is the Spec's Next relation turned into a deterministic function.
    // One match arm per named action, mirroring FORMAL_SPEC §6 one-to-one;
    // splitting the Next relation across helpers would hurt Spec readability.
    // The Match arm is restructured in plan_v0.2.0.md WS3 (edit-ops journal).
    #[allow(clippy::too_many_lines)]
    pub fn apply(
        &self,
        mut state: State,
        action: Action,
        a: Condition,
    ) -> Result<State, AriaError> {
        match action {
            Action::OpticalStep => {
                // ψ' = U_t(ψ); UNCHANGED ⟨z, G, t⟩ — FORMAL_SPEC §6.1
                let prev_psi = state.psi.clone();
                let prev_prev_res = state.prev_res;
                let pre_residual = self.compute_residual(&state, a);

                state.psi = self.optical.unitary_step(state.t, &state.psi);
                // TLA history obligation: prevRes' = Res(psi, z, t)
                state.prev_res = pre_residual;

                let post_residual = self.compute_residual(&state, a);
                let report = invariants::check_all(
                    &state,
                    post_residual,
                    self.config.eps,
                    self.config.n_modes,
                    self.config.latent_dim,
                );
                if self.config.strict && !report.all_ok() {
                    if let Some(v) = invariants::violation_from_report(
                        &report, action, state.energy(), state.energy_0,
                        post_residual, state.prev_res, self.config.eps,
                    ) {
                        state.psi = prev_psi;
                        state.prev_res = prev_prev_res;
                        return Err(AriaError::InvariantViolation(v));
                    }
                }
            }

            Action::Predict => {
                // z' = P(I(ψ), a_t); UNCHANGED ⟨ψ, G, t⟩ — FORMAL_SPEC §6.2
                let prev_z = state.z.clone();
                let prev_prev_res = state.prev_res;
                let pre_residual = self.compute_residual(&state, a);

                state.z = self.predictor.predict(&self.predictor.embed(&state.psi), a);
                state.prev_res = pre_residual;

                let post_residual = self.compute_residual(&state, a);
                let report = invariants::check_all(
                    &state,
                    post_residual,
                    self.config.eps,
                    self.config.n_modes,
                    self.config.latent_dim,
                );
                if self.config.strict && !report.all_ok() {
                    if let Some(v) = invariants::violation_from_report(
                        &report, action, state.energy(), state.energy_0,
                        post_residual, state.prev_res, self.config.eps,
                    ) {
                        state.z = prev_z;
                        state.prev_res = prev_prev_res;
                        return Err(AriaError::InvariantViolation(v));
                    }
                }
            }

            Action::Match => {
                // G' = ED(G ⊕ z, G*); UNCHANGED ⟨ψ, z, t⟩ — FORMAL_SPEC §6.3
                let prev_g = state.g.clone();
                let prev_prev_res = state.prev_res;
                let pre_residual = self.compute_residual(&state, a);

                let mut g_with_z = state.g.clone();
                g_with_z.add_node(
                    format!("z_{}", state.t),
                    state.z.clone(),
                    Some("latent".into()),
                );

                // Enforce max graph size
                if g_with_z.size() > self.config.max_graph_size {
                    return Err(AriaError::Schedule(format!(
                        "graph size {} exceeds max {}",
                        g_with_z.size(),
                        self.config.max_graph_size
                    )));
                }

                state.g = self.graph_backend.edit(
                    &g_with_z,
                    &state.z,
                    self.config.match_policy,
                    None,
                );
                state.prev_res = pre_residual;

                let post_residual = self.compute_residual(&state, a);
                let report = invariants::check_all(
                    &state,
                    post_residual,
                    self.config.eps,
                    self.config.n_modes,
                    self.config.latent_dim,
                );
                if self.config.strict && !report.all_ok() {
                    if let Some(v) = invariants::violation_from_report(
                        &report, action, state.energy(), state.energy_0,
                        post_residual, state.prev_res, self.config.eps,
                    ) {
                        state.g = prev_g;
                        state.prev_res = prev_prev_res;
                        return Err(AriaError::InvariantViolation(v));
                    }
                }
            }

            Action::Diffuse => {
                // z' = Diff_G(z); t' = t+1; UNCHANGED ⟨ψ, G⟩ — FORMAL_SPEC §6.4
                let prev_z = state.z.clone();
                let prev_t = state.t;
                let prev_prev_res = state.prev_res;
                let pre_residual = self.compute_residual(&state, a);

                state.z = self.diffuser.diffuse(&state.g, &state.z, self.config.diff_policy);
                state.t = state
                    .t
                    .checked_add(1)
                    .ok_or_else(|| AriaError::Backend("t overflowed u64".into()))?;
                state.prev_res = pre_residual;

                let post_residual = self.compute_residual(&state, a);
                let report = invariants::check_all(
                    &state,
                    post_residual,
                    self.config.eps,
                    self.config.n_modes,
                    self.config.latent_dim,
                );
                if self.config.strict && !report.all_ok() {
                    if let Some(v) = invariants::violation_from_report(
                        &report, action, state.energy(), state.energy_0,
                        post_residual, state.prev_res, self.config.eps,
                    ) {
                        state.z = prev_z;
                        state.t = prev_t;
                        state.prev_res = prev_prev_res;
                        return Err(AriaError::InvariantViolation(v));
                    }
                }
            }

            Action::Stutter => {
                // UNCHANGED all vars — TLA stuttering (including prevRes)
                let residual = self.compute_residual(&state, a);

                let report = invariants::check_all(
                    &state,
                    residual,
                    self.config.eps,
                    self.config.n_modes,
                    self.config.latent_dim,
                );
                if self.config.strict && !report.all_ok() {
                    if let Some(v) = invariants::violation_from_report(
                        &report, action, state.energy(), state.energy_0,
                        residual, state.prev_res, self.config.eps,
                    ) {
                        return Err(AriaError::InvariantViolation(v));
                    }
                }
            }
        }

        Ok(state)
    }

    /// Step one full Φ-cycle: OpticalStep → Predict → Match → Diffuse (𝐂4).
    ///
    /// This is the preferred schedule. Each sub-step checks invariants.
    /// Returns the state after the full cycle, or the first invariant violation.
    pub fn step_phi(&self, state: State, a: Condition) -> Result<State, AriaError> {
        let s = state;
        let s = self.apply(s, Action::OpticalStep, a)?;
        let s = self.apply(s, Action::Predict, a)?;
        let s = self.apply(s, Action::Match, a)?;
        let s = self.apply(s, Action::Diffuse, a)?;
        Ok(s)
    }

    /// Run the engine for a number of steps with a scheduler.
    ///
    /// Returns the final state and a trace of all steps.
    pub fn run(
        &self,
        state: State,
        scheduler: &mut Scheduler,
        steps: u64,
        a: Condition,
    ) -> Result<(State, Trace), AriaError> {
        let (state, trace, _) = self.run_monitored(state, scheduler, steps, a)?;
        Ok((state, trace))
    }

    /// Run the engine, additionally monitoring the optional Inv5–Inv11 gates.
    ///
    /// The monitor is a passive observer: it sees each completed step and can
    /// only report. Enabling a gate never changes which actions are taken, so
    /// the set of admissible behaviors is exactly the same as for [`run`].
    pub fn run_monitored(
        &self,
        mut state: State,
        scheduler: &mut Scheduler,
        steps: u64,
        a: Condition,
    ) -> Result<(State, Trace, GateReport), AriaError> {
        let mut trace = Trace::new(self.config.n_modes, self.config.latent_dim, self.config.eps);
        let mut monitor = GateMonitor::new(self.config.gates.clone());

        for _ in 0..steps {
            let action = scheduler.next_action_budgeted();
            let t_before = state.t;

            state = self.apply(state, action, a)?;

            let residual = self.compute_residual(&state, a);
            let energy = state.energy();
            trace.push(
                t_before,
                action,
                residual,
                energy,
                state.g.size(),
                &format!("{a:?}").to_lowercase(),
            );
            monitor.observe(action, &state, residual, self.config.eps);
        }

        Ok((state, trace, monitor.finish()))
    }

    /// Check all invariants on the current state without applying an action.
    pub fn check(&self, state: &State, a: Condition) -> InvariantReport {
        let residual = self.compute_residual(state, a);
        invariants::check_all(state, residual, self.config.eps, self.config.n_modes, self.config.latent_dim)
    }

    /// Compute JEPA residual: Res(ψ, z, t) = d(z, P(I(ψ), a_t))
    fn compute_residual(&self, state: &State, a: Condition) -> f64 {
        let embedded = self.predictor.embed(&state.psi);
        let predicted = self.predictor.predict(&embedded, a);
        self.predictor.dist(&state.z, &predicted)
    }

    /// Access config.
    pub fn config(&self) -> &AriaConfig {
        &self.config
    }
}
