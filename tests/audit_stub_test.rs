//! Stub-server integration tests for the audit surface (AUDT-01..02,
//! SC-5, Phase 11).
//!
//! Test order follows task order: all 13 tests cover `audit list` — the four
//! passthrough filters (verbatim values, EMPTY values omitted from the wire,
//! P4), clamped pagination (limit 0→1, 150→100; NO --all), the
//! timestamp/actor/action/entity display contract with table-only
//! truncation, --json-only verbatim changes payload, the 422 passthrough
//! (no client-side enum gate), and the PINNED Forbidden probes proving the
//! server's gate-before-validation ordering: a non-admin 403 renders the
//! audit hint with NO filters AND with an invalid filter value.
//!
//! Wire keys are the server's snake_case query params (RESEARCH § Audit:
//! GET /api/v1/audit?entity_type&entity_id&actor_kind&workflow_run_id&offset&limit).
//!
//! Uses tests/common/mod.rs (shared Phase 9 helpers, no new crates).

mod common;

use predicates::prelude::*;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

// -- fixtures (verified wire shapes from RESEARCH § Audit) --

/// Single audit entry with a `user` actor — the verified stub fixture from
/// the plan's interfaces (changes = {field: {from, to}}).
fn audit_entry_user(id: &str, entity_type: &str, entity_id: &str, action: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "entity_type": entity_type,
        "entity_id": entity_id,
        "action": action,
        "changes": {"value": {"from": 100, "to": 200}},
        "actor_kind": "user",
        "actor_user_id": "u1",
        "workflow_run_id": null,
        "import_session_id": null,
        "created_at": "2026-09-01T12:00:00.000Z"
    })
}

/// Workflow-run actor variant: actor_user_id null, workflow_run_id set,
/// empty changes (legitimate — e.g. a create/delete entry).
fn audit_entry_workflow_run(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "entity_type": "deal",
        "entity_id": "d1",
        "action": "updated",
        "changes": {},
        "actor_kind": "workflow_run",
        "actor_user_id": null,
        "workflow_run_id": "wr9",
        "import_session_id": null,
        "created_at": "2026-09-01T12:00:00.000Z"
    })
}

/// Verified list envelope `{data, meta:{total, offset, limit}}`.
fn list_body(rows: Vec<serde_json::Value>, total: u64) -> String {
    serde_json::json!({
        "data": rows,
        "meta": {"total": total, "offset": 0, "limit": 50}
    })
    .to_string()
}

/// Verified RFC 7807 server 403 shape (same fixture family as 11-01's
/// interfaces — the audit gate fires BEFORE query validation, so the body
/// carries no filter-specific detail).
fn forbidden_403() -> String {
    serde_json::json!({
        "type": "https://api.pipelite.app/errors/FORBIDDEN",
        "title": "Forbidden",
        "status": 403,
        "detail": "You don't have access to this resource"
    })
    .to_string()
}

/// Verified server 422 shape with a populated errors[] array (invalid
/// entity_type enum value) — the CLI must render the joined field message
/// untouched (no client-side enum gate).
fn validation_422() -> String {
    r#"{"type":"https://api.pipelite.app/errors/VALIDATION_ERROR","title":"Validation Error","status":422,"detail":"Request validation failed","errors":[{"field":"entity_type","code":"invalid","message":"Invalid entity type"}]}"#
        .to_string()
}

/// The captured (lowercased) head of request `idx`.
fn captured_head(heads: &Mutex<Vec<String>>, idx: usize) -> String {
    heads
        .lock()
        .expect("heads lock")
        .get(idx)
        .expect("captured head")
        .clone()
}

// -- list defaults, filters, pagination (AUDT-01) --

