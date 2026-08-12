//! Aria simulated backends — electronic simulation of Spec operators.
//!
//! All operators are trait implementations of aria-core's backend traits.
//! Phase 1 uses ideal/simulated operators; later phases add GPU/hardware backends.
//!
//! [`runner`] holds the single reference run path shared by the CLI, the Python
//! extension, and the WASM module (Phase 2 parity).

pub mod data;
pub mod diffuser;
pub mod graph;
pub mod optical;
pub mod predictor;
pub mod runner;
pub mod spectral;
pub mod trained;

pub use data::{dataset_from_bytes, dataset_from_file, encode_corpus, encode_window, FieldDataset};
pub use diffuser::SimDiffuser;
pub use graph::SimGraphBackend;
pub use optical::{FftOptical, RefOptical, SimOptical};
pub use predictor::SimPredictor;
pub use runner::{
    engine_with, run, run_with, sim_engine, RefPredictor, RunOutcome, RunSummary, SimEngine,
};
pub use spectral::{
    project_spectral, power_iteration, Matrix, SpectralError, SpectralReport, DEFAULT_ITERATIONS,
};
pub use trained::{PredictorWeights, TrainedPredictor, WeightsError};

/// Convenience constructor for a full simulated backend suite.
pub fn sim_backends(
    n_modes: usize,
    latent_dim: usize,
) -> (SimOptical, SimPredictor, SimGraphBackend, SimDiffuser) {
    (
        SimOptical::new(n_modes),
        SimPredictor::new(n_modes, latent_dim),
        SimGraphBackend::new(latent_dim),
        SimDiffuser::new(latent_dim),
    )
}
