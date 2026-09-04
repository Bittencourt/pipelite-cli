//! Stub-server integration tests for the custom-field-definitions surface
//! (CFLD-01, Phase 12).
//!
//! The create-body-shape tests are the FIRST tests in this file
//! (task-order contract): the server validates NOTHING on the v1 path, so
//! the CLI's pre-HTTP enum/config checks are the only line of defense, and
//! the POST body shape (--key → wire "name", no position, no description,
//! config.options as a real JSON array) is pinned at the wire level.
//!
//! Uses tests/common/mod.rs (shared Phase 9 helpers, no new crates).

mod common;

use predicates::prelude::*;
use std::sync::atomic::Ordering;

// -- create body shape (SC-1: written FIRST, before the handler exists) --

/// The flag-path create POST must map --key to the wire "name" (blob keys
/// are definition NAMES), normalize the entity alias (deals → deal) BEFORE
/// any HTTP, always carry required/show_in_list, and NEVER carry a position
/// key (the server strips it and auto-assigns max+10000) or a description
/// key (no such server field anywhere on the model/serializer/schemas).
#[test]
fn create_body_shape() {
    let (url, counter, heads, bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    common::cmd_with_server(&url)
        .args([
            "custom-fields",
            "create",
            "--entity-type",
            "deals",
            "--key",
            "price",
            "--type",
            "number",
            "--format",
            "json",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("post /api/v1/custom-field-definitions"),
        "create must POST /api/v1/custom-field-definitions: {head}"
    );

    let raw = last_captured_body(&bodies);
    let body: serde_json::Value =
        serde_json::from_str(&raw).expect("captured create body parses");
    assert_eq!(
        body["name"], "price",
        "--key must map to the wire 'name' (blob keys are definition names)"
    );
    assert_eq!(
        body["entity_type"], "deal",
        "entity aliases must normalize to the singular server token BEFORE the POST"
    );
    assert_eq!(body["type"], "number");
    assert_eq!(body["required"], false, "flag-path create always sends required (default false)");
    assert_eq!(
        body["show_in_list"], false,
        "flag-path create always sends show_in_list (default false)"
    );
    // Key ABSENCE can only be proven on the RAW string (a parsed Value masks
    // missing keys): no position, no description may ever reach the POST.
    assert!(
        !raw.contains("\"position\""),
        "NO position key may reach the POST (server strips it and auto-assigns max+10000): {raw}"
    );
    assert!(
        !raw.contains("\"description\""),
        "NO description key may reach the POST (no such server field exists): {raw}"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

/// `--type select` is an alias normalized to the wire enum `single_select`
/// (the server has only single_select), and `--options a,b` builds
/// config.options as a REAL JSON array of two strings — never a comma-string.
#[test]
fn create_select_alias_body() {
    let (url, _counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf2", "deal", "stage_sel", "single_select")),
    )]);

    common::cmd_with_server(&url)
        .args([
            "custom-fields",
            "create",
            "--entity-type",
            "deals",
            "--key",
            "stage_sel",
            "--type",
            "select",
            "--options",
            "a,b",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    let raw = last_captured_body(&bodies);
    let body: serde_json::Value =
        serde_json::from_str(&raw).expect("captured create body parses");
    assert_eq!(
        body["type"], "single_select",
        "select must normalize to the wire enum single_select"
    );
    assert_eq!(
        body["config"],
        serde_json::json!({"options": ["a", "b"]}),
        "--options must build config.options as a JSON array, never a comma-string: {raw}"
    );
}

// -- create validation & vocabulary (pre-HTTP: the server validates nothing) --

/// multi_select shares the SelectConfig shape: config.options is a flat
/// string array with every segment in order.
#[test]
fn create_multi_select_body() {
    let (url, _counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf3", "deal", "tags", "multi_select")),
    )]);

    common::cmd_with_server(&url)
        .args([
            "custom-fields",
            "create",
            "--entity-type",
            "deals",
            "--key",
            "tags",
            "--type",
            "multi_select",
            "--options",
            "x,y,z",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    let body: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("captured body parses");
    assert_eq!(body["type"], "multi_select");
    assert_eq!(
        body["config"],
        serde_json::json!({"options": ["x", "y", "z"]})
    );
}

/// Unknown --type tokens are rejected BEFORE any request (exit 2, counter
/// == 0) with the accepted vocabulary in the hint — the server would
/// silently accept "string" and store a dead definition.
#[test]
fn create_unknown_type() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    let output = common::cmd_with_server(&url)
        .args([
            "custom-fields",
            "create",
            "--entity-type",
            "deals",
            "--key",
            "price",
            "--type",
            "string",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("single_select"))
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Connection failed"),
        "type rejection must be pre-HTTP:\n{stderr}"
    );

    assert_eq!(counter.load(Ordering::SeqCst), 0, "zero HTTP on unknown type");
}

/// Unknown --entity-type values are rejected BEFORE any request (exit 2,
/// counter == 0) with the four server tokens in the hint — the server
/// would 422, but the CLI refuses earlier with a better message.
#[test]
fn create_unknown_entity() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    common::cmd_with_server(&url)
        .args([
            "custom-fields",
            "create",
            "--entity-type",
            "bogus",
            "--key",
            "price",
            "--type",
            "number",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("organization"));

    assert_eq!(counter.load(Ordering::SeqCst), 0, "zero HTTP on unknown entity type");
}

/// A missing --key is a MissingInput (exit 2, zero HTTP) naming the flag.
#[test]
fn create_missing_key() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    common::cmd_with_server(&url)
        .args([
            "custom-fields",
            "create",
            "--entity-type",
            "deal",
            "--type",
            "number",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--key"));

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

/// --options on a non-select type is rejected (exit 2, zero HTTP) — a
/// silently-ignored flag is the exact dead-value bug class this milestone
/// removes.
#[test]
fn create_options_on_text() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    common::cmd_with_server(&url)
        .args([
            "custom-fields",
            "create",
            "--entity-type",
            "deals",
            "--key",
            "price",
            "--type",
            "text",
            "--options",
            "a",
        ])
        .assert()
        .code(2);

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

/// A select-family type WITHOUT --options is rejected (exit 2, zero HTTP)
/// — an options-less select definition would render an empty dropdown.
#[test]
fn create_select_without_options() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf2", "deal", "stage_sel", "single_select")),
    )]);

    common::cmd_with_server(&url)
        .args([
            "custom-fields",
            "create",
            "--entity-type",
            "deals",
            "--key",
            "stage_sel",
            "--type",
            "select",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--options"));

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

// -- list (CFLD-01: --entity-type filter normalized pre-HTTP, pagination) --

/// The unfiltered list GET carries default pagination and NO entity_type
/// param; the table renders the serializer-exact columns including the
/// fractional position as-is.
#[test]
fn list_table() {
    let body = definitions_list_body(vec![definition_json("cf1", "deal", "price", "number")]);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "list", "--format", "table"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("number"))
        .stdout(predicate::str::contains("10000.5"));

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("get /api/v1/custom-field-definitions?limit=50&offset=0"),
        "list must GET /api/v1/custom-field-definitions with default pagination: {head}"
    );
    assert!(
        !head.contains("entity_type="),
        "an unfiltered list must NOT send an entity_type param: {head}"
    );
}

/// --entity-type deals normalizes to the singular server token ON THE WIRE.
#[test]
fn list_entity_type_filter() {
    let body = definitions_list_body(vec![definition_json("cf1", "deal", "price", "number")]);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "list", "--entity-type", "deals", "--format", "json"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("entity_type=deal"),
        "the normalized singular token must reach the wire: {head}"
    );
}

/// Org/people aliases normalize too (two invocations, one stub server).
#[test]
fn list_entity_type_aliases() {
    let body = definitions_list_body(vec![]);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body), (200, &body)]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "list", "--entity-type", "orgs", "--format", "json"])
        .assert()
        .code(0);
    common::cmd_with_server(&url)
        .args(["custom-fields", "list", "--entity-type", "people", "--format", "json"])
        .assert()
        .code(0);

    let head0 = captured_head(&heads, 0);
    assert!(
        head0.contains("entity_type=organization"),
        "orgs must normalize to organization: {head0}"
    );
    let head1 = captured_head(&heads, 1);
    assert!(
        head1.contains("entity_type=person"),
        "people must normalize to person: {head1}"
    );
}

