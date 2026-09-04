//! Typed custom-field writing — wire-level stub tests (CFLD-02/03, Phase 12).
//!
//! The canonical `price=4` test is the FIRST test in this file (task-order
//! contract): the server validates NOTHING on the v1 API path, so the CLI
//! resolver is the only type gate in existence. These tests pin the WIRE
//! BODY: `--custom-field price=4` on a number definition must carry JSON
//! number 4 — never the string "4", never 4.0.
//!
//! Uses tests/common/mod.rs (shared Phase 9 helpers, no new crates).

mod common;

use predicates::prelude::*;
use std::sync::atomic::Ordering;

/// The canonical bug fix (SC-2): `--custom-field price=4` on a number
/// definition stores JSON number 4 — raw body contains "price":4, never
/// "price":"4" and never 4.0. The definitions GET + the POST are both
/// counted (cold cache: 2 requests).
#[test]
fn number_int_body() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, DEAL_DEFS),
        (201, &created_deal_envelope()),
    ]);

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "deals",
            "create",
            "--title",
            "T",
            "--stage",
            "s1",
            "--custom-field",
            "price=4",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    assert_eq!(
        counter.load(Ordering::SeqCst),
        2,
        "cold cache: definitions GET + create POST"
    );

    let raw = last_captured_body(&bodies);
    let body: serde_json::Value = serde_json::from_str(&raw).expect("captured POST body parses");
    let price = &body["custom_fields"]["price"];
    assert!(
        price.is_number(),
        "price=4 on a number definition must store a JSON number: {raw}"
    );
    assert_eq!(
        price.as_i64(),
        Some(4),
        "integer input must keep i64 precision (never 4.0): {raw}"
    );
    // Raw-string assertions: a parsed Value comparison can mask 4.0 vs 4.
    assert!(
        raw.contains("\"price\":4"),
        "raw wire body must contain \"price\":4: {raw}"
    );
    assert!(
        !raw.contains("\"price\":\"4\""),
        "the string form must be dead: {raw}"
    );
    assert!(!raw.contains("4.0"), "the float form must be dead: {raw}");
}

/// A float input passes through as f64 (as_f64 == 4.5) — still a JSON number.
#[test]
fn number_float_body() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, _counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, DEAL_DEFS),
        (201, &created_deal_envelope()),
    ]);

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "deals",
            "create",
            "--title",
            "T",
            "--stage",
            "s1",
            "--custom-field",
            "price=4.5",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    let raw = last_captured_body(&bodies);
    let body: serde_json::Value = serde_json::from_str(&raw).expect("captured POST body parses");
    assert_eq!(
        body["custom_fields"]["price"].as_f64(),
        Some(4.5),
        "float input stays a float number: {raw}"
    );
}

/// Dry-run honesty (SC-4): with a COLD cache, --dry-run NEVER fetches
/// (counter == 0 — not even the definitions GET is scripted), falls back to
/// raw strings, and says so on stderr with a warm-the-cache note.
#[test]
fn dry_run_cold_cache_zero_http() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &created_deal_envelope())]);

    let output = common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "deals",
            "create",
            "--title",
            "T",
            "--stage",
            "s1",
            "--custom-field",
            "price=4",
            "--dry-run",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .get_output()
        .clone();

    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "dry-run with a cold cache must make ZERO HTTP requests (no defs fetch)"
    );

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let preview: serde_json::Value =
        serde_json::from_str(&stdout).expect("dry-run JSON preview parses");
    assert_eq!(
        preview["body"]["custom_fields"]["price"], "4",
        "cache-only fallback sends the value as a raw string"
    );

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        stderr.contains("not cached"),
        "the dry-run cache-miss note must be visible on stderr: {stderr}"
    );
}

/// The dry-run cache-miss note is suppressed by --quiet (exactly the two
/// advisory lines are quiet-suppressible — nothing else).
#[test]
fn dry_run_quiet_suppresses_note() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &created_deal_envelope())]);

    let output = common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "deals",
            "create",
            "--title",
            "T",
            "--stage",
            "s1",
            "--custom-field",
            "price=4",
            "--dry-run",
            "--quiet",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .get_output()
        .clone();

    assert_eq!(counter.load(Ordering::SeqCst), 0, "still zero HTTP");

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        !stderr.contains("not cached"),
        "--quiet must suppress the dry-run cache-miss note: {stderr}"
    );
}

