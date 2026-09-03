use assert_cmd::Command;
use predicates::prelude::*;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Helper: pipelite command pointed at a live stub server URL (hermetic).
fn cmd_with_server(url: &str) -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", url);
    c.env("PIPELITE_SERVER_URL", url);
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

/// Full Deal JSON fixture (programmatic, for page generation).
fn deal_json(i: usize) -> String {
    format!(
        r#"{{"id":"deal_{i}","title":"Deal {i}","value":100.0,"stage_id":"stage_001","organization_id":null,"person_id":null,"owner_id":"user_001","position":null,"expected_close_date":null,"notes":null,"custom_fields":null,"created_at":"2026-01-15T10:30:00Z","updated_at":"2026-03-20T14:22:00Z"}}"#
    )
}

/// One page body with `count` deals starting at `offset`, server total `total`.
fn deals_page(offset: usize, count: usize, total: u64) -> String {
    let items: Vec<String> = (offset..offset + count).map(deal_json).collect();
    format!(
        r#"{{"data":[{}],"meta":{{"total":{total},"offset":{},"limit":{count}}}}}"#,
        items.join(","),
        offset as u64
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
fn deals_help_shows_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn deals_list_help_shows_filter_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stage"))
        .stdout(predicate::str::contains("--org"))
        .stdout(predicate::str::contains("--owner"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--offset"))
        .stdout(predicate::str::contains("--all"));
}

#[test]
fn deals_create_help_shows_required_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--title"))
        .stdout(predicate::str::contains("--stage"))
        .stdout(predicate::str::contains("--stdin"))
        .stdout(predicate::str::contains("--custom-field"));
}

#[test]
fn deals_get_help_shows_id_argument() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<ID>").or(predicate::str::contains("id")))
        .stdout(predicate::str::contains("--fields"))
        .stdout(predicate::str::contains("--expand"));
}

#[test]
fn deals_get_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "get"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn deals_delete_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "delete"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn deals_list_limit_zero_is_accepted() {
    // --limit 0 is a valid argument (clap accepts it).
    // The command will fail at runtime (no config), but NOT with exit code 2.
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "list", "--limit", "0"])
        .assert()
        .failure()
        .code(1); // Runtime error, not clap misuse
}

#[test]
fn deals_update_help_shows_optional_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--title"))
        .stdout(predicate::str::contains("--stage"))
        .stdout(predicate::str::contains("--value"))
        .stdout(predicate::str::contains("--custom-field"));
}

#[test]
fn deals_delete_help_shows_id_and_examples() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn main_help_shows_deals_subcommand() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("deals"));
}

// -- FIX-06: --all ceiling warning (guard stays conditional) --

#[test]
fn deals_list_all_at_ceiling_warns_and_exits_zero() {
    // total=1243 across 10 pages of 100: the loop must stop at the 1000-record
    // cap, warn on stderr, and still exit 0.
    let total: u64 = 1243;
    let script: Vec<(u16, String)> = (0..10u64)
        .map(|p| (200u16, deals_page((p * 100) as usize, 100, total)))
        .collect();
    let (url, counter) = spawn_stub_server(&script);

    cmd_with_server(&url)
        .args(["deals", "list", "--all"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "warning: --all stopped at 1000 records (server ceiling); results may be incomplete",
        ));

    assert_eq!(counter.load(Ordering::SeqCst), 10, "exactly 10 page requests");
}

#[test]
fn deals_list_all_under_ceiling_stays_silent() {
    // Negative case: total=450 across 5 pages is UNDER the cap — the warning
    // must NOT print (the `if total > max_records` guard stays conditional).
    let total: u64 = 450;
    let script: Vec<(u16, String)> = (0..5u64)
        .map(|p| (200u16, deals_page((p * 100) as usize, 100, total)))
        .collect();
    let (url, counter) = spawn_stub_server(&script);

    cmd_with_server(&url)
        .args(["deals", "list", "--all"])
        .assert()
        .success()
        .stderr(predicate::str::contains("server ceiling").not());

    assert_eq!(counter.load(Ordering::SeqCst), 5, "exactly 5 page requests");
}