#[test]
fn list_pagination() {
    let body = definitions_list_body(vec![]);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "list", "--limit", "2", "--offset", "4", "--format", "json"])
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(head.contains("limit=2&offset=4"), "pagination must reach the wire: {head}");
}

/// An empty list renders a creation hint on stderr (exit 0); --quiet
/// suppresses the hint but stays exit 0.
#[test]
fn list_empty_hint() {
    let body = definitions_list_body(vec![]);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body), (200, &body)]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "list", "--format", "json"])
        .assert()
        .code(0)
        .stderr(predicate::str::contains("No custom field definitions"));

    common::cmd_with_server(&url)
        .args(["custom-fields", "list", "--quiet", "--format", "json"])
        .assert()
        .code(0)
        .stderr(predicate::str::contains("No custom field definitions").not());
}

/// --format json preserves the FULL float (10000.5 — never truncated to
/// 10000 or stringified).
#[test]
fn list_json_position_full() {
    let body = definitions_list_body(vec![definition_json("cf1", "deal", "price", "number")]);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    let output = common::cmd_with_server(&url)
        .args(["custom-fields", "list", "--format", "json"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json list parses");
    assert_eq!(
        parsed[0]["position"], 10000.5,
        "the full float must survive in json output: {stdout}"
    );
}

// -- get --

#[test]
fn get_by_id() {
    let body = created_envelope(definition_json("cf1", "deal", "price", "number"));
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "get", "cf1", "--format", "json"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("price"));

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("get /api/v1/custom-field-definitions/cf1"),
        "get must GET /api/v1/custom-field-definitions/{{id}}: {head}"
    );
}

