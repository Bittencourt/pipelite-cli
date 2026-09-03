//! Stub-server integration tests for the workflow-templates surface
//! (TPL-01, 09-02).
//!
//! Proves through the real CLI binary, against the shared head/body-capturing
//! stub:
//! - list/get render template rows
//! - `templates create --workflow` maps triggers[0] -> trigger (the array
//!   NEVER leaks into the create payload), warns on stderr when the workflow
//!   has multiple triggers, and rejects zero-trigger workflows pre-POST
//! - `--stdin` passes a raw body through verbatim (single request)
//! - exactly-one trigger source is enforced BEFORE any HTTP (exit 2)
//! - `--workflow` + `--nodes` is rejected pre-HTTP (the snapshot's nodes
//!   cannot be overridden — WR-01)
//! - 422 errors[] passthrough (Phase 8 layer)
//! - the delete confirmation contract: --force bypass, non-TTY refusal
//!   (exit 1, zero HTTP), --dry-run preview
//! - the hidden `templates update` exits 2 with the locked
//!   delete-and-recreate hint before any HTTP
//!
//! Uses tests/common/mod.rs (shared Phase 9 helpers, no new crates).

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::sync::atomic::Ordering;

// -- Fixtures built from the verified wire shapes (09-RESEARCH § Server
//    Contract 3; serializeWorkflowTemplate emits exactly these fields) --

/// A complete workflow body (single envelope) with the given triggers array.
fn workflow_body(triggers: serde_json::Value) -> String {
    serde_json::json!({
        "data": {
            "id": "wf_1",
            "name": "Deal Alert",
            "description": null,
            "triggers": triggers,
            "nodes": [{"id": "n1"}],
            "active": false,
            "created_by": "user_1",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
    })
    .to_string()
}

/// One-trigger workflow: crm_event on deals (distinguishing keys/values the
/// POST-body assertions check).
fn one_trigger() -> serde_json::Value {
    serde_json::json!([{"type": "crm_event", "entity": "deal"}])
}

fn two_triggers() -> serde_json::Value {
    serde_json::json!([{"type": "crm_event", "entity": "deal"}, {"type": "schedule"}])
}

/// A complete template body (single envelope).
fn template_envelope(id: &str, name: &str) -> String {
    serde_json::json!({
        "data": {
            "id": id,
            "name": name,
            "description": "Snapshot of Deal Alert",
            "category": "sales",
            "trigger": {"type": "crm_event", "entity": "deal"},
            "nodes": [{"id": "n1"}],
            "created_at": "2026-01-01T00:00:00Z"
        }
    })
    .to_string()
}

fn templates_list_body() -> String {
    serde_json::json!({
        "data": [
            {
                "id": "tpl_1",
                "name": "Deal Alert",
                "description": null,
                "category": "sales",
                "trigger": {"type": "crm_event", "entity": "deal"},
                "nodes": [],
                "created_at": "2026-01-01T00:00:00Z"
            },
            {
                "id": "tpl_2",
                "name": "Nightly Digest",
                "description": null,
                "category": null,
                "trigger": {"type": "schedule"},
                "nodes": [],
                "created_at": "2026-01-02T00:00:00Z"
            }
        ],
        "meta": {"total": 2, "offset": 0, "limit": 50}
    })
    .to_string()
}

/// Verified server 422 shape: errors[] carries the real info.
fn body_422() -> String {
    serde_json::json!({
        "type": "https://api.pipelite.app/errors/VALIDATION_ERROR",
        "title": "Validation Error",
        "status": 422,
        "detail": "Request validation failed",
        "errors": [{"field": "name", "code": "invalid", "message": "Template name too long"}]
    })
    .to_string()
}

/// The raw body of the LAST captured request (split at the first \r\n\r\n).
fn last_captured_body(bodies: &std::sync::Mutex<Vec<String>>) -> String {
    let guard = bodies.lock().expect("bodies lock");
    let last = guard.last().expect("at least one captured request");
    last.split("\r\n\r\n").nth(1).unwrap_or("").to_string()
}

// -- list / get --

#[test]
fn templates_list_renders_ids_and_names() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &templates_list_body())]);

    common::cmd_with_server(&url)
        .args(["templates", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("tpl_1")
                .and(predicate::str::contains("Deal Alert"))
                .and(predicate::str::contains("tpl_2"))
                .and(predicate::str::contains("Nightly Digest")),
        );
}

