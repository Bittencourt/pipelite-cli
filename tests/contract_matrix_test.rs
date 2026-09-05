//! Phase 13 contract matrix — every Phase 7-12 surface × the v1.0 global
//! contract (ROADMAP SC-1/SC-2/SC-4).
//!
//! Table-driven: ONE #[test] per Case row, each delegating to a shared axis
//! runner. Loop-inside-one-fn designs fail the per-row count gates by intent:
//! all red rows must report individually.
//!
//! Axes:
//!   dry_run_zero_http                 --dry-run previews mutations with ZERO HTTP
//!   dry_run_typed_writing_cold_cache  Phase 12 special case (cache-only fallback)
//!   no_input_refusals                 --no-input/non-TTY locked refusal codes
//!   quiet_suppression                 --quiet drops chatter, keeps DATA (Task 2)
//!   no_color_zero_ansi                --no-color emits zero ANSI bytes (Task 2)
//!   csv_and_plain                     format coverage per list-bearing group (Task 2)
//!   v1.0 smoke rows                   original surfaces still behave (Task 3)
//!
//! Fix-and-pin protocol (13-CONTEXT, BINDING): a failing row whose grammar was
//! --help-verified is a candidate CONTRACT VIOLATION — fix the surface's src,
//! add a pinning test to the surface's own test file, log the deviation in
//! 13-01-SUMMARY.md. Never relax an expectation to make a row pass.
//!
//! Zero-HTTP proof pattern (batch_error_test.rs precedent): dry-run and
//! refusal rows run against the unreachable http://127.0.0.1:1 server, where
//! ANY connection attempt would surface "Connection failed" in stderr — its
//! absence proves no HTTP was attempted.
//!
//! Uses tests/common/mod.rs (shared Phase 9 helpers, no new crates).

mod common;

use assert_cmd::Command;
use std::sync::atomic::Ordering;

/// The unreachable-server proof: any HTTP attempt surfaces this in stderr.
const CONNECTION_FAILED: &str = "Connection failed";

// ------------------------------------------------------------------ table --

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    /// Side-effectful operation: --dry-run must preview with ZERO HTTP.
    Mutation,
    /// Side-effect-free read: EXEMPT from the dry-run axis. Reads are
    /// side-effect-free and the v1.0 suites never refused reads under
    /// --dry-run — --dry-run is a MUTATION-preview contract (v1.0 precedent).
    /// Read rows still ride the presentation axes (quiet/no-color/csv/plain).
    ReadOnly,
    /// Destructive: refuses under --no-input/non-TTY without --force.
    Destructive,
}

struct Case {
    surface: &'static str,
    name: &'static str,
    kind: Kind,
    /// Full CLI args INCLUDING the axis flag (--dry-run / --no-input) —
    /// every row's grammar was --help-verified at authoring time (Task 1).
    args: &'static [&'static str],
    stdin: Option<&'static str>,
    /// Substrings the row's output must contain: the dry-run preview
    /// (stdout) for Mutation rows, the refusal stderr for Destructive rows.
    expect: &'static [&'static str],
    /// Locked exit code: 0 for dry-run previews; 1 Validation / 2
    /// InvalidInput+MissingInput for refusals (the locked exit-code contract).
    code: i32,
}

const fn case(
    surface: &'static str,
    name: &'static str,
    kind: Kind,
    args: &'static [&'static str],
    stdin: Option<&'static str>,
    expect: &'static [&'static str],
    code: i32,
) -> Case {
    Case {
        surface,
        name,
        kind,
        args,
        stdin,
        expect,
        code,
    }
}

