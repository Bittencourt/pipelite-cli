use assert_cmd::Command;
use predicates::prelude::*;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[test]
fn stages_help_shows_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn stages_list_help_shows_pipeline_flag() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--pipeline"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--offset"));
}

#[test]
fn stages_create_help_shows_required_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--pipeline"))
        .stdout(predicate::str::contains("--color"))
        .stdout(predicate::str::contains("--type"));
}

#[test]
fn stages_get_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "get"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn stages_delete_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "delete"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn stages_update_help_shows_optional_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--color"))
        .stdout(predicate::str::contains("--type"))
        .stdout(predicate::str::contains("--description"));
}

#[test]
fn main_help_shows_stages_subcommand() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("stages"));
}

#[test]
fn stages_list_without_config_fails_with_exit_code_1() {
    // Running stages list with --pipeline should fail at runtime (no config),
    // NOT exit code 2 (args are valid).
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "list", "--pipeline", "pl_123"])
        .assert()
        .failure()
        .code(1);
}

// -- FIX-04: stages list all-mode (stub-based) --

/// Helper: pipelite command pointed at a live stub server URL. Env wins over
/// any real config on dev machines (hermetic).
fn cmd_with_server(url: &str) -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", url);
    c.env("PIPELITE_SERVER_URL", url);
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

/// Full Stage JSON per row (Pitfall 8: every row carries its own pipeline_id).
fn stage_json(id: &str, pipeline_id: &str, name: &str, position: u64) -> String {
    format!(
        r##"{{"id":"{id}","pipeline_id":"{pipeline_id}","name":"{name}","description":null,"color":"#00ff00","type":"open","position":{position},"created_at":"2026-01-15T10:30:00Z","updated_at":"2026-03-20T14:22:00Z"}}"##
    )
}

fn page_body(items: &[String], total: u64, offset: u64, limit: u64) -> String {
    format!(
        r#"{{"data":[{}],"meta":{{"total":{total},"offset":{offset},"limit":{limit}}}}}"#,
        items.join(",")
    )
}

/// Body-carrying stub server that also captures request heads (request line
/// + headers), so tests can assert exact query pairs. Generalization of the
/// error_layer_stub_test.rs helper — copied per test crate (no shared helpers).
fn spawn_capturing_stub_server(
    script: &[(u16, String)],
) -> (String, Arc<AtomicUsize>, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub server");
    let addr = listener.local_addr().expect("stub server address");
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let heads: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let heads_clone = Arc::clone(&heads);
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
            heads_clone
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&received).to_string());

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

    (format!("http://{addr}"), counter, heads)
}

#[test]
fn stages_list_without_pipeline_lists_all_stages_in_one_unfiltered_call() {
    let body = page_body(
        &[
            stage_json("stg_1", "pl_1", "Qualified", 1),
            stage_json("stg_2", "pl_2", "Discovery", 1),
        ],
        2,
        0,
        50,
    );
    let (url, counter, heads) = spawn_capturing_stub_server(&[(200, body)]);

    cmd_with_server(&url)
        .args(["stages", "list"])
        .assert()
        .success()
        // Rows from multiple pipelines are distinguishable via the column.
        .stdout(predicate::str::contains("pipeline_id"))
        .stdout(predicate::str::contains("pl_1"))
        .stdout(predicate::str::contains("pl_2"));

    assert_eq!(counter.load(Ordering::SeqCst), 1, "single unfiltered call");
    let heads = heads.lock().unwrap();
    assert_eq!(heads.len(), 1);
    assert!(
        !heads[0].contains("pipeline_id="),
        "no pipeline_id pair expected in all-mode, got: {}",
        heads[0]
    );
}

#[test]
fn stages_list_with_pipeline_still_filters() {
    let body = page_body(&[stage_json("stg_1", "pl_1", "Qualified", 1)], 1, 0, 50);
    let (url, counter, heads) = spawn_capturing_stub_server(&[(200, body)]);

    cmd_with_server(&url)
        .args(["stages", "list", "--pipeline", "pl_1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Qualified"));

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let heads = heads.lock().unwrap();
    assert!(
        heads[0].contains("pipeline_id=pl_1"),
        "expected pipeline_id filter, got: {}",
        heads[0]
    );
}
