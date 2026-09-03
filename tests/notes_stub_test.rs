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

fn note_envelope(note: serde_json::Value) -> String {
    serde_json::json!({ "data": note }).to_string()
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

/// Verified server 403 shape (author-or-admin gate, item routes).
fn forbidden_403() -> String {
    serde_json::json!({
        "type": "https://api.pipelite.app/errors/FORBIDDEN",
        "title": "Forbidden",
        "status": 403,
        "detail": "You don't have access to this resource"
    })
    .to_string()
}

/// Verified server 422 shape: errors[] carries the real info.
fn content_422() -> String {
    serde_json::json!({
        "type": "https://api.pipelite.app/errors/VALIDATION_ERROR",
        "title": "Validation Error",
        "status": 422,
        "detail": "Request validation failed",
        "errors": [{"field": "content", "code": "too_small", "message": "Note content is required"}]
    })
    .to_string()
}

/// Verified note 404 shape (missing or soft-deleted note).
fn note_404() -> String {
    serde_json::json!({
        "type": "https://api.pipelite.app/errors/ENTITY_NOT_FOUND",
        "title": "Not Found",
        "status": 404,
        "detail": "Note not found"
    })
    .to_string()
}

/// The raw body of the LAST captured request (split at the first \r\n\r\n).
fn last_captured_body(bodies: &std::sync::Mutex<Vec<String>>) -> String {
    let guard = bodies.lock().expect("bodies lock");
    let last = guard.last().expect("at least one captured request");
    last.split("\r\n\r\n").nth(1).unwrap_or("").to_string()
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

#[test]
fn list_empty_page_with_total_above_zero_stays_silent() {
    // WR-01: `--offset` paged past the end — the page is empty but notes
    // DO exist (total: 5), so the "No notes yet" hint would be factually
    // wrong. Exit 0, silent stderr; the empty table still renders.
    let paged_past_end = serde_json::json!({
        "data": [],
        "meta": {"total": 5, "offset": 10, "limit": 50}
    })
    .to_string();
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &paged_past_end)]);

    common::cmd_with_server(&url)
        .args(["notes", "list", "deals", "d1", "--offset", "10"])
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
        stdout.lines().all(|l| !l.trim().starts_with("get")),
        "notes --help must not advertise a get subcommand:\n{stdout}"
    );
}

// -- add (NOTE-02): body source matrix and the exact wire body --

#[test]
fn add_body_flag_posts_exactly_the_content_key() {
    let (url, counter, heads, bodies) = common::spawn_head_capturing_stub_server(&[(201,
        &note_envelope(note_json("n_new", "deal", "d1", "hello from flag")))]);

    common::cmd_with_server(&url)
        .args(["notes", "add", "deals", "d1", "--body", "hello from flag"])
        .assert()
        .success()
        .stdout(predicate::str::contains("n_new"));

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let heads = heads.lock().expect("heads lock");
    assert!(
        heads[0].contains("post /api/v1/deals/d1/notes"),
        "wire head: {}",
        heads[0]
    );
    // Pitfall 2: the wire field is `content` (not `body`) and it is the
    // ONLY key the CLI sends.
    let posted: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("POST body is JSON");
    assert_eq!(
        posted,
        serde_json::json!({"content": "hello from flag"}),
        "wire body must be exactly one content key: {posted}"
    );
}

#[test]
fn add_body_at_file_reads_the_file_text() {
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[(201,
        &note_envelope(note_json("n_new", "deal", "d1", "from file")))]);

    let file = tempfile::Builder::new().suffix(".md").tempfile().expect("tempfile");
    std::fs::write(file.path(), "from file").expect("write tempfile");

    common::cmd_with_server(&url)
        .args([
            "notes",
            "add",
            "deals",
            "d1",
            "--body",
            &format!("@{}", file.path().display()),
        ])
        .assert()
        .success();

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let posted: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("POST body is JSON");
    assert_eq!(posted["content"], "from file");
}

#[test]
fn add_unreadable_at_file_rejects_pre_http() {
    common::cmd()
        .args(["notes", "add", "deals", "d1", "--body", "@/nonexistent/nope.md"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("Check that the file path")
                .and(predicate::str::contains("Connection failed").not()),
        );
}

#[test]
fn add_stdin_reads_stdin_data() {
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[(201,
        &note_envelope(note_json("n_new", "deal", "d1", "from stdin")))]);

    common::cmd_with_server(&url)
        .args(["notes", "add", "deals", "d1", "--stdin"])
        .write_stdin("from stdin")
        .assert()
        .success();

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let posted: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("POST body is JSON");
    assert_eq!(posted["content"], "from stdin");
}

#[test]
fn add_body_at_dash_reads_stdin_data() {
    let (url, counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[(201,
        &note_envelope(note_json("n_new", "deal", "d1", "via at-dash")))]);

    common::cmd_with_server(&url)
        .args(["notes", "add", "deals", "d1", "--body", "@-"])
        .write_stdin("via at-dash")
        .assert()
        .success();

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let posted: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("POST body is JSON");
    assert_eq!(posted["content"], "via at-dash");
}

#[test]
fn add_body_and_stdin_is_two_sources_rejected_pre_http() {
    common::cmd()
        .args(["notes", "add", "deals", "d1", "--body", "x", "--stdin"])
        .write_stdin("ignored")
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("Multiple note body sources")
                .and(predicate::str::contains("exactly one"))
                .and(predicate::str::contains("Connection failed").not()),
        );
}

