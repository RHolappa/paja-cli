// SPDX-License-Identifier: Apache-2.0
use crate::{engine, error::Error};
use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Build, test, and improve your AI agent.",
    long_about = "Build, test, and improve your AI agent.\nLocal control prototype: deterministic fixtures only, no live models or workers."
)]
pub struct Cli {
    #[arg(short = 'C', long, global = true, default_value = ".")]
    pub directory: PathBuf,
    #[arg(long, global = true)]
    pub json: bool,
    #[arg(long, global = true)]
    pub project: Option<String>,
    #[arg(long, global = true)]
    pub revision: Option<u32>,
    #[arg(long, global = true)]
    pub request_id: Option<String>,
    #[command(subcommand)]
    pub command: Option<Command>,
}
#[derive(Debug, Clone, Subcommand, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum Command {
    /// Create a local meal-helper control project (no provider calls).
    Init {
        name: String,
        #[arg(
            long,
            required_unless_present = "goal_file",
            conflicts_with = "goal_file"
        )]
        goal: Option<String>,
        /// Read an edited UTF-8 goal from a file (relative to --directory).
        #[arg(long)]
        #[serde(skip_serializing_if = "Option::is_none")]
        goal_file: Option<String>,
    },
    /// Freeze the project specification and default suite into a runnable version.
    Build {
        #[arg(long, default_value = "paja.json")]
        file: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Execute host assertions against a frozen version.
    Test {
        #[arg(long)]
        suite: Option<String>,
        #[arg(long)]
        repeat: Option<u32>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Apply one deterministic fixture repair and compare with its baseline.
    Improve {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Invoke a built version using a demo phrase or structured JSON input.
    Run {
        #[arg(
            long,
            required_unless_present = "input_file",
            conflicts_with = "input_file"
        )]
        input: Option<String>,
        /// Read an edited UTF-8 prompt from a file (relative to --directory).
        #[arg(long)]
        #[serde(skip_serializing_if = "Option::is_none")]
        input_file: Option<String>,
    },
    /// Show current revision, latest evidence, and the next action.
    Status,
    /// Discover a local owner's project identity without mutating it.
    Identity,
    /// Inspect stored outcomes, traces, and optional baseline comparison.
    Results {
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        compare: Option<String>,
    },
}
impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Init { .. } => "init",
            Self::Build { .. } => "build",
            Self::Test { .. } => "test",
            Self::Improve { .. } => "improve",
            Self::Run { .. } => "run",
            Self::Status => "status",
            Self::Identity => "identity",
            Self::Results { .. } => "results",
        }
    }
    pub fn dry_run(&self) -> bool {
        matches!(
            self,
            Self::Build { dry_run: true, .. }
                | Self::Test { dry_run: true, .. }
                | Self::Improve { dry_run: true, .. }
        )
    }
    pub fn mutates(&self) -> bool {
        !matches!(self, Self::Identity | Self::Status | Self::Results { .. }) && !self.dry_run()
    }
}
pub fn error_response(error: &Error) -> Value {
    json!({"schema_version":"paja.result.v1","ok":false,"project":null,"run":null,"data":null,"next_actions":[],"error":error})
}
fn print_human(response: &Value) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    writeln!(
        out,
        "CONTROL FIXTURE — deterministic mechanics, not AI quality"
    )?;
    if let Some(project) = response.get("project").filter(|v| !v.is_null()) {
        writeln!(
            out,
            "{} · revision {} · {}",
            project["id"].as_str().unwrap_or(""),
            project["candidate_revision"],
            project["state"].as_str().unwrap_or("")
        )?;
    }
    let data = &response["data"];
    if let Some(message) = data["message"].as_str() {
        writeln!(out, "{message}")?;
    }
    if let Some(run) = response.get("run").filter(|v| !v.is_null()) {
        writeln!(
            out,
            "{} · {} · {}",
            run["id"].as_str().unwrap_or(""),
            run["kind"].as_str().unwrap_or(""),
            run["state"].as_str().unwrap_or("")
        )?;
    }
    let report = &data["report"];
    if let Some(trials) = report["trials"].as_array() {
        let counts = &report["counts"];
        writeln!(
            out,
            "Trials: {} passed, {} failed, {} blocked, {} skipped",
            counts["passed"], counts["failed"], counts["blocked"], counts["skipped"]
        )?;
        for trial in trials {
            writeln!(
                out,
                "  {} #{}: {}",
                trial["case_id"].as_str().unwrap_or(""),
                trial["repetition"],
                trial["outcome"].as_str().unwrap_or("")
            )?;
            if trial["outcome"] != "passed" {
                if let Some(checks) = trial["checks"].as_array() {
                    for check in checks.iter().filter(|c| c["passed"] == false) {
                        writeln!(out, "    failed: {}", check["assertion"])?;
                    }
                }
                writeln!(out, "    observed: {}", trial["execution"])?;
            }
        }
    }
    if let Some(comparison) = data.get("comparison") {
        writeln!(
            out,
            "Comparison: {}",
            serde_json::to_string_pretty(comparison).map_err(io::Error::other)?
        )?;
    }
    if let Some(changes) = data.get("changes") {
        writeln!(out, "Changes: {changes}")?;
    }
    if let Some(execution) = data.get("execution") {
        writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(execution).map_err(io::Error::other)?
        )?;
    }
    if let Some(next) = response["next_actions"].as_array() {
        for action in next {
            let operation = action["operation"].as_str().unwrap_or("status");
            match operation {
                "run" => writeln!(
                    out,
                    "Next: paja run --input \"Choose two lunches for a total of eight euros\""
                )?,
                "results" if action["arguments"]["run_id"].is_string() => writeln!(
                    out,
                    "Next: paja results --run {}",
                    action["arguments"]["run_id"].as_str().unwrap()
                )?,
                _ => writeln!(out, "Next: paja {operation}")?,
            }
        }
    }
    Ok(())
}
pub fn main() -> ExitCode {
    let wants_json = std::env::args_os().any(|arg| arg == "--json");
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            let _ = error.print();
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            if wants_json {
                println!("{}", error_response(&Error::invalid(error.to_string())));
            } else {
                let _ = error.print();
            }
            return ExitCode::from(2);
        }
    };
    match engine::dispatch(&cli) {
        Ok(response) => {
            let printed = if cli.json {
                writeln!(io::stdout().lock(), "{response}")
            } else {
                print_human(&response)
            };
            if printed.is_err() {
                return ExitCode::from(5);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            if cli.json {
                let _ = writeln!(io::stdout().lock(), "{}", error_response(&error));
            } else {
                eprintln!("{}: {}", error.code, error.message);
            }
            ExitCode::from(error.exit)
        }
    }
}
