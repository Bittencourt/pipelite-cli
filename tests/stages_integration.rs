use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn stages_help_shows_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn stages_list_help_shows_pipeline_flag() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--pipeline"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--offset"));
}

#[test]
fn stages_create_help_shows_required_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--pipeline"))
        .stdout(predicate::str::contains("--color"))
        .stdout(predicate::str::contains("--type"));
}

#[test]
fn stages_get_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "get"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn stages_delete_without_id_fails_with_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "delete"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn stages_update_help_shows_optional_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--color"))
        .stdout(predicate::str::contains("--type"))
        .stdout(predicate::str::contains("--description"));
}

#[test]
fn main_help_shows_stages_subcommand() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("stages"));
}

#[test]
fn stages_list_without_config_fails_with_exit_code_1() {
    // Running stages list with --pipeline should fail at runtime (no config),
    // NOT exit code 2 (args are valid).
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["stages", "list", "--pipeline", "pl_123"])
        .assert()
        .failure()
        .code(1);
}
