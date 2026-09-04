//! Stub-server integration tests for the webhooks surface (WHOK-01..03,
//! SC-1/SC-2, Phase 11).
//!
//! The secret tests are the FIRST tests in this file (task-order contract):
//! the 64-char signing secret must survive `webhooks create` in full, alone
//! on its own stdout line behind the save-it-now warning, in every format,
//! occurring exactly once — and never reach disk (HOME-redirected walk).
//!
//! Uses tests/common/mod.rs (shared Phase 9 helpers, no new crates).

mod common;

use predicates::prelude::*;
use std::sync::atomic::Ordering;

/// 64 lowercase hex chars — the verified secret shape (32 random bytes hex).
const SECRET: &str = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2";

// -- SC-2: the secret is shown exactly once, full, on its own line --
// (these two tests MUST stay at the top of the file)

/// T-11-01: create --format table under a piped (non-TTY) stdout renders the
/// table at a FIXED 120 columns (src/output/table.rs:49-51) — a 64-char
/// secret passed through a table cell would wrap/truncate. The secret must
/// bypass cells entirely: exactly ONE stdout line equals the full secret,
/// the line before it carries the save-it-now warning, the secret occurs
/// exactly once, and nothing under the redirected HOME persists it.
#[test]
fn secret_shown_once_full_on_own_line() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let body = create_body_json().to_string();
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &body)]);

    let output = common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args([
            "webhooks",
            "create",
            "--url",
            "https://example.com/hook",
            "--events",
            "deal.created",
            "--format",
            "table",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let lines: Vec<&str> = stdout.lines().collect();

    let secret_line_idxs: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == SECRET)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        secret_line_idxs.len(),
        1,
        "exactly one stdout line must equal the full 64-char secret (no wrap/truncation):\n{stdout}"
    );
    let idx = secret_line_idxs[0];
    assert!(idx > 0, "secret line must not be the first line:\n{stdout}");
    assert!(
        lines[idx - 1].contains("save it now"),
        "the line immediately before the secret must carry the save-it-now warning:\n{stdout}"
    );
    assert_eq!(
        stdout.matches(SECRET).count(),
        1,
        "the secret must occur exactly once in stdout:\n{stdout}"
    );
    assert!(
        !dir_contains_string(tmp.path(), SECRET),
        "the secret must never reach disk under the redirected HOME"
    );
}

/// --format json: the server body itself carries the secret (that IS the
/// show-once moment) — stdout parses as JSON with data.secret == fixture,
/// the secret occurs exactly once, and the save-it-now warning goes to
/// STDERR so `| jq` keeps working.
#[test]
fn secret_json_format() {
    let body = create_body_json().to_string();
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &body)]);

    let output = common::cmd_with_server(&url)
        .args([
            "webhooks",
            "create",
            "--url",
            "https://example.com/hook",
            "--events",
            "deal.created",
            "--format",
            "json",
        ])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("create --format json stdout parses as JSON");
    // render_single is envelope-free (templates/notes create convention) —
    // the FULL server data object (secret included) IS the rendered body.
    assert_eq!(parsed["secret"], SECRET, "json: {stdout}");
    assert_eq!(
        stdout.matches(SECRET).count(),
        1,
        "the secret must occur exactly once in stdout:\n{stdout}"
    );
    assert!(
        stderr.contains("save it now"),
        "the save-it-now warning must render on stderr in json mode:\n{stderr}"
    );
}

// -- helpers (kept BELOW the secret tests: the task-order contract pins the
//    first test fn in this file to the truncation proof) --

/// Recursively walk `dir`; return true if any readable file contains `needle`.
fn dir_contains_string(dir: &std::path::Path, needle: &str) -> bool {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    stack.push(entry.path());
                }
            }
        } else if let Ok(content) = std::fs::read_to_string(&path) {
            if content.contains(needle) {
                return true;
            }
        }
    }
    false
}

