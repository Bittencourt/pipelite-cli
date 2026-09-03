//! DOCS-01 integration coverage: the `pipelite docs` command fetches the
//! server's OpenAPI 3.1 spec over the wire WITHOUT an Authorization header
//! (T-09-08), pretty-prints it, saves with parent-dir creation / overwrite
//! refusal (T-09-09) / --force / quiet handling, and re-wraps 404 +
//! legacy-500 errors with the locked docs hint while preserving the served
//! detail.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::sync::atomic::Ordering;

use common::{cmd_with_server, spawn_head_capturing_stub_server};

const SPEC_BODY: &str = r#"{"openapi":"3.1.0","info":{"title":"Pipelite API","version":"1.0"}}"#;
const NOT_FOUND_BODY: &str = r#"{"error":"Not Found","statusCode":404}"#;
const LEGACY_500_BODY: &str = r#"{"error":"OpenAPI specification not available"}"#;

/// 200 spec stub: exit 0; the captured request head contains NO line
/// starting with "authorization:"; stdout shows the PRETTY form — the stub
/// serves COMPACT JSON, so `"openapi": "3.1.0"` with the space proves
/// pretty-printing happened.
#[test]
fn docs_fetches_spec_without_authorization_header() {
    let (url, counter, heads, _bodies) = spawn_head_capturing_stub_server(&[(200, SPEC_BODY)]);

    cmd_with_server(&url)
        .args(["docs"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""openapi": "3.1.0""#));

    let heads = heads.lock().expect("heads lock");
    assert_eq!(heads.len(), 1, "exactly one request expected");
    let auth_lines: Vec<&str> = heads[0]
        .lines()
        .filter(|l| l.starts_with("authorization:"))
        .collect();
    assert!(
        auth_lines.is_empty(),
        "docs request must NOT carry an Authorization header — got head: {}",
        heads[0]
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "exactly one docs request"
    );
}

/// --save into a missing nested directory: parent dirs created, the file
/// parses as JSON, and the one-line confirmation reaches stdout.
#[test]
fn docs_save_creates_parent_dirs_and_confirms() {
    let (url, _counter, _heads, _bodies) = spawn_head_capturing_stub_server(&[(200, SPEC_BODY)]);
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let path = tmp.path().join("sub").join("spec.json");

    cmd_with_server(&url)
        .args(["docs", "--save", path.to_str().expect("utf-8 path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("Wrote OpenAPI spec to"));

    let content = std::fs::read_to_string(&path).expect("written spec");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert_eq!(parsed["openapi"], "3.1.0");
}

/// --save onto an existing file WITHOUT --force: exit 2, the --force hint
/// on stderr, sentinel content untouched, and ZERO requests — the refusal
/// fires BEFORE the fetch (the server IS reachable, making the 0-request
/// assertion stronger than an unreachable-server trick).
#[test]
fn docs_save_refuses_existing_file_without_force_pre_http() {
    let (url, counter, _heads, _bodies) = spawn_head_capturing_stub_server(&[(200, SPEC_BODY)]);
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let path = tmp.path().join("spec.json");
    std::fs::write(&path, "sentinel").expect("seed sentinel");

    cmd_with_server(&url)
        .args(["docs", "--save", path.to_str().expect("utf-8 path")])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--force"));

    let content = std::fs::read_to_string(&path).expect("read back");
    assert_eq!(content, "sentinel", "existing file must be untouched");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "refusal must fire BEFORE any HTTP request"
    );
}

/// --save WITH --force overwrites the existing file with the spec.
#[test]
fn docs_save_force_overwrites_existing_file() {
    let (url, _counter, _heads, _bodies) = spawn_head_capturing_stub_server(&[(200, SPEC_BODY)]);
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let path = tmp.path().join("spec.json");
    std::fs::write(&path, "sentinel").expect("seed sentinel");

    cmd_with_server(&url)
        .args([
            "docs",
            "--save",
            path.to_str().expect("utf-8 path"),
            "--force",
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&path).expect("read back");
    assert!(
        content.contains(r#""openapi": "3.1.0""#),
        "file must hold the pretty spec, got: {content}"
    );
}

/// --save under --quiet: the file IS written but the confirmation line is
/// suppressed.
#[test]
fn docs_save_quiet_suppresses_confirmation() {
    let (url, _counter, _heads, _bodies) = spawn_head_capturing_stub_server(&[(200, SPEC_BODY)]);
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let path = tmp.path().join("spec.json");

    cmd_with_server(&url)
        .args(["-q", "docs", "--save", path.to_str().expect("utf-8 path")])
        .assert()
        .success()
        .stdout(predicate::str::contains("Wrote").not());

    assert!(path.exists(), "file must still be written under --quiet");
}

/// 404 (older server without the route): exit 1; the LOCKED docs hint
/// replaces the generic 404 hint; the served detail ("Not Found") is
/// preserved.
#[test]
fn docs_404_renders_locked_hint_with_preserved_detail() {
    let (url, _counter, _heads, _bodies) =
        spawn_head_capturing_stub_server(&[(404, NOT_FOUND_BODY)]);

    cmd_with_server(&url)
        .args(["docs"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("may not expose the docs endpoint"))
        .stderr(predicate::str::contains("Not Found"))
        .stderr(predicate::str::contains("The requested resource was not found.").not());
}

/// Legacy 500 body {"error": "..."} (spec file missing on an older server):
/// exit 1 (Api arm); the detail renders UNQUOTED — parse_rfc7807 extracts
/// via as_str, never Value::to_string.
#[test]
fn docs_500_legacy_body_renders_detail_unquoted() {
    let (url, _counter, _heads, _bodies) =
        spawn_head_capturing_stub_server(&[(500, LEGACY_500_BODY)]);

    cmd_with_server(&url)
        .args(["docs"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("OpenAPI specification not available"))
        .stderr(
            predicate::str::contains(r#""OpenAPI specification not available""#).not(),
        );
}

/// --format is accepted but IGNORED: stdout still carries the pretty JSON
/// (the spec is JSON, not tabular output).
#[test]
fn docs_format_flag_is_ignored() {
    let (url, _counter, _heads, _bodies) = spawn_head_capturing_stub_server(&[(200, SPEC_BODY)]);

    cmd_with_server(&url)
        .args(["docs", "--format", "csv"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""openapi": "3.1.0""#));
}
