use assert_cmd::Command;
use predicates::prelude::*;

/// Helper: build a pipelite command with fake config env vars.
/// Uses an unreachable server to prove dry-run never makes HTTP requests.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
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