/// `--dry-run` mutation rows: unreachable server, exit 0, zero HTTP, the
/// preview names the method + endpoint (render_dry_run output shape: the
/// "METHOD URL" header in text modes, the method/url JSON fields in json).
///
/// Grammar note (templates_create): `--trigger` resolves the source LOCALLY
/// (no fetch), so the row is honestly zero-HTTP. The `--workflow` source
/// GET is a READ needed to build the preview — exempt from the zero-mutation
/// contract and pinned separately in templates_stub_test.rs
/// (create_dry_run_previews_the_mapped_post_without_template_post: exactly 1
/// request, the GET; the POST is previewed).
const MUTATION_ROWS: &[Case] = &[
    case(
        "batch",
        "batch_deals_update_stdin",
        Kind::Mutation,
        &["deals", "update", "--stdin", "--dry-run"],
        Some(r#"[{"id":"deal_1","title":"New Title"}]"#),
        &["PUT", "/api/v1/deals/deal_1"],
        0,
    ),
    case(
        "batch",
        "batch_deals_delete_stdin",
        Kind::Mutation,
        &["deals", "delete", "--stdin", "--dry-run"],
        Some(r#"["deal_1","deal_2"]"#),
        &["deal_1", "deal_2"],
        0,
    ),
    case(
        "templates",
        "templates_create",
        Kind::Mutation,
        &[
            "templates",
            "create",
            "--name",
            "T",
            "--trigger",
            r#"{"type":"schedule"}"#,
            "--dry-run",
        ],
        None,
        &["POST", "/api/v1/workflow-templates"],
        0,
    ),
    case(
        "templates",
        "templates_delete",
        Kind::Mutation,
        &["templates", "delete", "t1", "--dry-run"],
        None,
        &["t1"],
        0,
    ),
    case(
        "notes",
        "notes_add",
        Kind::Mutation,
        &["notes", "add", "deals", "d1", "--body", "x", "--dry-run"],
        None,
        &["POST", "/api/v1/deals/d1/notes"],
        0,
    ),
    case(
        "notes",
        "notes_edit",
        Kind::Mutation,
        &["notes", "edit", "deals", "d1", "n1", "--body", "y", "--dry-run"],
        None,
        &["PATCH", "/api/v1/notes/n1"],
        0,
    ),
    case(
        "notes",
        "notes_delete",
        Kind::Mutation,
        &["notes", "delete", "deals", "d1", "n1", "--dry-run"],
        None,
        &["n1"],
        0,
    ),
    case(
        "webhooks",
        "webhooks_create",
        Kind::Mutation,
        &[
            "webhooks",
            "create",
            "--url",
            "https://example.com/hook",
            "--events",
            "deal.created",
            "--dry-run",
        ],
        None,
        &["POST", "/api/v1/webhooks"],
        0,
    ),
    case(
        "webhooks",
        "webhooks_update",
        Kind::Mutation,
        &["webhooks", "update", "wh1", "--active", "--dry-run"],
        None,
        &["PUT", "/api/v1/webhooks/wh1"],
        0,
    ),
    case(
        "trash",
        "trash_restore",
        Kind::Mutation,
        &["trash", "restore", "deal", "d1", "--dry-run"],
        None,
        &["POST", "d1"],
        0,
    ),
    case(
        "custom-fields",
        "custom_fields_create",
        Kind::Mutation,
        &[
            "custom-fields",
            "create",
            "--entity-type",
            "deals",
            "--key",
            "price",
            "--type",
            "number",
            "--dry-run",
        ],
        None,
        &["POST", "/api/v1/custom-field-definitions"],
        0,
    ),
    case(
        "custom-fields",
        "custom_fields_update",
        Kind::Mutation,
        &["custom-fields", "update", "cf1", "--name", "price2", "--dry-run"],
        None,
        &["PUT", "cf1"],
        0,
    ),
    case(
        "custom-fields",
        "custom_fields_delete",
        Kind::Mutation,
        &["custom-fields", "delete", "cf1", "--dry-run"],
        None,
        &["cf1"],
        0,
    ),
];

/// Read-only rows: exempt from the dry-run axis (see Kind::ReadOnly), but
/// members of the table so the presentation axes (Task 2) and the audit
/// trail can account for every surface×operation pairing.
const READ_ONLY_ROWS: &[Case] = &[
    case(
        "runs",
        "runs_list",
        Kind::ReadOnly,
        &["workflows", "runs", "list", "--workflow", "wf1"],
        None,
        &[],
        0,
    ),
    case(
        "runs",
        "runs_get",
        Kind::ReadOnly,
        &["workflows", "runs", "get", "run1", "--workflow", "wf1"],
        None,
        &[],
        0,
    ),
    case(
        "templates",
        "templates_list",
        Kind::ReadOnly,
        &["templates", "list"],
        None,
        &[],
        0,
    ),
    case(
        "notes",
        "notes_list",
        Kind::ReadOnly,
        &["notes", "list", "deals", "d1"],
        None,
        &[],
        0,
    ),
    case(
        "webhooks",
        "webhooks_list",
        Kind::ReadOnly,
        &["webhooks", "list"],
        None,
        &[],
        0,
    ),
    case(
        "webhooks",
        "webhooks_get",
        Kind::ReadOnly,
        &["webhooks", "get", "wh1"],
        None,
        &[],
        0,
    ),
    case(
        "trash",
        "trash_list",
        Kind::ReadOnly,
        &["trash", "list"],
        None,
        &[],
        0,
    ),
    case(
        "audit",
        "audit_list",
        Kind::ReadOnly,
        &["audit", "list"],
        None,
        &[],
        0,
    ),
    case(
        "custom-fields",
        "custom_fields_list",
        Kind::ReadOnly,
        &["custom-fields", "list", "--entity-type", "deals"],
        None,
        &[],
        0,
    ),
    case("docs", "docs", Kind::ReadOnly, &["docs"], None, &[], 0),
];

/// `--no-input` refusal rows (piped stdin via assert_cmd = non-TTY): locked
/// exit codes (1 Validation for batch + standard deletes; 2 InvalidInput for
/// the trash-purge outlier and 2 MissingInput for suppressed prompts), the
/// hint names the bypass flag, and the refusal fires BEFORE any HTTP.
const REFUSAL_ROWS: &[Case] = &[
    case(
        "batch",
        "batch_delete_stdin_refusal",
        Kind::Destructive,
        &["deals", "delete", "--stdin", "--no-input"],
        Some(r#"["deal_1","deal_2"]"#),
        &["Re-run with --force"],
        1,
    ),
    case(
        "webhooks",
        "webhooks_delete_refusal",
        Kind::Destructive,
        &["webhooks", "delete", "wh1", "--no-input"],
        None,
        &["--force"],
        1,
    ),
    case(
        "templates",
        "templates_delete_refusal",
        Kind::Destructive,
        &["templates", "delete", "t1", "--no-input"],
        None,
        &["--force"],
        1,
    ),
    case(
        "notes",
        "notes_delete_refusal",
        Kind::Destructive,
        &["notes", "delete", "deals", "d1", "n1", "--no-input"],
        None,
        &["--force"],
        1,
    ),
    case(
        "custom-fields",
        "custom_fields_delete_refusal",
        Kind::Destructive,
        &["custom-fields", "delete", "cf1", "--no-input"],
        None,
        &["--force"],
        1,
    ),
    // The locked exit-2 outlier: STRICTER than the standard delete-1 —
    // InvalidInput BEFORE the purge fan-out (STATE decision, Phase 11).
    case(
        "trash",
        "trash_purge_refusal",
        Kind::Destructive,
        &["trash", "purge", "--type", "deals", "--no-input"],
        None,
        &["--force"],
        2,
    ),
    // Prompt suppressed under --no-input → MissingInput exit 2 pre-HTTP
    // (no hang: assert_cmd would deadlock-timeout if a prompt ever ran).
    case(
        "notes",
        "notes_add_no_body_refusal",
        Kind::Destructive,
        &["notes", "add", "deals", "d1", "--no-input"],
        None,
        &["--body"],
        2,
    ),
];

fn row(name: &str) -> &'static Case {
    MUTATION_ROWS
        .iter()
        .chain(READ_ONLY_ROWS.iter())
        .chain(REFUSAL_ROWS.iter())
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("unknown matrix row: {name}"))
}

/// Build the pipelite command for a row against the unreachable server.
fn unreachable_cmd(c: &Case) -> Command {
    let mut cmd = common::cmd();
    cmd.args(c.args);
    if let Some(input) = c.stdin {
        cmd.write_stdin(input);
    }
    cmd
}

// ------------------------------------------------- axis: dry_run_zero_http --

/// Shared runner for every Mutation row: --dry-run previews with ZERO HTTP.
/// ReadOnly rows never reach this runner — reads are side-effect-free and
/// the v1.0 suites never refused reads under --dry-run (--dry-run is a
/// mutation-preview contract; v1.0 precedent). The guard below documents
/// that exemption at the type level instead of silently mis-running reads.
fn dry_run_zero_http(c: &'static Case) {
    assert_eq!(
        c.kind,
        Kind::Mutation,
        "row {} is not a mutation — reads are exempt from the dry-run axis (v1.0 precedent)",
        c.name
    );

    let output = unreachable_cmd(c)
        .assert()
        .code(c.code)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Zero-HTTP proof: on the unreachable server, any connection attempt
    // would print "Connection failed" — its absence proves the preview was
    // rendered entirely pre-HTTP.
    assert!(
        !stderr.contains(CONNECTION_FAILED),
        "--dry-run must make ZERO HTTP requests (no connection attempt):\n{stderr}"
    );
    for fragment in c.expect {
        assert!(
            stdout.contains(fragment),
            "dry-run preview must contain {fragment:?} (method + endpoint URL preview):\n{stdout}"
        );
    }
}

#[test]
fn dry_run_batch_deals_update_stdin() {
    dry_run_zero_http(row("batch_deals_update_stdin"));
}

#[test]
fn dry_run_batch_deals_delete_stdin() {
    dry_run_zero_http(row("batch_deals_delete_stdin"));
}

#[test]
fn dry_run_templates_create() {
    dry_run_zero_http(row("templates_create"));
}

#[test]
fn dry_run_templates_delete() {
    dry_run_zero_http(row("templates_delete"));
}

#[test]
fn dry_run_notes_add() {
    dry_run_zero_http(row("notes_add"));
}

#[test]
fn dry_run_notes_edit() {
    dry_run_zero_http(row("notes_edit"));
}

#[test]
fn dry_run_notes_delete() {
    dry_run_zero_http(row("notes_delete"));
}

#[test]
fn dry_run_webhooks_create() {
    dry_run_zero_http(row("webhooks_create"));
}

#[test]
fn dry_run_webhooks_update() {
    dry_run_zero_http(row("webhooks_update"));
}

#[test]
fn dry_run_trash_restore() {
    dry_run_zero_http(row("trash_restore"));
}

#[test]
fn dry_run_custom_fields_create() {
    dry_run_zero_http(row("custom_fields_create"));
}

#[test]
fn dry_run_custom_fields_update() {
    dry_run_zero_http(row("custom_fields_update"));
}

#[test]
fn dry_run_custom_fields_delete() {
    dry_run_zero_http(row("custom_fields_delete"));
}

// ------------------------------- axis: dry_run_typed_writing_cold_cache ----

/// Phase 12 SC-4 verbatim: `deals create --custom-field ... --dry-run` with a
/// COLD cache NEVER fetches (not even the definitions GET — counter-style
/// proof via the unreachable server), falls back to raw strings, and says so
/// on stderr with the warm-the-cache note.
#[test]
fn dry_run_typed_writing_cold_cache_zero_http_note_visible() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir (cold cache)");

    let output = common::cmd()
        .env("HOME", tmp.path())
        .args([
            "deals",
            "create",
            "--title",
            "T",
            "--stage",
            "stg1",
            "--custom-field",
            "price=4",
            "--dry-run",
        ])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        !stderr.contains(CONNECTION_FAILED),
        "cold-cache dry-run must be cache-only (zero HTTP, 12-02 SC-4):\n{stderr}"
    );
    assert!(
        stdout.contains("POST") && stdout.contains("/api/v1/deals"),
        "preview must name the method + endpoint:\n{stdout}"
    );
    assert!(
        stderr.contains("not cached"),
        "the dry-run cache-miss note must be visible on stderr (cold cache):\n{stderr}"
    );
}

