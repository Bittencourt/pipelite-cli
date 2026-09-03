use assert_cmd::Command;
use predicates::prelude::*;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Helper: build a pipelite command with fake config env vars.
/// Uses an unreachable server to prove dry-run never makes HTTP requests.
fn cmd() -> Command {
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

/// Full Workflow JSON fixture.
fn workflow_json(id: &str, name: &str, active: bool) -> String {
    format!(
        r#"{{"id":"{id}","name":"{name}","description":null,"triggers":[],"nodes":[],"active":{active},"created_by":"user_001","created_at":"2026-01-15T10:30:00Z","updated_at":"2026-03-20T14:22:00Z"}}"#
    )
}

fn page_body(items: &[String], total: u64, offset: u64, limit: u64) -> String {
    format!(
        r#"{{"data":[{}],"meta":{{"total":{total},"offset":{offset},"limit":{limit}}}}}"#,
        items.join(",")
    )
}

/// Body-carrying stub server with a request counter (pattern from
/// error_layer_stub_test.rs; copied per test crate).
fn spawn_stub_server(script: &[(u16, String)]) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub server");
    let addr = listener.local_addr().expect("stub server address");
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let script: Vec<(u16, String)> = script.to_vec();

    std::thread::spawn(move || {
        for (status, body) in script {
            let (mut stream, _) = match listener.accept() {
                Ok(conn) => conn,
                Err(_) => break,
            };

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

    (format!("http://{addr}"), counter)
}

#[test]
fn workflows_help_shows_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("trigger"));
}

#[test]
fn workflows_list_help_shows_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--active"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--offset"))
        .stdout(predicate::str::contains("--all"))
        .stdout(predicate::str::contains("--fields"));
}

#[test]
fn workflows_create_help_shows_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--triggers"))
        .stdout(predicate::str::contains("--nodes"))
        .stdout(predicate::str::contains("--stdin"))
        .stdout(predicate::str::contains("--description"));
}

#[test]
fn workflows_trigger_help_shows_data_flag() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "trigger", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--data"));
}

#[test]
fn workflows_get_requires_id() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "get"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn workflows_delete_requires_id() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "delete"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn workflows_trigger_requires_id() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "trigger"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn workflows_alias_w_works() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["w", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("trigger"));
}

#[test]
fn workflows_create_dry_run() {
    cmd()
        .write_stdin("")
        .args(["workflows", "create", "--name", "Test", "--dry-run", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("POST"))
        .stdout(predicate::str::contains("/api/v1/workflows"));
}

#[test]
fn workflows_trigger_dry_run() {
    cmd()
        .write_stdin("")
        .args(["workflows", "trigger", "wf_abc123", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("POST"))
        .stdout(predicate::str::contains("/api/v1/workflows/wf_abc123/run"));
}

#[test]
fn workflows_delete_dry_run() {
    cmd()
        .write_stdin("")
        .args(["workflows", "delete", "wf_abc123", "--dry-run", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DELETE"));
}

#[test]
fn workflows_create_headless_without_name_fails() {
    cmd()
        .write_stdin("")
        .args(["workflows", "create", "--no-input"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--name"));
}

#[test]
fn workflows_delete_help_shows_force_flag() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--force"));
}

// -- FIX-01: workflows list --active filters client-side (server ignores it) --

#[test]
fn workflows_list_active_true_filters_client_side_with_warning() {
    let body = page_body(
        &[
            workflow_json("wf_1", "Active One", true),
            workflow_json("wf_2", "Inactive One", false),
            workflow_json("wf_3", "Active Two", true),
        ],
        3,
        0,
        50,
    );
    let (url, counter) = spawn_stub_server(&[(200, body)]);

    cmd_with_server(&url)
        .args(["workflows", "list", "--active", "true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active One"))
        .stdout(predicate::str::contains("Active Two"))
        .stdout(predicate::str::contains("Inactive One").not())
        // Locked warning line — exactly this text, on stderr.
        .stderr(predicate::str::contains(
            "warning: --active filters client-side after fetching all records",
        ));

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "--active routes through fetch_all; total fits in one 100-batch, so one request"
    );
}

#[test]
fn workflows_list_active_covers_all_pages_before_filtering() {
    // CR-01 regression: --active must cover ALL records, not just the first
    // page. 150 workflows across 2 fetch_all pages (batch size 100); the only
    // active rows are "Page One Active" (page 1) and "Page Two Active"
    // (page 2). The page-2 active row MUST appear in the output and the
    // request counter MUST show both fetches.
    let total: u64 = 150;
    let page_one: Vec<String> = (1..=100usize)
        .map(|i| {
            if i == 1 {
                workflow_json("wf_1", "Page One Active", true)
            } else {
                workflow_json(&format!("wf_{i}"), &format!("Filler One {i}"), false)
            }
        })
        .collect();
    let page_two: Vec<String> = (101..=150usize)
        .map(|i| {
            if i == 150 {
                workflow_json("wf_150", "Page Two Active", true)
            } else {
                workflow_json(&format!("wf_{i}"), &format!("Filler Two {i}"), false)
            }
        })
        .collect();
    let script = [
        (200, page_body(&page_one, total, 0, 100)),
        (200, page_body(&page_two, total, 100, 100)),
    ];
    let (url, counter) = spawn_stub_server(&script);

    cmd_with_server(&url)
        .args(["workflows", "list", "--active", "true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Page One Active"))
        // The key assertion: an active record beyond page 1 must not be lost.
        .stdout(predicate::str::contains("Page Two Active"))
        .stdout(predicate::str::contains("Filler One 2").not())
        .stderr(predicate::str::contains(
            "warning: --active filters client-side after fetching all records",
        ));

    assert_eq!(counter.load(Ordering::SeqCst), 2, "both pages fetched before filtering");
}

#[test]
fn workflows_list_active_false_fetches_all_pages_before_filtering() {
    // --active false follows the same fetch_all route: with 150 records
    // across 2 pages the filter must see every record (all inactive here, so
    // all 150 survive) and both fetches must happen.
    let total: u64 = 150;
    let page_one: Vec<String> = (1..=100usize)
        .map(|i| workflow_json(&format!("wf_{i}"), &format!("Filler One {i}"), false))
        .collect();
    let page_two: Vec<String> = (101..=150usize)
        .map(|i| workflow_json(&format!("wf_{i}"), &format!("Filler Two {i}"), false))
        .collect();
    let script = [
        (200, page_body(&page_one, total, 0, 100)),
        (200, page_body(&page_two, total, 100, 100)),
    ];
    let (url, counter) = spawn_stub_server(&script);

    cmd_with_server(&url)
        .args(["workflows", "list", "--active", "false"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Filler One 2"))
        .stdout(predicate::str::contains("Filler Two 150"))
        .stderr(predicate::str::contains(
            "warning: --active filters client-side after fetching all records",
        ));

    assert_eq!(counter.load(Ordering::SeqCst), 2, "both pages fetched before filtering");
}

#[test]
fn workflows_list_without_active_stays_unfiltered_and_silent() {
    let body = page_body(
        &[
            workflow_json("wf_1", "Active One", true),
            workflow_json("wf_2", "Inactive One", false),
        ],
        2,
        0,
        50,
    );
    let (url, _counter) = spawn_stub_server(&[(200, body)]);

    cmd_with_server(&url)
        .args(["workflows", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Inactive One"))
        .stderr(predicate::str::contains("client-side").not());
}
