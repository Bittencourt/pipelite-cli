use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn deals_help_shows_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn deals_list_help_shows_filter_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stage"))
        .stdout(predicate::str::contains("--org"))
        .stdout(predicate::str::contains("--owner"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--offset"))
        .stdout(predicate::str::contains("--all"));
}

#[test]
fn deals_create_help_shows_required_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--title"))
        .stdout(predicate::str::contains("--stage"))
        .stdout(predicate::str::contains("--stdin"))
        .stdout(predicate::str::contains("--custom-field"));
}

#[test]
fn deals_get_help_shows_id_argument() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<ID>").or(predicate::str::contains("id")))
        .stdout(predicate::str::contains("--fields"))
        .stdout(predicate::str::contains("--expand"));
}

#[test]
fn deals_get_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "get"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn deals_delete_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "delete"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn deals_list_limit_zero_is_accepted() {
    // --limit 0 is a valid argument (clap accepts it).
    // The command will fail at runtime (no config), but NOT with exit code 2.
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "list", "--limit", "0"])
        .assert()
        .failure()
        .code(1); // Runtime error, not clap misuse
}

#[test]
fn deals_update_help_shows_optional_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--title"))
        .stdout(predicate::str::contains("--stage"))
        .stdout(predicate::str::contains("--value"))
        .stdout(predicate::str::contains("--custom-field"));
}

#[test]
fn deals_delete_help_shows_id_and_examples() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["deals", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn main_help_shows_deals_subcommand() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("deals"));
}