/// Same invocation with --quiet: the cache-miss note is chatter — suppressed.
#[test]
fn dry_run_typed_writing_cold_cache_quiet_suppresses_note() {
    let tmp = tempfile::TempDir::new().expect("temp HOME dir (cold cache)");

    let output = common::cmd()
        .env("HOME", tmp.path())
        .args([
            "deals",
            "create",
            "--title",
            "T",
            "--stage",
            "stg1",
            "--custom-field",
            "price=4",
            "--dry-run",
            "--quiet",
        ])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        !stderr.contains(CONNECTION_FAILED),
        "still zero HTTP under --quiet"
    );
    assert!(
        !stderr.contains("not cached"),
        "--quiet must suppress the dry-run cache-miss note:\n{stderr}"
    );
}

// ---------------------------------------------------- axis: no_input_refusals

/// Shared runner for every Destructive row: the locked refusal (exit code +
/// flag-naming hint) must fire BEFORE any HTTP. Piped stdin (assert_cmd
/// default) is a non-TTY, so the refusal path is exercised without --force.
fn no_input_refusals(c: &'static Case) {
    assert_eq!(c.kind, Kind::Destructive, "row {} is not destructive", c.name);

    let output = unreachable_cmd(c)
        .assert()
        .code(c.code)
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    for fragment in c.expect {
        assert!(
            stderr.contains(fragment),
            "refusal hint must name the bypass flag/contract {fragment:?} (locked exit {}):\n{stderr}",
            c.code
        );
    }
    assert!(
        !stderr.contains(CONNECTION_FAILED),
        "refusal must fire BEFORE any HTTP (unreachable-server proof):\n{stderr}"
    );
}

