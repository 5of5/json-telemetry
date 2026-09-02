//! Run Aria, then project a closed operator document.

use aria_engine_backends::ipo::{canonical_json, sha256_hex, IpoEdge, IpoNode, NodeRecord};
use aria_engine_backends::telemetry::{transform, TelemetryRequest};
use aria_engine_core::config::AriaConfig;
use aria_engine_core::error::AriaError;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::envelope::{OperatorEnvelope, OperatorNode, OperatorRel, OperatorSpec};
use crate::{OPERATOR_ENVELOPE_V1, OPERATOR_SCHEMA_VERSION};

/// Options a binary or test may pin. Defaults match `aria node`.
#[derive(Debug, Clone)]
pub struct RunOpts {
    /// Φ steps.
    pub steps: u64,
    /// Seed for byte-determinism.
    pub seed: Option<u64>,
    /// Optical modes. `None` uses `AriaConfig::default()`.
    pub n_modes: Option<usize>,
    /// Latent dim. `None` uses the config default.
    pub latent_dim: Option<usize>,
    /// Test-only 𝒮 escape.
    pub allow_sub_spec_dims: bool,
    /// Bind this plan hash. `None` hashes the payload.
    pub plan_hash: Option<String>,
    /// Coverage key.
    pub requirement_id: Option<String>,
    /// Embed `aria-telemetry-query-v1` under `telemetry` (sheet 09: optional).
    /// Default off: the Coordinator reads the vertical. Workers that need the
    /// spine pass `--telemetry`.
    pub include_telemetry: bool,
}

impl Default for RunOpts {
    fn default() -> Self {
        Self {
            steps: 32,
            seed: Some(1),
            n_modes: None,
            latent_dim: None,
            allow_sub_spec_dims: false,
            plan_hash: None,
            requirement_id: None,
            include_telemetry: false,
        }
    }
}

/// Operator-layer failure. Never a Trust write.
#[derive(Debug, thiserror::Error)]
pub enum OperatorError {
    /// Φ / ingest / config.
    #[error(transparent)]
    Aria(#[from] AriaError),
    /// Spec JSON would not parse.
    #[error("operator spec: {0}")]
    Spec(String),
    /// Envelope serialization.
    #[error("operator json: {0}")]
    Json(#[from] serde_json::Error),
    /// Catalog lookup missed.
    #[error("unknown binary: {0}")]
    UnknownBinary(String),
}

impl OperatorError {
    /// Coordinator exit vocabulary: 1 invariant, 2 config/spec, 3 unused here
    /// (I/O is the CLI's job).
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            OperatorError::Aria(AriaError::InvariantViolation(_)) => 1,
            OperatorError::Aria(AriaError::Io(_)) => 3,
            OperatorError::Aria(_)
            | OperatorError::Spec(_)
            | OperatorError::Json(_)
            | OperatorError::UnknownBinary(_) => 2,
        }
    }
}

/// Run the Aria transform and project this crate's closed operator JSON.
pub fn run_spec(
    spec_json: &str,
    payload: &[u8],
    opts: &RunOpts,
) -> Result<OperatorEnvelope, OperatorError> {
    let spec = parse_spec(spec_json)?;
    run_operator(&spec, payload, opts)
}

/// Gateway entry: run any catalog binary by `BIN.*` id.
pub fn run_binary(
    binary_id: &str,
    payload: &[u8],
    opts: &RunOpts,
) -> Result<OperatorEnvelope, OperatorError> {
    let spec = crate::spec_by_id(binary_id)
        .ok_or_else(|| OperatorError::UnknownBinary(binary_id.to_string()))?;
    run_operator(spec, payload, opts)
}

fn parse_spec(spec_json: &str) -> Result<OperatorSpec, OperatorError> {
    let v: Value = serde_json::from_str(spec_json)
        .map_err(|e| OperatorError::Spec(e.to_string()))?;
    OperatorSpec::from_catalog_value(v).map_err(|e| OperatorError::Spec(e.to_string()))
}

