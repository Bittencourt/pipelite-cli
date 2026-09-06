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

// -- Help text tests (verify --stdin and multiple IDs for all 7 entities) --

#[test]
fn deals_delete_help_shows_stdin_flag() {
    cmd()
        .args(["deals", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn orgs_delete_help_shows_stdin_flag() {
    cmd()
        .args(["orgs", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn people_delete_help_shows_stdin_flag() {
    cmd()
        .args(["people", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn activities_delete_help_shows_stdin_flag() {
    cmd()
        .args(["activities", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn pipelines_delete_help_shows_stdin_flag() {
    cmd()
        .args(["pipelines", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn stages_delete_help_shows_stdin_flag() {
    cmd()
        .args(["stages", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn workflows_delete_help_shows_stdin_flag() {
    cmd()
        .args(["workflows", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

// -- Dry-run tests for multi-ID delete (D-04) --

#[test]
fn deals_batch_delete_dry_run_multiple_ids() {
    // write_stdin("") creates a piped (non-TTY) stdin so the command does not
    // block waiting for terminal input. The empty content is not read because
    // positional IDs are provided (--stdin is not set).
    cmd()
        .write_stdin("")
        .args(["deals", "delete", "deal_1", "deal_2", "deal_3", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deal_1"))
        .stdout(predicate::str::contains("deal_2"))
        .stdout(predicate::str::contains("deal_3"));
}

// -- Dry-run tests for --stdin delete (D-04) --

#[test]
fn deals_batch_delete_dry_run_stdin() {
    let input = r#"["deal_1","deal_2"]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "delete", "--stdin", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deal_1"))
        .stdout(predicate::str::contains("deal_2"));
}

// -- Non-interactive batch deletes require --force (CR-01) --

#[test]
fn deals_batch_delete_stdin_without_force_refuses() {
    // Piped stdin cannot serve as both the ID source and the confirmation
    // prompt, so the batch delete must refuse without --force (CR-01).
    let input = r#"["deal_1","deal_2"]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "delete", "--stdin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Refusing to delete without confirmation"))
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn deals_batch_delete_stdin_with_force_proceeds() {
    // With --force the gate is skipped and the deletes are attempted (they
    // fail against the unreachable server, but NOT with the refusal error).
    let input = r#"["deal_1","deal_2"]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "delete", "--stdin", "--force"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed").or(predicate::str::contains("failed")));
}

#[test]
fn deals_batch_delete_positional_without_force_refuses() {
    // Multi-ID positional deletes on a piped (non-TTY) stdin are also gated.
    cmd()
        .write_stdin("")
        .args(["deals", "delete", "deal_1", "deal_2"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Refusing to delete without confirmation"));
}

// -- Invalid --stdin input --

#[test]
fn deals_batch_delete_invalid_json_fails() {
    cmd()
        .write_stdin("not json")
        .args(["deals", "delete", "--stdin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid JSON input"));
}

// -- Mutual exclusivity: positional IDs + --stdin --

#[test]
fn deals_delete_stdin_and_positional_fails() {
    cmd()
        .write_stdin(r#"["deal_1"]"#)
        .args(["deals", "delete", "deal_extra", "--stdin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("mutually exclusive"));
}

// -- No args and no --stdin should fail --

#[test]
fn deals_delete_no_args_fails() {
    // write_stdin("") creates a piped (non-TTY) stdin so the command does not
    // block waiting for terminal input. With no positional IDs and no --stdin
    // flag, clap should reject the command.
    cmd()
        .write_stdin("")
        .args(["deals", "delete"])
        .assert()
        .failure();
}