#[test]
fn templates_get_renders_name_and_category() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &template_envelope("tpl_1", "Deal Alert"))]);

    common::cmd_with_server(&url)
        .args(["templates", "get", "tpl_1"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Deal Alert").and(predicate::str::contains("sales")),
        );
}

// -- create: triggers[0] -> trigger mapping (T-09-05) --

#[test]
fn create_via_workflow_maps_first_trigger_into_the_post_body() {
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &workflow_body(one_trigger())),
        (201, &template_envelope("tpl_new", "T")),
    ]);

    common::cmd_with_server(&url)
        .args(["templates", "create", "--name", "T", "--workflow", "wf_1"])
        .assert()
        .success();

    // Request 1: the workflow GET. Request 2: the template POST.
    assert_eq!(counter.load(Ordering::SeqCst), 2, "workflow GET + template POST");

    let posted: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("POST body is JSON");
    // trigger == triggers[0] (distinguishing key/value of the first trigger).
    assert_eq!(posted["trigger"]["type"], "crm_event");
    assert_eq!(posted["trigger"]["entity"], "deal");
    assert_eq!(posted["name"], "T");
    // The workflow's nodes are cloned into the create payload.
    assert!(posted.get("nodes").is_some(), "nodes present: {posted}");
    assert_eq!(posted["nodes"][0]["id"], "n1");
    // The triggers ARRAY never leaks into the create payload.
    assert!(
        posted.get("triggers").is_none(),
        "triggers array must not appear: {posted}"
    );
}

