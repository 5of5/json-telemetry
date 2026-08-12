//! Aria CLI — Spec-faithful runtime runner.
//!
//! Usage:
//!   aria run --schedule opmd --steps 1000
//!   aria step --action OpticalStep --state state.json
//!   aria check --state state.json
//!
//! `run` delegates to `aria_engine_backends::runner::run`, the same code path
//! used by the Python extension and the WASM module (Phase 2 parity).

use aria_engine_backends::runner::{self, canonical_init, sim_engine, RefPredictor};
use aria_engine_backends::{SimPredictor, TrainedPredictor};
use aria_engine_core::action::Action;
use aria_engine_core::config::AriaConfig;
use aria_engine_core::gates::{Gate, GateConfig};
use aria_engine_core::scheduler::Scheduler;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "aria", version, about = "Aria — Ariadne Transformer runtime")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to TOML config file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the engine for N steps
    Run {
        /// Schedule string (default: "opmd")
        #[arg(long)]
        schedule: Option<String>,

        /// Number of steps
        #[arg(long, default_value = "100")]
        steps: u64,

        /// Epsilon tolerance
        #[arg(long)]
        eps: Option<f64>,

        /// Conditioning: token, diffusion, world_model
        #[arg(long)]
        condition: Option<String>,

        /// Number of optical modes
        #[arg(long)]
        n_modes: Option<usize>,

        /// Latent dimension
        #[arg(long)]
        latent_dim: Option<usize>,

        /// Stutter budget K
        #[arg(long)]
        stutter_k: Option<u64>,

        /// Seed for reproducibility
        #[arg(long)]
        seed: Option<u64>,

        /// Output trace file (JSONL)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Trained predictor weights (JSON from python/training/train_jepa.py).
        /// Omit to use the Phase 1 stub predictor.
        #[arg(long)]
        predictor: Option<PathBuf>,

        /// Optional operating gates Inv5–Inv11, e.g. "inv5,inv9" or "all".
        /// These are monitors, never Spec enlargement.
        #[arg(long)]
        gates: Option<String>,

        /// Exit non-zero if any enabled operating gate is breached
        #[arg(long)]
        strict_gates: bool,

        /// Disable strict invariant checking
        #[arg(long)]
        no_strict: bool,
    },

    /// Apply a single step to a state
    Step {
        /// Action: OpticalStep, Predict, Match, Diffuse, Stutter
        #[arg(long)]
        action: String,

        /// Conditioning
        #[arg(long)]
        condition: Option<String>,

        /// Path to state JSON
        #[arg(long)]
        state: Option<PathBuf>,

        /// N modes
        #[arg(long)]
        n_modes: Option<usize>,

        /// Latent dimension
        #[arg(long)]
        latent_dim: Option<usize>,

        /// Epsilon
        #[arg(long)]
        eps: Option<f64>,
    },

    /// Check invariants on a state
    Check {
        /// Path to state JSON
        #[arg(long)]
        state: PathBuf,

        /// Conditioning
        #[arg(long)]
        condition: Option<String>,

        /// Latent dimension
        #[arg(long)]
        latent_dim: Option<usize>,

        /// Trained checkpoint: load it, print the σ_max audit, and check
        /// against the trained backend that produced the state (plan WS1)
        #[arg(long)]
        predictor: Option<PathBuf>,
    },

    /// Measure Φ-cycle throughput across sizes (Phase 4 performance notes)
    Bench {
        /// Comma-separated N values to sweep
        #[arg(long, default_value = "16,64,256")]
        n_modes: String,

        /// Latent dimension
        #[arg(long, default_value = "64")]
        latent_dim: usize,

        /// Steps per measurement
        #[arg(long, default_value = "1000")]
        steps: u64,

        /// Also measure with every operating gate enabled
        #[arg(long)]
        with_gates: bool,
    },

    /// Export a training dataset for the Phase 3 JEPA loop.
    ///
    /// With `--input`, encodes a real corpus (text, code, any bytes) as optical
    /// fields — this is the production path. Without it, emits synthetic
    /// phase-ramp trajectories, which exist for smoke tests only and are not
    /// training data.
    Dataset {
        /// Real corpus file. Anything byte-readable: text, code, logs.
        #[arg(long)]
        input: Option<PathBuf>,

        /// Window stride in bytes (default: window size = non-overlapping)
        #[arg(long)]
        stride: Option<usize>,

        /// Number of synthetic trajectories (smoke-test path only)
        #[arg(long, default_value = "64")]
        trajectories: usize,

        /// Snapshots per synthetic trajectory (smoke-test path only)
        #[arg(long, default_value = "16")]
        length: usize,

        /// Number of optical modes (= bytes per window on the real-data path)
        #[arg(long)]
        n_modes: Option<usize>,

        /// Seed for the optical operator (synthetic path only)
        #[arg(long)]
        seed: Option<u64>,

        /// Output JSON file (stdout when omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    env_logger::init();
    let cli = Cli::parse();

    match real_main(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

// One block per subcommand (run/step/check/bench/dataset); WS6 of
// plan_v0.2.0.md adds `verify` and is the right moment to split this into
// per-subcommand handlers.
#[allow(clippy::too_many_lines)]
fn real_main(cli: Cli) -> Result<(), String> {
    let base = match cli.config {
        Some(ref path) => {
            let contents = fs::read_to_string(path)
                .map_err(|e| format!("failed to read config {}: {}", path.display(), e))?;
            AriaConfig::from_toml(&contents).map_err(|e| format!("failed to parse config: {e}"))?
        }
        None => AriaConfig::default(),
    };

    match cli.command {
        Commands::Run {
            schedule,
            steps,
            eps,
            condition,
            n_modes,
            latent_dim,
            stutter_k,
            seed,
            output,
            predictor,
            gates,
            strict_gates,
            no_strict,
        } => {
            let mut config = base;
            if let Some(v) = schedule {
                config.schedule = v;
            }
            if let Some(v) = eps {
                config.eps = v;
            }
            if let Some(v) = n_modes {
                config.n_modes = v;
            }
            if let Some(v) = latent_dim {
                config.latent_dim = v;
            }
            if let Some(v) = stutter_k {
                config.stutter_k = v;
            }
            if let Some(v) = seed {
                config.seed = Some(v);
            }
            if let Some(ref v) = condition {
                config.condition = runner::parse_condition(v).map_err(|e| e.to_string())?;
            }
            config.strict = !no_strict;
            if let Some(ref list) = gates {
                config.gates.enabled = GateConfig::parse_list(list)?;
                config.gates.stutter_k = config.stutter_k;
            }

            let predictor = match predictor {
                Some(ref path) => {
                    let trained = TrainedPredictor::from_file(path)
                        .map_err(|e| format!("failed to load {}: {}", path.display(), e))?;
                    // The checkpoint fixes N and dim(Z); adopt them.
                    config.n_modes = trained.n_modes();
                    config.latent_dim = trained.latent_dim();
                    let lip = trained.measured_lipschitz().map_err(|e| e.to_string())?;
                    eprintln!(
                        "Predictor: trained weights from {} (Lip(P) = {:.4})",
                        path.display(),
                        lip
                    );
                    RefPredictor::Trained(trained)
                }
                None => RefPredictor::Sim(SimPredictor::new(config.n_modes, config.latent_dim)),
            };

            eprintln!(
                "Aria run: {} steps, schedule={}, eps={}, N={}, dim(Z)={}, condition={:?}",
                steps, config.schedule, config.eps, config.n_modes, config.latent_dim, config.condition
            );

            let outcome = runner::run_with(config, steps, predictor).map_err(|e| e.to_string())?;
            let s = &outcome.summary;

            eprintln!("Completed {} steps successfully.", s.steps);
            eprintln!(
                "Final: t={}, |G|={}, energy={:.6}, residual={:.6}, invariants={}",
                s.t,
                s.graph_size,
                s.energy,
                s.residual,
                if s.invariants_ok { "OK" } else { "FAILED" }
            );

            // Phase 1 audit (plan WS1): σ_max per weight matrix, present iff
            // the run used the trained backend.
            if let Some(r) = &s.spectral_report {
                eprintln!(
                    "σ_max audit: token={:.9} diffusion={:.9} world_model={:.9} embed={:.9}",
                    r.token, r.diffusion, r.world_model, r.embed
                );
            }

            if !s.invariants_ok {
                for f in &s.failures {
                    eprintln!("  {f}");
                }
                return Err("invariant check failed on final state".into());
            }

            if !s.gates.enabled.is_empty() {
                eprintln!(
                    "Operating gates [{}]: {} breach(es)",
                    s.gates.enabled.join(", "),
                    s.gates.breaches.len()
                );
                for b in &s.gates.breaches {
                    eprintln!("  {} @ step {}: {}", b.gate, b.step, b.detail);
                }
                if strict_gates && !s.gates.all_ok() {
                    return Err("operating gate breached (--strict-gates)".into());
                }
            }

            let jsonl = outcome.trace.to_jsonl();
            match output {
                Some(path) => {
                    fs::write(&path, &jsonl)
                        .map_err(|e| format!("failed to write trace {}: {}", path.display(), e))?;
                    eprintln!("Trace written to {}", path.display());
                }
                None => print!("{jsonl}"),
            }
            Ok(())
        }

        Commands::Step {
            action,
            condition,
            state: state_path,
            n_modes,
            latent_dim,
            eps,
        } => {
            let mut config = base;
            if let Some(v) = n_modes {
                config.n_modes = v;
            }
            if let Some(v) = latent_dim {
                config.latent_dim = v;
            }
            if let Some(v) = eps {
                config.eps = v;
            }
            let cond = match condition {
                Some(ref v) => runner::parse_condition(v).map_err(|e| e.to_string())?,
                None => config.condition,
            };
            let action = parse_action(&action)?;

            // `aria step --state` never calls Engine::init, so the 𝒮 bounds
            // are enforced here for that path (plan WS0: every CLI entry).
            config.validate().map_err(|e| e.to_string())?;
            let engine = sim_engine(config);
            let state = match state_path {
                Some(p) => {
                    let s = fs::read_to_string(&p)
                        .map_err(|e| format!("failed to read state {}: {}", p.display(), e))?;
                    serde_json::from_str(&s).map_err(|e| format!("failed to parse state: {e}"))?
                }
                None => canonical_init(&engine, cond).map_err(|e| e.to_string())?,
            };

            let new_state = engine.apply(state, action, cond).map_err(|e| e.to_string())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&new_state)
                    .map_err(|e| format!("failed to serialize state: {e}"))?
            );
            Ok(())
        }

        Commands::Check {
            state,
            condition,
            latent_dim,
            predictor,
        } => {
            let mut config = base;
            if let Some(v) = latent_dim {
                config.latent_dim = v;
            }
            let cond = match condition {
                Some(ref v) => runner::parse_condition(v).map_err(|e| e.to_string())?,
                None => config.condition,
            };

            let contents = fs::read_to_string(&state)
                .map_err(|e| format!("failed to read state {}: {}", state.display(), e))?;
            let state: aria_engine_core::state::State =
                serde_json::from_str(&contents).map_err(|e| format!("failed to parse state: {e}"))?;

            // `aria check` never calls Engine::init; enforce the 𝒮 bounds
            // here for the same reason as `step` (plan WS0).
            config.validate().map_err(|e| e.to_string())?;

            let engine = match predictor {
                Some(ref path) => {
                    let trained = TrainedPredictor::from_file(path)
                        .map_err(|e| format!("failed to load {}: {}", path.display(), e))?;
                    config.n_modes = trained.n_modes();
                    config.latent_dim = trained.latent_dim();
                    let report = trained.spectral_report().map_err(|e| e.to_string())?;
                    println!(
                        "σ_max audit ({}) — token={:.9} diffusion={:.9} world_model={:.9} embed={:.9}",
                        path.display(),
                        report.token,
                        report.diffusion,
                        report.world_model,
                        report.embed
                    );
                    runner::engine_with(config, RefPredictor::Trained(trained))
                }
                None => sim_engine(config),
            };
            let report = engine.check(&state, cond);
            if report.all_ok() {
                println!("All invariants hold: Inv1 ✓ Inv2 ✓ Inv3 ✓ Inv4 ✓");
                Ok(())
            } else {
                for failure in report.failures() {
                    println!("  {failure}");
                }
                Err("invariant violations".into())
            }
        }

        Commands::Bench {
            n_modes,
            latent_dim,
            steps,
            with_gates,
        } => {
            let sizes: Vec<usize> = n_modes
                .split(',')
                .map(|s| {
                    s.trim()
                        .parse::<usize>()
                        .map_err(|e| format!("bad --n-modes value '{}': {}", s.trim(), e))
                })
                .collect::<Result<_, _>>()?;

            println!(
                "{:>8}  {:>8}  {:>10}  {:>12}  {:>12}  {:>10}",
                "N", "dim(Z)", "steps", "setup (ms)", "run (ms)", "steps/s"
            );
            for n in sizes {
                let mut config = base.clone();
                config.n_modes = n;
                config.latent_dim = latent_dim.min(2 * n);
                if with_gates {
                    config.gates.enabled = Gate::ALL.to_vec();
                }

                // Setup (the O(N³) unitary build) is timed separately from the
                // Φ-cycle loop; conflating them hides which one actually scales.
                let t_setup = std::time::Instant::now();
                let engine = runner::sim_engine(config.clone());
                let state = runner::canonical_init(&engine, config.condition)
                    .map_err(|e| e.to_string())?;
                let setup_ms = t_setup.elapsed().as_secs_f64() * 1000.0;

                let mut scheduler =
                    Scheduler::from_string(&config.schedule, config.stutter_k)?;

                let t_run = std::time::Instant::now();
                let (final_state, _, _) = engine
                    .run_monitored(state, &mut scheduler, steps, config.condition)
                    .map_err(|e| e.to_string())?;
                let run = t_run.elapsed();

                let report = engine.check(&final_state, config.condition);
                if !report.all_ok() {
                    return Err(format!("N={}: invariants failed: {:?}", n, report.failures()));
                }

                println!(
                    "{:>8}  {:>8}  {:>10}  {:>12.1}  {:>12.1}  {:>10.0}",
                    n,
                    config.latent_dim,
                    steps,
                    setup_ms,
                    run.as_secs_f64() * 1000.0,
                    steps as f64 / run.as_secs_f64()
                );
            }
            Ok(())
        }

        Commands::Dataset {
            input,
            stride,
            trajectories,
            length,
            n_modes,
            seed,
            output,
        } => {
            let n_modes = n_modes.unwrap_or(base.n_modes);

            let (json, summary) = if let Some(ref path) = input {
                // Production path: real bytes → spectral fields.
                let stride = stride.unwrap_or(n_modes);
                let dataset = aria_engine_backends::dataset_from_file(path, n_modes, stride)?;
                let summary = format!(
                    "{} frames from {} bytes of {} (spectral-dft, N={}, stride={})",
                    dataset.trajectories[0].len(),
                    dataset.source_bytes,
                    dataset.source,
                    n_modes,
                    stride
                );
                let json = serde_json::to_string(&dataset)
                    .map_err(|e| format!("failed to serialize dataset: {e}"))?;
                (json, summary)
            } else {
                // Smoke-test path: synthetic phase-ramp trajectories. These
                // exercise the training plumbing; they are not data.
                eprintln!(
                    "note: no --input given — emitting synthetic phase-ramp data for smoke tests only"
                );
                let seed = seed.or(base.seed).unwrap_or(42);
                let dataset = runner::optical_dataset(n_modes, seed, trajectories, length);
                let summary = format!(
                    "{trajectories} synthetic trajectories × {length} snapshots (N={n_modes}, seed={seed})"
                );
                let json = serde_json::to_string(&dataset)
                    .map_err(|e| format!("failed to serialize dataset: {e}"))?;
                (json, summary)
            };

            match output {
                Some(path) => {
                    fs::write(&path, &json)
                        .map_err(|e| format!("failed to write dataset {}: {}", path.display(), e))?;
                    eprintln!("Dataset written to {}: {}", path.display(), summary);
                }
                None => println!("{json}"),
            }
            Ok(())
        }
    }
}

fn parse_action(s: &str) -> Result<Action, String> {
    match s.to_lowercase().as_str() {
        "opticalstep" | "optical_step" | "o" => Ok(Action::OpticalStep),
        "predict" | "p" => Ok(Action::Predict),
        "match" | "m" => Ok(Action::Match),
        "diffuse" | "d" => Ok(Action::Diffuse),
        "stutter" | "s" => Ok(Action::Stutter),
        other => Err(format!(
            "unknown action '{other}' (expected OpticalStep|Predict|Match|Diffuse|Stutter)"
        )),
    }
}