/// The default list: limit=50&offset=0 on the wire, and the table renders
/// the entity cell (type/id), the actor kind, the action, and the
/// timestamp. The table layer renders `*_at` fields as relative time
/// (pre-existing format.rs convention), so the exact timestamp value is
/// asserted via --json.
#[test]
fn list_default() {
    let body = list_body(vec![audit_entry_user("a1", "deal", "d1", "updated")], 1);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body), (200, &body)]);

    common::cmd_with_server(&url)
        .args(["audit", "list", "--format", "table"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("deal/d1"))
        .stdout(predicate::str::contains("user"))
        .stdout(predicate::str::contains("updated"))
        .stdout(predicate::str::contains("ago"));

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("get /api/v1/audit?limit=50&offset=0"),
        "default pagination must reach the wire on /api/v1/audit:\n{head}"
    );

    // The created_at value itself is visible verbatim in json output.
    common::cmd_with_server(&url)
        .args(["audit", "list", "--format", "json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("2026-09-01T12:00:00.000Z"));
}

/// All four filters pass through VERBATIM as the server's snake_case query
/// params — the CLI validates nothing (the server owns enum validation).
#[test]
fn filters_verbatim() {
    let body = list_body(vec![audit_entry_workflow_run("a1")], 1);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args([
            "audit",
            "list",
            "--entity-type",
            "deal",
            "--entity-id",
            "d1",
            "--actor-kind",
            "workflow_run",
            "--workflow-run-id",
            "wr9",
            "--format",
            "json",
        ])
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("entity_type=deal"),
        "--entity-type must reach the wire verbatim:\n{head}"
    );
    assert!(
        head.contains("entity_id=d1"),
        "--entity-id must reach the wire verbatim:\n{head}"
    );
    assert!(
        head.contains("actor_kind=workflow_run"),
        "--actor-kind must reach the wire verbatim:\n{head}"
    );
    assert!(
        head.contains("workflow_run_id=wr9"),
        "--workflow-run-id must reach the wire verbatim:\n{head}"
    );
}

/// An EMPTY --entity-id value is OMITTED from the query entirely (P4 — the
/// server 422s `entity_id=`, min length 1); the request still succeeds
/// against a 200 stub.
#[test]
fn empty_flag_omitted() {
    let body = list_body(vec![], 0);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["audit", "list", "--entity-id", "", "--format", "json"])
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        !head.contains("entity_id="),
        "empty --entity-id must be OMITTED from the wire (server 422s entity_id=):\n{head}"
    );
    assert!(
        head.contains("get /api/v1/audit?limit=50&offset=0"),
        "the request must still carry default pagination:\n{head}"
    );
}

/// --limit 150 clamps DOWN to the server's page max (100) on the wire.
#[test]
fn limit_clamped_high() {
    let body = list_body(vec![], 0);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["audit", "list", "--limit", "150", "--format", "json"])
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("limit=100"),
        "--limit 150 must clamp to limit=100 (server page max):\n{head}"
    );
    assert!(
        !head.contains("limit=150"),
        "the unclamped value must never reach the wire:\n{head}"
    );
}

/// --limit 0 clamps UP to 1 on the wire (server clamps [1,100]).
#[test]
fn limit_clamped_low() {
    let body = list_body(vec![], 0);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["audit", "list", "--limit", "0", "--format", "json"])
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("limit=1"),
        "--limit 0 must clamp to limit=1:\n{head}"
    );
    assert!(
        !head.contains("limit=0"),
        "the unclamped value must never reach the wire:\n{head}"
    );
}

// -- --json display contract (AUDT-01) --

