//! Trained predictor backend — loads weights learned by the Phase 3 Python loop.
//!
//! The Spec constrains this backend in exactly one way, ℙ2: `E[Lip(P)] ≤ 1`.
//! Training is not trusted to respect that, so the loader *enforces* it by
//! spectral projection. A checkpoint that would break Inv2 is scaled down at
//! load time rather than being allowed to fail mid-run — training must never
//! bypass an invariant.
//!
//! # Why the bound is `ε/2` and not `1`
//!
//! Inv2 is `Res(ψ',z',t') ≤ Res(ψ,z,t) + ε`. The worst case is an OpticalStep,
//! which replaces ψ by an arbitrary unit vector while leaving z fixed:
//!
//! ```text
//! Res(ψ',z,t) = ‖z − P(I(ψ'))‖ ≤ ‖z‖ + ‖P(I(ψ'))‖ ≤ 2·Lip(P)·‖I‖·‖ψ₀‖
//! ```
//!
//! With `‖I‖ ≤ 1` and `‖ψ₀‖ = 1` this is `2·Lip(P)`, so `Lip(P) ≤ ε/2` makes
//! Inv2 hold unconditionally. [`TrainedPredictor::max_residual_jump`] reports
//! that bound so a config can be checked against it before a run.

use aria_engine_core::condition::Condition;
use aria_engine_core::engine::Predictor;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};

use crate::spectral::{
    power_iteration, project_spectral, SpectralError, SpectralReport, DEFAULT_ITERATIONS,
};

/// On-disk weight format written by `python/training/train_jepa.py`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictorWeights {
    /// Format tag; must be `aria-predictor-v1`.
    pub format: String,
    pub n_modes: usize,
    pub latent_dim: usize,
    /// Target Lipschitz bound for P. Keep `≤ eps/2` to guarantee Inv2.
    pub lipschitz_bound: f64,
    /// I : H → Z, shape [latent_dim × 2·n_modes], applied to [re₀, im₀, re₁, …].
    pub embed: Vec<Vec<f64>>,
    /// P : Z × Condition → Z, one [latent_dim × latent_dim] matrix per conditioning.
    pub predict: ConditionedWeights,
}

/// One predictor matrix per conditioning (𝐂2: conditioning, not architecture).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionedWeights {
    pub token: Vec<Vec<f64>>,
    pub diffusion: Vec<Vec<f64>>,
    pub world_model: Vec<Vec<f64>>,
}

/// Error cases when loading a checkpoint.
#[derive(Debug, thiserror::Error)]
pub enum WeightsError {
    #[error("unsupported weight format '{0}' (expected 'aria-predictor-v1')")]
    Format(String),
    #[error("{name}: expected shape [{want_rows} × {want_cols}], got [{got_rows} × {got_cols}]")]
    Shape {
        name: &'static str,
        want_rows: usize,
        want_cols: usize,
        got_rows: usize,
        got_cols: usize,
    },
    #[error("{0} contains a non-finite weight")]
    NonFinite(&'static str),
    #[error("lipschitz_bound must be finite and > 0, got {0}")]
    Bound(f64),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("spectral projection failed: {0}")]
    Spectral(#[from] SpectralError),
}

/// A `Predictor` backed by learned weights, with ℙ2 enforced at load time.
#[derive(Debug)]
pub struct TrainedPredictor {
    n_modes: usize,
    latent_dim: usize,
    lipschitz_bound: f64,
    embed: Vec<Vec<f64>>,
    token: Vec<Vec<f64>>,
    diffusion: Vec<Vec<f64>>,
    world_model: Vec<Vec<f64>>,
}

impl TrainedPredictor {
    /// Load a checkpoint from JSON, validating shapes and enforcing ℙ2.
    pub fn from_json(src: &str) -> Result<Self, WeightsError> {
        Self::from_weights(serde_json::from_str(src)?)
    }

    /// Load a checkpoint from a file path.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, WeightsError> {
        Self::from_json(&std::fs::read_to_string(path)?)
    }

