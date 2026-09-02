use assert_cmd::Command;
use predicates::prelude::*;

/// Per CLAUDE.md: tests use 127.0.0.1:1 + fake-test-key.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

#[test]
fn deals_update_help_shows_stdin() {
    cmd()
        .args(["deals", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn deals_delete_help_shows_stdin() {
    cmd()
        .args(["deals", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}
