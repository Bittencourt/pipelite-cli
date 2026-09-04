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

// The `trash` help page must be truthful (ROADMAP SC-3/SC-4): it carries
// examples, lists all three subcommands, and surfaces the purge warning at
// GROUP level ("permanently destroys").
#[test]
fn trash_help_truthful() {
    let output = Command::cargo_bin("pipelite")
        .unwrap()
        .args(["trash", "--help"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Examples:"), "help: {stdout}");
    for sub in ["list", "restore", "purge"] {
        assert!(
            stdout.lines().any(|l| l.trim().starts_with(sub)),
            "trash --help must list the {sub} subcommand:\n{stdout}"
        );
    }
    assert!(
        stdout.contains("permanently destroys"),
        "the purge warning must surface at group level:\n{stdout}"
    );
}

// The list help must advertise --all/--type, carry the 10,000 offset-cap
// note and the jq round-trip example; restore help must carry the
// no-confirmation semantics; purge help must carry --force and the
// permanent-destruction warning.
#[test]
fn trash_subcommand_help_specifics() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["trash", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--all"))
        .stdout(predicate::str::contains("--type"))
        .stdout(predicate::str::contains("10,000"))
        .stdout(predicate::str::contains("jq"));

    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["trash", "restore", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no confirmation"));

    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["trash", "purge", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--force"))
        .stdout(predicate::str::contains("permanently"));
}

// Every trash subcommand help page carries Examples (after_help).
#[test]
fn trash_subcommands_have_examples() {
    for args in [
        ["trash", "list"],
        ["trash", "restore"],
        ["trash", "purge"],
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

// The audit group help must state the admin gating, carry Examples, list
// the `list` subcommand — and NOT advertise an --all flag (audit pages are
// capped at 100; deep history is reached by iterating --offset).
#[test]
fn audit_help_truthful() {
    let output = Command::cargo_bin("pipelite")
        .unwrap()
        .args(["audit", "--help"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Examples:"), "help: {stdout}");
    assert!(stdout.contains("admin"), "help must state the admin gating: {stdout}");
    assert!(
        stdout.lines().any(|l| l.trim().starts_with("list")),
        "audit --help must list the list subcommand:\n{stdout}"
    );
    assert!(
        !stdout.contains("--all"),
        "audit --help must not advertise an --all flag:\n{stdout}"
    );
}

// The audit list help must document all four filter flags, every enum
// value (6 entity types incl. import_session + export; 5 actor kinds; the
// merged action), and the no---all/iterate---offset paging note.
#[test]
fn audit_list_help_specifics() {
    let output = Command::cargo_bin("pipelite")
        .unwrap()
        .args(["audit", "list", "--help"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);

    for flag in [
        "--entity-type",
        "--entity-id",
        "--actor-kind",
        "--workflow-run-id",
    ] {
        assert!(stdout.contains(flag), "list help must document {flag}:\n{stdout}");
    }
    for value in [
        "import_session", "export", "api_key", "system", "workflow_run", "merged",
    ] {
        assert!(
            stdout.contains(value),
            "list help must document the '{value}' enum value:\n{stdout}"
        );
    }
    assert!(
        stdout.contains("iterate --offset") || stdout.contains("no --all"),
        "list help must explain paging without --all:\n{stdout}"
    );
}

// Every audit help page (group + subcommand) carries Examples (after_help).
#[test]
fn audit_subcommands_have_examples() {
    for args in [vec!["audit"], vec!["audit", "list"]] {
        Command::cargo_bin("pipelite")
            .unwrap()
            .args(args)
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Examples:"));
    }
}

// The `custom-fields` help pages must be truthful (CFLD-01, Phase 12):
// the group help carries examples + the tombstone note and does NOT
// advertise a `--description` flag (no such server field exists anywhere
// on the model/serializer/schemas); create help advertises --key/--options
// and the max+10000 auto-position; update help advertises --position and
// the nothing-to-update refusal but NOT the immutable --entity-type/--type
// flags; delete help advertises --force + the tombstone note; list help
// documents the entity filter with the 4 server tokens.
#[test]
fn custom_fields_help_truthful() {
    let run_help = |args: &[&str]| {
        let output = Command::cargo_bin("pipelite")
            .unwrap()
            .args(args)
            .arg("--help")
            .assert()
            .success()
            .get_output()
            .clone();
        String::from_utf8_lossy(&output.stdout).to_string()
    };

    let group = run_help(&["custom-fields"]);
    assert!(group.contains("Examples:"), "group help: {group}");
    assert!(
        group.contains("does not mark them"),
        "group help must carry the tombstone note: {group}"
    );
    assert!(
        !group.contains("--description"),
        "group help must not advertise a --description flag:\n{group}"
    );

    let create = run_help(&["custom-fields", "create"]);
    for flag in ["--key", "--options", "--entity-type", "--stdin"] {
        assert!(create.contains(flag), "create help must document {flag}:\n{create}");
    }
    assert!(
        create.contains("max+10000"),
        "create help must state the auto-assigned position:\n{create}"
    );

    let update = run_help(&["custom-fields", "update"]);
    for flag in ["--position", "--config"] {
        assert!(update.contains(flag), "update help must document {flag}:\n{update}");
    }
    assert!(
        update.contains("nothing to update"),
        "update help must state the nothing-to-update refusal:\n{update}"
    );
    assert!(
        !update.contains("--entity-type") && !update.contains("--type"),
        "update help must NOT advertise the immutable --entity-type/--type flags:\n{update}"
    );

    let delete = run_help(&["custom-fields", "delete"]);
    assert!(delete.contains("--force"), "delete help must document --force:\n{delete}");
    assert!(
        delete.contains("does not mark them"),
        "delete help must carry the tombstone note:\n{delete}"
    );

    let list = run_help(&["custom-fields", "list"]);
    assert!(
        list.contains("--entity-type"),
        "list help must document the entity filter:\n{list}"
    );
    for token in ["deal", "organization", "person", "activity"] {
        assert!(
            list.contains(token),
            "list help must document the server token '{token}':\n{list}"
        );
    }
}

// Every custom-fields subcommand help page carries Examples (after_help).
#[test]
fn custom_fields_subcommands_have_examples() {
    for args in [
        ["custom-fields", "list"],
        ["custom-fields", "get"],
        ["custom-fields", "create"],
        ["custom-fields", "update"],
        ["custom-fields", "delete"],
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