#[test]
fn create_via_workflow_warns_when_two_triggers_but_still_maps_the_first() {
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &workflow_body(two_triggers())),
        (201, &template_envelope("tpl_new", "T")),
    ]);

    common::cmd_with_server(&url)
        .args(["templates", "create", "--name", "T", "--workflow", "wf_1"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "warning: workflow has 2 triggers; only the first was captured",
        ));

    let posted: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("POST body is JSON");
    assert_eq!(
        posted["trigger"]["type"], "crm_event",
        "the FIRST trigger is captured: {posted}"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn create_zero_trigger_workflow_rejected_after_single_fetch() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &workflow_body(serde_json::json!([])))]);

    common::cmd_with_server(&url)
        .args(["templates", "create", "--name", "T", "--workflow", "wf_1"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("no triggers to snapshot")
                .and(predicate::str::contains("--stdin")),
        );

    // Exactly 1 request: the workflow GET — no template POST attempted.
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn create_stdin_passes_the_body_through_verbatim() {
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[(201,
        &template_envelope("tpl_new", "S"))]);

    common::cmd_with_server(&url)
        .args(["templates", "create", "--stdin"])
        .write_stdin(r#"{"name":"S","trigger":{"type":"schedule"}}"#)
        .assert()
        .success();

    // Exactly 1 request: the POST — no workflow fetch.
    assert_eq!(counter.load(Ordering::SeqCst), 1, "stdin: POST only");

    let posted: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("POST body is JSON");
    assert_eq!(posted["name"], "S");
    assert_eq!(posted["trigger"]["type"], "schedule");
}

#[test]
fn create_422_passes_the_joined_field_message_through() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(422, &body_422())]);

    common::cmd_with_server(&url)
        .args([
            "templates",
            "create",
            "--name",
            "T",
            "--trigger",
            r#"{"type":"x"}"#,
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Template name too long"));
}

#[test]
fn create_without_source_rejected_pre_http() {
    // Unreachable server: the handler must reject BEFORE any HTTP.
    common::cmd()
        .args(["templates", "create"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("--workflow")
                .and(predicate::str::contains("--trigger"))
                .and(predicate::str::contains("--stdin"))
                .and(predicate::str::contains("Connection failed").not()),
        );
}

#[test]
fn create_with_two_sources_rejected_pre_http() {
    common::cmd()
        .args([
            "templates",
            "create",
            "--name",
            "T",
            "--workflow",
            "wf_1",
            "--trigger",
            r#"{"type":"x"}"#,
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Connection failed").not());
}

#[test]
fn create_workflow_with_nodes_rejected_pre_http() {
    // WR-01: --workflow snapshots the workflow's own nodes, so an explicit
    // --nodes must be REJECTED (exit 2) before any HTTP instead of being
    // silently discarded in favor of the snapshot. Unreachable server: any
    // request would surface as "Connection failed".
    common::cmd()
        .args([
            "templates",
            "create",
            "--name",
            "T",
            "--workflow",
            "wf_1",
            "--nodes",
            r#"[{"id":"n9"}]"#,
        ])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("--workflow and --nodes are mutually exclusive")
                .and(predicate::str::contains(
                    "--nodes cannot be combined with --workflow",
                ))
                .and(predicate::str::contains("Connection failed").not()),
        );
}

#[test]
fn create_dry_run_previews_the_mapped_post_without_template_post() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &workflow_body(one_trigger()))]);

    common::cmd_with_server(&url)
        .args([
            "--dry-run",
            "templates",
            "create",
            "--name",
            "T",
            "--workflow",
            "wf_1",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("POST")
                .and(predicate::str::contains("/api/v1/workflow-templates")),
        );

    // Only the workflow GET is served — the template POST is previewed.
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

// -- delete: dry-run -> confirm -> force ordering --

#[test]
fn delete_force_against_204_succeeds_with_one_request() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(204, "")]);

    common::cmd_with_server(&url)
        .args(["templates", "delete", "tpl_1", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted workflow template tpl_1"));

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn delete_non_tty_without_force_refuses_with_zero_http() {
    // Unreachable server + non-TTY stdin: the refusal must fire BEFORE any
    // HTTP (exit 1 via CliError::Validation, matching workflows delete).
    common::cmd()
        .args(["templates", "delete", "tpl_1"])
        .assert()
        .code(1)
        .stderr(
            predicate::str::contains("--force")
                .and(predicate::str::contains("Connection failed").not()),
        );
}

#[test]
fn delete_dry_run_previews_the_url_with_zero_http() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    // JSON format renders the exact URL in the preview document.
    common::cmd_with_server(&url)
        .args([
            "templates",
            "delete",
            "tpl_1",
            "--dry-run",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("/api/v1/workflow-templates/tpl_1"));

    assert_eq!(counter.load(Ordering::SeqCst), 0, "--dry-run makes no requests");
}

// -- hidden update: parse-then-error, zero HTTP (T-09-07) --

#[test]
fn update_hidden_variant_rejects_with_hint_before_any_http() {
    // Unreachable server: exit 2 with the locked hint — and never a
    // Connection failure (zero HTTP).
    common::cmd()
        .args(["templates", "update", "tpl_1", "--name", "X"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("delete and recreate")
                .and(predicate::str::contains("Connection failed").not()),
        );
}

// -- truthful help --

#[test]
fn templates_help_states_no_update_and_hides_the_subcommand() {
    let output = Command::cargo_bin("pipelite")
        .unwrap()
        .args(["templates", "--help"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("no template update"), "help: {stdout}");
    assert!(
        stdout.lines().all(|l| !l.trim().starts_with("update")),
        "templates --help must not advertise an update subcommand:\n{stdout}"
    );
}