/// A missing definition 404s through the STANDARD NotFound path (exit 1) —
/// no special casing for tombstones (re-delete/re-get of soft-deleted rows
/// behaves exactly like any absent record).
#[test]
fn get_404() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(404, &not_found_body())]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "get", "cf_absent"])
        .assert()
        .code(1);

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

// -- create: stdin verbatim + dry-run --

/// --stdin passes the body through VERBATIM: no defaults injected, ordering
/// per stdin, config null preserved.
#[test]
fn create_stdin_verbatim() {
    let (url, counter, heads, bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    let stdin_json = r#"{"name":"price","entity_type":"deal","type":"number","config":null,"required":true,"show_in_list":false}"#;
    common::cmd_with_server(&url)
        .args(["custom-fields", "create", "--stdin"])
        .write_stdin(stdin_json)
        .assert()
        .code(0);

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let head = captured_head(&heads, 0);
    assert!(
        head.contains("post /api/v1/custom-field-definitions"),
        "stdin create must POST /api/v1/custom-field-definitions: {head}"
    );
    let raw = last_captured_body(&bodies);
    let captured: serde_json::Value = serde_json::from_str(&raw).expect("captured body parses");
    let expected: serde_json::Value = serde_json::from_str(stdin_json).expect("stdin parses");
    assert_eq!(
        captured, expected,
        "captured body must equal the stdin JSON verbatim (no default injection): {raw}"
    );
}

/// An invalid type inside --stdin JSON is rejected pre-HTTP (exit 2,
/// counter == 0) — full control does not include poisoned vocabularies.
#[test]
fn create_stdin_bad_type() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "create", "--stdin"])
        .write_stdin(r#"{"name":"price","entity_type":"deal","type":"string"}"#)
        .assert()
        .code(2);

    assert_eq!(counter.load(Ordering::SeqCst), 0, "zero HTTP on bad stdin type");
}

