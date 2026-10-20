// SPDX-License-Identifier: Apache-2.0
use crate::{
    cli::{Cli, Command},
    error::{Error, Result},
    model::{self, Agent, Input, Package, Strategy, Suite},
    runtime::{self, Report},
    store::{self, Project, Run, Store},
};
use rusqlite::{Connection, TransactionBehavior};
use serde_json::{Value, json};
use std::{fs, io::Read};

// Resolve prompt contents before computing request identity. Retrying with changed
// text must conflict even if the input filename stays the same.
fn resolve_prompt(cli: &Cli, command: &Command) -> Result<Command> {
    fn read(cli: &Cli, file: &str) -> Result<String> {
        let mut text = String::new();
        fs::File::open(cli.directory.join(file))?
            .take(1_048_577)
            .read_to_string(&mut text)?;
        if text.len() > 1_048_576 || text.trim().is_empty() {
            return Err(Error::invalid(
                "Prompt files must contain 1–1048576 bytes of UTF-8 text",
            ));
        }
        Ok(text.trim().to_owned())
    }
    Ok(match command {
        Command::Init {
            name,
            goal: None,
            goal_file: Some(file),
        } => Command::Init {
            name: name.clone(),
            goal: Some(read(cli, file)?),
            goal_file: None,
        },
        Command::Run {
            input: None,
            input_file: Some(file),
        } => Command::Run {
            input: Some(read(cli, file)?),
            input_file: None,
        },
        _ => command.clone(),
    })
}

fn response(project: &Project, run: Option<&Run>, data: Value, next: &str) -> Value {
    let scope = match next {
        "build" | "improve" => "project:build",
        "test" | "run" => "project:execute",
        _ => "project:read",
    };
    let mut arguments = json!({"project_id":project.id,"revision":project.candidate_revision});
    if next == "results" {
        arguments["run_id"] = run
            .map(|r| json!(r.id))
            .unwrap_or_else(|| data["baseline_run_id"].clone());
    } else if next == "improve"
        && let Some(run) = run
    {
        arguments["from_run_id"] = json!(run.id);
    } else if next == "run" {
        arguments["input"] = json!("Choose two lunches for a total of eight euros");
    }
    json!({"schema_version":"paja.result.v1","ok":true,"project":project,"run":run,"data":data,
        "next_actions":[{"operation":next,"arguments":arguments,"required_scope":scope}],"error":null})
}
fn validate_request(cli: &Cli, command: &Command) -> Result<()> {
    if let Some(key) = &cli.request_id
        && (key.is_empty()
            || key.len() > 128
            || !key
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._:-".contains(&c)))
    {
        return Err(Error::invalid(
            "Request IDs must be 1–128 ASCII letters, digits, dots, underscores, colons, or hyphens",
        ));
    }
    if matches!(command, Command::Init { .. }) {
        if cli.project.is_some() || cli.revision.is_some() {
            return Err(Error::invalid(
                "Init creates the project identity; omit --project and --revision",
            ));
        }
        if cli.json && cli.request_id.is_none() {
            return Err(Error::missing("JSON init requires --request-id"));
        }
        return Ok(());
    }
    if (cli.json || cli.request_id.is_some())
        && cli.project.is_none()
        && !matches!(command, Command::Identity)
    {
        return Err(Error::missing("Machine requests require --project"));
    }
    if (cli.json || cli.request_id.is_some())
        && (command.mutates() || command.dry_run())
        && cli.revision.is_none()
    {
        return Err(Error::missing(
            "Machine operations require an explicit --revision",
        ));
    }
    if cli.json && command.mutates() && cli.request_id.is_none() {
        return Err(Error::missing("Machine mutations require --request-id"));
    }
    if cli.json && matches!(command, Command::Improve { from: None, .. }) {
        return Err(Error::missing("JSON improve requires --from <test-run-id>"));
    }
    if cli.json && matches!(command, Command::Results { run: None, .. }) {
        return Err(Error::missing("JSON results requires --run <id>"));
    }
    Ok(())
}
fn expected_current(cli: &Cli, project: &Project) -> Result<()> {
    if cli
        .revision
        .is_some_and(|r| r != project.candidate_revision)
    {
        return Err(Error::conflict(
            "Candidate changed; read status and decide whether to submit new work",
        ));
    }
    Ok(())
}
fn init(cli: &Cli, command: &Command, name: &str, goal: &str) -> Result<Value> {
    if name.is_empty()
        || name.len() > 80
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
    {
        return Err(Error::invalid(
            "Project name must be 1–80 ASCII letters, digits, hyphens, or underscores",
        ));
    }
    let mut agent: Agent = serde_json::from_str(model::AGENT_EXAMPLE)?;
    agent.name = name.into();
    agent.goal = goal.into();
    agent.validate()?;
    let fingerprint = model::digest(command)?;
    let root = cli.directory.join(name);
    if root.exists() {
        if cli.request_id.is_some() && root.join(".paja/state.sqlite3").is_file() {
            let store = Store::open(&root)?;
            if let Some(saved) =
                store::receipt(&store.db, "init", cli.request_id.as_deref(), &fingerprint)?
            {
                return Ok(saved);
            }
        }
        return Err(Error::new(
            "project_exists",
            "Destination already exists; initialization never overwrites files",
            4,
        ));
    }
    fs::create_dir(&root)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    }
    let mut store = Store::create(&root, &store::new_project_id(name)?)?;
    fs::write(
        root.join("paja.json"),
        format!("{}\n", serde_json::to_string_pretty(&agent)?),
    )?;
    fs::write(root.join("cases.json"), model::SUITE_EXAMPLE)?;
    fs::write(root.join(".gitignore"), ".paja/\n")?;
    let tx = store
        .db
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let result = response(
        &store::project(&tx)?,
        None,
        json!({"evidence":"control_fixture","directory":name,
        "message":"Created the meal-helper control project. The goal is metadata; this adapter does not generate an AI agent. Change into the project directory, then build."}),
        "build",
    );
    store::save_receipt(
        &tx,
        "init",
        cli.request_id.as_deref(),
        &fingerprint,
        &result,
    )?;
    tx.commit()?;
    Ok(result)
}
fn report(db: &Connection, id: &str) -> Result<(Run, Report)> {
    let (run, data) = store::run(db, id)?;
    if !matches!(run.kind.as_str(), "test" | "improve") {
        return Err(Error::invalid("Run has no test evidence"));
    }
    let report: Report = serde_json::from_value(data["report"].clone())?;
    let (_, hash) = store::package(db, run.candidate_revision)?;
    if report.package_digest != hash || report.suite_digest != model::digest(&report.suite)? {
        return Err(Error::integrity(
            "Test evidence does not match its package or frozen suite",
        ));
    }
    Ok((run, report))
}
fn baseline_id(db: &Connection, project: &Project, from: &Option<String>) -> Result<String> {
    from.clone()
        .or(store::latest_test(db, project.candidate_revision)?)
        .ok_or_else(|| Error::missing("Test the current candidate before improving it"))
}