#[test]
fn add_at_dash_with_stdin_is_two_sources_rejected_pre_http() {
    // Open question 3: --body @- and --stdin are BOTH stdin-backed — two
    // explicit sources, rejected before any read (Pitfall 7).
    common::cmd()
        .args(["notes", "add", "deals", "d1", "--body", "@-", "--stdin"])
        .write_stdin("ignored")
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("Multiple note body sources")
                .and(predicate::str::contains("Connection failed").not()),
        );
}

#[test]
fn add_no_input_without_source_is_missing_input_pre_http() {
    common::cmd()
        .args(["notes", "add", "deals", "d1", "--no-input"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("--body")
                .and(predicate::str::contains("Connection failed").not()),
        );
}

#[test]
fn add_dry_run_previews_the_post_with_zero_http() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(201,
        &note_envelope(note_json("n", "deal", "d1", "x")))]);

    common::cmd_with_server(&url)
        .args(["--dry-run", "notes", "add", "deals", "d1", "--body", "preview me"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("POST")
                .and(predicate::str::contains("/api/v1/deals/d1/notes"))
                .and(predicate::str::contains("preview me")),
        );

    assert_eq!(counter.load(Ordering::SeqCst), 0, "--dry-run makes no requests");
}

// -- edit (NOTE-03): PATCH by note ID only --

#[test]
fn edit_patches_the_note_url_with_the_content_body() {
    let (url, counter, heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &note_envelope(note_json("n1", "deal", "d1", "new text"))),
        (200, &note_envelope(note_json("n1", "deal", "d1", "new text"))),
    ]);

    // Piped default (json): the updated note renders as data — the id is
    // present.
    common::cmd_with_server(&url)
        .args(["notes", "edit", "deals", "d1", "n1", "--body", "new text"])
        .assert()
        .success()
        .stdout(predicate::str::contains("n1"));

    // Table mode (human default on a TTY): the quiet-suppressible
    // confirmation line.
    common::cmd_with_server(&url)
        .args([
            "notes",
            "edit",
            "deals",
            "d1",
            "n1",
            "--body",
            "new text",
            "--format",
            "table",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated note n1"));

    assert_eq!(counter.load(Ordering::SeqCst), 2);
    let heads = heads.lock().expect("heads lock");
    assert!(
        heads[0].contains("patch /api/v1/notes/n1"),
        "wire head: {}",
        heads[0]
    );
    assert!(
        !heads[0].contains("/deals/"),
        "PATCH URL must carry ONLY the note id: {}",
        heads[0]
    );
    let posted: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("PATCH body is JSON");
    assert_eq!(
        posted,
        serde_json::json!({"content": "new text"}),
        "wire body must be exactly one content key: {posted}"
    );
}

#[test]
fn edit_403_renders_the_registered_notes_hint() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(403, &forbidden_403())]);

    common::cmd_with_server(&url)
        .args(["notes", "edit", "deals", "d1", "n1", "--body", "x"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("You can only modify your own notes"));
}

#[test]
fn edit_whitespace_body_passes_through_and_422_renders_server_message() {
    // No client-side trim (locked): whitespace-only bodies REACH the server,
    // and the 422 errors[] message renders via the Phase 8 layer.
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(422, &content_422())]);

    common::cmd_with_server(&url)
        .args(["notes", "edit", "deals", "d1", "n1", "--body", "   "])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Note content is required"));

    assert_eq!(counter.load(Ordering::SeqCst), 1, "body reached the server");
}

#[test]
fn edit_dry_run_previews_the_patch_with_zero_http() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(200,
        &note_envelope(note_json("n1", "deal", "d1", "x")))]);

    common::cmd_with_server(&url)
        .args(["--dry-run", "notes", "edit", "deals", "d1", "n1", "--body", "preview"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("PATCH")
                .and(predicate::str::contains("/api/v1/notes/n1")),
        );

    assert_eq!(counter.load(Ordering::SeqCst), 0, "--dry-run makes no requests");
}

// -- delete (NOTE-04): exact templates delete contract --

#[test]
fn delete_force_against_204_succeeds_with_one_request() {
    let (url, counter, heads, _bodies) = common::spawn_head_capturing_stub_server(&[(204, "")]);

    common::cmd_with_server(&url)
        .args(["notes", "delete", "deals", "d1", "n1", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted note n1"));

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let heads = heads.lock().expect("heads lock");
    assert!(
        heads[0].contains("delete /api/v1/notes/n1"),
        "wire head: {}",
        heads[0]
    );
}

#[test]
fn delete_non_tty_without_force_refuses_with_zero_http() {
    // Unreachable server + non-TTY stdin: the refusal must fire BEFORE any
    // HTTP (exit 1 via CliError::Validation).
    common::cmd()
        .args(["notes", "delete", "deals", "d1", "n1"])
        .assert()
        .code(1)
        .stderr(
            predicate::str::contains("--force")
                .and(predicate::str::contains("Connection failed").not()),
        );
}

#[test]
fn delete_dry_run_previews_with_zero_http() {
    let (url, counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[(204, "")]);

    common::cmd_with_server(&url)
        .args(["notes", "delete", "deals", "d1", "n1", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("/api/v1/notes/n1"));

    assert_eq!(counter.load(Ordering::SeqCst), 0, "--dry-run makes no requests");
}

#[test]
fn delete_then_redelete_404s_as_not_found() {
    // Pitfall 5: soft delete is idempotent only under a concurrent race —
    // a SEQUENTIAL re-delete hits the server 404 ("Note not found") and
    // renders as the normal NotFound failure path.
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, ""), (404, &note_404())]);

    common::cmd_with_server(&url)
        .args(["notes", "delete", "deals", "d1", "n1", "--force"])
        .assert()
        .success();

    common::cmd_with_server(&url)
        .args(["notes", "delete", "deals", "d1", "n1", "--force"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Note not found"));
}