/// Verified wire shape for POST /api/v1/webhooks (201): Webhook + secret —
/// the ONLY server response containing the secret.
fn create_body_json() -> serde_json::Value {
    serde_json::json!({
        "data": {
            "id": "wh1",
            "url": "https://example.com/hook",
            "events": ["deal.created"],
            "active": true,
            "created_at": "2026-09-01T10:00:00.000Z",
            "updated_at": "2026-09-01T10:00:00.000Z",
            "secret": SECRET
        }
    })
}

/// Verified Webhook wire shape (list/get/PUT serializers — NO secret, NO
/// description).
fn webhook_json(id: &str, url: &str, events: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "url": url,
        "events": events,
        "active": true,
        "created_at": "2026-09-01T10:00:00.000Z",
        "updated_at": "2026-09-01T10:00:00.000Z"
    })
}

/// Verified server 403 shape (ownership gate — even admins get 403).
fn forbidden_403() -> String {
    serde_json::json!({
        "type": "https://api.pipelite.app/errors/FORBIDDEN",
        "title": "Forbidden",
        "status": 403,
        "detail": "You don't have access to this resource"
    })
    .to_string()
}

/// Verified list envelope.
fn webhooks_list_body(webhooks: Vec<serde_json::Value>) -> String {
    let total = webhooks.len();
    serde_json::json!({
        "data": webhooks,
        "meta": {"total": total, "offset": 0, "limit": 50}
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

// -- create (WHOK-01, SC-1) --

#[test]
fn create_table_basic() {
    let body = create_body_json().to_string();
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &body)]);

    common::cmd_with_server(&url)
        .args([
            "webhooks",
            "create",
            "--url",
            "https://example.com/hook",
            "--events",
            "deal.created",
            "--format",
            "table",
        ])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("wh1"))
        .stdout(predicate::str::contains("https://example.com/hook"));

    assert!(
        captured_head(&heads, 0).contains("post /api/v1/webhooks"),
        "create must POST /api/v1/webhooks"
    );
}

/// --stdin passes the body through VERBATIM: the captured POST body parses
/// to exactly the stdin JSON (no flag merging, no key injection).
#[test]
fn create_stdin_verbatim() {
    let body = create_body_json().to_string();
    let (url, _counter, heads, bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &body)]);

    let stdin_json = r#"{"url":"https://example.com/hook","events":["deal.created"]}"#;
    common::cmd_with_server(&url)
        .args(["webhooks", "create", "--stdin"])
        .write_stdin(stdin_json)
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("post /api/v1/webhooks"),
        "stdin create must POST /api/v1/webhooks: {head}"
    );
    let captured: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("captured body parses");
    assert_eq!(
        captured,
        serde_json::json!({"url": "https://example.com/hook", "events": ["deal.created"]}),
        "captured body must equal the stdin JSON verbatim"
    );
}

/// Unknown events inside --stdin JSON are rejected pre-HTTP (exit 2,
/// counter == 0) — full control does not include dead webhooks.
#[test]
fn create_stdin_bad_event() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &create_body_json().to_string())]);

    common::cmd_with_server(&url)
        .args(["webhooks", "create", "--stdin"])
        .write_stdin(r#"{"url":"https://x/e","events":["deal.archived"]}"#)
        .assert()
        .code(2);

    assert_eq!(counter.load(Ordering::SeqCst), 0, "zero HTTP on bad stdin");
}

/// Non-string entries inside --stdin events arrays are rejected pre-HTTP
/// (exit 2, counter == 0) — WR-01: they previously slipped past the
/// allow-list via filter_map and reached the wire as-is.
#[test]
fn create_stdin_non_string_event() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &create_body_json().to_string())]);

    common::cmd_with_server(&url)
        .args(["webhooks", "create", "--stdin"])
        .write_stdin(r#"{"url":"https://x/e","events":[123]}"#)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("must be strings"))
        .stderr(predicate::str::contains("deal.stage_changed"));

    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "zero HTTP on non-string event entry"
    );
}