/// --dry-run previews the POST with ZERO HTTP.
#[test]
fn create_dry_run() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(
        201,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    let output = common::cmd_with_server(&url)
        .args([
            "custom-fields",
            "create",
            "--entity-type",
            "deals",
            "--key",
            "price",
            "--type",
            "number",
            "--dry-run",
        ])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert_eq!(counter.load(Ordering::SeqCst), 0, "dry-run must be zero HTTP");
    assert!(stdout.contains("POST"), "preview must name the method:\n{stdout}");
    assert!(
        stdout.contains("/api/v1/custom-field-definitions"),
        "preview must carry the endpoint URL:\n{stdout}"
    );
}

// -- update: partial PUT, immutable guard (CFLD-01) --

/// --position sends ONLY position on the PUT body, as a real f64 — and
/// NEVER carries the immutable entity_type/type keys.
#[test]
fn update_position_f64() {
    let (url, counter, heads, bodies) = common::spawn_head_capturing_stub_server(&[(
        200,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf1", "--position", "10000.5", "--format", "json"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("put /api/v1/custom-field-definitions/cf1"),
        "update must PUT /api/v1/custom-field-definitions/{{id}}: {head}"
    );
    let raw = last_captured_body(&bodies);
    let body: serde_json::Value = serde_json::from_str(&raw).expect("PUT body parses");
    assert_eq!(
        body["position"].as_f64(),
        Some(10000.5),
        "position must round-trip as f64: {raw}"
    );
    assert!(
        !raw.contains("entity_type"),
        "entity_type is immutable and must NEVER be sent: {raw}"
    );
    assert!(
        !raw.contains("\"type\""),
        "type is immutable and must NEVER be sent: {raw}"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 1, "exactly ONE request (the PUT)");
}

/// A single flag produces a body with EXACTLY that key — partial PUT,
/// nothing else (the server merges only what arrives).
#[test]
fn update_partial_body() {
    let (url, _counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[(
        200,
        &created_envelope(definition_json("cf1", "deal", "newname", "text")),
    )]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf1", "--name", "newname"])
        .assert()
        .code(0);

    let body: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("PUT body parses");
    assert_eq!(
        body,
        serde_json::json!({"name": "newname"}),
        "the PUT body must carry exactly the provided keys"
    );
}

/// Tri-state required: --no-required → false, --required → true, no flag →
/// the key is absent entirely (three invocations).
#[test]
fn update_required_tri_state() {
    let env = created_envelope(definition_json("cf1", "deal", "price", "number"));
    let (url, counter, _heads, bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &env), (200, &env), (200, &env)]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf1", "--no-required"])
        .assert()
        .code(0);
    common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf1", "--required"])
        .assert()
        .code(0);
    common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf1", "--name", "x"])
        .assert()
        .code(0);

    let body0: serde_json::Value =
        serde_json::from_str(&captured_body(&bodies, 0)).expect("body 0 parses");
    let body1: serde_json::Value =
        serde_json::from_str(&captured_body(&bodies, 1)).expect("body 1 parses");
    let raw2 = captured_body(&bodies, 2);
    let body2: serde_json::Value = serde_json::from_str(&raw2).expect("body 2 parses");

    assert_eq!(body0, serde_json::json!({"required": false}));
    assert_eq!(body1, serde_json::json!({"required": true}));
    assert!(
        body2.get("required").is_none() && !raw2.contains("\"required\""),
        "no tri-state flag given → the key must not be sent: {raw2}"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

/// Tri-state show_in_list mirrors required.
#[test]
fn update_show_in_list_tri_state() {
    let env = created_envelope(definition_json("cf1", "deal", "price", "number"));
    let (url, _counter, _heads, bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &env), (200, &env)]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf1", "--show-in-list"])
        .assert()
        .code(0);
    common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf1", "--no-show-in-list"])
        .assert()
        .code(0);

    let body0: serde_json::Value =
        serde_json::from_str(&captured_body(&bodies, 0)).expect("body 0 parses");
    let body1: serde_json::Value =
        serde_json::from_str(&captured_body(&bodies, 1)).expect("body 1 parses");
    assert_eq!(body0, serde_json::json!({"show_in_list": true}));
    assert_eq!(body1, serde_json::json!({"show_in_list": false}));
}

/// --config must be a JSON OBJECT and passes through verbatim; arrays are
/// rejected pre-HTTP (exit 2, zero HTTP for the bad run).
#[test]
fn update_config_object() {
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[(
        200,
        &created_envelope(definition_json("cf2", "deal", "stage_sel", "single_select")),
    )]);

    common::cmd_with_server(&url)
        .args([
            "custom-fields",
            "update",
            "cf2",
            "--config",
            r#"{"options":["a","b"]}"#,
        ])
        .assert()
        .code(0);

    let body: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("PUT body parses");
    assert_eq!(
        body["config"],
        serde_json::json!({"options": ["a", "b"]}),
        "config must pass through verbatim"
    );

    common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf2", "--config", "[1,2]"])
        .assert()
        .code(2);

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "the non-object config run must be zero HTTP"
    );
}

/// Update with no flags at all refuses (exit 2, zero HTTP) — never an
/// empty-body PUT.
#[test]
fn update_no_flags() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(
        200,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf1"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("nothing to update"));

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

/// --stdin strips immutable entity_type/type keys with ONE stderr warning
/// and PUTs the rest verbatim — exactly ONE request (no GET first).
#[test]
fn update_stdin_strips_immutable() {
    let (url, counter, heads, bodies) = common::spawn_head_capturing_stub_server(&[(
        200,
        &created_envelope(definition_json("cf1", "deal", "x", "number")),
    )]);

    let output = common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf1", "--stdin", "--format", "json"])
        .write_stdin(r#"{"type":"number","name":"x"}"#)
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        stderr.contains("immutable"),
        "the strip must warn on stderr:\n{stderr}"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "exactly ONE request (the PUT — stdin never GETs first)"
    );
    let head = captured_head(&heads, 0);
    assert!(
        head.contains("put /api/v1/custom-field-definitions/cf1"),
        "single request must be the PUT: {head}"
    );
    let raw = last_captured_body(&bodies);
    let body: serde_json::Value = serde_json::from_str(&raw).expect("PUT body parses");
    assert_eq!(body["name"], "x", "the non-immutable key must survive: {raw}");
    assert!(
        !raw.contains("\"type\""),
        "the immutable type key must be stripped: {raw}"
    );
}

/// --stdin together with update flags is mutually exclusive (exit 2, zero
/// HTTP).
#[test]
fn update_stdin_flag_mix() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(
        200,
        &created_envelope(definition_json("cf1", "deal", "price", "number")),
    )]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "update", "cf1", "--stdin", "--name", "x"])
        .write_stdin(r#"{"name":"y"}"#)
        .assert()
        .code(2);

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

// -- delete: the standard confirm contract (dry-run → TTY → --force →
//    non-TTY Validation exit-1 refusal) --

#[test]
fn delete_force() {
    let (url, counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "delete", "cf1", "--force"])
        .assert()
        .code(0);

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let head = captured_head(&heads, 0);
    assert!(
        head.contains("delete /api/v1/custom-field-definitions/cf1"),
        "delete must DELETE /api/v1/custom-field-definitions/{{id}}: {head}"
    );
}

