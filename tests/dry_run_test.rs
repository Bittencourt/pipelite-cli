use assert_cmd::Command;
use predicates::prelude::*;

/// Helper: build a pipelite command with fake config env vars.
/// Uses an unreachable server to prove dry-run never makes HTTP requests.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", "http://localhost:1");
    c.env("PIPELITE_API_KEY", "test_key_fake");
    c
}

#[test]
fn deals_create_dry_run_shows_request() {
    // Dry-run shows method, URL, and body content without making an API call
    cmd()
        .write_stdin("")
        .arg("deals")
        .arg("create")
        .arg("--title")
        .arg("Test")
        .arg("--stage")
        .arg("stg_001")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("POST"))
        .stdout(predicate::str::contains("/api/v1/deals"))
        .stdout(predicate::str::contains("Test"));
}

#[test]
fn deals_create_dry_run_json_format() {
    cmd()
        .write_stdin("")
        .arg("deals")
        .arg("create")
        .arg("--title")
        .arg("Test")
        .arg("--stage")
        .arg("stg_001")
        .arg("--dry-run")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dry_run\": true"))
        .stdout(predicate::str::contains("\"method\": \"POST\""));
}

#[test]
fn deals_delete_dry_run() {
    // In non-TTY (piped stdout), format defaults to JSON
    cmd()
        .write_stdin("")
        .arg("deals")
        .arg("delete")
        .arg("deal_123")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dry_run\": true"))
        .stdout(predicate::str::contains("\"method\": \"DELETE\""))
        .stdout(predicate::str::contains("deal_123"));
}

#[test]
fn deals_update_dry_run() {
    cmd()
        .write_stdin("")
        .arg("deals")
        .arg("update")
        .arg("deal_123")
        .arg("--title")
        .arg("New")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("PUT"))
        .stdout(predicate::str::contains("/api/v1/deals/deal_123"));
}

#[test]
fn orgs_create_dry_run() {
    cmd()
        .write_stdin("")
        .arg("orgs")
        .arg("create")
        .arg("--name")
        .arg("Acme")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("POST"))
        .stdout(predicate::str::contains("organizations"))
        .stdout(predicate::str::contains("Acme"));
}

#[test]
fn dry_run_does_not_need_server() {
    // PIPELITE_URL points to unreachable server (port 1).
    // Exit 0 proves no HTTP request was made.
    cmd()
        .write_stdin("")
        .arg("deals")
        .arg("create")
        .arg("--title")
        .arg("Test")
        .arg("--stage")
        .arg("stg_001")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn people_create_dry_run() {
    cmd()
        .write_stdin("")
        .arg("people")
        .arg("create")
        .arg("--first-name")
        .arg("Jane")
        .arg("--last-name")
        .arg("Doe")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("POST"))
        .stdout(predicate::str::contains("Jane"));
}
