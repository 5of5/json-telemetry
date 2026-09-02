//! The work gateway — JSON CLI / nervous system.
//!
//! Workers (and Aria compiling a hosted command list) pass JSON. Each catalog
//! binary remains its own crate. This process is the only expand point.

use aria_operator::{
    catalog, commands_json, endpoint_by_binary_id, endpoint_by_operator, endpoint_by_package,
    execute_work, looks_like_work_command, run_binary, RunOpts, WorkRequest,
};
use clap::Parser;
use serde_json::Value;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    name = "work",
    about = "Nervous-system gateway: JSON-CLI for every catalog operator. One telemetry base; 535 crates."
)]
struct Cli {
    /// Catalog `BIN.*`.
    #[arg(long)]
    binary: Option<String>,
    /// Operator name.
    #[arg(long)]
    operator: Option<String>,
    /// Cargo package.
    #[arg(long)]
    package: Option<String>,
    /// Payload JSON, or a work command JSON. `-` / omit = stdin.
    #[arg(long)]
    r#in: Option<PathBuf>,
    /// Write the result here (stdout when omitted).
    #[arg(short, long)]
    out: Option<PathBuf>,
    /// Tab-separated catalog (human).
    #[arg(long)]
    list: bool,
    /// Hosted command list as JSON (what Aria compiles against).
    #[arg(long)]
    commands: bool,
    /// Treat stdin/`--in` as a work command JSON (`work` / `ops` / `commands`).
    #[arg(long)]
    json: bool,
    /// Φ steps.
    #[arg(long, default_value_t = 32)]
    steps: u64,
    /// Seed.
    #[arg(long)]
    seed: Option<u64>,
    /// Observation Plan bind.
    #[arg(long)]
    plan_hash: Option<String>,
    /// Coverage key.
    #[arg(long)]
    requirement_id: Option<String>,
    /// Embed the Aria telemetry spine. Off by default.
    #[arg(long)]
    telemetry: bool,
}

fn main() -> ExitCode {
    ExitCode::from(u8::try_from(run()).unwrap_or(2))
}

fn run() -> i32 {
    let cli = Cli::parse();
    if cli.commands {
        return dump(&commands_json(), cli.out.as_deref());
    }
    if cli.list {
        return list_tsv();
    }

    let raw = match read_payload(cli.r#in.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("work: {e}");
            return 3;
        }
    };

    let named = cli.binary.is_some() || cli.operator.is_some() || cli.package.is_some();
    let as_json = cli.json || (!named && looks_like_bytes(&raw));

    if as_json {
        return run_json(&raw, &cli);
    }

    let binary_id = match resolve(&cli) {
        Ok(id) => id,
        Err(msg) => {
            eprintln!("work: {msg}");
            return 2;
        }
    };
    let opts = opts_from(&cli);
    match run_binary(&binary_id, &raw, &opts) {
        Ok(env) => dump_bytes(&env, cli.out.as_deref()),
        Err(e) => {
            eprintln!("work: {e}");
            e.exit_code()
        }
    }
}

fn run_json(raw: &[u8], cli: &Cli) -> i32 {
    let req: WorkRequest = match serde_json::from_slice(raw) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("work: command JSON: {e}");
            return 2;
        }
    };
    let opts = opts_from(cli);
    match execute_work(&req, &opts) {
        Ok(v) => dump(&v, cli.out.as_deref()),
        Err(e) => {
            eprintln!("work: {e}");
            e.exit_code()
        }
    }
}

fn opts_from(cli: &Cli) -> RunOpts {
    RunOpts {
        steps: cli.steps,
        seed: cli.seed,
        plan_hash: cli.plan_hash.clone(),
        requirement_id: cli.requirement_id.clone(),
        include_telemetry: cli.telemetry,
        ..RunOpts::default()
    }
}

fn resolve(cli: &Cli) -> Result<String, String> {
    let n = usize::from(cli.binary.is_some())
        + usize::from(cli.operator.is_some())
        + usize::from(cli.package.is_some());
    if n != 1 {
        return Err(
            "name --binary, --operator, or --package; or pass JSON {work|ops|commands}"
                .into(),
        );
    }
    if let Some(id) = &cli.binary {
        endpoint_by_binary_id(id)
            .map(|e| e.binary_id)
            .ok_or_else(|| format!("unknown binary {id}"))
    } else if let Some(op) = &cli.operator {
        endpoint_by_operator(op)
            .map(|e| e.binary_id)
            .ok_or_else(|| format!("unknown operator {op}"))
    } else if let Some(pkg) = &cli.package {
        endpoint_by_package(pkg)
            .map(|e| e.binary_id)
            .ok_or_else(|| format!("unknown package {pkg}"))
    } else {
        Err("unreachable".into())
    }
}

fn looks_like_bytes(raw: &[u8]) -> bool {
    serde_json::from_slice::<Value>(raw)
        .ok()
        .is_some_and(|v| looks_like_work_command(&v))
}

fn list_tsv() -> i32 {
    let mut rows: Vec<_> = catalog().iter().collect();
    rows.sort_by(|a, b| a.binary_id.cmp(&b.binary_id));
    for s in rows {
        println!(
            "{}\t{}\t{}\t{}",
            s.binary_id, s.operator, s.package, s.layer
        );
    }
    0
}

fn dump(v: &Value, path: Option<&std::path::Path>) -> i32 {
    match serde_json::to_vec(v) {
        Ok(b) => match write_sink(path, &b) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("work: {e}");
                3
            }
        },
        Err(e) => {
            eprintln!("work: serialize: {e}");
            2
        }
    }
}

fn dump_bytes<T: serde::Serialize>(v: &T, path: Option<&std::path::Path>) -> i32 {
    match serde_json::to_vec(v) {
        Ok(b) => match write_sink(path, &b) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("work: {e}");
                3
            }
        },
        Err(e) => {
            eprintln!("work: serialize: {e}");
            2
        }
    }
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
        Some(p) => std::fs::write(p, bytes),
    }
}
