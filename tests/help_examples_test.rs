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

// The `workflows runs` help pages must each carry Examples (after_help):
// `pipelite workflows runs --help`, `pipelite workflows runs list --help`
// (advertising --include-dry-run), and `pipelite workflows runs get --help`
// (advertising the required --workflow flag).
#[test]
fn workflows_runs_help_has_examples() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "runs", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"));

    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "runs", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("--include-dry-run"));

    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["workflows", "runs", "get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("--workflow"));
}

// The `pipelite docs --help` page must carry examples and state that
// --format is ignored (the OpenAPI spec is JSON, not tabular output).
#[test]
fn docs_help_has_examples() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["docs", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("--save"))
        .stdout(predicate::str::contains("ignored"));
}

// The `templates` help page must be truthful (ROADMAP SC-4): it carries
// examples, states that the server has no template update, and does NOT
// advertise an update subcommand (the hidden Update variant is a
// parse-then-error rejection, never a real command).
#[test]
fn templates_help_has_examples_and_no_update_advertisement() {
    let output = Command::cargo_bin("pipelite")
        .unwrap()
        .args(["templates", "--help"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Examples:"), "help: {stdout}");
    assert!(stdout.contains("no template update"), "help: {stdout}");
    assert!(
        stdout.lines().all(|l| !l.trim().starts_with("update")),
        "templates --help must not advertise an update subcommand:\n{stdout}"
    );
}