/// Non-TTY without --force refuses BEFORE any HTTP (exit 1 Validation —
/// the STANDARD delete contract, deliberately distinct from trash purge's
/// exit-2 refusal).
#[test]
fn delete_non_tty_refusal() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "delete", "cf1"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("--force"));

    assert_eq!(counter.load(Ordering::SeqCst), 0, "refusal must precede any HTTP");
}

#[test]
fn delete_dry_run() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    let output = common::cmd_with_server(&url)
        .args(["custom-fields", "delete", "cf1", "--dry-run", "--format", "json"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert_eq!(counter.load(Ordering::SeqCst), 0, "dry-run must be zero HTTP");
    assert!(
        stdout.contains("/api/v1/custom-field-definitions/cf1"),
        "preview must carry the endpoint URL:\n{stdout}"
    );
}

/// Soft delete: the first delete 204s; re-deleting the SAME definition
/// 404s through the STANDARD NotFound path (exit 1, no special casing).
#[test]
fn delete_then_redelete_404() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, ""), (404, &not_found_body())]);

    common::cmd_with_server(&url)
        .args(["custom-fields", "delete", "cf1", "--force"])
        .assert()
        .code(0);

    common::cmd_with_server(&url)
        .args(["custom-fields", "delete", "cf1", "--force"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

/// list --entity-type deals warms the custom_fields_deal cache file;
/// delete --force invalidates the whole custom_fields_ prefix (two runs
/// against ONE stub server, HOME redirected for cache hermeticity).
#[test]
fn delete_invalidates_cache() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let list_body = definitions_list_body(vec![definition_json("cf1", "deal", "price", "number")]);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &list_body), (204, "")]);

    let cache_file = tmp.path().join(".pipelite/cache/custom_fields_deal.json");
    assert!(!cache_file.exists(), "cache file must not pre-exist");

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args(["custom-fields", "list", "--entity-type", "deals", "--format", "json"])
        .assert()
        .code(0);
    assert!(
        cache_file.exists(),
        "a filtered list must warm the per-entity cache file"
    );

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args(["custom-fields", "delete", "cf1", "--force"])
        .assert()
        .code(0);
    assert!(
        !cache_file.exists(),
        "delete must invalidate (remove) the custom_fields_deal cache file"
    );
}

