//! Closed operator document (Binary Repository v1 / sheet 09) and the frozen
//! catalog row that defines one crate.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::OPERATOR_ENVELOPE_V1;

/// One catalog operator. Frozen into each crate's `spec.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorSpec {
    /// `BIN.*` identity.
    pub binary_id: String,
    /// Operator enum/name on the envelope.
    pub operator: String,
    /// Family / HOST / TRANSFORM / RESIDUAL / DEEP_TAG.
    pub layer: String,
    /// NODE / REL / TAG / PROP / ENTITY / …
    pub class: String,
    /// Parent family or envelope, when declared.
    #[serde(default)]
    pub parent: String,
    /// Catalog crate field (underscored).
    #[serde(rename = "crate")]
    pub crate_name: String,
    /// Cargo package name (hyphenated).
    pub package: String,
    /// `aria/people@v1` (or `pcvc/…@dev` for host tools).
    #[serde(default)]
    pub telemetry_fork: String,
    /// Sheet VERIFY T/F.
    pub verify: bool,
    /// Whether a Neo4j template is declared (availability, never Trust).
    pub neo4j_pass: bool,
    /// Result-limit when the sheet gave a number.
    #[serde(default)]
    pub default_limit: Option<usize>,
    /// Node kinds this operator may emit.
    #[serde(default)]
    pub node_types: Vec<String>,
    /// Relationship kinds this operator may emit.
    #[serde(default)]
    pub relationship_types: Vec<String>,
    /// Declared anchor tags.
    #[serde(default)]
    pub anchor_tags: Vec<String>,
    /// Plan result type this operator is allowed to satisfy.
    #[serde(default)]
    pub result_definition_ref: String,
    /// Catalog retrieval step (memory / lookup / lift / …).
    #[serde(default)]
    pub retrieval_step: String,
    /// TRANSFORM and HOST pass the Aria graph through without a type filter.
    #[serde(default)]
    pub pass_through: bool,
    /// PROP operators expose this property key only.
    #[serde(default)]
    pub property_key: Option<String>,
    /// Deep-tag taxonomy block.
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Residual/deep wave letter.
    #[serde(default)]
    pub wave: Option<String>,
}

impl OperatorSpec {
    /// Parse one catalog / spec.json object.
    pub fn from_catalog_value(v: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(v)
    }
}

/// One typed node on the operator envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorNode {
    /// Arena id (stable within the envelope).
    pub id: u64,
    /// Declared kind for this operator.
    #[serde(rename = "kind")]
    pub kind: String,
}

/// One typed relationship on the operator envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorRel {
    /// Source id.
    pub from: u64,
    /// Target id.
    pub to: u64,
    /// Declared rel type.
    #[serde(rename = "type")]
    pub rel_type: String,
}

/// Closed operator JSON. Unknown fields are never written.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorEnvelope {
    /// Always [`OPERATOR_ENVELOPE_V1`].
    pub schema: String,
    /// Catalog `BIN.*`.
    pub binary_id: String,
    /// Catalog operator name.
    pub operator: String,
    /// Envelope semver.
    pub schema_version: String,
    /// Catalog crate name.
    #[serde(rename = "crate")]
    pub crate_name: String,
    /// Observation Plan bind (hex). Payload hash when unbound.
    pub plan_hash: String,
    /// Coverage key. `"unbound"` when the host did not supply one.
    pub requirement_id: String,
    /// Subject ids looked up.
    pub subject_ids: Vec<u64>,
    /// Plan result type.
    #[serde(rename = "resultDefinitionRef")]
    pub result_definition_ref: String,
    /// Tags that justified this return.
    pub anchor_tags: Vec<String>,
    /// Neo4j availability only. Never Trust.
    pub neo4j_hit: bool,
    /// Operator-typed nodes.
    pub nodes: Vec<OperatorNode>,
    /// Operator-typed relationships.
    pub relationships: Vec<OperatorRel>,
    /// Operator-typed properties (PROP binaries; otherwise empty object).
    pub properties: Map<String, Value>,
    /// Sheet VERIFY bit after projection.
    pub verify: bool,
    /// `proposal` | `no-finding` | `limitation` | `truncation` | `failure`.
    pub coverage_state: String,
    /// Required when `nodes` is empty and state is no-finding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_finding_reason: Option<String>,
    /// Distinct from failure.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub limitations: Vec<String>,
    /// Hash of the operator payload (nodes + rels + properties), not notes.
    pub content_hash: String,
    /// Shared Aria spine. Not an API contract; PCVC/Supervisor may read it.
    pub telemetry: Value,
}

impl OperatorEnvelope {
    /// Format tag constructor helper.
    #[must_use]
    pub fn schema_tag() -> &'static str {
        OPERATOR_ENVELOPE_V1
    }
}
