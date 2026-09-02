//! Closed operator JSON over the Aria telemetry transform.
//!
//! Each catalog binary is a distinct crate that includes its own `spec.json`
//! and calls [`run_spec`]. AriA is the underlying transformer in every
//! binary: Φ is not copied, it is linked.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod dispatch;
mod envelope;
mod run;

pub use dispatch::{
    endpoint_by_binary_id, endpoint_by_operator, endpoint_by_package, WorkerEndpoint,
};
pub use envelope::{OperatorEnvelope, OperatorNode, OperatorRel, OperatorSpec};
pub use run::{run_binary, run_spec, OperatorError, RunOpts};

use clap::Parser;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::OnceLock;

/// Schema tag for the operator envelope (Binary Repository v1 / sheet 09).
pub const OPERATOR_ENVELOPE_V1: &str = "aria-operator-envelope-v1";
/// Closed envelope semver carried on every operator document.
pub const OPERATOR_SCHEMA_VERSION: &str = "1.0.0";
/// Frozen catalog shipped with this crate (all 535 rows).
pub const CATALOG_JSON: &str = include_str!("../catalog/operators.json");

static CATALOG: OnceLock<Vec<OperatorSpec>> = OnceLock::new();

/// Frozen catalog (535 rows). Parsed once per process.
#[must_use]
pub fn catalog() -> &'static [OperatorSpec] {
    CATALOG
        .get_or_init(|| {
            serde_json::from_str(CATALOG_JSON)
                .expect("catalog/operators.json is generated valid JSON")
        })
        .as_slice()
}

/// Look up one catalog row.
#[must_use]
pub fn spec_by_id(binary_id: &str) -> Option<&'static OperatorSpec> {
    catalog().iter().find(|s| s.binary_id == binary_id)
}

#[derive(Parser, Debug)]
#[command(
    name = "aria-operator",
    about = "Run one catalog operator: Aria transform + closed operator JSON"
)]
struct Cli {
    /// Payload JSON. `-` or omitted reads stdin.
    #[arg(long)]
    r#in: Option<PathBuf>,
    /// Write the operator envelope here (stdout when omitted).
    #[arg(short, long)]
    out: Option<PathBuf>,
    /// Φ steps.
    #[arg(long, default_value_t = 32)]
    steps: u64,
    /// Seed. Equal payload + spec + seed ⇒ equal bytes.
    #[arg(long)]
    seed: Option<u64>,
    /// Bind this Observation Plan hash (hex). Unbound runs hash the payload.
    #[arg(long)]
    plan_hash: Option<String>,
    /// Coverage key from the sealed plan.
    #[arg(long)]
    requirement_id: Option<String>,
    /// Embed the Aria telemetry spine (sheet 09: optional). Off by default.
    #[arg(long)]
    telemetry: bool,
}

/// CLI entry used by every generated `[[bin]]`. Returns a process exit code.
#[must_use]
pub fn bin_main(spec_json: &str) -> i32 {
    match bin_main_inner(spec_json) {
        Ok(()) => 0,
        Err(code) => code,
    }
}

fn bin_main_inner(spec_json: &str) -> Result<(), i32> {
    let cli = Cli::parse();
    let payload = read_payload(cli.r#in.as_deref()).map_err(|e| {
        eprintln!("aria-operator: {e}");
        3
    })?;
    let opts = RunOpts {
        steps: cli.steps,
        seed: cli.seed,
        plan_hash: cli.plan_hash,
        requirement_id: cli.requirement_id,
        include_telemetry: cli.telemetry,
        ..RunOpts::default()
    };

    let envelope = run_spec(spec_json, &payload, &opts).map_err(|e| {
        eprintln!("aria-operator: {e}");
        e.exit_code()
    })?;
    let bytes = serde_json::to_vec(&envelope).map_err(|e| {
        eprintln!("aria-operator: serialize: {e}");
        2
    })?;
    write_sink(cli.out.as_deref(), &bytes).map_err(|e| {
        eprintln!("aria-operator: {e}");
        3
    })?;
    Ok(())
}

fn read_payload(path: Option<&std::path::Path>) -> io::Result<Vec<u8>> {
    match path {
        None => {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf)?;
            Ok(buf)
        }
        Some(p) if p.as_os_str() == "-" => {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf)?;
            Ok(buf)
        }
        Some(p) => std::fs::read(p),
    }
}

fn write_sink(path: Option<&std::path::Path>, bytes: &[u8]) -> io::Result<()> {
    match path {
        None => {
            io::stdout().write_all(bytes)?;
            Ok(())
        }
        Some(p) => {
            std::fs::write(p, bytes)?;
            Ok(())
        }
    }
}

/// `ExitCode` wrapper for callers that prefer the std type.
#[must_use]
pub fn bin_exit(spec_json: &str) -> ExitCode {
    ExitCode::from(u8::try_from(bin_main(spec_json)).unwrap_or(2))
}