/// create also invalidates the whole custom_fields_ prefix (two runs, one
/// stub server).
#[test]
fn create_invalidates_cache() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let list_body = definitions_list_body(vec![definition_json("cf1", "deal", "price", "number")]);
    let created = created_envelope(definition_json("cf9", "deal", "x", "text"));
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &list_body), (201, &created)]);

    let cache_file = tmp.path().join(".pipelite/cache/custom_fields_deal.json");

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args(["custom-fields", "list", "--entity-type", "deals", "--format", "json"])
        .assert()
        .code(0);
    assert!(cache_file.exists(), "seed run must warm the cache file");

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "custom-fields",
            "create",
            "--entity-type",
            "deals",
            "--key",
            "x",
            "--type",
            "text",
        ])
        .assert()
        .code(0);
    assert!(
        !cache_file.exists(),
        "create must invalidate (remove) the custom_fields_deal cache file"
    );
}

// -- helpers (kept BELOW the body-shape tests: the task-order contract pins
//    the first fn in this file to create_body_shape) --

/// Verified CustomFieldDefinition wire shape (serialize.ts:143-156):
/// position is a JSON float (or null); NO deleted_at, NO description.
fn definition_json(id: &str, entity_type: &str, name: &str, type_: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "entity_type": entity_type,
        "name": name,
        "type": type_,
        "config": null,
        "required": false,
        "position": 10000.5,
        "show_in_list": false,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z"
    })
}

/// Verified create-response envelope (201 {data}).
fn created_envelope(def: serde_json::Value) -> String {
    serde_json::json!({ "data": def }).to_string()
}

/// Verified list envelope: {data: [...], meta: {total, offset, limit}}.
fn definitions_list_body(defs: Vec<serde_json::Value>) -> String {
    let total = defs.len();
    serde_json::json!({
        "data": defs,
        "meta": {"total": total, "offset": 0, "limit": 50}
    })
    .to_string()
}

/// Verified server 404 shape (RFC 7807) — the standard NotFound path.
fn not_found_body() -> String {
    serde_json::json!({
        "type": "https://api.pipelite.app/errors/NOT_FOUND",
        "title": "Not Found",
        "status": 404,
        "detail": "custom field definition not found"
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

/// The raw body of the request at `idx` (split at the first \r\n\r\n).
fn captured_body(bodies: &std::sync::Mutex<Vec<String>>, idx: usize) -> String {
    let guard = bodies.lock().expect("bodies lock");
    let body = guard.get(idx).expect("captured body");
    body.split("\r\n\r\n").nth(1).unwrap_or("").to_string()
}
