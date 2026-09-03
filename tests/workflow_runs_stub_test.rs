//! Stub-server integration tests for the workflow-runs read surface
//! (WRUN-01/02, 09-01).
//!
//! Proves through the real CLI binary, against a head-capturing stub:
//! - `--status` and `--include-dry-run` produce the right QUERY PARAMS ON
//!   THE WIRE (the contract — not the flag), and dry_run is ABSENT by default
//! - the locked empty-result hints: valid-statuses hint (pass-through, no
//!   client-side rejection) and the hidden-test-runs hint backed by exactly
//!   one probe (never on non-empty pages), quiet-suppressed
//! - run detail flattens every step to a row (node/status/input/output/
//!   error/duration) above/beside the run summary; --format json passes the
//!   run+steps document through verbatim
//! - missing required --workflow exits 2 BEFORE any HTTP
//!
//! Uses tests/common/mod.rs (shared Phase 9 helpers, no new crates).

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::sync::atomic::Ordering;

// -- Fixtures built from the verified wire shapes (09-RESEARCH § Server
//    Contract; serializer emits exactly these fields, ordered steps ASC) --

/// A complete 11-field run row.
fn run_row(id: &str, status: &str, dry_run: bool) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "workflow_id": "wf_1",
        "status": status,
        "trigger_data": {"dealId": "deal_001"},
        "error": null,
        "depth": 0,
        "dry_run": dry_run,
        "current_node_id": null,
        "started_at": "2026-01-01T00:00:00Z",
        "completed_at": "2026-01-01T00:00:05Z",
        "created_at": "2026-01-01T00:00:00Z"
    })
}

fn list_envelope(rows: Vec<serde_json::Value>, total: u64) -> String {
    serde_json::json!({
        "data": rows,
        "meta": {"total": total, "offset": 0, "limit": 50}
    })
    .to_string()
}

/// Detail body: run fields at the top level + two ordered steps —
/// one completed (3s duration, payload input/output), one failed.
fn detail_body() -> String {
    serde_json::json!({
        "data": {
            "id": "run_detail_1",
            "workflow_id": "wf_1",
            "status": "completed",
            "trigger_data": {"dealId": "deal_001"},
            "error": null,
            "depth": 0,
            "dry_run": false,
            "current_node_id": null,
            "started_at": "2026-01-01T00:00:00Z",
            "completed_at": "2026-01-01T00:00:05Z",
            "created_at": "2026-01-01T00:00:00Z",
            "steps": [
                {
                    "id": "step_1",
                    "run_id": "run_detail_1",
                    "node_id": "fetch_deal",
                    "status": "completed",
                    "input": {"dealId": "deal_001"},
                    "output": {"ok": true},
                    "error": null,
                    "resume_at": null,
                    "started_at": "2026-01-01T00:00:00Z",
                    "completed_at": "2026-01-01T00:05:00Z",
                    "created_at": "2026-01-01T00:00:00Z"
                },
                {
                    "id": "step_2",
                    "run_id": "run_detail_1",
                    "node_id": "send_email",
                    "status": "failed",
                    "input": null,
                    "output": null,
                    "error": "SMTP timeout",
                    "resume_at": null,
                    "started_at": null,
                    "completed_at": null,
                    "created_at": "2026-01-01T00:00:03Z"
                }
            ]
        }
    })
    .to_string()
}

// -- WRUN-01: runs list --

