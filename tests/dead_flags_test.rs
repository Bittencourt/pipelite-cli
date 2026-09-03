//! Dead-flag removal tests (FIX-01, T-08-04).
//!
//! Proves through the real CLI binary that the 5 removed flags:
//! - reject with `CliError::InvalidInput` (exit 2) BEFORE any HTTP call
//!   (asserted against an unreachable server: zero "Connection failed")
//! - carry a replacement hint naming what to use instead
//! - are hidden from `--help` (parse-then-error, not clap rejection)
//! - and that `workflows create` neither prompts nor sends `active` on any
//!   path (flag path and stdin path, against a live stub server).
//!
//! Uses the hand-rolled TcpListener stub pattern (no new crates).

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Helper: pipelite command pointed at an unreachable server. Any HTTP call
/// would fail with "Connection failed" — tests assert that text is absent.
fn cmd_unreachable() -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_SERVER_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

/// Helper: pipelite command pointed at a live stub server URL (hermetic).
fn cmd_with_server(url: &str) -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", url);
    c.env("PIPELITE_SERVER_URL", url);
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

/// Body-carrying stub server that captures every full request (head + body),
/// so tests can assert exact serialized request payloads.
fn spawn_capturing_stub_server(
    script: &[(u16, String)],
) -> (String, Arc<AtomicUsize>, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub server");
    let addr = listener.local_addr().expect("stub server address");
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let requests: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let requests_clone = Arc::clone(&requests);
    let script: Vec<(u16, String)> = script.to_vec();

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
                    Ok(n) => {
                        body_read += n;
                        received.extend_from_slice(&buf[..n]);
                    }
                }
            }

            requests_clone
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&received).to_string());

            let reason = if status == 200 { "OK" } else { "Error" };
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

    (format!("http://{addr}"), counter, requests)
}

/// The server's response for a successful workflow create.
fn workflow_created_body() -> String {
    r#"{"data":{"id":"wf_new","name":"W","description":null,"triggers":[],"nodes":[],"active":false,"created_by":"user_001","created_at":"2026-01-15T10:30:00Z","updated_at":"2026-03-20T14:22:00Z"}}"#.to_string()
}

// -- people list --org / --owner: exit 2 + hint, zero HTTP --

#[test]
fn people_list_org_flag_rejects_before_http_with_replacement_hint() {
    cmd_unreachable()
        .args(["people", "list", "--org", "org_123"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--org/--owner were removed"))
        .stderr(predicate::str::contains("filter client-side"))
        .stderr(predicate::str::contains("jq"))
        // Zero HTTP (Pitfall 7): unreachable server, yet no connection error.
        .stderr(predicate::str::contains("Connection failed").not());
}

#[test]
fn people_list_owner_flag_rejects_before_http_with_replacement_hint() {
    cmd_unreachable()
        .args(["people", "list", "--owner", "usr_001"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--org/--owner were removed"))
        .stderr(predicate::str::contains("filter client-side"))
        .stderr(predicate::str::contains("Connection failed").not());
}

// -- pipelines/stages create --custom-field: exit 2 + hint, zero HTTP --

#[test]
fn pipelines_create_custom_field_rejects_before_http_with_replacement_hint() {
    cmd_unreachable()
        .args(["pipelines", "create", "--name", "P", "--custom-field", "k=v"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--custom-field was removed"))
        .stderr(predicate::str::contains("pipelines do not support custom fields"))
        .stderr(predicate::str::contains("deals/orgs/people/activities"))
        .stderr(predicate::str::contains("Connection failed").not());
}

#[test]
fn stages_create_custom_field_rejects_before_http_with_replacement_hint() {
    cmd_unreachable()
        .args([
            "stages",
            "create",
            "--name",
            "S",
            "--pipeline",
            "pl_1",
            "--custom-field",
            "k=v",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--custom-field was removed"))
        .stderr(predicate::str::contains("stages do not support custom fields"))
        .stderr(predicate::str::contains("Connection failed").not());
}

// -- workflows create --active: exit 2 + activation path hint --

#[test]
fn workflows_create_active_flag_rejects_with_update_activation_hint() {
    cmd_unreachable()
        .args(["workflows", "create", "--name", "W", "--active", "true"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--active was removed"))
        .stderr(predicate::str::contains("pipelite workflows update"))
        .stderr(predicate::str::contains("--active true"))
        .stderr(predicate::str::contains("Connection failed").not());
}

// -- --help hiding: removed flags must not be advertised --

#[test]
fn people_list_help_hides_removed_filter_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["people", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--org").not())
        .stdout(predicate::str::contains("--owner").not())
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--offset"));
}

#[test]
fn stages_create_help_hides_removed_custom_field() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--custom-field").not())
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--pipeline"));
}

#[test]
fn pipelines_create_help_hides_removed_custom_field() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["pipelines", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--custom-field").not())
        .stdout(predicate::str::contains("--name"));
}

#[test]
fn workflows_create_help_hides_removed_active_flag() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--active").not())
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--triggers"));
}

// -- workflows create: no prompt, no `active` key on any path --

#[test]
fn workflows_create_flag_path_sends_no_active_and_never_prompts() {
    let (url, counter, requests) =
        spawn_capturing_stub_server(&[(200, workflow_created_body())]);

    // Non-TTY run (assert_cmd pipes stdio): must succeed without prompting.
    cmd_with_server(&url)
        .args(["workflows", "create", "--name", "W", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wf_new"));

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    let req_body = requests[0]
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("");
    assert!(
        !req_body.contains("\"active\""),
        "request body must not carry active: {req_body}"
    );
    assert!(req_body.contains("\"name\":\"W\""), "body: {req_body}");
}

#[test]
fn workflows_create_stdin_with_active_key_ignores_it_and_sends_none() {
    let (url, counter, requests) =
        spawn_capturing_stub_server(&[(200, workflow_created_body())]);

    // A3: serde ignores unknown stdin keys — `active` in the payload is
    // dropped, never forwarded to the server.
    cmd_with_server(&url)
        .args(["workflows", "create", "--stdin"])
        .write_stdin(r#"{"name":"W","active":true}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("wf_new"));

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let requests = requests.lock().unwrap();
    let req_body = requests[0]
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("");
    assert!(
        !req_body.contains("\"active\""),
        "request body must not carry active: {req_body}"
    );
}