    /// Build from an in-memory checkpoint, validating shapes and enforcing ℙ2.
    pub fn from_weights(w: PredictorWeights) -> Result<Self, WeightsError> {
        if w.format != "aria-predictor-v1" {
            return Err(WeightsError::Format(w.format));
        }
        if !w.lipschitz_bound.is_finite() || w.lipschitz_bound <= 0.0 {
            return Err(WeightsError::Bound(w.lipschitz_bound));
        }

        let input_dim = 2 * w.n_modes;
        check_shape("embed", &w.embed, w.latent_dim, input_dim)?;
        check_shape("predict.token", &w.predict.token, w.latent_dim, w.latent_dim)?;
        check_shape("predict.diffusion", &w.predict.diffusion, w.latent_dim, w.latent_dim)?;
        check_shape("predict.world_model", &w.predict.world_model, w.latent_dim, w.latent_dim)?;

        // 𝔸2: I is an isometry, so ‖I‖ ≤ 1. Project if training drifted above it.
        let embed = project_spectral(w.embed, 1.0)?;
        // ℙ2: Lip(P) ≤ lipschitz_bound, enforced rather than trusted.
        let token = project_spectral(w.predict.token, w.lipschitz_bound)?;
        let diffusion = project_spectral(w.predict.diffusion, w.lipschitz_bound)?;
        let world_model = project_spectral(w.predict.world_model, w.lipschitz_bound)?;

        Ok(TrainedPredictor {
            n_modes: w.n_modes,
            latent_dim: w.latent_dim,
            lipschitz_bound: w.lipschitz_bound,
            embed,
            token,
            diffusion,
            world_model,
        })
    }

    pub fn n_modes(&self) -> usize {
        self.n_modes
    }

    pub fn latent_dim(&self) -> usize {
        self.latent_dim
    }

    pub fn lipschitz_bound(&self) -> f64 {
        self.lipschitz_bound
    }

    /// Worst-case residual jump across one action: `2·Lip(P)·‖I‖·‖ψ₀‖`.
    ///
    /// Inv2 holds for every schedule when `eps ≥ max_residual_jump(‖ψ₀‖)`.
    pub fn max_residual_jump(&self, psi_norm: f64) -> Result<f64, SpectralError> {
        Ok(2.0
            * self.lipschitz_bound
            * power_iteration(&self.embed, DEFAULT_ITERATIONS)?
            * psi_norm)
    }

    /// The largest measured Lipschitz constant across the conditioned matrices.
    pub fn measured_lipschitz(&self) -> Result<f64, SpectralError> {
        let sigmas = [&self.token, &self.diffusion, &self.world_model]
            .into_iter()
            .map(|m| power_iteration(m, DEFAULT_ITERATIONS))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sigmas.into_iter().fold(0.0, f64::max))
    }

    /// σ_max per weight matrix — the Phase-1 audit surface (plan WS1).
    ///
    /// Under the load-time hard projection every reported value is ≤ its
    /// bound: `embed ≤ 1.0` (𝔸2) and the three conditioned matrices ≤
    /// `lipschitz_bound` (ℙ2) — ε = 0.0 in weight space (𝕋4). Surfaced by
    /// `aria check --predictor` and the run summary.
    pub fn spectral_report(&self) -> Result<SpectralReport, SpectralError> {
        Ok(SpectralReport {
            embed: power_iteration(&self.embed, DEFAULT_ITERATIONS)?,
            token: power_iteration(&self.token, DEFAULT_ITERATIONS)?,
            diffusion: power_iteration(&self.diffusion, DEFAULT_ITERATIONS)?,
            world_model: power_iteration(&self.world_model, DEFAULT_ITERATIONS)?,
        })
    }

    fn matrix_for(&self, a: Condition) -> &Vec<Vec<f64>> {
        match a {
            Condition::Token => &self.token,
            Condition::Diffusion => &self.diffusion,
            Condition::WorldModel => &self.world_model,
        }
    }
}

impl Predictor for TrainedPredictor {
    fn embed(&self, psi: &[Complex64]) -> Vec<f64> {
        let mut flat = Vec::with_capacity(psi.len() * 2);
        for c in psi {
            flat.push(c.re);
            flat.push(c.im);
        }
        mat_vec(&self.embed, &flat)
    }

    fn predict(&self, z: &[f64], a: Condition) -> Vec<f64> {
        mat_vec(self.matrix_for(a), z)
    }

