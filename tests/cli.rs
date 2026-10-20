// SPDX-License-Identifier: Apache-2.0
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::TempDir;

#[test]
fn reviewed_prompt_files_preserve_request_identity_and_never_execute_text() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("reviewed prompt.txt");
    fs::write(
        &file,
        "A goal with 'quotes' and $(touch should-not-exist)\n",
    )
    .unwrap();
    let args = [
        "init",
        "spoken",
        "--goal-file",
        file.to_str().unwrap(),
        "--json",
        "--request-id",
        "voice-init",
    ];
    let first = decode(invoke(temp.path(), &args), 0);
    assert_eq!(decode(invoke(temp.path(), &args), 0), first);
    assert!(!temp.path().join("should-not-exist").exists());
    fs::write(&file, "Changed reviewed goal").unwrap();
    assert_eq!(
        decode(invoke(temp.path(), &args), 4)["error"]["code"],
        "idempotency_conflict"
    );
    let root = temp.path().join("spoken");
    let identity = decode(invoke(&root, &["identity", "--json"]), 0);
    assert_eq!(identity["project"], first["project"]);
    let id = first["project"]["id"].as_str().unwrap();
    decode(
        invoke(
            &root,
            &[
                "build",
                "--json",
                "--project",
                id,
                "--revision",
                "0",
                "--request-id",
                "build",
            ],
        ),
        0,
    );
    fs::write(&file, "Choose two lunches for a total of 8 euros.\n").unwrap();
    let run = [
        "run",
        "--input-file",
        file.to_str().unwrap(),
        "--json",
        "--project",
        id,
        "--revision",
        "1",
        "--request-id",
        "spoken-run",
    ];
    let result = decode(invoke(&root, &run), 0);
    assert_eq!(result["data"]["execution"]["output"]["total_cents"], 1100);
    assert_eq!(decode(invoke(&root, &run), 0), result);
    fs::write(&file, "Choose a lunch and buy it for me").unwrap();
    assert_eq!(
        decode(invoke(&root, &run), 4)["error"]["code"],
        "idempotency_conflict"
    );
    assert_eq!(
        decode(
            invoke(
                &root,
                &[
                    "run",
                    "--input",
                    "text",
                    "--input-file",
                    file.to_str().unwrap(),
                    "--json"
                ]
            ),
            2
        )["ok"],
        false
    );
    fs::write(&file, "x".repeat(1_048_577)).unwrap();
    assert_eq!(
        decode(invoke(temp.path(), &args), 2)["error"]["code"],
        "invalid_input"
    );
}

fn invoke(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_paja"))
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("execute paja")
}
fn decode(output: Output, exit: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(exit),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "JSON mode must not log normal output to stderr"
    );
    let result: Value = serde_json::from_slice(&output.stdout).expect("exactly one JSON response");
    assert_eq!(result["schema_version"], "paja.result.v1");
    for field in ["ok", "project", "run", "data", "next_actions", "error"] {
        assert!(result.get(field).is_some(), "{field}");
    }
    result
}
struct Project {
    _temp: TempDir,
    root: PathBuf,
    id: String,
}
impl Project {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let result = decode(
            invoke(
                temp.path(),
                &[
                    "init",
                    "meal",
                    "--goal",
                    "Choose lunches within budget",
                    "--request-id",
                    "init-1",
                    "--json",
                ],
            ),
            0,
        );
        Self {
            root: temp.path().join("meal"),
            id: result["project"]["id"].as_str().unwrap().into(),
            _temp: temp,
        }
    }
    fn call(&self, args: &[&str], exit: i32) -> Value {
        let mut arguments = vec!["--json", "--project", &self.id];
        arguments.extend_from_slice(args);
        decode(invoke(&self.root, &arguments), exit)
    }
    fn edit(&self, file: &str, change: impl FnOnce(&mut Value)) {
        let path = self.root.join(file);
        let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        change(&mut value);
        fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }
    fn build(&self) -> Value {
        self.call(&["build", "--revision", "0", "--request-id", "build-1"], 0)
    }
    fn test(&self) -> Value {
        self.call(&["test", "--revision", "1", "--request-id", "test-1"], 0)
    }
    fn improve(&self) -> Value {
        self.call(
            &[
                "improve",
                "--from",
                "run_000002",
                "--revision",
                "1",
                "--request-id",
                "improve-1",
            ],
            0,
        )
    }
    fn sql_count(&self, table: &str) -> u32 {
        let db = rusqlite::Connection::open(self.root.join(".paja/state.sqlite3")).unwrap();
        db.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }
}