#[test]
fn create_missing_url() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &create_body_json().to_string())]);

    common::cmd_with_server(&url)
        .args(["webhooks", "create", "--events", "deal.created"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--url"));

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[test]
fn create_missing_events() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &create_body_json().to_string())]);

    common::cmd_with_server(&url)
        .args(["webhooks", "create", "--url", "https://example.com/hook"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--events"));

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

/// SC-1: https mirrored client-side — a plain-http URL exits 2 BEFORE any
/// request (the server would 422; the CLI refuses earlier with zero HTTP).
#[test]
fn create_http_url() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &create_body_json().to_string())]);

    let output = common::cmd_with_server(&url)
        .args([
            "webhooks",
            "create",
            "--url",
            "http://example.com/hook",
            "--events",
            "deal.created",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("https"))
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Connection failed"),
        "https rejection must be pre-HTTP:\n{stderr}"
    );

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

/// WHOK-02: unknown event names exit 2 pre-HTTP with ALL 13 valid events
/// listed (the server would silently accept them and never fire).
#[test]
fn create_unknown_event() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &create_body_json().to_string())]);

    common::cmd_with_server(&url)
        .args([
            "webhooks",
            "create",
            "--url",
            "https://example.com/hook",
            "--events",
            "deal.created,deal.archived",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("deal.archived"))
        .stderr(predicate::str::contains("deal.stage_changed"))
        .stderr(predicate::str::contains("activity.deleted"));

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

// -- list (WHOK-01) --

#[test]
fn list_table() {
    let body = webhooks_list_body(vec![webhook_json("wh1", "https://example.com/hook", &[
        "deal.created",
    ])]);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["webhooks", "list", "--format", "table"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("wh1"))
        .stdout(predicate::str::contains("https://example.com/hook"))
        .stdout(predicate::str::contains("(shown once at creation)"));

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("get /api/v1/webhooks?limit=50&offset=0"),
        "list must GET /api/v1/webhooks with default pagination: {head}"
    );
}

/// List/get NEVER carry the secret: the json `secret` key holds the
/// self-documenting placeholder and no 64-hex string appears anywhere.
#[test]
fn list_json_placeholder() {
    let body = webhooks_list_body(vec![webhook_json("wh1", "https://example.com/hook", &[
        "deal.created",
    ])]);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    let output = common::cmd_with_server(&url)
        .args(["webhooks", "list", "--format", "json"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json list parses");
    assert_eq!(parsed[0]["secret"], "(shown once at creation)");
    assert!(
        !stdout.contains("a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6"),
        "no 64-hex secret fragment anywhere in list stdout:\n{stdout}"
    );
}

#[test]
fn list_pagination() {
    let body = webhooks_list_body(vec![]);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["webhooks", "list", "--limit", "5", "--offset", "10", "--format", "json"])
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("limit=5&offset=10"),
        "pagination must reach the wire: {head}"
    );
}

// -- get (WHOK-01) --

#[test]
fn get_placeholder() {
    let body = serde_json::json!({
        "data": webhook_json("wh1", "https://example.com/hook", &["deal.created"])
    })
    .to_string();
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["webhooks", "get", "wh1", "--format", "json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("(shown once at creation)"));

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("get /api/v1/webhooks/wh1"),
        "get must GET /api/v1/webhooks/{{id}}: {head}"
    );
}

/// Ownership 403 renders the registered ownership hint — webhooks have NO
/// admin bypass (exit 1, Forbidden).
#[test]
fn get_foreign_403() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(403, &forbidden_403())]);

    common::cmd_with_server(&url)
        .args(["webhooks", "get", "wh_foreign"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "This webhook belongs to another user.",
        ));
}

// -- empty-list hint --

#[test]
fn list_empty_hint() {
    let body = webhooks_list_body(vec![]);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body), (200, &body)]);

    common::cmd_with_server(&url)
        .args(["webhooks", "list", "--format", "json"])
        .assert()
        .code(0)
        .stderr(predicate::str::contains("No webhooks yet"));

    // Same run under --quiet suppresses the hint (but stays exit 0).
    common::cmd_with_server(&url)
        .args(["webhooks", "list", "--quiet", "--format", "json"])
        .assert()
        .code(0)
        .stderr(predicate::str::contains("No webhooks yet").not());
}

// -- update: get→merge→PUT (WHOK-01, locked D-decision) --