/// With a WARM cache, --dry-run still writes TYPED values (cache-only is
/// not strings-only). The cache is warmed by `custom-fields list` —
/// exactly ONE request (a live create would make two). The dry-run run
/// itself adds ZERO requests; the typed value is proven on the dry-run
/// preview body.
#[test]
fn dry_run_warm_cache_types() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, DEAL_DEFS), (201, &created_deal_envelope())]);

    // Run 1: warm the definitions cache (one list GET).
    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "custom-fields",
            "list",
            "--entity-type",
            "deals",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);
    assert_eq!(counter.load(Ordering::SeqCst), 1, "warm-up is one list GET");

    // Run 2: dry-run with the warm cache — typed value, zero HTTP, no note.
    let output = common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "deals",
            "create",
            "--title",
            "T",
            "--stage",
            "s1",
            "--custom-field",
            "price=4",
            "--dry-run",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .get_output()
        .clone();

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "dry-run with a warm cache adds ZERO requests"
    );

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let preview: serde_json::Value =
        serde_json::from_str(&stdout).expect("dry-run JSON preview parses");
    assert_eq!(
        preview["body"]["custom_fields"]["price"].as_i64(),
        Some(4),
        "warm cache types correctly under dry-run: {stdout}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        !stderr.contains("not cached"),
        "warm cache must NOT print the cache-miss note: {stderr}"
    );
}

/// Client-side validation is pre-HTTP: a non-numeric value on a number
/// definition exits 2 with ZERO HTTP when the cache is warm (the type
/// check happens before any request would be needed).
#[test]
fn number_bad_warm_zero_post() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, DEAL_DEFS), (200, DEAL_DEFS)]);

    // Run 1: warm the cache.
    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args(["custom-fields", "list", "--entity-type", "deals"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);
    assert_eq!(counter.load(Ordering::SeqCst), 1, "warm-up is one list GET");

    // Run 2: invalid number on a warm cache — exit 2, ZERO HTTP.
    let output = common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args(["deals", "update", "d1", "--custom-field", "price=abc"])
        .env("NO_COLOR", "1")
        .assert()
        .code(2)
        .get_output()
        .clone();

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "validation failure on a warm cache must make ZERO HTTP requests"
    );

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        stderr.contains("number"),
        "the error must name the field's type: {stderr}"
    );
}

/// deals update resolves through the same resolver: cold cache GETs the
/// definitions first, then PUTs /api/v1/deals/d1 with the typed value.
#[test]
fn update_typed_body() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, _counter, heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, DEAL_DEFS),
        (200, &created_deal_envelope()),
    ]);

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "deals",
            "update",
            "d1",
            "--custom-field",
            "price=4",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    let head0 = captured_head(&heads, 0);
    assert!(
        head0.contains("get /api/v1/custom-field-definitions"),
        "cold cache must GET the definitions first: {head0}"
    );
    let head1 = captured_head(&heads, 1);
    assert!(
        head1.contains("put /api/v1/deals/d1"),
        "then PUT the deal: {head1}"
    );

    let raw = last_captured_body(&bodies);
    let body: serde_json::Value = serde_json::from_str(&raw).expect("captured PUT body parses");
    assert_eq!(
        body["custom_fields"]["price"].as_i64(),
        Some(4),
        "update stores the typed number: {raw}"
    );
}

// -- Task 2: fan-out (orgs / people / activities + the 8th call site) --

/// orgs create resolves through the same resolver: a number definition on
/// entity_type "organization" stores JSON number 7 (2 requests, cold).
#[test]
fn orgs_create_typed() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &definitions_list(vec![definition("co1", "organization", "employees", "number", None)])),
        (201, &created_org_envelope()),
    ]);

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "orgs",
            "create",
            "--name",
            "N",
            "--custom-field",
            "employees=7",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    assert_eq!(counter.load(Ordering::SeqCst), 2, "defs GET + POST");
    let raw = last_captured_body(&bodies);
    let body: serde_json::Value = serde_json::from_str(&raw).expect("captured body parses");
    assert_eq!(
        body["custom_fields"]["employees"].as_i64(),
        Some(7),
        "org number field stores a JSON number: {raw}"
    );
}

