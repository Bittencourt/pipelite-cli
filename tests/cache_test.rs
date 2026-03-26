use assert_cmd::Command;
use predicates::prelude::*;

/// Helper: build a pipelite command with fake config env vars.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", "http://localhost:1");
    c.env("PIPELITE_API_KEY", "test_key_fake");
    c
}

#[test]
fn cache_help_shows_subcommands() {
    cmd()
        .arg("cache")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("clear"))
        .stdout(predicate::str::contains("refresh"));
}

#[test]
fn cache_clear_runs_without_error() {
    // Cache clear is local-only -- does not need a real server
    cmd()
        .arg("cache")
        .arg("clear")
        .assert()
        .success()
        .stderr(predicate::str::contains("Cache cleared."));
}

#[test]
fn cache_clear_no_existing_dir_succeeds() {
    // Even with a temp home that has no .pipelite/cache/, clear should succeed
    // because CacheStore::new() creates the directory
    let dir = tempfile::TempDir::new().unwrap();
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("HOME", dir.path().to_str().unwrap());
    c.env("PIPELITE_API_KEY", "test_key_fake");
    c.env("PIPELITE_URL", "http://localhost:1");
    c.arg("cache").arg("clear").assert().success();
}

#[test]
fn cache_alias_c_works() {
    cmd()
        .arg("c")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("clear"))
        .stdout(predicate::str::contains("refresh"));
}
