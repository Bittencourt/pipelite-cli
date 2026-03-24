use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn main_help_lists_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("ping"))
        .stdout(predicate::str::contains("config"));
}

#[test]
fn init_help_has_examples() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn ping_help_has_examples() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["ping", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn config_help_shows_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("set"));
}

#[test]
fn config_show_help_has_examples() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["config", "show", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"));
}
