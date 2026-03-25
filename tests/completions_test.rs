use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("pipelite").unwrap()
}

#[test]
fn bash_completions_generated() {
    // Completions do not require config/auth
    cmd()
        .arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("pipelite"))
        .stderr(predicate::str::contains("bashrc"));
}

#[test]
fn zsh_completions_generated() {
    cmd()
        .arg("completions")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not())
        .stderr(predicate::str::contains("zshrc"));
}

#[test]
fn fish_completions_generated() {
    cmd()
        .arg("completions")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not())
        .stderr(predicate::str::contains("fish"));
}

#[test]
fn invalid_shell_rejected() {
    cmd()
        .arg("completions")
        .arg("invalid")
        .assert()
        .failure();
}