/// people create: number definition on entity_type "person" → as_i64 == 9.
#[test]
fn people_create_typed() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &definitions_list(vec![definition("cp1", "person", "salary", "number", None)])),
        (201, &created_person_envelope()),
    ]);

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "people",
            "create",
            "--first-name",
            "F",
            "--last-name",
            "L",
            "--custom-field",
            "salary=9",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    assert_eq!(counter.load(Ordering::SeqCst), 2, "defs GET + POST");
    let raw = last_captured_body(&bodies);
    let body: serde_json::Value = serde_json::from_str(&raw).expect("captured body parses");
    assert_eq!(
        body["custom_fields"]["salary"].as_i64(),
        Some(9),
        "person number field stores a JSON number: {raw}"
    );
}

/// activities create: boolean definition → JSON Bool(true) on the wire.
#[test]
fn activities_create_typed() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (
            200,
            &definitions_list(vec![definition(
                "ca1",
                "activity",
                "follow_up",
                "boolean",
                None,
            )]),
        ),
        (201, &created_activity_envelope()),
    ]);

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "activities",
            "create",
            "--title",
            "S",
            "--type",
            "call",
            "--custom-field",
            "follow_up=true",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    assert_eq!(counter.load(Ordering::SeqCst), 2, "defs GET + POST");
    let raw = last_captured_body(&bodies);
    let body: serde_json::Value = serde_json::from_str(&raw).expect("captured body parses");
    assert_eq!(
        body["custom_fields"]["follow_up"], serde_json::Value::Bool(true),
        "activity boolean field stores a JSON bool: {raw}"
    );
}

/// The 8th call site (Pitfall 6): --mark-undone resolves ONCE at the top of
/// activities update and the typed value is carried into the RAW payload —
/// the same body carries "completed_at":null AND the typed custom_fields
/// value, inserted exactly once (not dropped, not doubled).
#[test]
fn activities_mark_undone_raw_path() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (
            200,
            &definitions_list(vec![definition(
                "ca1",
                "activity",
                "follow_up",
                "boolean",
                None,
            )]),
        ),
        (200, &created_activity_envelope()),
    ]);

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "activities",
            "update",
            "a1",
            "--mark-undone",
            "--custom-field",
            "follow_up=true",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    assert_eq!(counter.load(Ordering::SeqCst), 2, "defs GET + raw PUT");
    let raw = last_captured_body(&bodies);
    assert!(
        raw.contains("\"completed_at\":null"),
        "the raw payload must clear completed_at: {raw}"
    );
    assert!(
        raw.contains("\"follow_up\":true"),
        "the raw payload must carry the TYPED boolean: {raw}"
    );
    assert_eq!(
        raw.matches("\"custom_fields\"").count(),
        1,
        "custom_fields must be inserted exactly once (never doubled): {raw}"
    );
}

/// --custom-field-json joins every --stdin exclusivity rule: exit 2 with
/// ZERO HTTP (the check precedes any request and any stdin read).
#[test]
fn exclusivity_json_stdin() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[]);

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "deals",
            "create",
            "--stdin",
            "--custom-field-json",
            "{}",
        ])
        .env("NO_COLOR", "1")
        .write_stdin(r#"[{"title":"A","stage_id":"s1"}]"#)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("mutually exclusive"));

    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "exclusivity refusal is pre-HTTP"
    );
}

/// Both custom-field flags together → exit 2 BEFORE the definitions fetch:
/// on a warm cache the failing run adds ZERO requests (required flags are
/// present so check_missing cannot be the exit source — the both-flags
/// check is).
#[test]
fn both_flags_zero_http_warm() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, DEAL_DEFS)]);

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args(["custom-fields", "list", "--entity-type", "deals"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);
    assert_eq!(counter.load(Ordering::SeqCst), 1, "warm-up list GET");

    let output = common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "deals",
            "create",
            "--title",
            "T",
            "--stage",
            "s1",
            "--custom-field",
            "a=1",
            "--custom-field-json",
            "{}",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(2)
        .get_output()
        .clone();

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "the both-flags refusal must add ZERO requests (fires before the fetch)"
    );
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        stderr.contains("mutually exclusive"),
        "both-flags detail must say mutually exclusive: {stderr}"
    );
}

/// The update no-flags gate counts --custom-field-json as "has flags":
/// the flag alone proceeds to the resolver and PUTs the verbatim object
/// (NOT rejected as "nothing to update").
#[test]
fn update_no_flags_gate_counts_json() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let (url, counter, heads, bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &created_deal_envelope())]);

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "deals",
            "update",
            "d1",
            "--custom-field-json",
            "{\"score\":1}",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("put /api/v1/deals/d1"),
        "json alone counts as has-flags and PUTs: {head}"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "the json bypass needs NO definitions fetch"
    );
    let raw = last_captured_body(&bodies);
    let body: serde_json::Value = serde_json::from_str(&raw).expect("captured body parses");
    assert_eq!(
        body["custom_fields"],
        serde_json::json!({"score": 1}),
        "verbatim object passes through untouched: {raw}"
    );
}

