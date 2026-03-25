use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn dashboard_help_shows_description() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["dashboard", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pipeline overview"))
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn dashboard_json_format_with_unreachable_server_returns_connection_error() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["dashboard", "--format", "json"])
        .env("PIPELITE_API_KEY", "pk_test_fake")
        .env("PIPELITE_SERVER_URL", "http://127.0.0.1:19999")
        .assert()
        .failure()
        .stderr(predicate::str::contains("connect"));
}

#[test]
fn main_help_shows_dashboard_subcommand() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("dashboard"));
}
