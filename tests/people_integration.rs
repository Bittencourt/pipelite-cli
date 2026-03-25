use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn people_help_shows_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["people", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn people_list_help_shows_filter_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["people", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--org"))
        .stdout(predicate::str::contains("--owner"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--offset"));
}

#[test]
fn people_create_help_shows_required_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["people", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--first-name"))
        .stdout(predicate::str::contains("--last-name"))
        .stdout(predicate::str::contains("--stdin"));
}

#[test]
fn people_get_help_shows_id_argument() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["people", "get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<ID>").or(predicate::str::contains("id")))
        .stdout(predicate::str::contains("--fields"))
        .stdout(predicate::str::contains("--expand"));
}

#[test]
fn people_get_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["people", "get"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn people_delete_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["people", "delete"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn people_update_help_shows_optional_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["people", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--first-name"))
        .stdout(predicate::str::contains("--last-name"))
        .stdout(predicate::str::contains("--email"))
        .stdout(predicate::str::contains("--phone"));
}

#[test]
fn main_help_shows_people_subcommand() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("people"));
}