#[test]
fn no_input_batch_delete_stdin_refuses_exit_1() {
    no_input_refusals(row("batch_delete_stdin_refusal"));
}

#[test]
fn no_input_webhooks_delete_refuses_exit_1() {
    no_input_refusals(row("webhooks_delete_refusal"));
}

#[test]
fn no_input_templates_delete_refuses_exit_1() {
    no_input_refusals(row("templates_delete_refusal"));
}

#[test]
fn no_input_notes_delete_refuses_exit_1() {
    no_input_refusals(row("notes_delete_refusal"));
}

#[test]
fn no_input_custom_fields_delete_refuses_exit_1() {
    no_input_refusals(row("custom_fields_delete_refusal"));
}

#[test]
fn no_input_trash_purge_refuses_exit_2_stricter() {
    no_input_refusals(row("trash_purge_refusal"));
}

#[test]
fn no_input_notes_add_without_body_missing_input_exit_2() {
    no_input_refusals(row("notes_add_no_body_refusal"));
}

/// Control row (anti-refusal): `batch update --stdin` PROCEEDS — stdin IS the
/// input, there is nothing to refuse. Against the unreachable server the run
/// must therefore REACH HTTP ("Connection failed" present) and NOT surface
/// the batch-delete-style confirmation refusal.
#[test]
fn no_input_batch_update_stdin_proceeds_no_refusal() {
    let output = common::cmd()
        .args(["deals", "update", "--stdin"])
        .write_stdin(r#"[{"id":"deal_1","title":"New Title"}]"#)
        .assert()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        !stderr.contains("Refusing"),
        "batch update must never hit a confirmation refusal (stdin IS the input):\n{stderr}"
    );
    assert!(
        stderr.contains(CONNECTION_FAILED),
        "batch update --stdin must PROCEED to HTTP (the failure here is the unreachable server, not a refusal):\n{stderr}"
    );
}

// -- request-count proof for the unreachable server (belt-and-braces on top
//    of the "Connection failed" string assertions): the stub-backed variant
//    pins counter == 0 for one representative dry-run row per mechanism. --

/// Stub-backed counter proof for the dry-run zero-HTTP axis: the stub is
/// scripted but must never be contacted (counter == 0) — the same invariant
/// the unreachable-server rows prove by string absence.
#[test]
fn dry_run_stub_counter_stays_zero_webhooks_create() {
    let body = serde_json::json!({
        "data": {
            "id": "wh1",
            "url": "https://example.com/hook",
            "events": ["deal.created"],
            "active": true,
            "created_at": "2026-09-01T10:00:00.000Z",
            "updated_at": "2026-09-01T10:00:00.000Z",
            "secret": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2"
        }
    })
    .to_string();
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &body)]);

    common::cmd_with_server(&url)
        .args([
            "webhooks",
            "create",
            "--url",
            "https://example.com/hook",
            "--events",
            "deal.created",
            "--dry-run",
        ])
        .assert()
        .code(0);

    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "dry-run must not contact even a LIVE server"
    );
}
