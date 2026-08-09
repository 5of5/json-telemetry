use aria_engine_core::engine::GraphBackend;
use aria_engine_core::graph::Graph;
use aria_engine_core::policy::MatchPolicy;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

/// Simulated graph backend.
///
/// Implements ElementaryEdit operations (add/delete/relabel/edge) per ℙ3.
/// Supports Match policies: identity, one_edit, rebuild_gstar.
#[derive(Debug)]
pub struct SimGraphBackend {
    latent_dim: usize,
}

impl SimGraphBackend {
    pub fn new(latent_dim: usize) -> Self {
        SimGraphBackend {
            latent_dim,
        }
    }

    pub fn with_seed(latent_dim: usize, _seed: u64) -> Self {
        SimGraphBackend {
            latent_dim,
        }
    }
}


impl GraphBackend for SimGraphBackend {
    fn edit(
        &self,
        g: &Graph,
        _z: &[f64],
        policy: MatchPolicy,
        _target: Option<&Graph>,
    ) -> Graph {
        let mut rng = StdRng::seed_from_u64(789);
        let mut g2 = g.clone();

        match policy {
            MatchPolicy::Identity => {
                // Leave G unchanged
            }
            MatchPolicy::OneEdit => {
                // Pick one random elementary edit
                let node_ids: Vec<String> = g2.nodes.keys().cloned().collect();
                let choice: u32 = rng.gen::<u32>() % 6;

                match choice {
                    0 if !node_ids.is_empty() => {
                        // Relabel a random node
                        let idx = rng.gen::<usize>() % node_ids.len();
                        let new_emb: Vec<f64> = (0..self.latent_dim)
                            .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
                            .collect();
                        g2.relabel_node(&node_ids[idx], new_emb);
                    }
                    1 | 2 => {
                        // Add a node
                        let id = format!("n{}", g2.node_count());
                        if !g2.nodes.contains_key(&id) {
                            let emb: Vec<f64> = (0..self.latent_dim)
                                .map(|_| rng.gen::<f64>() * 2.0 - 1.0)
                                .collect();
                            g2.add_node(id.clone(), emb, Some("latent".into()));

                            // Possibly add an edge from an existing node
                            if node_ids.len() > 1 {
                                let from = &node_ids[rng.gen::<usize>() % node_ids.len()];
                                g2.add_edge(from.clone(), id, "morph".into());
                            }
                        }
                    }
                    3 if !node_ids.is_empty() => {
                        // Delete a random node
                        let idx = rng.gen::<usize>() % node_ids.len();
                        g2.delete_node(&node_ids[idx]);
                    }
                    4 if node_ids.len() >= 2 => {
                        // Add an edge
                        let a = &node_ids[rng.gen::<usize>() % node_ids.len()];
                        let b = &node_ids[rng.gen::<usize>() % node_ids.len()];
                        if a != b {
                            g2.add_edge(a.clone(), b.clone(), "morph".into());
                        }
                    }
                    _ if !g2.edges.is_empty() => {
                        // Delete a random edge
                        let edges: Vec<_> = g2.edges.iter().cloned().collect();
                        let idx = rng.gen::<usize>() % edges.len();
                        g2.delete_edge(&edges[idx].from, &edges[idx].to, &edges[idx].edge_type);
                    }
                    _ => {} // fallback: no-op if conditions not met
                }
            }
            MatchPolicy::RebuildGStar => {
                // Rebuild to empty target (G* not configured in Phase 1 stubs)
                // In practice, Match rebuilds toward G*; here we use a fresh empty graph
                g2 = Graph::empty();
            }
        }

        g2
    }

    fn ok(&self, g: &Graph) -> bool {
        g.ok(self.latent_dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_edit_preserves_graph() {
        let backend = SimGraphBackend::new(4);
        let mut g = Graph::empty();
        g.add_node("a".into(), vec![1.0, 0.0, 0.0, 0.0], None);
        g.add_node("b".into(), vec![0.0, 1.0, 0.0, 0.0], None);
        g.add_edge("a".into(), "b".into(), "morph".into());

        let g2 = backend.edit(&g, &[1.0, 0.0, 0.0, 0.0], MatchPolicy::Identity, None);
        assert_eq!(g.node_count(), g2.node_count());
        assert_eq!(g.edge_count(), g2.edge_count());
        assert!(backend.ok(&g2));
    }

    #[test]
    fn one_edit_preserves_graph_ok() {
        let backend = SimGraphBackend::new(4);
        let mut g = Graph::empty();
        g.add_node("a".into(), vec![1.0, 0.0, 0.0, 0.0], None);
        g.add_node("b".into(), vec![0.0, 1.0, 0.0, 0.0], None);
        g.add_edge("a".into(), "b".into(), "morph".into());

        let g2 = backend.edit(&g, &[0.5; 4], MatchPolicy::OneEdit, None);
        assert!(backend.ok(&g2));
    }
}