// -- fixtures (kept BELOW the tests: the task-order contract pins the first
//    fn in this file to number_int_body) --

/// One verified CustomFieldDefinition wire row (serialize.ts shape):
/// position is a JSON float, NO deleted_at, NO description.
fn definition(
    id: &str,
    entity_type: &str,
    name: &str,
    type_: &str,
    config: Option<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "entity_type": entity_type,
        "name": name,
        "type": type_,
        "config": config,
        "required": false,
        "position": 10000.0,
        "show_in_list": false,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z"
    })
}

/// Verified list envelope: {data: [...], meta: {total, offset, limit}}.
fn definitions_list(defs: Vec<serde_json::Value>) -> String {
    let total = defs.len();
    serde_json::json!({
        "data": defs,
        "meta": {"total": total, "offset": 0, "limit": 100}
    })
    .to_string()
}

/// Verified organization create/update response envelope.
fn created_org_envelope() -> String {
    serde_json::json!({
        "data": {
            "id": "o1",
            "name": "N",
            "website": null,
            "industry": null,
            "notes": null,
            "owner_id": "u1",
            "custom_fields": null,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
    })
    .to_string()
}

/// Verified person create/update response envelope.
fn created_person_envelope() -> String {
    serde_json::json!({
        "data": {
            "id": "p1",
            "first_name": "F",
            "last_name": "L",
            "full_name": null,
            "email": null,
            "phone": null,
            "notes": null,
            "organization_id": null,
            "owner_id": "u1",
            "custom_fields": null,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
    })
    .to_string()
}

/// Verified activity create/update response envelope.
fn created_activity_envelope() -> String {
    serde_json::json!({
        "data": {
            "id": "a1",
            "title": "S",
            "type_id": "call",
            "deal_id": null,
            "owner_id": "u1",
            "due_at": null,
            "completed_at": null,
            "notes": null,
            "custom_fields": null,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
    })
    .to_string()
}

/// Six-definition deal fixture (verified serializer shape, entity_type
/// "deal"): number, boolean, single_select with options, multi_select with
/// options, date, formula.
const DEAL_DEFS: &str = r#"{"data":[
 {"id":"cf1","entity_type":"deal","name":"price","type":"number","config":null,"required":false,"position":10000.0,"show_in_list":false,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"},
 {"id":"cf2","entity_type":"deal","name":"done","type":"boolean","config":null,"required":false,"position":20000.0,"show_in_list":false,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"},
 {"id":"cf3","entity_type":"deal","name":"status","type":"single_select","config":{"options":["new","won","lost"]},"required":false,"position":30000.0,"show_in_list":false,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"},
 {"id":"cf4","entity_type":"deal","name":"tags","type":"multi_select","config":{"options":["a","b","c"]},"required":false,"position":40000.0,"show_in_list":false,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"},
 {"id":"cf5","entity_type":"deal","name":"close_by","type":"date","config":null,"required":false,"position":50000.0,"show_in_list":false,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"},
 {"id":"cf6","entity_type":"deal","name":"calc_total","type":"formula","config":null,"required":false,"position":60000.0,"show_in_list":false,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}
],"meta":{"total":6,"offset":0,"limit":100}}"#;

/// Verified deal create/update response envelope (201/200 {data}).
fn created_deal_envelope() -> String {
    serde_json::json!({
        "data": {
            "id": "d1",
            "title": "T",
            "value": null,
            "stage_id": "s1",
            "organization_id": null,
            "person_id": null,
            "owner_id": "u1",
            "position": null,
            "expected_close_date": null,
            "notes": null,
            "custom_fields": null,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
    })
    .to_string()
}

/// The raw body of the LAST captured request (split at the first \r\n\r\n).
fn last_captured_body(bodies: &std::sync::Mutex<Vec<String>>) -> String {
    let guard = bodies.lock().expect("bodies lock");
    let last = guard.last().expect("at least one captured request");
    last.split("\r\n\r\n").nth(1).unwrap_or("").to_string()
}

fn captured_head(heads: &std::sync::Mutex<Vec<String>>, idx: usize) -> String {
    heads
        .lock()
        .expect("heads lock")
        .get(idx)
        .expect("captured head")
        .clone()
}
