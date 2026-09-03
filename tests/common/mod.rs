//! Shared hermetic helpers for the Phase 9 stub-server integration tests.
// Used by workflow_runs/templates/docs stub test files
//
// Generalizes the per-file helpers of error_layer_stub_test.rs and
// batch_error_test.rs so the three Phase 9 test files (workflow runs,
// templates, docs) share one copy of the hand-rolled TcpListener pattern
// (no new crates). Tests/ subdirectory modules are not compiled as
// separate test binaries, so `mod common;` inclusion is free.

use assert_cmd::Command;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Helper: pipelite command pointed at a live stub server URL.
/// Sets PIPELITE_URL + PIPELITE_SERVER_URL + fake key — hermetic in the
/// no-config suite default; the env override wins over any real config
/// on dev machines.
pub fn cmd_with_server(url: &str) -> Command {
    let mut c = Command::cargo_bin("pipelite").expect("cargo binary pipelite");
    c.env("PIPELITE_URL", url);
    c.env("PIPELITE_SERVER_URL", url);
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

/// Helper: pipelite command with an unreachable server (port 1 on
/// loopback) — for pre-HTTP validation tests (clap must reject BEFORE any
/// connection attempt, so no Connection error may ever appear).
pub fn cmd() -> Command {
    cmd_with_server("http://127.0.0.1:1")
}

/// Hand-rolled HTTP stub server (no new crates): serves one scripted
/// `(status, raw body)` response per accepted connection, in order, and
/// RECORDS each request head (request line + headers, lowercased) so tests
/// can assert query strings on the wire and header presence/absence.
///
/// Returns `(base_url, request_counter, captured_heads)`.
pub fn spawn_head_capturing_stub_server(
    script: &[(u16, &str)],
) -> (String, Arc<AtomicUsize>, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub server");
    let addr = listener.local_addr().expect("stub server address");
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let heads: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let heads_clone = Arc::clone(&heads);
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

            // Record the full lowercased head (request line + headers)
            // BEFORE responding, for wire-level assertions.
            let header_end = received
                .windows(4)
                .position(|w| w == b"\r\n\r\n")
                .map(|p| p + 4)
                .unwrap_or(received.len());
            let head = String::from_utf8_lossy(&received[..header_end]).to_lowercase();
            heads_clone.lock().expect("heads lock").push(head);

            // Read exactly Content-Length body bytes, if any.
            let headers = &heads_clone.lock().expect("heads lock")[0];
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

    (format!("http://{addr}"), counter, heads)
}
