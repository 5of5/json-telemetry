//! Data-efficiency gate for operator JSON.
//!
//! The vertical (declared types only) must be cheaper than the shared
//! telemetry spine it sits on. Embeddings stay off unless asked. A PEOPLE
//! worker must not pay for COMPANY nodes.

use aria_operator::{catalog, run_spec, RunOpts};
use serde_json::{json, Value};
use std::time::Instant;

fn opts() -> RunOpts {
    RunOpts {
        steps: 8,
        seed: Some(1),
        n_modes: Some(16),
        latent_dim: Some(16),
        allow_sub_spec_dims: true,
        plan_hash: None,
        requirement_id: None,
    }
}

fn spec_json(id: &str) -> String {
    let spec = catalog()
        .iter()
        .find(|s| s.binary_id == id)
        .unwrap_or_else(|| panic!("missing {id}"));
    serde_json::to_string(spec).unwrap()
}

fn mixed_payload() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "nodes": [
            {"id": 1, "type": "Person", "label": "Ada", "notes": "founder"},
            {"id": 2, "type": "Company", "label": "Acme", "notes": "infra"},
            {"id": 3, "type": "Person", "label": "Bob", "notes": "engineer"}
        ],
        "edges": [
            {"from": 1, "to": 2, "type": "WORKS_AT"},
            {"from": 3, "to": 2, "type": "WORKS_AT"}
        ]
    }))
    .unwrap()
}

#[test]
fn operator_vertical_is_smaller_than_the_shared_telemetry_spine() {
    let payload = mixed_payload();
    let t0 = Instant::now();
    let people = run_spec(&spec_json("BIN.PEOPLE"), &payload, &opts()).unwrap();
    let company = run_spec(&spec_json("BIN.COMPANY"), &payload, &opts()).unwrap();
    let aria = run_spec(&spec_json("BIN.ARIA"), &payload, &opts()).unwrap();
    let elapsed_ms = t0.elapsed().as_millis();

    let people_bytes = serde_json::to_vec(&people).unwrap();
    let company_bytes = serde_json::to_vec(&company).unwrap();
    let aria_bytes = serde_json::to_vec(&aria).unwrap();
    let people_telem = serde_json::to_vec(&people.telemetry).unwrap();
    let people_vertical = serde_json::to_vec(&json!({
        "nodes": people.nodes,
        "relationships": people.relationships,
        "properties": people.properties,
    }))
    .unwrap();

    eprintln!(
        "efficiency: people_full={}B people_vertical={}B people_telem={}B company_full={}B aria_full={}B elapsed={}ms",
        people_bytes.len(),
        people_vertical.len(),
        people_telem.len(),
        company_bytes.len(),
        aria_bytes.len(),
        elapsed_ms
    );

    // Vertical is the cheap document the Coordinator reads first.
    assert!(
        people_vertical.len() < people_telem.len(),
        "operator vertical {}B must be smaller than nested telemetry {}B",
        people_vertical.len(),
        people_telem.len()
    );
    assert!(
        people_vertical.len() < 512,
        "tiny mixed payload vertical should stay under 512B, got {}",
        people_vertical.len()
    );

    // PEOPLE does not emit Company nodes; COMPANY does not emit Person nodes.
    assert_eq!(people.nodes.len(), 2);
    assert!(people.nodes.iter().all(|n| n.kind.eq_ignore_ascii_case("person")));
    assert_eq!(company.nodes.len(), 1);
    assert!(company
        .nodes
        .iter()
        .all(|n| n.kind.eq_ignore_ascii_case("company")));

    // Pass-through AriA returns the whole graph; a vertical is strictly smaller.
    assert!(aria.nodes.len() >= people.nodes.len() + company.nodes.len());
    assert!(people.nodes.len() < aria.nodes.len());

    // Default telemetry omits embeddings (G10).
    let omitted = people.telemetry["graph"]["embeddings_omitted"]
        .as_bool()
        .expect("embeddings_omitted present");
    assert!(omitted, "default operator run must omit embeddings");
    let telem: Value = people.telemetry.clone();
    if let Some(nodes) = telem["graph"]["nodes"].as_array() {
        for n in nodes {
            let emb = n.get("embedding").and_then(Value::as_array);
            assert!(
                emb.is_none() || emb.unwrap().is_empty(),
                "default envelope must not carry embedding vectors"
            );
        }
    }

    // Three small operators on a 3-node payload stay interactive.
    assert!(
        elapsed_ms < 5_000,
        "three operator runs on a tiny payload took {elapsed_ms}ms"
    );

    // Full envelope is telemetry-dominated, not vertical-dominated.
    assert!(
        people_telem.len() * 2 > people_bytes.len(),
        "nested telemetry should dominate the envelope ({} telem vs {} full)",
        people_telem.len(),
        people_bytes.len()
    );
}
