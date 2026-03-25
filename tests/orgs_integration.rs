use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn orgs_help_shows_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["orgs", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn orgs_list_help_shows_filter_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["orgs", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--owner"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--offset"))
        .stdout(predicate::str::contains("--all"));
}

#[test]
fn orgs_create_help_shows_required_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["orgs", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--stdin"))
        .stdout(predicate::str::contains("--custom-field"));
}

#[test]
fn orgs_get_help_shows_id_argument() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["orgs", "get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<ID>").or(predicate::str::contains("id")))
        .stdout(predicate::str::contains("--fields"))
        .stdout(predicate::str::contains("--expand"));
}

#[test]
fn orgs_get_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["orgs", "get"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn orgs_delete_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["orgs", "delete"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn orgs_update_help_shows_optional_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["orgs", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--website"))
        .stdout(predicate::str::contains("--industry"))
        .stdout(predicate::str::contains("--notes"));
}

#[test]
fn main_help_shows_orgs_subcommand() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("orgs"));
}
