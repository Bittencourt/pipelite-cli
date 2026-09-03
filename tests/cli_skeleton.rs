use assert_cmd::Command;
use predicates::prelude::*;

/// Helper: pipelite command with hermetic env (WR-03). Without these
/// overrides, a machine with a real ~/.pipelite/config.toml made this suite
/// issue real API calls (`ping`) or MODIFY the real config (`config set`).
/// The nonexistent config path + unreachable endpoint make failures
/// deterministic (runtime error = exit 1; clap misuse would be 2) without
/// touching the network or any live config.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_CONFIG", "/tmp/pipelite-test-no-such-config.toml");
    c.env("PIPELITE_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_SERVER_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

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
    // config set should parse key and value as positional args.
    // The command itself fails at runtime (no config at the hermetic path),
    // but flags are accepted by clap.
    cmd()
        .args(["config", "set", "output.format", "json"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn global_flags_parse_without_error() {
    // Global flags should parse without clap rejecting them.
    // The command itself fails at runtime (unreachable server), but the
    // flags are accepted by clap (clap misuse would exit 2).
    cmd()
        .args(["--format", "json", "--no-color", "-q", "-v", "ping"])
        .assert()
        .failure()
        .code(1); // Runtime error, not clap misuse (code 2)
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
