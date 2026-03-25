use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn pipelines_help_shows_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["pipelines", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn pipelines_list_help_shows_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["pipelines", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--offset"))
        .stdout(predicate::str::contains("--all"));
}

#[test]
fn pipelines_create_help_shows_required_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["pipelines", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--default"))
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn pipelines_get_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["pipelines", "get"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn pipelines_delete_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["pipelines", "delete"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn pipelines_update_help_shows_optional_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["pipelines", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--default"));
}

#[test]
fn main_help_shows_pipelines_subcommand() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("pipelines"));
}