fn get_envelope(url_str: &str, events: &[&str], active: bool) -> String {
    let mut wh = webhook_json("wh1", url_str, events);
    wh["active"] = serde_json::json!(active);
    serde_json::json!({ "data": wh }).to_string()
}

#[test]
fn update_merges() {
    let (url, _counter, heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &get_envelope("https://old.example.com/hook", &["deal.created"], true)),
        (200, &get_envelope("https://new.example.com/hook", &["deal.created"], true)),
    ]);

    common::cmd_with_server(&url)
        .args(["webhooks", "update", "wh1", "--url", "https://new.example.com/hook"])
        .assert()
        .code(0);

    let head0 = captured_head(&heads, 0);
    let head1 = captured_head(&heads, 1);
    assert!(
        head0.contains("get /api/v1/webhooks/wh1"),
        "update must GET first: {head0}"
    );
    assert!(
        head1.contains("put /api/v1/webhooks/wh1"),
        "then PUT: {head1}"
    );

    let put_body: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("PUT body parses");
    assert_eq!(
        put_body,
        serde_json::json!({
            "url": "https://new.example.com/hook",
            "events": ["deal.created"],
            "active": true
        }),
        "fetched events + active must be preserved (merge, not replace)"
    );
}

#[test]
fn update_events_flag() {
    let (url, _counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &get_envelope("https://old.example.com/hook", &["deal.created"], true)),
        (200, &get_envelope("https://old.example.com/hook", &["deal.updated", "deal.created"], true)),
    ]);

    common::cmd_with_server(&url)
        .args([
            "webhooks",
            "update",
            "wh1",
            "--events",
            "deal.updated,deal.created",
        ])
        .assert()
        .code(0);

    let put_body: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("PUT body parses");
    assert_eq!(put_body["events"], serde_json::json!(["deal.updated", "deal.created"]), "order as given");
    assert_eq!(
        put_body["url"],
        "https://old.example.com/hook",
        "url must be merged from the fetch"
    );
}

#[test]
fn update_inactive() {
    let (url, _counter, _heads, bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &get_envelope("https://example.com/hook", &["deal.created"], true)),
        (200, &get_envelope("https://example.com/hook", &["deal.created"], false)),
    ]);

    common::cmd_with_server(&url)
        .args(["webhooks", "update", "wh1", "--inactive"])
        .assert()
        .code(0);

    let put_body: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("PUT body parses");
    assert_eq!(put_body["active"], serde_json::json!(false));
}

#[test]
fn update_no_flags() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &get_envelope("https://example.com/hook", &["deal.created"], true))]);

    common::cmd_with_server(&url)
        .args(["webhooks", "update", "wh1"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("nothing to update"));

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

/// --stdin is a verbatim PUT with NO GET first (bypasses the merge).
#[test]
fn update_stdin_no_get() {
    let (url, counter, heads, bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &get_envelope("https://new.example.com/hook", &["deal.created"], false))]);

    common::cmd_with_server(&url)
        .args(["webhooks", "update", "wh1", "--stdin"])
        .write_stdin(r#"{"url":"https://new.example.com/hook","active":false}"#)
        .assert()
        .code(0);

    assert_eq!(counter.load(Ordering::SeqCst), 1, "exactly ONE request (no GET)");
    let head = captured_head(&heads, 0);
    assert!(head.contains("put /api/v1/webhooks/wh1"), "single request must be the PUT: {head}");
    let captured: serde_json::Value =
        serde_json::from_str(&last_captured_body(&bodies)).expect("PUT body parses");
    assert_eq!(
        captured,
        serde_json::json!({"url": "https://new.example.com/hook", "active": false}),
        "stdin body must be PUT verbatim"
    );
}

