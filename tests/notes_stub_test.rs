//! Stub-server integration tests for the notes surface (NOTE-01..04, SC-5,
//! Phase 10).
//!
//! Proves through the real CLI binary, against the shared head/body-capturing
//! stub:
//! - `notes list` renders id/created_at/truncated-flattened content (table)
//!   and full raw text via --format json; --limit/--offset reach the wire
//! - parent 404s render the server detail; empty pages hint on stderr
//!   (suppressed by --quiet)
//! - non-capable entity types reject pre-HTTP (exit 2, zero HTTP)
//! - the hidden `notes get` rejects pre-HTTP with the no-single-GET hint and
//!   stays out of `notes --help`
//!
//! Uses tests/common/mod.rs (shared Phase 9 helpers, no new crates).

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::sync::atomic::Ordering;

// -- Fixtures built from the verified wire shapes (10-RESEARCH § Code
//    Examples; SerializedNote emits exactly these fields — entity_type is
//    SINGULAR, author_id nullable, NO deleted_at) --

fn note_json(id: &str, entity_type: &str, entity_id: &str, content: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "entity_type": entity_type,
        "entity_id": entity_id,
        "content": content,
        "author_id": "u1",
        "source": "user",
        "created_at": "2026-09-01T10:00:00.000Z",
        "updated_at": "2026-09-01T10:00:00.000Z"
    })
}

fn notes_list_body(notes: Vec<serde_json::Value>) -> String {
    let total = notes.len();
    serde_json::json!({
        "data": notes,
        "meta": {"total": total, "offset": 0, "limit": 50}
    })
    .to_string()
}

/// Verified server 404 shape for a missing parent (label per segment).
fn parent_404(label: &str) -> String {
    serde_json::json!({
        "type": "https://api.pipelite.app/errors/ENTITY_NOT_FOUND",
        "title": "Not Found",
        "status": 404,
        "detail": format!("{label} not found")
    })
    .to_string()
}

// -- list (NOTE-01) --

#[test]
fn list_renders_table_with_id_created_at_header_and_content() {
    let (url, counter, heads, _bodies) = common::spawn_head_capturing_stub_server(&[(200,
        &notes_list_body(vec![note_json("n1", "deal", "d1", "hello from stub")]))]);

    common::cmd_with_server(&url)
        .args(["notes", "list", "deals", "d1", "--format", "table"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("n1")
                .and(predicate::str::contains("created_at"))
                .and(predicate::str::contains("hello from stub")),
        );

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let heads = heads.lock().expect("heads lock");
    assert!(
        heads[0].contains("get /api/v1/deals/d1/notes?limit=50&offset=0"),
        "wire head: {}",
        heads[0]
    );
}

#[test]
fn list_passes_limit_and_offset_to_the_wire() {
    let (url, _counter, heads, _bodies) = common::spawn_head_capturing_stub_server(&[(200,
        &notes_list_body(vec![]))]);

    common::cmd_with_server(&url)
        .args(["notes", "list", "orgs", "o1", "--limit", "10", "--offset", "5"])
        .assert()
        .success();

    let heads = heads.lock().expect("heads lock");
    assert!(
        heads[0].contains("get /api/v1/organizations/o1/notes?limit=10&offset=5"),
        "wire head: {}",
        heads[0]
    );
}

#[test]
fn list_table_flattens_newlines_in_content() {
    // T-10-01: multi-line content must render as ONE row — \n/\r flattened
    // to spaces in the table builder only.
    let (url, _counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(200,
        &notes_list_body(vec![note_json("n1", "deal", "d1", "line1\nline2")]))]);

    common::cmd_with_server(&url)
        .args(["notes", "list", "deals", "d1", "--format", "table"])
        .assert()
        .success()
        .stdout(predicate::str::contains("line1 line2"));
}

#[test]
fn list_table_truncates_long_content_but_json_keeps_full_text() {
    // 120-char content with a distinctive tail: table truncates to ~80 and
    // appends "...", dropping the final 20 chars; --format json prints the
    // FULL raw text (the view-before-edit path).
    let tail = "B".repeat(20);
    let content = format!("{}{tail}", "a".repeat(100));
    let (url, _counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &notes_list_body(vec![note_json("n1", "deal", "d1", &content)])),
        (200, &notes_list_body(vec![note_json("n1", "deal", "d1", &content)])),
    ]);

    common::cmd_with_server(&url)
        .args(["notes", "list", "deals", "d1", "--format", "table"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("...")
                .and(predicate::str::contains(tail.as_str()).not()),
        );

    common::cmd_with_server(&url)
        .args(["notes", "list", "deals", "d1", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(tail.as_str()));
}

#[test]
fn list_parent_404_renders_server_detail() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(404, &parent_404("Deal"))]);

    common::cmd_with_server(&url)
        .args(["notes", "list", "deals", "missing"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Deal not found"));
}

#[test]
fn list_empty_page_hints_and_quiet_suppresses() {
    let empty = serde_json::json!({
        "data": [],
        "meta": {"total": 0, "offset": 0, "limit": 50}
    })
    .to_string();
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &empty), (200, &empty)]);

    common::cmd_with_server(&url)
        .args(["notes", "list", "deals", "d1"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No notes"));

    common::cmd_with_server(&url)
        .args(["notes", "list", "deals", "d1", "--quiet"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No notes").not());
}

// -- SC-5: rejection surfaces (zero HTTP) --

#[test]
fn list_non_capable_entity_type_rejects_pre_http() {
    // Unreachable server: the entity-type gate must fire BEFORE any HTTP —
    // exit 2 with the locked message, never a Connection failure.
    common::cmd()
        .args(["notes", "list", "pipelines", "123"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("notes are only available on")
                .and(predicate::str::contains("Connection failed").not()),
        );
}

#[test]
fn get_hidden_variant_rejects_and_help_hides_it() {
    // The hidden Get variant is parse-then-error: exit 2 with the locked
    // hint BEFORE any HTTP (stub counter stays 0).
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[]);

    common::cmd_with_server(&url)
        .args(["notes", "get", "deals", "123"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("no single-note GET")
                .and(predicate::str::contains("Connection failed").not()),
        );
    assert_eq!(counter.load(Ordering::SeqCst), 0, "zero HTTP for notes get");

    // `notes --help` must not advertise a get subcommand (truthful help).
    let output = Command::cargo_bin("pipelite")
        .unwrap()
        .args(["notes", "--help"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("get"),
        "notes --help must not advertise a get subcommand:\n{stdout}"
    );
}
