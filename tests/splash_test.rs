use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn no_args_non_tty_shows_clap_help_not_splash() {
    // assert_cmd pipes stdout, so it's non-TTY -- splash should NOT appear
    Command::cargo_bin("pipelite")
        .unwrap()
        .assert()
        .failure() // clap requires a subcommand
        .stderr(
            predicate::str::contains("Usage")
                .or(predicate::str::contains("Manage your Pipelite CRM")),
        );
}

#[test]
fn help_flag_shows_normal_help() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage your Pipelite CRM"))
        .stdout(predicate::str::contains("init"));
}