pub fn dispatch(cli: &Cli) -> Result<Value> {
    let default = Command::Status;
    let resolved = resolve_prompt(cli, cli.command.as_ref().unwrap_or(&default))?;
    let command = &resolved;
    validate_request(cli, command)?;
    if let Command::Init {
        name,
        goal: Some(goal),
        ..
    } = command
    {
        return init(cli, command, name, goal);
    }
    let mut store = Store::open(&cli.directory)?;
    let tx = store
        .db
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let project = store::project(&tx)?;
    if cli.project.as_ref().is_some_and(|id| *id != project.id) {
        return Err(Error::new(
            "project_mismatch",
            "Project ID does not match this directory",
            3,
        ));
    }
    let revision = cli.revision.unwrap_or(project.candidate_revision);

    // Freeze normalized inputs before dispatch. All accepted mutations and their receipts
    // commit together; this runtime performs no external or paid effects.
    let payload = match command {
        Command::Build { file, .. } => {
            let agent: Agent = model::read_json(&cli.directory, file)?;
            agent.validate()?;
            let suite: Suite = model::read_json(&cli.directory, &agent.tests.suite)?;
            runtime::preflight(&agent, &suite, agent.tests.repetitions)?;
            json!({"package":Package {schema_version:"paja.package.v1".into(),agent,default_suite:suite}})
        }
        Command::Test { suite, repeat, .. } => {
            let (package, hash) = store::package(&tx, revision)?;
            let suite = match suite {
                Some(file) => model::read_json(&cli.directory, file)?,
                None => package.default_suite.clone(),
            };
            let repetitions = repeat.unwrap_or(package.agent.tests.repetitions);
            runtime::preflight(&package.agent, &suite, repetitions)?;
            json!({"package":package,"package_digest":hash,"suite":suite,"repetitions":repetitions})
        }
        Command::Run {
            input: Some(input), ..
        } => {
            let (package, hash) = store::package(&tx, revision)?;
            json!({"package":package,"package_digest":hash,"input":Input::parse(input)?})
        }
        Command::Improve { from, .. } => {
            json!({"baseline_run_id":baseline_id(&tx, &project, from)?})
        }
        _ => Value::Null,
    };
    let fingerprint = model::digest(&(command, &cli.project, cli.revision, &payload))?;
    if command.mutates()
        && let Some(saved) =
            store::receipt(&tx, command.name(), cli.request_id.as_deref(), &fingerprint)?
    {
        return Ok(saved);
    }
    let result = match command {
        Command::Build { dry_run, .. } => {
            expected_current(cli, &project)?;
            let package: Package = serde_json::from_value(payload["package"].clone())?;
            if *dry_run {
                response(
                    &project,
                    None,
                    json!({"dry_run":true,"package_digest":model::digest(&package)?,"message":"Specification and default suite are valid; no version created."}),
                    "build",
                )
            } else {
                let next_revision = store::next_revision(&tx)?;
                let hash = store::save_package(&tx, next_revision, &package)?;
                store::set_current(&tx, next_revision, "built")?;
                let run = Run {
                    id: store::next_run_id(&tx)?,
                    kind: "build".into(),
                    candidate_revision: next_revision,
                    state: "succeeded".into(),
                };
                let data = json!({"evidence":"control_fixture","package_digest":hash,"message":"Built an immutable fixture package. Try it with run, then test."});
                store::save_run(&tx, &run, &data)?;
                response(&store::project(&tx)?, Some(&run), data, "test")
            }
        }
        Command::Test { dry_run, .. } => {
            let package: Package = serde_json::from_value(payload["package"].clone())?;
            let suite: Suite = serde_json::from_value(payload["suite"].clone())?;
            let repetitions: u32 = serde_json::from_value(payload["repetitions"].clone())?;
            if *dry_run {
                response(
                    &project,
                    None,
                    json!({"dry_run":true,"trials":suite.cases.len() as u32 * repetitions,"suite_digest":model::digest(&suite)?,"message":"Frozen test inputs are valid; no trials executed."}),
                    "test",
                )
            } else {
                let id = store::next_run_id(&tx)?;
                let report = runtime::evaluate(
                    &package.agent,
                    payload["package_digest"].as_str().unwrap(),
                    &suite,
                    repetitions,
                    &id,
                )?;
                let run = Run {
                    id,
                    kind: "test".into(),
                    candidate_revision: revision,
                    state: if report.counts.blocked == 0 {
                        "succeeded"
                    } else {
                        "failed"
                    }
                    .into(),
                };
                if revision == project.candidate_revision {
                    store::set_current(
                        &tx,
                        revision,
                        if report.counts.blocked == 0 {
                            "tested"
                        } else {
                            "blocked"
                        },
                    )?;
                }
                let data =
                    json!({"report":report,"historical":revision != project.candidate_revision});
                store::save_run(&tx, &run, &data)?;
                response(&store::project(&tx)?, Some(&run), data, "improve")
            }
        }
        Command::Run { .. } => {
            let package: Package = serde_json::from_value(payload["package"].clone())?;
            let input: Input = serde_json::from_value(payload["input"].clone())?;
            let execution = runtime::execute(
                &package.agent,
                &package.default_suite.fixtures,
                &input,
                &mut runtime::Budget::new(&package.agent),
            );
            let run = Run {
                id: store::next_run_id(&tx)?,
                kind: "run".into(),
                candidate_revision: revision,
                state: if execution.error.is_none() {
                    "succeeded"
                } else {
                    "failed"
                }
                .into(),
            };
            let data = json!({"evidence":"control_fixture","package_digest":payload["package_digest"],"input":input,"execution":execution});
            store::save_run(&tx, &run, &data)?;
            response(&project, Some(&run), data, "test")
        }
        Command::Improve { dry_run, .. } => {
            expected_current(cli, &project)?;
            let id = payload["baseline_run_id"].as_str().unwrap();
            let (baseline_run, baseline) = report(&tx, id)?;
            if baseline_run.candidate_revision != project.candidate_revision {
                return Err(Error::conflict(
                    "Baseline evidence belongs to a different candidate; test the current version",
                ));
            }
            if baseline.counts.blocked > 0 || baseline.counts.skipped > 0 {
                return Err(Error::missing(
                    "Resolve execution failures before attempting an improvement",
                ));
            }
            let (mut package, _) = store::package(&tx, revision)?;
            let mut changes = Vec::new();
            if baseline.counts.failed > 0 {
                // Repairs depend on observed failures, and cannot edit assertions or tool grants.
                let budget_failed = baseline.trials.iter().flat_map(|t| &t.checks).any(|c| {
                    !c.passed
                        && matches!(
                            c.assertion,
                            model::Assertion::FixturePriceTotalAtMost { .. }
                        )
                });
                let purchase_failed = baseline.trials.iter().flat_map(|t| &t.checks).any(|c| {
                    !c.passed
                        && matches!(
                            c.assertion,
                            model::Assertion::NoDisallowedToolRequests
                                | model::Assertion::PurchaseNotPerformed
                        )
                });
                if budget_failed && package.agent.runtime.strategy != Strategy::Cheapest {
                    changes.push(json!({"field":"runtime.strategy","before":package.agent.runtime.strategy,"after":"cheapest"}));
                    package.agent.runtime.strategy = Strategy::Cheapest;
                }
                if purchase_failed && package.agent.runtime.request_purchase {
                    changes.push(
                        json!({"field":"runtime.request_purchase","before":true,"after":false}),
                    );
                    package.agent.runtime.request_purchase = false;
                }
            }
            if changes.is_empty() {
                response(
                    &project,
                    None,
                    json!({"dry_run":dry_run,"changed":false,"baseline_run_id":id,"message":if baseline.counts.failed == 0 {"No failed cases; no change needed."} else {"No supported fixture repair for these failures; no candidate created."}}),
                    "results",
                )
            } else if *dry_run {
                response(
                    &project,
                    None,
                    json!({"dry_run":true,"changes":changes,"baseline_run_id":id,"suite_digest":baseline.suite_digest,"message":"One fixture repair and comparison planned; no candidate created."}),
                    "improve",
                )
            } else {
                let next_revision = store::next_revision(&tx)?;
                let hash = store::save_package(&tx, next_revision, &package)?;
                let run_id = store::next_run_id(&tx)?;
                let candidate = runtime::evaluate(
                    &package.agent,
                    &hash,
                    &baseline.suite,
                    baseline.repetitions,
                    &run_id,
                )?;
                let comparison = runtime::compare(&baseline, &candidate);
                let adopted = comparison.comparable
                    && !comparison.improved.is_empty()
                    && comparison.regressed.is_empty()
                    && comparison.execution_failures.is_empty();
                if adopted {
                    store::set_current(&tx, next_revision, "tested")?;
                }
                let run = Run {
                    id: run_id,
                    kind: "improve".into(),
                    candidate_revision: next_revision,
                    state: if candidate.counts.blocked == 0 {
                        "succeeded"
                    } else {
                        "failed"
                    }
                    .into(),
                };
                let data = json!({"baseline_run_id":id,"baseline_revision":baseline_run.candidate_revision,"report":candidate,"comparison":comparison,"changes":changes,"adopted":adopted,
                    "message":if adopted {"Candidate became current. Baseline and source files are preserved; results contains the exact changes."} else {"Candidate retained for inspection; baseline remains current because the comparison did not establish a regression-free improvement."}});
                store::save_run(&tx, &run, &data)?;
                response(&store::project(&tx)?, Some(&run), data, "results")
            }
        }
        Command::Identity => response(
            &project,
            None,
            json!({"message":"Local owner project identity."}),
            "status",
        ),
        Command::Status => {
            let latest = store::latest_test(&tx, project.candidate_revision)?;
            let data = if let Some(id) = latest {
                let (_, report) = report(&tx, &id)?;
                json!({"evidence":"control_fixture","latest_test_run":id,"report":report,"message":"Local owner mode. Worker sessions and live model adapters are not implemented."})
            } else {
                json!({"evidence":"control_fixture","latest_test_run":null,"message":"No test evidence for the current candidate."})
            };
            response(
                &project,
                None,
                data,
                if project.candidate_revision == 0 {
                    "build"
                } else {
                    "test"
                },
            )
        }
        Command::Results { run, compare } => {
            let id = baseline_id(&tx, &project, run)?;
            let (run, mut data) = store::run(&tx, &id)?;
            if let Some(baseline_id) = compare {
                let (_, baseline) = report(&tx, baseline_id)?;
                let (_, candidate) = report(&tx, &id)?;
                data["comparison"] = serde_json::to_value(runtime::compare(&baseline, &candidate))?;
                data["baseline_run_id"] = json!(baseline_id);
            }
            response(&project, Some(&run), data, "run")
        }
        Command::Init { .. } => unreachable!(),
    };
    if command.mutates() {
        store::save_receipt(
            &tx,
            command.name(),
            cli.request_id.as_deref(),
            &fingerprint,
            &result,
        )?;
    }
    tx.commit()?;
    Ok(result)
}
