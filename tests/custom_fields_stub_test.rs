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
