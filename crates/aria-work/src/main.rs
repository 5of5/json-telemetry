//! The work gateway. A worker names one catalog binary and passes JSON.
//! Each binary remains its own crate under `crates/operators/`.

use aria_operator::{
    catalog, endpoint_by_binary_id, endpoint_by_operator, endpoint_by_package, run_binary,
    RunOpts,
};
use clap::Parser;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "work",
    about = "Worker gateway: pass any catalog binary as work. One JSON telemetry base; 535 separate crates."
)]
struct Cli {
    /// Catalog `BIN.*` (the work definition).
    #[arg(long)]
    binary: Option<String>,
    /// Operator name (`PEOPLE`, `TAG.PERSON_FOUNDER`, …).
    #[arg(long)]
    operator: Option<String>,
    /// Cargo package (`aria-telemetry-people`).
    #[arg(long)]
    package: Option<String>,
    /// Payload JSON. `-` or omitted reads stdin.
    #[arg(long)]
    r#in: Option<PathBuf>,
    /// Write the operator envelope here (stdout when omitted).
    #[arg(short, long)]
    out: Option<PathBuf>,
    /// List every catalog binary and exit.
    #[arg(long)]
    list: bool,
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
    if cli.list {
        return list();
    }
    let binary_id = match resolve(&cli) {
        Ok(id) => id,
        Err(msg) => {
            eprintln!("work: {msg}");
            return 2;
        }
    };
    let payload = match read_payload(cli.r#in.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("work: {e}");
            return 3;
        }
    };
    let opts = RunOpts {
        steps: cli.steps,
        seed: cli.seed,
        plan_hash: cli.plan_hash,
        requirement_id: cli.requirement_id,
        include_telemetry: cli.telemetry,
        ..RunOpts::default()
    };
    match run_binary(&binary_id, &payload, &opts) {
        Ok(env) => match serde_json::to_vec(&env) {
            Ok(bytes) => match write_sink(cli.out.as_deref(), &bytes) {
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
        },
        Err(e) => {
            eprintln!("work: {e}");
            e.exit_code()
        }
    }
}

fn resolve(cli: &Cli) -> Result<String, String> {
    let n = usize::from(cli.binary.is_some())
        + usize::from(cli.operator.is_some())
        + usize::from(cli.package.is_some());
    if n != 1 {
        return Err(
            "name exactly one of --binary, --operator, or --package (or pass --list)"
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

fn list() -> i32 {
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
