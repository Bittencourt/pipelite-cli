use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_contains_pipelite_and_rustc() {
    Command::cargo_bin("pipelite")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pipelite"))
        .stdout(predicate::str::contains("rustc"));
}
