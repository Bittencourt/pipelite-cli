use assert_cmd::Command;
use predicates::prelude::*;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Helper: build a pipelite command with fake config env vars.
/// Per CLAUDE.md: tests use 127.0.0.1:1 + fake-test-key.
///
/// Note: `PIPELITE_SERVER_URL` is the env var the app actually reads
/// (see src/config.rs env override); `PIPELITE_URL` is kept for
/// consistency with the rest of the test suite's cmd() helpers.
/// Setting it matters here: these tests make real connection attempts
/// against the unreachable 127.0.0.1:1 endpoint to exercise error paths
/// — without the override the default production URL would be used.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_SERVER_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

/// Helper: pipelite command pointed at a live stub server URL.
fn cmd_with_server(url: &str) -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", url);
    c.env("PIPELITE_SERVER_URL", url);
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

/// A complete ApiSingleResponse<Deal> body (all Deal fields required).
const DEAL_SUCCESS_BODY: &str = r#"{"data":{"id":"deal_1","title":"Updated","value":null,"stage_id":"s1","organization_id":null,"person_id":null,"owner_id":"u1","position":null,"expected_close_date":null,"notes":null,"custom_fields":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}}"#;

/// Hand-rolled HTTP stub server (no new crates): serves one scripted
/// `(status, Retry-After)` response per accepted connection, in order.
///
/// Returns the base URL and a shared counter of served responses so tests
/// can assert exactly how many requests the CLI made (one retry = 2 total).
fn spawn_stub_server(script: &[(u16, Option<&str>)]) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub server");
    let addr = listener.local_addr().expect("stub server address");
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let script: Vec<(u16, Option<String>)> = script
        .iter()
        .map(|(status, retry_after)| (*status, retry_after.map(|s| s.to_string())))
        .collect();

    std::thread::spawn(move || {
        for (status, retry_after) in script {
            let (mut stream, _) = match listener.accept() {
                Ok(conn) => conn,
                Err(_) => break,
            };

            // Read the request head (headers end at the first \r\n\r\n).
            let mut received = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        received.extend_from_slice(&buf[..n]);
                        if received.windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }
                }
            }

            // Read exactly Content-Length body bytes, if any (the PUT body).
            let header_end = received
                .windows(4)
                .position(|w| w == b"\r\n\r\n")
                .map(|p| p + 4)
                .unwrap_or(received.len());
            let headers = String::from_utf8_lossy(&received[..header_end]).to_lowercase();
            let content_length = headers
                .lines()
                .find_map(|l| l.strip_prefix("content-length:"))
                .and_then(|v| v.trim().parse::<usize>().ok())
                .unwrap_or(0);
            let mut body_read = received.len().saturating_sub(header_end);
            while body_read < content_length {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => body_read += n,
                }
            }

            let reason = if status == 429 {
                "Too Many Requests"
            } else {
                "OK"
            };
            let body = if status == 429 {
                r#"{"error":"rate limited"}"#
            } else {
                DEAL_SUCCESS_BODY
            };
            let mut response = format!("HTTP/1.1 {status} {reason}\r\n");
            if let Some(ra) = &retry_after {
                response.push_str(&format!("Retry-After: {ra}\r\n"));
            }
            response.push_str(&format!("Content-Length: {}\r\n", body.len()));
            response.push_str("Content-Type: application/json\r\n");
            response.push_str("Connection: close\r\n\r\n");
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.write_all(body.as_bytes());
            let _ = stream.flush();
            drop(stream);
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    (format!("http://{addr}"), counter)
}

// -- Batch update against unreachable server -> all fail, non-zero exit (D-06, D-07) --

#[test]
fn batch_update_unreachable_server_exits_nonzero() {
    // All items are structurally valid; failures are per-item (unreachable
    // server), so the batch exits 1 — never 2 (exit-2 is structural only).
    let input = r#"[{"id":"deal_1","title":"New"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("failed").or(predicate::str::contains("Failed")));
}

// -- Batch update silent no-ops must fail (WR-03) --

#[test]
fn batch_update_empty_array_fails() {
    // An empty array runs zero operations and must not exit 0 silently.
    // Structural input failure -> exit 2 (BATCH-04 exit-code matrix).
    cmd()
        .write_stdin("[]")
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Empty update list"));
}

#[test]
fn batch_update_invalid_json_exits_2() {
    // Malformed JSON is a structural input failure -> exit 2.
    cmd()
        .write_stdin("not json")
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Invalid JSON input"));
}