/// --format json renders the changes payload VERBATIM ({field: {from, to}})
/// and the actor id in full — the table never carries either (AUDT-03 diff
/// rendering deliberately deferred).
#[test]
fn json_changes_payload_verbatim() {
    let body = list_body(vec![audit_entry_user("a1", "deal", "d1", "updated")], 1);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    let output = common::cmd_with_server(&url)
        .args(["audit", "list", "--format", "json"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json list parses");

    assert_eq!(
        parsed[0]["changes"]["value"]["from"], 100,
        "changes must pass through verbatim: {stdout}"
    );
    assert_eq!(parsed[0]["changes"]["value"]["to"], 200);
    assert_eq!(parsed[0]["actor_user_id"], "u1");
    assert!(parsed[0]["workflow_run_id"].is_null());
}

/// --format json exposes ALL actor id fields: the workflow-run entry shows
/// workflow_run_id with actor_user_id null.
#[test]
fn json_shows_all_actor_ids() {
    let body = list_body(vec![audit_entry_workflow_run("a1")], 1);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    let output = common::cmd_with_server(&url)
        .args(["audit", "list", "--format", "json"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json list parses");

    assert_eq!(parsed[0]["workflow_run_id"], "wr9");
    assert!(parsed[0]["actor_user_id"].is_null());
    assert_eq!(parsed[0]["actor_kind"], "workflow_run");
}

// -- PINNED Forbidden probes (AUDT-02, gate-before-validation ordering) --

/// A non-admin key 403s even with NO filters — the server gates BEFORE
/// query-string validation, so the registered audit hint renders on the
/// bare list (PINNED probe).
#[test]
fn forbidden_empty_filter_renders_audit_hint() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(403, &forbidden_403())]);

    common::cmd_with_server(&url)
        .args(["audit", "list"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "The audit log requires an admin API key.",
        ));
}

/// The SAME hint renders with an INVALID filter value — the CLI adds no
/// validation layer that could reorder or mask the gate: it passed the
/// bogus value through and the server 403'd before validating it
/// (ordering proven end-to-end; PINNED probe).
#[test]
fn forbidden_invalid_filter_still_gate() {
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(403, &forbidden_403())]);

    common::cmd_with_server(&url)
        .args(["audit", "list", "--entity-type", "bogus"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "The audit log requires an admin API key.",
        ));

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("entity_type=bogus"),
        "the invalid value must reach the wire (no client-side gate):\n{head}"
    );
}

// -- server validation passthrough + display tolerance --

/// An invalid enum value 422s SERVER-SIDE and the errors[] field message
/// renders untouched — proof the CLI does no client-side enum validation
/// (Responsibility Map: audit filter validation = API server tier).
#[test]
fn validation_422_passthrough() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(422, &validation_422())]);

    common::cmd_with_server(&url)
        .args(["audit", "list", "--entity-type", "bogus"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "entity_type: Invalid entity type (invalid)",
        ));
}

/// The `merged` action (4th value of the server's closed union) renders
/// like any other — the action column is a plain String, not an enum.
#[test]
fn merged_action_tolerated() {
    let body = list_body(vec![audit_entry_user("a1", "deal", "d1", "merged")], 1);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["audit", "list", "--format", "table"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("merged"));
}

/// A 100-char entity_id truncates in the TABLE cell (ellipsis, tail
/// dropped) while --format json keeps the FULL id — table-only truncation
/// in both directions (T-11 style display contract).
#[test]
fn entity_cell_truncation() {
    let long_id = format!("{}{}", "x".repeat(91), "tail9xyz");
    let body = list_body(
        vec![audit_entry_user("a1", "deal", &long_id, "updated")],
        1,
    );
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body), (200, &body)]);

    common::cmd_with_server(&url)
        .args(["audit", "list", "--format", "table"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("..."))
        .stdout(predicate::str::contains("tail9xyz").not());

    common::cmd_with_server(&url)
        .args(["audit", "list", "--format", "json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("tail9xyz"));
}

// -- empty list --

/// An empty audit log prints one stderr hint (filters/offset pointer),
/// suppressed under --quiet, exit 0 either way.
#[test]
fn list_empty_hint() {
    let body = list_body(vec![], 0);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body), (200, &body)]);

    common::cmd_with_server(&url)
        .args(["audit", "list", "--format", "json"])
        .assert()
        .code(0)
        .stderr(predicate::str::contains("No audit entries"));

    common::cmd_with_server(&url)
        .args(["audit", "list", "--quiet", "--format", "json"])
        .assert()
        .code(0)
        .stderr(predicate::str::contains("No audit entries").not());
}
