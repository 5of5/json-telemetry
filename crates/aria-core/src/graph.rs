use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A node in the experience graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node identifier
    pub id: String,
    /// Embedding in latent space Z
    pub embedding: Vec<f64>,
    /// Optional node type label
    pub node_type: Option<String>,
}

/// A typed edge between two nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
}

/// Experience/thought graph G — 𝔸3
///
/// Nodes carry embeddings in Z (latent space).
/// Edges are typed morphisms.
/// Only Match mutates G.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: HashSet<GraphEdge>,
}

impl Graph {
    /// Create an empty graph.
    pub fn empty() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashSet::new(),
        }
    }

    /// Create a seed graph with a single root node.
    pub fn seed(node: GraphNode) -> Self {
        let mut g = Graph::empty();
        g.nodes.insert(node.id.clone(), node);
        g
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Total size |G| = |V| + |E|.
    pub fn size(&self) -> usize {
        self.node_count() + self.edge_count()
    }

    /// GraphOK — Inv3: typed nodes/edges, embeddings in Z.
    ///
    /// Checks:
    /// - Every edge endpoint is an existing node
    /// - All embeddings have matching dimension
    /// - All embeddings are finite (in Z)
    pub fn ok(&self, latent_dim: usize) -> bool {
        // Every edge endpoint must be a node
        for edge in &self.edges {
            if !self.nodes.contains_key(&edge.from) || !self.nodes.contains_key(&edge.to) {
                return false;
            }
        }

        // All embeddings must have matching dimension and be finite
        for node in self.nodes.values() {
            if node.embedding.len() != latent_dim {
                return false;
            }
            for &v in &node.embedding {
                if !v.is_finite() {
                    return false;
                }
            }
        }

        true
    }

    /// Add a node with a latent embedding.
    pub fn add_node(&mut self, id: String, embedding: Vec<f64>, node_type: Option<String>) {
        self.nodes.insert(
            id.clone(),
            GraphNode {
                id,
                embedding,
                node_type,
            },
        );
    }

    /// Delete a node and all incident edges.
    pub fn delete_node(&mut self, id: &str) {
        self.nodes.remove(id);
        self.edges.retain(|e| e.from != id && e.to != id);
    }

    /// Add a typed edge.
    pub fn add_edge(&mut self, from: String, to: String, edge_type: String) -> bool {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return false;
        }
        self.edges.insert(GraphEdge {
            from,
            to,
            edge_type,
        });
        true
    }

    /// Delete an edge.
    pub fn delete_edge(&mut self, from: &str, to: &str, edge_type: &str) -> bool {
        self.edges.remove(&GraphEdge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type: edge_type.to_string(),
        })
    }

    /// Relabel a node's embedding.
    pub fn relabel_node(&mut self, id: &str, new_embedding: Vec<f64>) -> bool {
        if let Some(node) = self.nodes.get_mut(id) {
            node.embedding = new_embedding;
            true
        } else {
            false
        }
    }

    /// Replace the entire graph with a target (rebuild to G*).
    pub fn rebuild(&mut self, target: &Graph) {
        self.nodes = target.nodes.clone();
        self.edges = target.edges.clone();
    }

    /// Get a node's embedding.
    pub fn get_embedding(&self, id: &str) -> Option<&Vec<f64>> {
        self.nodes.get(id).map(|n| &n.embedding)
    }

    /// Whether the directed dependency spine is acyclic (Inv7 candidate gate).
    ///
    /// Kahn's algorithm: a DAG has a topological order covering every node.
    pub fn is_acyclic(&self) -> bool {
        let mut indegree: HashMap<&str, usize> =
            self.nodes.keys().map(|k| (k.as_str(), 0)).collect();
        let mut out: HashMap<&str, Vec<&str>> = HashMap::new();

        for e in &self.edges {
            // Ignore dangling edges here; Inv3 owns that failure mode.
            let (Some(d), true) = (
                indegree.get_mut(e.to.as_str()),
                self.nodes.contains_key(&e.from),
            ) else {
                continue;
            };
            *d += 1;
            out.entry(e.from.as_str()).or_default().push(e.to.as_str());
        }

        let mut queue: Vec<&str> = indegree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&k, _)| k)
            .collect();

        let mut visited = 0;
        while let Some(n) = queue.pop() {
            visited += 1;
            for &m in out.get(n).into_iter().flatten() {
                if let Some(d) = indegree.get_mut(m) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push(m);
                    }
                }
            }
        }

        visited == self.nodes.len()
    }
}

impl Default for Graph {
    fn default() -> Self {
        Graph::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_graph() -> Graph {
        let mut g = Graph::empty();
        g.add_node("n1".into(), vec![1.0, 0.0], None);
        g.add_node("n2".into(), vec![0.0, 1.0], None);
        g.add_edge("n1".into(), "n2".into(), "morph".into());
        g
    }

    #[test]
    fn graph_ok_valid() {
        let g = test_graph();
        assert!(g.ok(2));
    }

    #[test]
    fn graph_ok_wrong_dim() {
        let g = test_graph();
        assert!(!g.ok(3));
    }

    #[test]
    fn acyclic_chain_is_acyclic() {
        assert!(test_graph().is_acyclic());
    }

    #[test]
    fn a_cycle_is_detected() {
        let mut g = test_graph();
        g.add_edge("n2".into(), "n1".into(), "morph".into());
        assert!(!g.is_acyclic());
    }

    #[test]
    fn a_self_loop_is_a_cycle() {
        let mut g = test_graph();
        g.add_edge("n1".into(), "n1".into(), "morph".into());
        assert!(!g.is_acyclic());
    }

    #[test]
    fn graph_ok_dangling_edge() {
        let mut g = test_graph();
        g.edges.insert(GraphEdge {
            from: "n1".into(),
            to: "ghost".into(),
            edge_type: "morph".into(),
        });
        assert!(!g.ok(2));
    }
}
