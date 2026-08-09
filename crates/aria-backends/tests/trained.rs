//! Phase 3 — the trained predictor must be Spec-admissible, not just accurate.
//!
//! These tests use a synthetic checkpoint so they run without PyTorch. The
//! end-to-end learning demonstration (held-out residual actually falls) lives in
//! `python/tests/test_training.py`.

use aria_engine_backends::runner::{self, RefPredictor};
use aria_engine_backends::trained::{ConditionedWeights, PredictorWeights};
use aria_engine_backends::TrainedPredictor;
use aria_engine_core::config::AriaConfig;

const N_MODES: usize = 8;
const LATENT_DIM: usize = 16;

/// A deterministic, well-shaped checkpoint whose P matrices have norm `scale`.
fn checkpoint(scale: f64, bound: f64) -> PredictorWeights {
    let input_dim = 2 * N_MODES;

    // I: a partial isometry — the first `LATENT_DIM` rows of the identity.
    let embed: Vec<Vec<f64>> = (0..LATENT_DIM)
        .map(|i| (0..input_dim).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();

    // P: a scaled cyclic permutation, so its spectral norm is exactly `scale`.
    let p: Vec<Vec<f64>> = (0..LATENT_DIM)
        .map(|i| {
            (0..LATENT_DIM)
                .map(|j| if j == (i + 1) % LATENT_DIM { scale } else { 0.0 })
                .collect()
        })
        .collect();

    PredictorWeights {
        format: "aria-predictor-v1".into(),
        n_modes: N_MODES,
        latent_dim: LATENT_DIM,
        lipschitz_bound: bound,
        embed,
        predict: ConditionedWeights {
            token: p.clone(),
            diffusion: p.clone(),
            world_model: p,
        },
    }
}

fn config() -> AriaConfig {
    AriaConfig {
        n_modes: N_MODES,
        latent_dim: LATENT_DIM,
        ..AriaConfig::test_config()
    }
}

#[test]
fn trained_predictor_runs_1000_opmd_steps_green() {
    let trained = TrainedPredictor::from_weights(checkpoint(0.49, 0.49)).unwrap();
    assert!(trained.max_residual_jump(1.0) <= config().eps);

    let outcome = runner::run_with(config(), 1000, RefPredictor::Trained(trained)).unwrap();

    assert!(
        outcome.summary.invariants_ok,
        "trained weights violated an invariant: {:?}",
        outcome.summary.failures
    );
    assert_eq!(outcome.summary.t, 250);
    assert_eq!(outcome.summary.action_sequence, "OPMD".repeat(250));
}

#[test]
fn swapping_the_predictor_needs_no_spec_change() {
    // Exit4 in miniature: same config, same schedule, different backend.
    let stub = runner::run(config(), 200).unwrap();
    let trained = TrainedPredictor::from_weights(checkpoint(0.4, 0.49)).unwrap();
    let learned = runner::run_with(config(), 200, RefPredictor::Trained(trained)).unwrap();

    assert!(stub.summary.invariants_ok);
    assert!(learned.summary.invariants_ok);
    assert_eq!(stub.summary.action_sequence, learned.summary.action_sequence);
    assert_eq!(stub.summary.t, learned.summary.t);
    assert_eq!(stub.summary.graph_size, learned.summary.graph_size);
}

#[test]
fn an_over_lipschitz_checkpoint_is_projected_not_trusted() {
    // Training claims Lip(P) = 4.0 while declaring a 0.49 bound. The loader
    // must clamp it, and the run must still be green.
    let trained = TrainedPredictor::from_weights(checkpoint(4.0, 0.49)).unwrap();
    assert!(trained.measured_lipschitz() <= 0.49 + 1e-9);

    let outcome = runner::run_with(config(), 400, RefPredictor::Trained(trained)).unwrap();
    assert!(
        outcome.summary.invariants_ok,
        "{:?}",
        outcome.summary.failures
    );
}

#[test]
fn all_conditionings_share_the_one_architecture() {
    // 𝐂2 / A4: conditioning selects a matrix, never a second engine.
    for name in ["token", "diffusion", "world_model"] {
        let mut cfg = config();
        cfg.condition = runner::parse_condition(name).unwrap();
        let trained = TrainedPredictor::from_weights(checkpoint(0.45, 0.49)).unwrap();
        let outcome = runner::run_with(cfg, 200, RefPredictor::Trained(trained)).unwrap();
        assert!(
            outcome.summary.invariants_ok,
            "{name}: {:?}",
            outcome.summary.failures
        );
    }
}
