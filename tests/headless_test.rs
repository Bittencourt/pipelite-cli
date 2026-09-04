use assert_cmd::Command;
use predicates::prelude::*;

/// Helper: build a pipelite command with fake config env vars so AppContext::build() succeeds.
/// All tests pipe stdin (via write_stdin) making stdin non-TTY, which auto-implies --no-input.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", "http://localhost:9999");
    c.env("PIPELITE_API_KEY", "test_key_fake");
    c
}

// ─── Deals (MissingInput / exit code 2 / batch error reporting) ──────────────

#[test]
fn deals_create_missing_required_fields_lists_all() {
    // HEAD-02: piped stdin (non-TTY) with no flags -> exit 2, lists --title AND --stage
    cmd()
        .write_stdin("")
        .arg("deals")
        .arg("create")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--title"))
        .stderr(predicate::str::contains("--stage"));
}

#[test]
fn deals_create_partial_flags_lists_remaining() {
    // Provide --title but not --stage -> exit 2
    // The "Missing required flags:" line should only contain --stage
    cmd()
        .write_stdin("")
        .arg("deals")
        .arg("create")
        .arg("--title")
        .arg("Test Deal")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Missing required flags: --stage"));
}

#[test]
fn no_input_flag_prevents_prompts() {
    // --no-input flag is accepted; since assert_cmd pipes stdin anyway,
    // behavior is the same as non-TTY: exit 2, lists missing fields
    cmd()
        .write_stdin("")
        .arg("--no-input")
        .arg("deals")
        .arg("create")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--title"))
        .stderr(predicate::str::contains("--stage"));
}

// ─── Orgs (MissingInput / exit code 2 / batch reporting) ────────────────────

#[test]
fn orgs_create_missing_name() {
    cmd()
        .write_stdin("")
        .arg("orgs")
        .arg("create")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--name"));
}

// ─── People (MissingInput / exit code 2 / batch reporting) ──────────────────

#[test]
fn people_create_missing_both_names() {
    // People create lists ALL missing required fields at once
    cmd()
        .write_stdin("")
        .arg("people")
        .arg("create")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--first-name"))
        .stderr(predicate::str::contains("--last-name"));
}

#[test]
fn people_create_with_first_name_lists_remaining() {
    // With --first-name provided, only --last-name should be missing
    cmd()
        .write_stdin("")
        .arg("people")
        .arg("create")
        .arg("--first-name")
        .arg("John")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Missing required flags: --last-name"));
}

// ─── Activities (MissingInput / exit code 2 / batch reporting) ──────────────

#[test]
fn activities_create_missing_required() {
    // Activities create lists --title AND --type as missing
    cmd()
        .write_stdin("")
        .arg("activities")
        .arg("create")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--title"))
        .stderr(predicate::str::contains("--type"));
}

// ─── Stages (MissingInput / exit code 2 / batch reporting) ──────────────────

#[test]
fn stages_create_missing_required() {
    // Stages create lists ALL missing: --name and --pipeline
    cmd()
        .write_stdin("")
        .arg("stages")
        .arg("create")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--name"))
        .stderr(predicate::str::contains("--pipeline"));
}

// ─── Pipelines (MissingInput / exit code 2 / batch reporting) ───────────────

#[test]
fn pipelines_create_missing_name() {
    cmd()
        .write_stdin("")
        .arg("pipelines")
        .arg("create")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--name"));
}

// ─── --stdin mutual exclusivity ─────────────────────────────────────────────

#[test]
fn stdin_and_flags_mutually_exclusive() {
    // --stdin + --title fail with the mutual exclusivity error at exit 2
    // (InvalidInput — normalized from the old Validation exit 1 so all 8
    // create/update handlers share one structural exit code).
    cmd()
        .write_stdin(r#"[{"title":"A","stage_id":"stg_001"}]"#)
        .arg("deals")
        .arg("create")
        .arg("--stdin")
        .arg("--title")
        .arg("Test")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("mutually exclusive"));
}