#[test]
fn complete_loop_preserves_baseline_and_actual_tool_evidence() {
    let project = Project::new();
    let source = fs::read(project.root.join("paja.json")).unwrap();
    let suite = fs::read(project.root.join("cases.json")).unwrap();
    project.build();
    let baseline = project.test();
    assert_eq!(
        baseline["data"]["report"]["counts"],
        json!({"passed":2,"failed":2,"blocked":0,"skipped":0})
    );
    let improved = project.improve();
    assert_eq!(improved["project"]["candidate_revision"], 2);
    assert_eq!(improved["data"]["report"]["counts"]["passed"], 4);
    assert_eq!(
        improved["data"]["comparison"]["improved"],
        json!(["within_budget"])
    );
    assert_eq!(improved["data"]["adopted"], true);
    assert_eq!(
        improved["data"]["report"]["suite_digest"],
        baseline["data"]["report"]["suite_digest"]
    );
    assert_eq!(improved["data"]["report"]["usage"]["model_calls"], 0);
    assert_eq!(improved["data"]["report"]["evidence"], "control_fixture");
    assert_eq!(fs::read(project.root.join("paja.json")).unwrap(), source);
    assert_eq!(fs::read(project.root.join("cases.json")).unwrap(), suite);
    let saved = project.call(&["results", "--run", "run_000002"], 0);
    assert_eq!(saved["data"], baseline["data"]);
    let run = project.call(
        &[
            "run",
            "--revision",
            "2",
            "--request-id",
            "run-1",
            "--input",
            "Choose two lunches for a total of eight euros",
        ],
        0,
    );
    assert_eq!(
        run["data"]["execution"]["output"]["selected_items"],
        json!(["soup", "soup"])
    );
    assert_eq!(run["data"]["execution"]["output"]["total_cents"], 800);
    assert_eq!(
        run["data"]["execution"]["tool_calls"][1]["tool"],
        "calculate"
    );
    let unchanged = project.call(
        &[
            "improve",
            "--revision",
            "2",
            "--request-id",
            "improve-2",
            "--from",
            "run_000003",
        ],
        0,
    );
    assert_eq!(unchanged["data"]["changed"], false);
    assert_eq!(project.sql_count("versions"), 2);
}

#[test]
fn duplicate_requests_reuse_receipts_even_after_revision_advances() {
    let project = Project::new();
    let build = project.build();
    let test = project.test();
    let improve = project.improve();
    assert_eq!(project.build(), build);
    assert_eq!(project.test(), test);
    assert_eq!(project.improve(), improve);
    assert_eq!(project.sql_count("versions"), 2);
    assert_eq!(project.sql_count("runs"), 3);
    project.edit("paja.json", |v| v["instructions"] = json!("Changed input"));
    let error = project.call(&["build", "--revision", "0", "--request-id", "build-1"], 4);
    assert_eq!(error["error"]["code"], "idempotency_conflict");
}

#[test]
fn stale_mutations_and_historical_tests_keep_current_evidence_separate() {
    let project = Project::new();
    project.build();
    project.test();
    project.improve();
    let error = project.call(&["build", "--revision", "1", "--request-id", "stale"], 4);
    assert_eq!(error["error"]["code"], "revision_conflict");
    let historical = project.call(
        &["test", "--revision", "1", "--request-id", "historical"],
        0,
    );
    assert_eq!(historical["data"]["historical"], true);
    assert_eq!(historical["run"]["candidate_revision"], 1);
    assert_eq!(historical["project"]["candidate_revision"], 2);
    let status = project.call(&["status"], 0);
    assert_eq!(status["data"]["latest_test_run"], "run_000003");
    project.call(
        &[
            "improve",
            "--revision",
            "2",
            "--request-id",
            "wrong-baseline",
            "--from",
            "run_000004",
        ],
        4,
    );
}