#[test]
fn batch_delete_empty_list_exits_2() {
    // Empty ID array is a structural input failure -> exit 2.
    cmd()
        .write_stdin("[]")
        .args(["deals", "delete", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Empty ID list"));
}

#[test]
fn batch_delete_stdin_with_positional_ids_exits_2() {
    // --stdin + positional IDs conflict is a structural input failure -> exit 2.
    cmd()
        .write_stdin(r#"["deal_1"]"#)
        .args(["deals", "delete", "deal_1", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("mutually exclusive"));
}

#[test]
fn batch_update_stdin_with_field_flag_exits_2() {
    // --stdin + individual field flags conflict is structural -> exit 2
    // (deals stand in for all 7 entities, which share the identical path).
    cmd()
        .write_stdin(r#"[{"id":"deal_1","title":"X"}]"#)
        .args(["deals", "update", "--stdin", "--title", "X"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("mutually exclusive"));
}

#[test]
fn batch_update_noop_item_fails() {
    // "titel" is not a known field; the Update model ignores it, so the item
    // would PUT an empty {} body and report success. WR-03: report failure.
    let input = r#"[{"id":"deal_1","titel":"New"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no recognizable update fields"));
}

// -- Batch delete against unreachable server -> all fail, non-zero exit (D-06, D-07) --

#[test]
fn batch_delete_unreachable_server_exits_nonzero() {
    // write_stdin("") creates a piped (non-TTY) stdin, which makes this a
    // non-interactive run: CR-01 requires --force for batch deletes when the
    // prompt cannot be shown. The empty content is not read because positional
    // IDs are provided (--stdin is not set). Both deletes fail against the
    // unreachable server.
    cmd()
        .write_stdin("")
        .args(["deals", "delete", "deal_1", "deal_2", "--force"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("failed").or(predicate::str::contains("Failed")));
}

// -- Unit test for BatchOutcome (via module test, not integration) --
// BatchOutcome finalize with failures returns error
// This is tested indirectly via the above integration tests.
// The error summary format is: "N/M entity operationd, K failed"

#[test]
fn batch_update_missing_id_in_item_shows_error() {
    // An item without a string "id" is a STRUCTURAL input failure: the whole
    // batch must be rejected with exit 2 before any HTTP call fires.
    let input = r#"[{"title":"No ID Here"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("missing a string 'id' field")
                .and(predicate::str::contains(": 1")),
        );
}

#[test]
fn batch_update_mixed_batch_missing_id_zero_http() {
    // THE core regression for verification gap 1: a valid item followed by an
    // id-less item must NOT mutate the valid item. The unreachable
    // 127.0.0.1:1 server guarantees any HTTP attempt prints "Connection
    // failed", so its absence from stderr proves zero requests fired.
    let input = r#"[{"id":"deal_1","title":"X"},{"title":"No ID Here"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("missing a string 'id' field")
                .and(predicate::str::contains(": 2"))
                .and(predicate::str::contains("Connection failed").not())
                .and(predicate::str::contains("[1/2] Failed").not()),
        );
}

#[test]
fn batch_update_non_string_id_exits_2() {
    // Non-string ids are treated as missing (matches the in-loop
    // as_str() extraction semantics) -> structural exit 2.
    let input = r#"[{"id":123,"title":"X"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("missing a string 'id' field"));
}

#[test]
fn batch_update_dry_run_missing_id_exits_2() {
    // The pre-scan runs BEFORE the dry-run block: --dry-run is how users
    // validate input, so structurally broken input is rejected there too —
    // no per-item dry-run output on stdout.
    let input = r#"[{"id":"deal_1","title":"X"},{"title":"No ID Here"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--dry-run", "--stdin"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("missing a string 'id' field"));
}

// -- 429 Retry-After retry-once (verification gap 2) --
// The retry lives in the shared API client, so the deals entity alone
// proves the behavior for all 7 entities.

#[test]
fn batch_update_429_retries_once_then_succeeds() {
    // Request 1 -> 429 (Retry-After: 0); request 2 (the one retry) -> 200.
    // The item must SUCCEED (exit 0, no failure line) after exactly one retry.
    let (url, count) = spawn_stub_server(&[(429, Some("0")), (200, None)]);
    let input = r#"[{"id":"deal_1","title":"New"}]"#;
    let output = cmd_with_server(&url)
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .output()
        .expect("run pipelite against stub server");
    assert_eq!(
        output.status.code(),
        Some(0),
        "retry after 429 should make the item succeed; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Failed"),
        "no failure line expected after successful retry: {stderr}"
    );
    assert_eq!(
        count.load(Ordering::SeqCst),
        2,
        "expected exactly one retry (2 requests total)"
    );
}

#[test]
fn batch_update_still_429_after_retry_fails_item_with_detail() {
    // 429 twice: exactly one retry (never more), then the item is reported
    // as a failed item carrying rate-limit detail, and the batch exits 1.
    let (url, count) = spawn_stub_server(&[(429, Some("0")), (429, Some("0"))]);
    let input = r#"[{"id":"deal_1","title":"New"}]"#;
    let output = cmd_with_server(&url)
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .output()
        .expect("run pipelite against stub server");
    assert_eq!(
        output.status.code(),
        Some(1),
        "still-429 after retry must fail the batch"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("429") || stderr.contains("Rate limited"),
        "expected rate-limit detail on the failure line: {stderr}"
    );
    assert_eq!(
        count.load(Ordering::SeqCst),
        2,
        "exactly one retry, not two (no retry storm)"
    );
}