    fn dist(&self, a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

fn check_shape(
    name: &'static str,
    m: &[Vec<f64>],
    rows: usize,
    cols: usize,
) -> Result<(), WeightsError> {
    if m.len() != rows || m.iter().any(|r| r.len() != cols) {
        return Err(WeightsError::Shape {
            name,
            want_rows: rows,
            want_cols: cols,
            got_rows: m.len(),
            got_cols: m.first().map_or(0, std::vec::Vec::len),
        });
    }
    if m.iter().flatten().any(|v| !v.is_finite()) {
        return Err(WeightsError::NonFinite(name));
    }
    Ok(())
}

fn mat_vec(m: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    m.iter()
        .map(|row| row.iter().zip(x).map(|(a, b)| a * b).sum())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(n: usize) -> Vec<Vec<f64>> {
        (0..n)
            .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect()
    }

    fn weights(latent_dim: usize, n_modes: usize, scale: f64, bound: f64) -> PredictorWeights {
        let input_dim = 2 * n_modes;
        let embed = (0..latent_dim)
            .map(|i| (0..input_dim).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect();
        let p: Vec<Vec<f64>> = identity(latent_dim)
            .into_iter()
            .map(|r| r.into_iter().map(|v| v * scale).collect())
            .collect();
        PredictorWeights {
            format: "aria-predictor-v1".into(),
            n_modes,
            latent_dim,
            lipschitz_bound: bound,
            embed,
            predict: ConditionedWeights {
                token: p.clone(),
                diffusion: p.clone(),
                world_model: p,
            },
        }
    }

    #[test]
    fn spectral_norm_of_identity_is_one() {
        assert!((power_iteration(&identity(6), DEFAULT_ITERATIONS).unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn spectral_norm_of_scaled_identity() {
        let m: Vec<Vec<f64>> = identity(5)
            .into_iter()
            .map(|r| r.into_iter().map(|v| v * 3.5).collect())
            .collect();
        assert!((power_iteration(&m, DEFAULT_ITERATIONS).unwrap() - 3.5).abs() < 1e-12);
    }

    #[test]
    fn loader_projects_an_over_lipschitz_checkpoint() {
        // Training produced Lip(P) = 5.0 but the bound is 0.49.
        let p = TrainedPredictor::from_weights(weights(8, 8, 5.0, 0.49)).unwrap();
        let lip = p.measured_lipschitz().unwrap();
        assert!(lip <= 0.49 + 1e-12, "loader must enforce P2, got {lip}");
    }

    #[test]
    fn loader_leaves_a_compliant_checkpoint_alone() {
        let p = TrainedPredictor::from_weights(weights(8, 8, 0.25, 0.49)).unwrap();
        assert!((p.measured_lipschitz().unwrap() - 0.25).abs() < 1e-12);
    }

    #[test]
    fn max_residual_jump_bounds_eps() {
        let p = TrainedPredictor::from_weights(weights(8, 8, 0.49, 0.49)).unwrap();
        // ‖ψ₀‖ = 1 ⇒ jump ≤ 2·0.49 = 0.98 ≤ ε = 1.0.
        assert!(p.max_residual_jump(1.0).unwrap() <= 1.0);
    }

    #[test]
    fn projection_is_entry_exact_on_the_cyclic_fixture() {
        // The migration fixture: σ(4·I) = 4 exactly under any start vector,
        // so the WS1 estimator must scale 4·I to 0.49 entry-exactly — the
        // same weights the pre-WS1 loader produced (observable behavior
        // stays identical on this fixture, plan WS1 tests-at-risk).
        let p = TrainedPredictor::from_weights(weights(8, 8, 4.0, 0.49)).unwrap();
        let report = p.spectral_report().unwrap();
        for m in [&p.token, &p.diffusion, &p.world_model] {
            for (i, row) in m.iter().enumerate() {
                for (j, v) in row.iter().enumerate() {
                    let want = if i == j { 0.49 } else { 0.0 };
                    assert!((v - want).abs() < 1e-12, "entry [{i}][{j}] = {v}");
                }
            }
        }
        assert!((report.token - 0.49).abs() < 1e-12);
        assert!((report.diffusion - 0.49).abs() < 1e-12);
        assert!((report.world_model - 0.49).abs() < 1e-12);
        assert!((report.embed - 1.0).abs() < 1e-12);
    }

    #[test]
    fn rejects_a_bad_format_tag() {
        let mut w = weights(4, 4, 0.5, 0.49);
        w.format = "keras-h5".into();
        assert!(matches!(
            TrainedPredictor::from_weights(w),
            Err(WeightsError::Format(_))
        ));
    }

    #[test]
    fn rejects_a_shape_mismatch() {
        let mut w = weights(4, 4, 0.5, 0.49);
        w.latent_dim = 5;
        assert!(matches!(
            TrainedPredictor::from_weights(w),
            Err(WeightsError::Shape { .. })
        ));
    }

    #[test]
    fn rejects_non_finite_weights() {
        let mut w = weights(4, 4, 0.5, 0.49);
        w.predict.token[0][0] = f64::NAN;
        assert!(matches!(
            TrainedPredictor::from_weights(w),
            Err(WeightsError::NonFinite(_))
        ));
    }

    #[test]
    fn json_round_trip() {
        let w = weights(4, 4, 0.3, 0.49);
        let src = serde_json::to_string(&w).unwrap();
        let p = TrainedPredictor::from_json(&src).unwrap();
        assert_eq!(p.latent_dim(), 4);
        assert_eq!(p.n_modes(), 4);
    }
}