#[test]
fn improvement_uses_frozen_suite_despite_working_file_tampering() {
    let project = Project::new();
    project.build();
    let baseline = project.test();
    project.edit("cases.json", |v| {
        v["cases"][0]["assertions"][2]["amount_cents"] = json!(99999);
        v["fixtures"]["catalog"][0]["price_cents"] = json!(1);
    });
    let improved = project.improve();
    assert_eq!(
        improved["data"]["report"]["suite"],
        baseline["data"]["report"]["suite"]
    );
    assert_eq!(
        improved["data"]["report"]["trials"][0]["execution"]["output"]["total_cents"],
        800
    );
}

#[test]
fn a_regression_is_visible_and_candidate_does_not_replace_baseline() {
    let project = Project::new();
    project.edit("cases.json", |v| {
        v["cases"].as_array_mut().unwrap().push(json!({
        "id":"variety", "input":{"count":2,"budget_cents":1200,"purchase":false},
        "assertions":[{"kind":"distinct_item_count","equals":2}]}))
    });
    project.build();
    project.test();
    let improved = project.improve();
    assert_eq!(
        improved["data"]["comparison"]["improved"],
        json!(["within_budget"])
    );
    assert_eq!(
        improved["data"]["comparison"]["regressed"],
        json!(["variety"])
    );
    assert_eq!(improved["data"]["adopted"], false);
    assert_eq!(improved["project"]["candidate_revision"], 1);
    assert_eq!(improved["run"]["candidate_revision"], 2);
    assert_eq!(project.sql_count("versions"), 2);
}

#[test]
fn trial_preflight_and_tool_budget_failures_are_not_quality_scores() {
    let project = Project::new();
    project.edit("paja.json", |v| v["limits"]["max_tool_calls"] = json!(1));
    project.build();
    let error = project.call(
        &[
            "test",
            "--repeat",
            "100",
            "--revision",
            "1",
            "--request-id",
            "over-budget",
        ],
        7,
    );
    assert_eq!(error["error"]["code"], "budget_exhausted");
    assert_eq!(project.sql_count("runs"), 1);
    let test = project.test();
    assert_eq!(test["run"]["state"], "failed");
    assert_eq!(test["data"]["report"]["counts"]["blocked"], 4);
    assert_eq!(test["data"]["report"]["counts"]["failed"], 0);
    assert_eq!(test["data"]["report"]["usage"]["tool_calls"], 1);
    project.call(
        &[
            "improve",
            "--revision",
            "1",
            "--request-id",
            "blocked",
            "--from",
            "run_000002",
        ],
        7,
    );
}

#[test]
fn denied_purchase_is_observed_and_repaired_without_new_authority() {
    let project = Project::new();
    project.edit("paja.json", |v| {
        v["runtime"]["request_purchase"] = json!(true)
    });
    project.build();
    let baseline = project.test();
    let purchase_calls = baseline["data"]["report"]["trials"][2]["execution"]["tool_calls"]
        .as_array()
        .unwrap();
    assert_eq!(purchase_calls[2]["tool"], "purchase.create");
    assert_eq!(purchase_calls[2]["allowed"], false);
    let improved = project.improve();
    assert_eq!(improved["data"]["report"]["counts"]["passed"], 4);
    assert_eq!(
        improved["data"]["report"]["conditions_digest"],
        baseline["data"]["report"]["conditions_digest"]
    );
    assert_eq!(improved["data"]["changes"].as_array().unwrap().len(), 2);
}

