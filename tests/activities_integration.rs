use assert_cmd::Command;
use predicates::prelude::*;

fn pipelite() -> Command {
    Command::cargo_bin("pipelite").unwrap()
}

#[test]
fn activities_help_shows_subcommands() {
    pipelite()
        .args(["activities", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("list")
                .and(predicate::str::contains("get"))
                .and(predicate::str::contains("create"))
                .and(predicate::str::contains("update"))
                .and(predicate::str::contains("delete")),
        );
}

#[test]
fn activities_list_help_shows_filter_flags() {
    pipelite()
        .args(["activities", "list", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("--type")
                .and(predicate::str::contains("--deal"))
                .and(predicate::str::contains("--done"))
                .and(predicate::str::contains("--limit"))
                .and(predicate::str::contains("--offset")),
        );
}

#[test]
fn activities_create_help_shows_required_flags() {
    pipelite()
        .args(["activities", "create", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("--title")
                .and(predicate::str::contains("--type"))
                .and(predicate::str::contains("--stdin")),
        );
}

#[test]
fn activities_get_help_shows_id_argument() {
    pipelite()
        .args(["activities", "get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<ID>").or(predicate::str::contains("id")));
}

#[test]
fn activities_get_without_id_fails_with_exit_code_2() {
    pipelite()
        .args(["activities", "get"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn activities_delete_without_id_fails_with_exit_code_2() {
    pipelite()
        .args(["activities", "delete"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn activities_update_help_shows_flags() {
    pipelite()
        .args(["activities", "update", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("--title")
                .and(predicate::str::contains("--type"))
                .and(predicate::str::contains("--mark-done"))
                .and(predicate::str::contains("--mark-undone"))
                .and(predicate::str::contains("--completed-at")),
        );
}

#[test]
fn main_help_shows_activities_subcommand() {
    pipelite()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("activities"));
}
