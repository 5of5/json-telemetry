use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::gates::GateConfig;
use crate::policy::{DiffPolicy, MatchPolicy};

/// Aria runtime configuration — TOML (PRD FR-10).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AriaConfig {
    /// Number of optical modes N (𝔸1)
    #[serde(default = "default_n_modes")]
    pub n_modes: usize,

    /// Latent dimension dim(Z)
    #[serde(default = "default_latent_dim")]
    pub latent_dim: usize,

    /// Contractivity tolerance ε (ℙ2, Inv2)
    #[serde(default = "default_eps")]
    pub eps: f64,

    /// Stutter budget K (𝐂5)
    #[serde(default = "default_stutter_k")]
    pub stutter_k: u64,

    /// Preferred schedule: "opmd" or custom action list
    #[serde(default = "default_schedule")]
    pub schedule: String,

    /// Match policy
    #[serde(default)]
    pub match_policy: MatchPolicy,

    /// Diff policy
    #[serde(default)]
    pub diff_policy: DiffPolicy,

    /// Conditioning
    #[serde(default)]
    pub condition: Condition,

    /// Which invariants to check: `["inv1","inv2","inv3","inv4"]`
    #[serde(default = "default_check_inv")]
    pub check_inv: Vec<String>,

    /// Optional operating gates Inv5–Inv11 (PRD §4.3). Off by default; these
    /// are monitors over a run, never Spec enlargement.
    #[serde(default)]
    pub gates: GateConfig,

    /// Maximum graph size |G| (nodes + edges)
    #[serde(default = "default_max_graph_size")]
    pub max_graph_size: usize,

    /// Seed for deterministic mode (None = random)
    pub seed: Option<u64>,

    /// Strict mode: invariant violations are hard errors
    #[serde(default = "default_true")]
    pub strict: bool,
}

fn default_n_modes() -> usize {
    256
}
fn default_latent_dim() -> usize {
    64
}
fn default_eps() -> f64 {
    1.0
}
fn default_stutter_k() -> u64 {
    2
}
fn default_schedule() -> String {
    "opmd".into()
}
fn default_check_inv() -> Vec<String> {
    vec![
        "inv1".into(),
        "inv2".into(),
        "inv3".into(),
        "inv4".into(),
    ]
}
fn default_max_graph_size() -> usize {
    10_000
}
fn default_true() -> bool {
    true
}

impl Default for AriaConfig {
    fn default() -> Self {
        AriaConfig {
            n_modes: default_n_modes(),
            latent_dim: default_latent_dim(),
            eps: default_eps(),
            stutter_k: default_stutter_k(),
            schedule: default_schedule(),
            match_policy: MatchPolicy::default(),
            diff_policy: DiffPolicy::default(),
            condition: Condition::default(),
            check_inv: default_check_inv(),
            gates: GateConfig::default(),
            max_graph_size: default_max_graph_size(),
            seed: None,
            strict: true,
        }
    }
}

impl AriaConfig {
    /// Quick test config with small dimensions.
    pub fn test_config() -> Self {
        AriaConfig {
            n_modes: 8,
            latent_dim: 16,
            eps: 1.0,
            stutter_k: 2,
            schedule: "opmd".into(),
            match_policy: MatchPolicy::Identity,
            diff_policy: DiffPolicy::Identity,
            condition: Condition::Token,
            check_inv: default_check_inv(),
            gates: GateConfig::default(),
            max_graph_size: 5000,
            seed: Some(42),
            strict: true,
        }
    }

    /// Parse from TOML string.
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}