fn run_operator(
    spec: &OperatorSpec,
    payload: &[u8],
    opts: &RunOpts,
) -> Result<OperatorEnvelope, OperatorError> {
    let mut config = AriaConfig::default();
    if let Some(n) = opts.n_modes {
        config.n_modes = n;
    }
    if let Some(d) = opts.latent_dim {
        config.latent_dim = d;
    }
    config.seed = opts.seed;
    config.allow_sub_spec_dims = opts.allow_sub_spec_dims;

    let mut req = TelemetryRequest::new(payload.to_vec());
    req.config = config;
    req.steps = opts.steps;
    let telem = transform(req)?;
    let telem_value = serde_json::to_value(&telem)?;

    let (nodes, relationships, properties) = project(spec, &telem.graph.nodes, &telem.graph.edges, &telem.records);

    let truncated = spec
        .default_limit
        .is_some_and(|lim| nodes.len() > lim);
    let nodes = match spec.default_limit {
        Some(lim) if nodes.len() > lim => nodes.into_iter().take(lim).collect(),
        _ => nodes,
    };
    let keep: BTreeSet<u64> = nodes.iter().map(|n| n.id).collect();
    let relationships: Vec<OperatorRel> = if spec.pass_through {
        relationships
    } else {
        relationships
            .into_iter()
            .filter(|r| keep.contains(&r.from) && keep.contains(&r.to))
            .collect()
    };

    let (coverage_state, no_finding_reason, limitations) = if !spec.verify {
        (
            "limitation".to_string(),
            None,
            vec!["catalog VERIFY=F — operator listed but not product-accepted".into()],
        )
    } else if nodes.is_empty() && properties.is_empty() && !spec.pass_through {
        (
            "no-finding".to_string(),
            Some(format!(
                "{} declared types not present in the Aria graph (empty is valid)",
                spec.binary_id
            )),
            Vec::new(),
        )
    } else if truncated {
        ("truncation".to_string(), None, Vec::new())
    } else {
        ("proposal".to_string(), None, Vec::new())
    };

    let payload_obj = serde_json::json!({
        "nodes": nodes,
        "relationships": relationships,
        "properties": properties,
    });
    let content_hash = sha256_hex(&canonical_json(&payload_obj));
    let plan_hash = opts
        .plan_hash
        .clone()
        .unwrap_or_else(|| sha256_hex(payload));
    let requirement_id = opts
        .requirement_id
        .clone()
        .unwrap_or_else(|| "unbound".into());
    let subject_ids: Vec<u64> = nodes.iter().map(|n| n.id).collect();

    Ok(OperatorEnvelope {
        schema: OPERATOR_ENVELOPE_V1.into(),
        binary_id: spec.binary_id.clone(),
        operator: spec.operator.clone(),
        schema_version: OPERATOR_SCHEMA_VERSION.into(),
        crate_name: spec.crate_name.clone(),
        plan_hash,
        requirement_id,
        subject_ids,
        result_definition_ref: spec.result_definition_ref.clone(),
        anchor_tags: spec.anchor_tags.clone(),
        neo4j_hit: false,
        nodes,
        relationships,
        properties,
        verify: spec.verify,
        coverage_state,
        no_finding_reason,
        limitations,
        content_hash,
        telemetry: if opts.include_telemetry {
            Some(telem_value)
        } else {
            None
        },
    })
}

