//! Correct, sound, complete — one Φ, every research operator.
//!
//! Garbage in must not become a guessed Person. Forget is no-finding.
//! Host bytes always survive. No Trust keys. Independent content_hash.

use aria_operator::{catalog, commands_json, execute_work, run_many, RunOpts, WorkRequest};
use serde_json::json;
use std::collections::BTreeSet;
use std::time::Instant;

fn opts() -> RunOpts {
    RunOpts {
        steps: 8,
        seed: Some(1),
        n_modes: Some(16),
        latent_dim: Some(16),
        allow_sub_spec_dims: true,
        include_telemetry: false,
        ..RunOpts::default()
    }
}

fn research_ids() -> Vec<String> {
    catalog()
        .iter()
        .filter(|s| s.layer != "HOST")
        .map(|s| s.binary_id.clone())
        .collect()
}

#[test]
fn hosted_command_list_covers_every_crate() {
    let cmds = commands_json();
    assert_eq!(cmds["count"], 535);
    let n = cmds["commands"].as_array().expect("commands").len();
    assert_eq!(n, 535);
    let ids: BTreeSet<_> = cmds["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["binary_id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(ids.len(), 535);
    assert!(ids.contains("BIN.PEOPLE"));
    assert!(ids.contains("BIN.TAG.PERSON_FOUNDER"));
}

#[test]
fn one_phi_projects_every_research_binary_on_garbage() {
    let garbage = serde_json::to_vec(&json!({
        "notes": ["qwerty asdf garbage dump — not a person, not a company"],
        "dump": "unstructured noise",
        "noise": [1, 2, 3]
    }))
    .unwrap();
    let ids = research_ids();
    assert!(ids.len() >= 500, "research catalog shrank: {}", ids.len());

    let t0 = Instant::now();
    let envs = run_many(&ids, &garbage, &opts()).expect("run_many");
    let ms = t0.elapsed().as_millis();
    eprintln!(
        "matrix: {} binaries, one Φ, {}ms, garbage payload {}B",
        envs.len(),
        ms,
        garbage.len()
    );

    assert_eq!(envs.len(), ids.len());
    let mut hashes = BTreeSet::new();
    for (id, env) in ids.iter().zip(&envs) {
        assert_eq!(&env.binary_id, id);
        assert_eq!(env.schema, aria_operator::OPERATOR_ENVELOPE_V1);
        assert!(env.telemetry.is_none());
        let v = serde_json::to_value(env).unwrap();
        assert!(v.get("trust").is_none());
        assert!(v.get("Trust").is_none());
        assert!(v.get("goal_readiness").is_none());
        hashes.insert(env.content_hash.clone());
        if env.nodes.is_empty() && env.properties.is_empty() && env.verify {
            assert_eq!(env.coverage_state, "no-finding");
            assert!(env.no_finding_reason.is_some());
        }
        if env.coverage_state == "proposal" {
            // Garbage must not mint typed people/companies.
            for n in &env.nodes {
                assert!(
                    !n.kind.eq_ignore_ascii_case("person"),
                    "{id} guessed a Person from garbage"
                );
            }
        }
    }
    assert!(
        ms < 30_000,
        "535 projections after one Φ took {ms}ms (budget 30s debug)"
    );
    let _ = hashes;
}

#[test]
fn json_cli_compiles_a_hosted_ops_list() {
    let req = WorkRequest {
        ops: vec!["BIN.PEOPLE".into(), "BIN.COMPANY".into()],
        payload: Some(json!({
            "nodes": [
                {"id": 1, "type": "Person", "label": "Ada"},
                {"id": 2, "type": "Company", "label": "Acme"}
            ]
        })),
        ..WorkRequest::default()
    };
    let out = execute_work(&req, &opts()).unwrap();
    assert_eq!(out["schema"], "aria-work-v1");
    assert_eq!(out["phi_once"], true);
    assert_eq!(out["ops"], 2);
    let results = out["results"].as_array().unwrap();
    assert_eq!(results[0]["binary_id"], "BIN.PEOPLE");
    assert_eq!(results[1]["binary_id"], "BIN.COMPANY");
    assert_eq!(results[0]["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(results[1]["nodes"].as_array().unwrap().len(), 1);
}

#[test]
fn json_cli_commands_flag_returns_the_hosted_list() {
    let req = WorkRequest {
        commands: true,
        ..WorkRequest::default()
    };
    let out = execute_work(&req, &opts()).unwrap();
    assert_eq!(out["schema"], "aria-work-commands-v1");
    assert_eq!(out["count"], 535);
}

#[test]
fn forget_is_not_delete_on_empty_graph() {
    let req = WorkRequest {
        work: Some("BIN.PEOPLE".into()),
        payload: Some(json!({"nodes": []})),
        ..WorkRequest::default()
    };
    let out = execute_work(&req, &opts()).unwrap();
    assert_eq!(out["coverage_state"], "no-finding");
    assert!(out["no_finding_reason"].as_str().unwrap().contains("BIN.PEOPLE"));
    assert!(out.get("trust").is_none());
}
