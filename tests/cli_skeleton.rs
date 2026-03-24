use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_shows_rich_info() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pipelite 0.1.0"))
        .stdout(predicate::str::contains("rustc"))
        .stdout(predicate::str::contains("linux-x86_64"));
}

#[test]
fn help_shows_description_and_subcommands() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage your Pipelite CRM from the terminal",
        ))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("ping"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("Get started:"));
}

#[test]
fn init_help_shows_url_and_key_flags() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--url"))
        .stdout(predicate::str::contains("--key"))
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
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("get"));
}

#[test]
fn config_set_parses_positional_args() {
    // config set should parse key and value as positional args
    // The handler uses todo!() so it will panic, but clap parsing should succeed
    // We verify clap doesn't reject the args by checking for the todo panic message
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["config", "set", "output.format", "json"])
        .assert()
        .failure() // todo!() causes panic -> non-zero exit
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn global_flags_parse_without_error() {
    // Global flags should parse; command will todo!() but flags are accepted
    Command::cargo_bin("pipelite")
        .unwrap()
        .args(["--format", "json", "--no-color", "-q", "-v", "ping"])
        .assert()
        .failure() // todo!() in ping
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn invalid_subcommand_returns_exit_code_2() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("nonexistent")
        .assert()
        .failure()
        .code(2);
}
