use assert_cmd::Command;
use predicates::prelude::*;

/// Helper: build a pipelite command with fake config env vars.
/// Per CLAUDE.md: tests use 127.0.0.1:1 + fake-test-key.
///
/// Note: `PIPELITE_SERVER_URL` is the env var the app actually reads
/// (see src/config.rs env override); `PIPELITE_URL` is kept for
/// consistency with the rest of the test suite's cmd() helpers.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_SERVER_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

// -- Help text tests (verify --stdin flag appears for all 7 entities) --

#[test]
fn deals_update_help_shows_stdin_flag() {
    cmd()
        .args(["deals", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn orgs_update_help_shows_stdin_flag() {
    cmd()
        .args(["orgs", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn people_update_help_shows_stdin_flag() {
    cmd()
        .args(["people", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn activities_update_help_shows_stdin_flag() {
    cmd()
        .args(["activities", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn pipelines_update_help_shows_stdin_flag() {
    cmd()
        .args(["pipelines", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn stages_update_help_shows_stdin_flag() {
    cmd()
        .args(["stages", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn workflows_update_help_shows_stdin_flag() {
    cmd()
        .args(["workflows", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

// -- Dry-run tests (verify batch update --stdin --dry-run renders PUT requests) --

#[test]
fn deals_batch_update_dry_run_shows_put_requests() {
    let input = r#"[{"id":"deal_1","title":"New Title"},{"id":"deal_2","value":50000}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PUT"))
        .stdout(predicate::str::contains("/api/v1/deals/deal_1"))
        .stdout(predicate::str::contains("/api/v1/deals/deal_2"))
        .stdout(predicate::str::contains("New Title"));
}

#[test]
fn orgs_batch_update_dry_run_shows_put_requests() {
    let input = r#"[{"id":"org_1","name":"Acme Corp"}]"#;
    cmd()
        .write_stdin(input)
        .args(["orgs", "update", "--stdin", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PUT"))
        .stdout(predicate::str::contains("/api/v1/organizations/org_1"))
        .stdout(predicate::str::contains("Acme Corp"));
}

// -- Invalid input tests --

#[test]
fn deals_batch_update_invalid_json_fails() {
    cmd()
        .write_stdin("not json")
        .args(["deals", "update", "--stdin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid JSON input"));
}

// -- Mutual exclusivity: --stdin + positional id --

#[test]
fn deals_update_stdin_with_empty_pipe_fails() {
    // write_stdin("") creates a piped (non-TTY) stdin with empty content.
    // This means is_terminal() returns false, but the empty string fails JSON parsing.
    cmd()
        .write_stdin("")
        .args(["deals", "update", "--stdin"])
        .assert()
        .failure();
}