#[test]
fn runs_list_renders_rows_and_statuses() {
    let (url, _counter, _heads) = common::spawn_head_capturing_stub_server(&[(
        200,
        &list_envelope(vec![run_row("run_a", "completed", false), run_row("run_b", "failed", false)], 2),
    )]);
    common::cmd_with_server(&url)
        .args(["workflows", "runs", "list", "--workflow", "wf_1"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("run_a")
                .and(predicate::str::contains("completed"))
                .and(predicate::str::contains("run_b"))
                .and(predicate::str::contains("failed")),
        );
}

#[test]
fn runs_list_sends_status_and_dry_run_params_on_the_wire() {
    let (url, _counter, heads) = common::spawn_head_capturing_stub_server(&[(200, &list_envelope(vec![], 0))]);
    common::cmd_with_server(&url)
        .args([
            "workflows",
            "runs",
            "list",
            "--workflow",
            "wf_1",
            "--status",
            "failed",
            "--include-dry-run",
        ])
        .assert()
        .success();

    let heads = heads.lock().expect("heads lock");
    let request_line = &heads[0];
    // Contract assertions on the wire, order-tolerant.
    assert!(
        request_line.contains("status=failed"),
        "missing status=failed in: {request_line}"
    );
    assert!(
        request_line.contains("dry_run=true"),
        "missing dry_run=true in: {request_line}"
    );
}

#[test]
fn runs_list_omits_dry_run_param_by_default() {
    let (url, _counter, heads) = common::spawn_head_capturing_stub_server(&[(
        200,
        &list_envelope(vec![run_row("run_a", "pending", false)], 1),
    )]);
    common::cmd_with_server(&url)
        .args(["workflows", "runs", "list", "--workflow", "wf_1"])
        .assert()
        .success();

    let heads = heads.lock().expect("heads lock");
    assert!(
        !heads[0].contains("dry_run="),
        "dry_run must be absent without --include-dry-run: {}",
        heads[0]
    );
}

#[test]
fn runs_list_bogus_status_hints_valid_statuses_without_rejection() {
    let (url, counter, _heads) =
        common::spawn_head_capturing_stub_server(&[(200, &list_envelope(vec![], 0))]);
    // Pass-through: exit 0, hint on stderr — never a client-side rejection.
    common::cmd_with_server(&url)
        .args(["workflows", "runs", "list", "--workflow", "wf_1", "--status", "bogus"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Valid statuses: pending, running, completed, failed, waiting",
        ));

    // The status hint path fires NO probe.
    assert_eq!(counter.load(Ordering::SeqCst), 1, "exactly 1 request");
}

#[test]
fn runs_list_hidden_dry_run_hint_reports_true_count_via_one_probe() {
    let empty = list_envelope(vec![], 0);
    let probe = list_envelope(vec![run_row("run_dry", "completed", true)], 2);
    let (url, counter, heads) = common::spawn_head_capturing_stub_server(&[(200, &empty), (200, &probe)]);

    common::cmd_with_server(&url)
        .args(["workflows", "runs", "list", "--workflow", "wf_1"])
        .assert()
        .success()
        .stderr(
            predicate::str::contains("2 test run(s) hidden")
                .and(predicate::str::contains("--include-dry-run")),
        );

    // First page + EXACTLY ONE probe.
    assert_eq!(counter.load(Ordering::SeqCst), 2, "page + one probe");
    let heads = heads.lock().expect("heads lock");
    assert!(heads[1].contains("dry_run=true"), "probe opts in: {}", heads[1]);
    assert!(heads[1].contains("limit=1"), "probe is limit 1: {}", heads[1]);
}

#[test]
fn runs_list_quiet_suppresses_hidden_hint_but_still_probes() {
    let empty = list_envelope(vec![], 0);
    let probe = list_envelope(vec![run_row("run_dry", "completed", true)], 2);
    let (url, counter, _heads) = common::spawn_head_capturing_stub_server(&[(200, &empty), (200, &probe)]);

    // -q is a global flag — placed before the subcommand.
    common::cmd_with_server(&url)
        .args(["-q", "workflows", "runs", "list", "--workflow", "wf_1"])
        .assert()
        .success()
        .stderr(predicate::str::contains("hidden").not());

    // Quiet suppresses the HINT, not the probe logic.
    assert_eq!(counter.load(Ordering::SeqCst), 2, "still page + probe");
}

#[test]
fn runs_list_probe_never_fires_on_non_empty_page() {
    // Server also holds dry runs — the CLI must still send exactly one
    // request when the (server-filtered) page is non-empty.
    let (url, counter, heads) = common::spawn_head_capturing_stub_server(&[(
        200,
        &list_envelope(vec![run_row("run_a", "pending", false)], 5),
    )]);

    common::cmd_with_server(&url)
        .args(["workflows", "runs", "list", "--workflow", "wf_1"])
        .assert()
        .success();

    assert_eq!(counter.load(Ordering::SeqCst), 1, "no probe on non-empty page");
    let heads = heads.lock().expect("heads lock");
    assert!(!heads[0].contains("dry_run="));
}

// -- WRUN-02: runs get --

#[test]
fn runs_get_flattens_steps_to_rows_with_run_summary() {
    let (url, _counter, _heads) =
        common::spawn_head_capturing_stub_server(&[(200, &detail_body())]);

    common::cmd_with_server(&url)
        .args([
            "workflows",
            "runs",
            "get",
            "run_detail_1",
            "--workflow",
            "wf_1",
            "--format",
            "table",
        ])
        .assert()
        .success()
        .stdout(
            // Run summary present.
            predicate::str::contains("run_detail_1")
                // Steps table header row (locked columns).
                .and(predicate::str::contains("node"))
                .and(predicate::str::contains("status"))
                .and(predicate::str::contains("input"))
                .and(predicate::str::contains("output"))
                .and(predicate::str::contains("error"))
                .and(predicate::str::contains("duration"))
                // One row per step: node ids, statuses, payload + error cells.
                .and(predicate::str::contains("fetch_deal"))
                .and(predicate::str::contains("send_email"))
                .and(predicate::str::contains("SMTP timeout"))
                // Computed client-side duration for the 5-minute step
                // (sub-minute deltas render as "now").
                .and(predicate::str::contains("5 minutes")),
        );
}

#[test]
fn runs_get_json_passthrough_is_verbatim() {
    let (url, _counter, _heads) =
        common::spawn_head_capturing_stub_server(&[(200, &detail_body())]);

    let output = common::cmd_with_server(&url)
        .args([
            "workflows",
            "runs",
            "get",
            "run_detail_1",
            "--workflow",
            "wf_1",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("stdout is JSON");
    // Flattened run fields + steps — no synthetic wrapper key.
    assert_eq!(parsed["id"], "run_detail_1");
    assert_eq!(parsed["depth"], 0);
    assert_eq!(parsed["workflow_id"], "wf_1");
    let steps = parsed["steps"].as_array().expect("steps array");
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0]["node_id"], "fetch_deal");
    assert_eq!(steps[1]["node_id"], "send_email");
    assert!(parsed.get("run").is_none(), "no wrapper key: {parsed}");
}

#[test]
fn runs_get_without_workflow_fails_pre_http_with_exit_2() {
    // Unreachable server: if the CLI made ANY request, the error would be a
    // Connection failure with exit 1. clap must reject the missing required
    // --workflow flag first (exit 2, zero HTTP).
    common::cmd()
        .args(["workflows", "runs", "get", "run_1"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Connection failed").not());
}
