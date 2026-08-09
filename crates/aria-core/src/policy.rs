use serde::{Deserialize, Serialize};

/// Match policy for ED (graph edit distance) — ℙ3
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchPolicy {
    /// Leave G unchanged
    #[default]
    Identity,
    /// Apply one elementary edit (add/delete/relabel node or edge)
    OneEdit,
    /// Rebuild G to match target G*
    RebuildGStar,
}

/// Diffusion policy for Diff_G(z)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffPolicy {
    /// Leave z unchanged
    #[default]
    Identity,
    /// Flip/perturb latent
    Flip,
    /// Graph-conditioned diffusion step
    GraphConditioned,
}