#[allow(clippy::too_many_lines)]
fn project(
    spec: &OperatorSpec,
    inodes: &[IpoNode],
    iedges: &[IpoEdge],
    records: &BTreeMap<u64, NodeRecord>,
) -> (Vec<OperatorNode>, Vec<OperatorRel>, Map<String, Value>) {
    if spec.pass_through {
        let nodes = inodes
            .iter()
            .map(|n| OperatorNode {
                id: n.id,
                kind: n.node_type.as_str().to_string(),
            })
            .collect();
        let rels = iedges
            .iter()
            .map(|e| OperatorRel {
                from: e.from,
                to: e.to,
                rel_type: e.edge_type.as_str().to_string(),
            })
            .collect();
        return (nodes, rels, Map::new());
    }

    let mut properties = Map::new();
    if let Some(key) = spec.property_key.as_deref() {
        for rec in records.values() {
            if let Some(v) = rec.properties.get(key) {
                properties.insert(key.to_string(), v.clone());
                break;
            }
        }
    }

    let allowed_nodes: Vec<String> = spec
        .node_types
        .iter()
        .map(|s| s.to_ascii_lowercase())
        .collect();
    let allowed_rels: Vec<String> = spec
        .relationship_types
        .iter()
        .map(|s| s.to_ascii_lowercase())
        .collect();
    let allowed_tags: Vec<String> = spec
        .anchor_tags
        .iter()
        .map(|s| s.to_ascii_lowercase())
        .collect();

    let nodes: Vec<OperatorNode> = inodes
        .iter()
        .filter_map(|n| {
            let kind = n.node_type.as_str();
            let rec = records.get(&n.id);
            if spec.class == "TAG" || spec.layer == "DEEP_TAG" {
                if tag_hits(rec, &allowed_tags, kind) {
                    return Some(OperatorNode {
                        id: n.id,
                        kind: kind.to_string(),
                    });
                }
                return None;
            }
            if spec.class == "PROP" {
                return None;
            }
            if spec.class == "REL" {
                return None;
            }
            if allowed_nodes.is_empty() {
                return None;
            }
            if matches_kind(kind, rec, &allowed_nodes) {
                Some(OperatorNode {
                    id: n.id,
                    kind: kind.to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    let keep: BTreeSet<u64> = if spec.class == "REL" {
        inodes.iter().map(|n| n.id).collect()
    } else {
        nodes.iter().map(|n| n.id).collect()
    };

    let mut relationships: Vec<OperatorRel> = iedges
        .iter()
        .filter_map(|e| {
            if !allowed_rels.is_empty() && !matches_token(e.edge_type.as_str(), &allowed_rels) {
                return None;
            }
            if spec.class == "REL" {
                return Some(OperatorRel {
                    from: e.from,
                    to: e.to,
                    rel_type: e.edge_type.as_str().to_string(),
                });
            }
            if keep.contains(&e.from) && keep.contains(&e.to) {
                Some(OperatorRel {
                    from: e.from,
                    to: e.to,
                    rel_type: e.edge_type.as_str().to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    // REL operators also list the endpoints they actually used.
    let nodes = if spec.class == "REL" {
        let used: BTreeSet<u64> = relationships
            .iter()
            .flat_map(|r| [r.from, r.to])
            .collect();
        inodes
            .iter()
            .filter(|n| used.contains(&n.id))
            .map(|n| OperatorNode {
                id: n.id,
                kind: n.node_type.as_str().to_string(),
            })
            .collect()
    } else {
        nodes
    };

    if spec.class == "REL" {
        relationships.retain(|r| {
            nodes.iter().any(|n| n.id == r.from) && nodes.iter().any(|n| n.id == r.to)
        });
    }

    (nodes, relationships, properties)
}

fn matches_token(got: &str, allowed: &[String]) -> bool {
    let g = got.to_ascii_lowercase();
    allowed.iter().any(|a| g == *a || g.replace('-', "_") == a.replace('-', "_"))
}

fn matches_kind(kind: &str, rec: Option<&NodeRecord>, allowed: &[String]) -> bool {
    if matches_token(kind, allowed) {
        return true;
    }
    if let Some(r) = rec {
        if r.label
            .as_deref()
            .is_some_and(|l| matches_token(l, allowed))
        {
            return true;
        }
        if let Some(Value::String(k)) = r.properties.get("kind").or_else(|| r.properties.get("type"))
        {
            return matches_token(k, allowed);
        }
    }
    false
}

fn tag_hits(rec: Option<&NodeRecord>, tags: &[String], kind: &str) -> bool {
    if tags.is_empty() {
        return false;
    }
    if matches_token(kind, tags) {
        return true;
    }
    let Some(r) = rec else {
        return false;
    };
    if r.label
        .as_deref()
        .is_some_and(|l| matches_token(l, tags))
    {
        return true;
    }
    match r.properties.get("tags") {
        Some(Value::Array(arr)) => arr.iter().any(|v| {
            v.as_str()
                .is_some_and(|s| matches_token(s, tags))
        }),
        Some(Value::String(s)) => matches_token(s, tags),
        _ => r
            .properties
            .get("tag")
            .and_then(Value::as_str)
            .is_some_and(|s| matches_token(s, tags)),
    }
}