/// --dry-run under --stdin ALSO previews with ZERO HTTP (CR-01: the stdin
/// path previously returned execute() before the dry-run intercept and
/// fired the real PUT). Mirrors update_dry_run_zero_http for stdin mode.
#[test]
fn update_stdin_dry_run_zero_http() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &get_envelope("https://example.com/hook", &["deal.created"], true))]);

    let output = common::cmd_with_server(&url)
        .args(["webhooks", "update", "wh1", "--stdin", "--dry-run"])
        .write_stdin(r#"{"url":"https://example.com/hook","active":false}"#)
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert_eq!(counter.load(Ordering::SeqCst), 0, "stdin dry-run must be zero HTTP");
    assert!(
        !stderr.contains("Connection failed"),
        "no connection may be attempted under --dry-run:\n{stderr}"
    );
    assert!(stdout.contains("PUT"), "preview must name the method:\n{stdout}");
    assert!(
        stdout.contains("/api/v1/webhooks/wh1"),
        "preview must target the webhook:\n{stdout}"
    );
    assert!(
        stdout.contains("\"active\": false"),
        "preview must carry the verbatim stdin body:\n{stdout}"
    );
}

/// --dry-run previews with ZERO HTTP (CLAUDE.md global contract).
#[test]
fn update_dry_run_zero_http() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &get_envelope("https://example.com/hook", &["deal.created"], true))]);

    let output = common::cmd_with_server(&url)
        .args([
            "webhooks",
            "update",
            "wh1",
            "--url",
            "https://example.com/hook",
            "--dry-run",
        ])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert_eq!(counter.load(Ordering::SeqCst), 0, "dry-run must be zero HTTP");
    assert!(stdout.contains("PUT"), "preview must name the method:\n{stdout}");
    assert!(
        stdout.contains("https://example.com/hook"),
        "preview must carry the webhook URL:\n{stdout}"
    );
}

/// Bad flags are rejected BEFORE the fetch (zero HTTP).
#[test]
fn update_flag_validation_precedes_get() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &get_envelope("https://example.com/hook", &["deal.created"], true))]);

    common::cmd_with_server(&url)
        .args(["webhooks", "update", "wh1", "--events", "deal.archived"])
        .assert()
        .code(2);

    assert_eq!(counter.load(Ordering::SeqCst), 0, "no GET may be issued");
}

#[test]
fn update_foreign_403() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(403, &forbidden_403())]);

    common::cmd_with_server(&url)
        .args(["webhooks", "update", "wh_foreign", "--url", "https://example.com/hook"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "This webhook belongs to another user.",
        ));
}

// -- delete: standard confirm contract (dry-run first, TTY confirm, --force,
//    non-TTY refusal exit 1 Validation) --

#[test]
fn delete_force() {
    let (url, counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    common::cmd_with_server(&url)
        .args(["webhooks", "delete", "wh1", "--force"])
        .assert()
        .code(0);

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let head = captured_head(&heads, 0);
    assert!(
        head.contains("delete /api/v1/webhooks/wh1"),
        "delete must DELETE /api/v1/webhooks/{{id}}: {head}"
    );
}

#[test]
fn delete_non_tty_refusal() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    common::cmd_with_server(&url)
        .args(["webhooks", "delete", "wh1"])
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
        .args(["webhooks", "delete", "wh1", "--dry-run", "--format", "json"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert_eq!(counter.load(Ordering::SeqCst), 0, "dry-run must be zero HTTP");
    assert!(
        stdout.contains("/api/v1/webhooks/wh1"),
        "preview must carry the webhook URL:\n{stdout}"
    );
}

/// list writes the (id, url) cache file; delete --force removes it.
#[test]
fn delete_invalidates_cache() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir");
    let list_body = webhooks_list_body(vec![webhook_json(
        "wh1",
        "https://example.com/hook",
        &["deal.created"],
    )]);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &list_body), (204, "")]);

    let cache_file = tmp.path().join(".pipelite/cache/webhooks.json");
    assert!(!cache_file.exists(), "cache file must not pre-exist");

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args(["webhooks", "list", "--format", "json"])
        .assert()
        .code(0);
    assert!(
        cache_file.exists(),
        "list must write the webhooks cache file"
    );

    common::cmd_with_server(&url)
        .env("HOME", tmp.path())
        .args(["webhooks", "delete", "wh1", "--force"])
        .assert()
        .code(0);
    assert!(
        !cache_file.exists(),
        "delete must invalidate (remove) the webhooks cache file"
    );
}
