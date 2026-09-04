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

// The `notes` help page must be truthful (ROADMAP SC-5): it carries
// examples, states that the server has no single-note GET, does NOT
// advertise a `get` subcommand (hidden parse-then-error variant), and does
// NOT advertise an `--all` flag (server page cap is 100 — iterate --offset).
#[test]
fn notes_help_truthful() {
    let output = Command::cargo_bin("pipelite")
        .unwrap()
        .args(["notes", "--help"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Examples:"), "help: {stdout}");
    assert!(stdout.contains("no single-note GET"), "help: {stdout}");
    assert!(stdout.contains("--body"), "help: {stdout}");
    assert!(
        !stdout.contains("--all"),
        "notes --help must not advertise an --all flag:\n{stdout}"
    );
    assert!(
        stdout.lines().all(|l| !l.trim().starts_with("get")),
        "notes --help must not advertise a get subcommand:\n{stdout}"
    );
}

// Every notes subcommand help page carries Examples (after_help).
#[test]
fn notes_subcommands_have_examples() {
    for args in [
        ["notes", "list"],
        ["notes", "add"],
        ["notes", "edit"],
        ["notes", "delete"],
    ] {
        Command::cargo_bin("pipelite")
            .unwrap()
            .args(args)
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Examples:"));
    }
}

// The `webhooks` help page must be truthful (ROADMAP SC-1/SC-2): it carries
// examples, mentions the show-once signing secret, lists all five
// subcommands, and does NOT advertise a `--description` flag (the server has
// no such field and would silently strip it) or an `--all` flag (server page
// cap is 100 — iterate --offset).
#[test]
fn webhooks_help_truthful() {
    let output = Command::cargo_bin("pipelite")
        .unwrap()
        .args(["webhooks", "--help"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Examples:"), "help: {stdout}");
    assert!(stdout.contains("shown"), "help must mention the show-once secret: {stdout}");
    for sub in ["list", "get", "create", "update", "delete"] {
        assert!(
            stdout.lines().any(|l| l.trim().starts_with(sub)),
            "webhooks --help must list the {sub} subcommand:\n{stdout}"
        );
    }
    assert!(
        !stdout.contains("--description"),
        "webhooks --help must not advertise a --description flag:\n{stdout}"
    );
    assert!(
        !stdout.contains("--all"),
        "webhooks --help must not advertise an --all flag:\n{stdout}"
    );
}

// The create help must carry the full 13-event list, the https rule and the
// --stdin escape hatch; update help must state the merge semantics and the
// verbatim --stdin path; delete help must carry --force.
#[test]
fn webhooks_create_update_delete_help_specifics() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["webhooks", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--url"))
        .stdout(predicate::str::contains("--events"))
        .stdout(predicate::str::contains("--stdin"))
        .stdout(predicate::str::contains("deal.stage_changed"));

    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["webhooks", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"))
        .stdout(predicate::str::contains("unchanged"));

    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["webhooks", "delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--force"));
}

// Every webhooks subcommand help page carries Examples (after_help).
#[test]
fn webhooks_subcommands_have_examples() {
    for args in [
        ["webhooks", "list"],
        ["webhooks", "get"],
        ["webhooks", "create"],
        ["webhooks", "update"],
        ["webhooks", "delete"],
    ] {
        Command::cargo_bin("pipelite")
            .unwrap()
            .args(args)
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Examples:"));
    }
}