#[test]
fn comparisons_reject_different_suites_repetitions_and_limits() {
    let project = Project::new();
    project.build();
    project.test();
    project.edit("cases.json", |v| v["version"] = json!(2));
    project.call(
        &[
            "test",
            "--suite",
            "cases.json",
            "--revision",
            "1",
            "--request-id",
            "new-suite",
        ],
        0,
    );
    let different_suite = project.call(
        &["results", "--run", "run_000003", "--compare", "run_000002"],
        0,
    );
    assert_eq!(different_suite["data"]["comparison"]["comparable"], false);
    project.call(
        &[
            "test",
            "--repeat",
            "1",
            "--revision",
            "1",
            "--request-id",
            "one-repeat",
        ],
        0,
    );
    let different_repeat = project.call(
        &["results", "--run", "run_000004", "--compare", "run_000002"],
        0,
    );
    assert_eq!(different_repeat["data"]["comparison"]["comparable"], false);
    project.edit("paja.json", |v| v["limits"]["max_tool_calls"] = json!(64));
    // Restore the identical suite, isolating the budget-setting difference.
    let source: Value = serde_json::from_str(include_str!("../examples/cases.json")).unwrap();
    fs::write(
        project.root.join("cases.json"),
        serde_json::to_vec(&source).unwrap(),
    )
    .unwrap();
    project.call(
        &["build", "--revision", "1", "--request-id", "new-budget"],
        0,
    );
    project.call(
        &["test", "--revision", "2", "--request-id", "test-budget"],
        0,
    );
    let different_limits = project.call(
        &["results", "--run", "run_000006", "--compare", "run_000002"],
        0,
    );
    assert_eq!(different_limits["data"]["comparison"]["comparable"], false);
}

#[test]
fn dry_runs_do_not_create_versions_trials_or_receipts() {
    let project = Project::new();
    project.call(&["build", "--revision", "0", "--dry-run"], 0);
    assert_eq!(project.sql_count("versions"), 0);
    project.build();
    project.call(&["test", "--revision", "1", "--dry-run"], 0);
    assert_eq!(project.sql_count("runs"), 1);
    project.test();
    let planned = project.call(
        &[
            "improve",
            "--revision",
            "1",
            "--from",
            "run_000002",
            "--dry-run",
        ],
        0,
    );
    assert_eq!(planned["data"]["changes"].as_array().unwrap().len(), 1);
    assert_eq!(project.sql_count("versions"), 1);
    assert_eq!(project.sql_count("runs"), 2);
    assert_eq!(project.sql_count("receipts"), 3);
}

#[test]
fn machine_requests_are_explicit_and_parser_errors_are_structured() {
    let project = Project::new();
    decode(invoke(&project.root, &["status", "--json"]), 7);
    project.call(&["build", "--revision", "0"], 7);
    project.call(&["build", "--request-id", "missing-revision"], 7);
    project.call(
        &["improve", "--revision", "0", "--request-id", "missing-run"],
        7,
    );
    project.call(&["results"], 7);
    let parse = project.call(&["test", "--repeat", "not-a-number"], 2);
    assert_eq!(parse["error"]["code"], "invalid_input");
    decode(
        invoke(
            &project.root,
            &["status", "--project", "prj_wrong", "--json"],
        ),
        3,
    );
    project.call(&["tasks", "claim"], 2);
    assert_eq!(project.sql_count("versions"), 0);
}

#[test]
fn strict_schemas_tools_and_paths_fail_before_mutation() {
    let project = Project::new();
    project.edit("paja.json", |v| v["runtime"]["adapter"] = json!("live"));
    let error = project.call(&["build", "--revision", "0", "--request-id", "live"], 2);
    assert_eq!(error["error"]["code"], "unsupported_capability");
    project.edit("paja.json", |v| {
        v["runtime"]["adapter"] = json!("fixture");
        v["tools"]["allowed"] = json!(["purchase.create"]);
    });
    project.call(&["build", "--revision", "0", "--request-id", "tool"], 2);
    project.edit("paja.json", |v| {
        v["tools"]["allowed"] = json!(["catalog.read", "calculate"]);
        v["schema_version"] = json!("paja.agent.v999");
    });
    project.call(&["build", "--revision", "0", "--request-id", "schema"], 2);
    project.edit("paja.json", |v| {
        v["schema_version"] = json!("paja.agent.v1");
        v["tests"]["suite"] = json!("../outside.json");
    });
    project.call(&["build", "--revision", "0", "--request-id", "escape"], 2);
    assert_eq!(project.sql_count("versions"), 0);
}

