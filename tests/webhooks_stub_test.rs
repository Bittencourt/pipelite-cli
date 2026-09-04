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

/// 64 lowercase hex chars — the verified secret shape (32 random bytes hex).
const SECRET: &str = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2";

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
    assert_eq!(parsed["data"]["secret"], SECRET, "json: {stdout}");
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
