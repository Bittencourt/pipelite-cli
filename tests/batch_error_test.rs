use assert_cmd::Command;
use predicates::prelude::*;

/// Helper: build a pipelite command with fake config env vars.
/// Per CLAUDE.md: tests use 127.0.0.1:1 + fake-test-key.
///
/// Note: `PIPELITE_SERVER_URL` is the env var the app actually reads
/// (see src/config.rs env override); `PIPELITE_URL` is kept for
/// consistency with the rest of the test suite's cmd() helpers.
/// Setting it matters here: these tests make real connection attempts
/// against the unreachable 127.0.0.1:1 endpoint to exercise error paths
/// — without the override the default production URL would be used.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("pipelite").unwrap();
    c.env("PIPELITE_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_SERVER_URL", "http://127.0.0.1:1");
    c.env("PIPELITE_API_KEY", "fake-test-key");
    c
}

// -- Batch update against unreachable server -> all fail, non-zero exit (D-06, D-07) --

#[test]
fn batch_update_unreachable_server_exits_nonzero() {
    // All items are structurally valid; failures are per-item (unreachable
    // server), so the batch exits 1 — never 2 (exit-2 is structural only).
    let input = r#"[{"id":"deal_1","title":"New"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("failed").or(predicate::str::contains("Failed")));
}

// -- Batch update silent no-ops must fail (WR-03) --

#[test]
fn batch_update_empty_array_fails() {
    // An empty array runs zero operations and must not exit 0 silently.
    // Structural input failure -> exit 2 (BATCH-04 exit-code matrix).
    cmd()
        .write_stdin("[]")
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Empty update list"));
}

#[test]
fn batch_update_invalid_json_exits_2() {
    // Malformed JSON is a structural input failure -> exit 2.
    cmd()
        .write_stdin("not json")
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Invalid JSON input"));
}

#[test]
fn batch_delete_empty_list_exits_2() {
    // Empty ID array is a structural input failure -> exit 2.
    cmd()
        .write_stdin("[]")
        .args(["deals", "delete", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Empty ID list"));
}

#[test]
fn batch_delete_stdin_with_positional_ids_exits_2() {
    // --stdin + positional IDs conflict is a structural input failure -> exit 2.
    cmd()
        .write_stdin(r#"["deal_1"]"#)
        .args(["deals", "delete", "deal_1", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("mutually exclusive"));
}

#[test]
fn batch_update_stdin_with_field_flag_exits_2() {
    // --stdin + individual field flags conflict is structural -> exit 2
    // (deals stand in for all 7 entities, which share the identical path).
    cmd()
        .write_stdin(r#"[{"id":"deal_1","title":"X"}]"#)
        .args(["deals", "update", "--stdin", "--title", "X"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("mutually exclusive"));
}

#[test]
fn batch_update_noop_item_fails() {
    // "titel" is not a known field; the Update model ignores it, so the item
    // would PUT an empty {} body and report success. WR-03: report failure.
    let input = r#"[{"id":"deal_1","titel":"New"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no recognizable update fields"));
}

// -- Batch delete against unreachable server -> all fail, non-zero exit (D-06, D-07) --

#[test]
fn batch_delete_unreachable_server_exits_nonzero() {
    // write_stdin("") creates a piped (non-TTY) stdin, which makes this a
    // non-interactive run: CR-01 requires --force for batch deletes when the
    // prompt cannot be shown. The empty content is not read because positional
    // IDs are provided (--stdin is not set). Both deletes fail against the
    // unreachable server.
    cmd()
        .write_stdin("")
        .args(["deals", "delete", "deal_1", "deal_2", "--force"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("failed").or(predicate::str::contains("Failed")));
}

// -- Unit test for BatchOutcome (via module test, not integration) --
// BatchOutcome finalize with failures returns error
// This is tested indirectly via the above integration tests.
// The error summary format is: "N/M entity operationd, K failed"

#[test]
fn batch_update_missing_id_in_item_shows_error() {
    // An item without a string "id" is a STRUCTURAL input failure: the whole
    // batch must be rejected with exit 2 before any HTTP call fires.
    let input = r#"[{"title":"No ID Here"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("missing a string 'id' field")
                .and(predicate::str::contains(": 1")),
        );
}

#[test]
fn batch_update_mixed_batch_missing_id_zero_http() {
    // THE core regression for verification gap 1: a valid item followed by an
    // id-less item must NOT mutate the valid item. The unreachable
    // 127.0.0.1:1 server guarantees any HTTP attempt prints "Connection
    // failed", so its absence from stderr proves zero requests fired.
    let input = r#"[{"id":"deal_1","title":"X"},{"title":"No ID Here"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("missing a string 'id' field")
                .and(predicate::str::contains(": 2"))
                .and(predicate::str::contains("Connection failed").not())
                .and(predicate::str::contains("[1/2] Failed").not()),
        );
}

#[test]
fn batch_update_non_string_id_exits_2() {
    // Non-string ids are treated as missing (matches the in-loop
    // as_str() extraction semantics) -> structural exit 2.
    let input = r#"[{"id":123,"title":"X"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--stdin"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("missing a string 'id' field"));
}

#[test]
fn batch_update_dry_run_missing_id_exits_2() {
    // The pre-scan runs BEFORE the dry-run block: --dry-run is how users
    // validate input, so structurally broken input is rejected there too —
    // no per-item dry-run output on stdout.
    let input = r#"[{"id":"deal_1","title":"X"},{"title":"No ID Here"}]"#;
    cmd()
        .write_stdin(input)
        .args(["deals", "update", "--dry-run", "--stdin"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("missing a string 'id' field"));
}