#[test]
fn storage_is_append_only_and_detects_corruption() {
    let project = Project::new();
    project.build();
    project.test();
    let db = rusqlite::Connection::open(project.root.join(".paja/state.sqlite3")).unwrap();
    assert!(db.execute("UPDATE versions SET package='{}'", []).is_err());
    assert!(db.execute("DELETE FROM runs", []).is_err());
    // Simulate corruption by a filesystem owner; this is not a security boundary
    // against that owner. A forged report without its original digest is rejected.
    db.execute_batch(
        "DROP TRIGGER immutable_runs_update; UPDATE runs SET data='{}' WHERE kind='test';",
    )
    .unwrap();
    drop(db);
    let error = project.call(&["results", "--run", "run_000002"], 5);
    assert_eq!(error["error"]["code"], "integrity_error");
}

#[test]
fn coordinator_lock_fences_competing_clients_and_recovers_after_release() {
    let project = Project::new();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(project.root.join(".paja/coordinator.lock"))
        .unwrap();
    lock.lock().unwrap();
    let busy = project.call(&["build", "--revision", "0", "--request-id", "build-1"], 6);
    assert_eq!(busy["error"]["code"], "coordinator_busy");
    drop(lock);
    project.build();
    assert_eq!(project.sql_count("versions"), 1);
}

#[test]
fn initialization_retries_do_not_overwrite_existing_files() {
    let temp = tempfile::tempdir().unwrap();
    let args = [
        "init",
        "meal",
        "--goal",
        "Synthetic task",
        "--json",
        "--request-id",
        "init-1",
    ];
    let first = decode(invoke(temp.path(), &args), 0);
    assert_eq!(decode(invoke(temp.path(), &args), 0), first);
    fs::write(temp.path().join("meal/keep.txt"), "preserve me").unwrap();
    decode(
        invoke(
            temp.path(),
            &[
                "init",
                "meal",
                "--goal",
                "Synthetic task",
                "--json",
                "--request-id",
                "init-2",
            ],
        ),
        4,
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("meal/keep.txt")).unwrap(),
        "preserve me"
    );
}

#[test]
fn human_flow_is_runnable_and_always_labels_fixture_evidence() {
    let temp = tempfile::tempdir().unwrap();
    for (root, args) in [
        (
            temp.path().to_path_buf(),
            vec!["init", "meal", "--goal", "Choose lunches"],
        ),
        (temp.path().join("meal"), vec!["build"]),
        (
            temp.path().join("meal"),
            vec![
                "run",
                "--input",
                "Choose two lunches for a total of eight euros",
            ],
        ),
        (temp.path().join("meal"), vec!["test"]),
        (temp.path().join("meal"), vec!["improve"]),
        (temp.path().join("meal"), vec!["results"]),
        (temp.path().join("meal"), vec![]),
    ] {
        let output = invoke(&root, &args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("CONTROL FIXTURE"));
    }
}

#[test]
fn racing_duplicate_builds_commit_one_version_and_receipt() {
    let project = Project::new();
    let spawn = || {
        Command::new(env!("CARGO_BIN_EXE_paja"))
            .arg("-C")
            .arg(&project.root)
            .args([
                "--json",
                "--project",
                &project.id,
                "build",
                "--revision",
                "0",
                "--request-id",
                "build-1",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap()
    };
    let first = spawn();
    let second = spawn();
    for child in [first, second] {
        let output = child.wait_with_output().unwrap();
        let code = output.status.code().unwrap();
        assert!(matches!(code, 0 | 6));
        decode(output, code);
    }
    let retried = project.build();
    assert_eq!(retried["project"]["candidate_revision"], 1);
    assert_eq!(project.sql_count("versions"), 1);
    assert_eq!(project.sql_count("runs"), 1);
    assert_eq!(project.sql_count("receipts"), 2);
}

#[test]
fn uncommitted_state_is_rolled_back_on_connection_loss() {
    let project = Project::new();
    project.build();
    {
        let db = rusqlite::Connection::open(project.root.join(".paja/state.sqlite3")).unwrap();
        db.execute_batch("BEGIN IMMEDIATE; UPDATE project SET revision=99, state='blocked';")
            .unwrap();
        // Drop an uncommitted coordinator connection, as happens on an aborted operation.
    }
    let status = project.call(&["status"], 0);
    assert_eq!(status["project"]["candidate_revision"], 1);
    assert_eq!(status["project"]["state"], "built");
}
