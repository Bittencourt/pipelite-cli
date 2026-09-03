//! Stub-server integration tests for the RFC 7807 error layer (FIX-05).
//!
//! Proves through the real CLI binary that:
//! - 403 renders Forbidden + a permission hint (never the bad-key hint)
//! - the ping path (third mapping site, `check_auth_status`) splits too
//! - 401 still renders Authentication failed + the init hint (regression)
//! - 409 on an inactive workflow trigger carries the activation hint with
//!   the server's verbatim detail (no JSON quotes)
//! - 422 renders the joined errors[] text, never the generic detail
//! - 404 renders the server's detail instead of "HTTP 404"
//!
//! Uses the hand-rolled TcpListener stub pattern (no new crates).

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Helper: pipelite command pointed at a live stub server URL.
/// Sets PIPELITE_URL + PIPELITE_SERVER_URL + fake key — hermetic in the
/// no-config suite default; the env override wins over any real config
/// on dev machines.
fn cmd_with_server(url: &str) -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", url);
    c.env("PIPELITE_SERVER_URL", url);
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

// -- Verified RFC 7807 fixtures (RESEARCH § API Contract Details) --

const BODY_403: &str = r#"{"type":"https://api.pipelite.app/errors/FORBIDDEN","title":"Forbidden","status":403,"detail":"You don't have access to this resource"}"#;

const BODY_401: &str = r#"{"type":"https://api.pipelite.app/errors/UNAUTHORIZED","title":"Unauthorized","status":401,"detail":"Authentication required"}"#;

const BODY_409_INACTIVE: &str = r#"{"type":"https://api.pipelite.app/errors/CONFLICT","title":"Conflict","status":409,"detail":"Workflow is not active. Activate the workflow before triggering a run."}"#;

const BODY_422_STAGE: &str = r#"{"type":"https://api.pipelite.app/errors/VALIDATION_ERROR","title":"Validation Error","status":422,"detail":"Request validation failed","errors":[{"field":"stage_id","code":"invalid","message":"Stage does not exist"}]}"#;

const BODY_422_EMPTY_ERRORS: &str = r#"{"type":"https://api.pipelite.app/errors/VALIDATION_ERROR","title":"Validation Error","status":422,"detail":"Request validation failed","errors":[]}"#;

const BODY_404_DEAL: &str = r#"{"type":"https://api.pipelite.app/errors/ENTITY_NOT_FOUND","title":"Not Found","status":404,"detail":"deal not found"}"#;

/// Hand-rolled HTTP stub server (no new crates): serves one scripted
/// `(status, raw body)` response per accepted connection, in order.
///
/// Generalization of the batch_error_test.rs helper: each script entry is a
/// full response body served verbatim as application/json. Keeps the
/// head-reading + Content-Length loop and the request counter exactly as
/// the existing helper.
fn spawn_body_stub_server(script: &[(u16, &str)]) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub server");
    let addr = listener.local_addr().expect("stub server address");
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let script: Vec<(u16, String)> = script
        .iter()
        .map(|(status, body)| (*status, body.to_string()))
        .collect();

    std::thread::spawn(move || {
        for (status, body) in script {
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

            // Read exactly Content-Length body bytes, if any.
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

            let reason = match status {
                401 => "Unauthorized",
                403 => "Forbidden",
                404 => "Not Found",
                409 => "Conflict",
                422 => "Unprocessable Entity",
                _ => "OK",
            };
            let mut response = format!("HTTP/1.1 {status} {reason}\r\n");
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

// -- 403 on the entity path: Forbidden + permission hint, never bad-key --

#[test]
fn forbidden_on_deal_get_shows_permission_hint() {
    let (url, _counter) = spawn_body_stub_server(&[(403, BODY_403)]);
    cmd_with_server(&url)
        .args(["deals", "get", "deal_1"])
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("Forbidden")
                .and(predicate::str::contains("permission"))
                // Server's voice preserved (detail threaded through).
                .and(predicate::str::contains("You don't have access to this resource"))
                // The two permission tiers must be distinguishable (ROADMAP SC-1).
                .and(predicate::str::contains("Check your API key").not()),
        );
}

// -- 403 on the ping path: third mapping site (check_auth_status) split --

#[test]
fn forbidden_on_ping_shows_permission_hint() {
    let (url, _counter) = spawn_body_stub_server(&[(403, BODY_403)]);
    cmd_with_server(&url)
        .args(["ping"])
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("Forbidden")
                .and(predicate::str::contains("permission"))
                .and(predicate::str::contains("Check your API key").not()),
        );
}

// -- 401 regression: still Auth with the init hint (tiers distinguishable) --

#[test]
fn unauthorized_on_deal_get_stays_auth_with_init_hint() {
    let (url, _counter) = spawn_body_stub_server(&[(401, BODY_401)]);
    cmd_with_server(&url)
        .args(["deals", "get", "deal_1"])
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("Authentication failed")
                .and(predicate::str::contains("Check your API key")),
        );
}

// -- 409 on inactive workflow trigger: activation hint + quote-free detail --

#[test]
fn conflict_on_workflow_trigger_shows_activation_hint() {
    let (url, _counter) = spawn_body_stub_server(&[(409, BODY_409_INACTIVE)]);
    cmd_with_server(&url)
        .args(["workflows", "trigger", "wf_1"])
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("Workflow is not active")
                // The hint names the activation command (ROADMAP SC-2).
                .and(predicate::str::contains("pipelite workflows update"))
                .and(predicate::str::contains("--active true"))
                // Detail rendered without JSON quotes (Pitfall 2).
                .and(predicate::str::contains("\"Workflow is not active").not()),
        );
}

// -- 422 renders the joined errors[] text, never the generic detail --

#[test]
fn validation_error_renders_errors_array_not_generic_detail() {
    let (url, _counter) = spawn_body_stub_server(&[(422, BODY_422_STAGE)]);
    // --stage is client-side required (cli/deals.rs): without it the CLI
    // exits 2 pre-HTTP and the stub is never reached. --no-input keeps the
    // run TTY-hermetic (optional-field prompts would hang a terminal run).
    cmd_with_server(&url)
        .args(["deals", "create", "--title", "X", "--stage", "stg_1", "--no-input"])
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("stage_id: Stage does not exist (invalid)")
                .and(predicate::str::contains("Request validation failed").not()),
        );
}

// -- 422 with empty errors[] falls back to the detail string --

#[test]
fn validation_error_empty_errors_array_falls_back_to_detail() {
    let (url, _counter) = spawn_body_stub_server(&[(422, BODY_422_EMPTY_ERRORS)]);
    cmd_with_server(&url)
        .args(["deals", "create", "--title", "X", "--stage", "stg_1", "--no-input"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Request validation failed"));
}

// -- 404 renders the server's detail instead of "HTTP 404" --

#[test]
fn not_found_renders_server_detail_not_http_status() {
    let (url, _counter) = spawn_body_stub_server(&[(404, BODY_404_DEAL)]);
    cmd_with_server(&url)
        .args(["deals", "get", "deal_1"])
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("deal not found")
                .and(predicate::str::contains("HTTP 404").not()),
        );
}
