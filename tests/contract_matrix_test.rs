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

// ===================================================== Task 2: presentation
// axes — --quiet suppression, --no-color zero-ANSI, csv/plain per group.
// Stub fixtures copy the known-good row shapes from each surface's own stub
// test file (tests/{webhooks,trash,notes,templates,audit,custom_fields,
// workflow_runs}_stub_test.rs) rather than inventing new shapes.

use common::cmd_with_server;
use predicates::prelude::*;

/// Verified empty-page envelope.
const EMPTY_PAGE: &str = r#"{"data":[],"meta":{"total":0,"offset":0,"limit":50}}"#;

// ---------------------------------------------------- axis: quiet_suppression

/// Shared runner for the two-variant empty-page chatter rows: the SAME
/// list command against the SAME empty-page stub must print the
/// informational hint on stderr WITHOUT --quiet, and stay silent WITH it
/// (exit 0 both ways — the empty page itself is not an error).
fn quiet_empty_page_hint(args: &[&str], hint_fragment: &str) {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, EMPTY_PAGE)]);

    cmd_with_server(&url)
        .args(args)
        .assert()
        .code(0)
        .stderr(predicate::str::contains(hint_fragment));

    let (url_quiet, _c2, _h2, _b2) =
        common::spawn_head_capturing_stub_server(&[(200, EMPTY_PAGE)]);

    let output = cmd_with_server(&url_quiet)
        .args(args)
        .args(["--quiet"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        !stderr.contains(hint_fragment),
        "--quiet must suppress the informational empty-hint {hint_fragment:?}:\n{stderr}"
    );
}

// Per-group empty-page chatter (grammar + wording verified in each surface's
// handler). runs list uses --status so the group's empty-hint fires without
// triggering the hidden-test-runs probe (which is itself pinned in
// workflow_runs_stub_test.rs). templates list has NO empty-hint by design
// (its render is silent data-only) — the surface's quiet-suppressible
// chatter is the create multi-trigger warning, covered by
// quiet_templates_create_multi_trigger_warning below.

#[test]
fn quiet_runs_list_empty_hint_suppressed() {
    quiet_empty_page_hint(
        &["workflows", "runs", "list", "--workflow", "wf1", "--status", "failed"],
        "No runs match status",
    );
}

#[test]
fn quiet_notes_list_empty_hint_suppressed() {
    quiet_empty_page_hint(&["notes", "list", "deals", "d1"], "No notes on deals d1");
}

#[test]
fn quiet_webhooks_list_empty_hint_suppressed() {
    quiet_empty_page_hint(&["webhooks", "list"], "No webhooks yet");
}

#[test]
fn quiet_trash_list_empty_hint_suppressed() {
    quiet_empty_page_hint(&["trash", "list"], "No trashed records");
}

#[test]
fn quiet_audit_list_empty_hint_suppressed() {
    quiet_empty_page_hint(&["audit", "list"], "No audit entries");
}

#[test]
fn quiet_custom_fields_list_empty_hint_suppressed() {
    quiet_empty_page_hint(
        &["custom-fields", "list", "--entity-type", "deals"],
        "No custom field definitions yet",
    );
}

/// templates chatter row: the create multi-trigger warning is informational
/// stderr — present without --quiet, suppressed with it (the surface has no
/// list empty-hint; this is its only quiet-suppressible chatter).
#[test]
fn quiet_templates_create_multi_trigger_warning_suppressed() {
    let get_body = serde_json::json!({
        "data": {
            "id": "wf_1",
            "name": "Deal Alert",
            "description": null,
            "triggers": [
                {"type": "crm_event", "entity": "deal"},
                {"type": "schedule"}
            ],
            "nodes": [{"id": "n1"}],
            "active": false,
            "created_by": "user_1",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
    })
    .to_string();
    let created = serde_json::json!({
        "data": {
            "id": "tpl_new",
            "name": "T",
            "description": null,
            "category": null,
            "trigger": {"type": "crm_event", "entity": "deal"},
            "nodes": [{"id": "n1"}],
            "created_at": "2026-01-01T00:00:00Z"
        }
    })
    .to_string();

    let (url, _counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &get_body),
        (201, &created),
    ]);

    cmd_with_server(&url)
        .args(["templates", "create", "--name", "T", "--workflow", "wf_1"])
        .assert()
        .code(0)
        .stderr(predicate::str::contains(
            "warning: workflow has 2 triggers",
        ));

    let (url_q, _c2, _h2, _b2) = common::spawn_head_capturing_stub_server(&[
        (200, &get_body),
        (201, &created),
    ]);

    let output = cmd_with_server(&url_q)
        .args(["templates", "create", "--name", "T", "--workflow", "wf_1", "--quiet"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        !stderr.contains("warning: workflow has 2 triggers"),
        "--quiet must suppress the multi-trigger warning:\n{stderr}"
    );
}

/// SC-2 anchor (data): the show-once webhook secret is DATA, not chatter —
/// `webhooks create` carries it on stdout in BOTH modes; only the
/// save-it-now warning is quiet-suppressible (stub-level anchor for the
/// 13-02 live E2E).
#[test]
fn quiet_webhooks_create_secret_present_without_quiet() {
    let secret = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2";
    let body = serde_json::json!({
        "data": {
            "id": "wh1",
            "url": "https://example.com/hook",
            "events": ["deal.created"],
            "active": true,
            "created_at": "2026-09-01T10:00:00.000Z",
            "updated_at": "2026-09-01T10:00:00.000Z",
            "secret": secret
        }
    })
    .to_string();
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &body)]);

    cmd_with_server(&url)
        .args([
            "webhooks",
            "create",
            "--url",
            "https://example.com/hook",
            "--events",
            "deal.created",
            "--format",
            "json",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(secret))
        .stderr(predicate::str::contains("save it now"));
}

#[test]
fn quiet_webhooks_create_secret_survives_quiet() {
    let secret = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2a3b4c5d6a7b8c9d0e1f2";
    let body = serde_json::json!({
        "data": {
            "id": "wh1",
            "url": "https://example.com/hook",
            "events": ["deal.created"],
            "active": true,
            "created_at": "2026-09-01T10:00:00.000Z",
            "updated_at": "2026-09-01T10:00:00.000Z",
            "secret": secret
        }
    })
    .to_string();
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(201, &body)]);

    let output = cmd_with_server(&url)
        .args([
            "webhooks",
            "create",
            "--url",
            "https://example.com/hook",
            "--events",
            "deal.created",
            "--format",
            "json",
            "--quiet",
        ])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        stdout.contains(secret),
        "the show-once secret is DATA — it must survive --quiet on stdout:\n{stdout}"
    );
    assert!(
        !stderr.contains("save it now"),
        "--quiet suppresses the save-it-now warning (the secret itself stays):\n{stderr}"
    );
}

/// custom-fields delete confirmation: chatter — present without --quiet,
/// suppressed with it (exit 0 both ways).
#[test]
fn quiet_custom_fields_delete_confirmation_present_without_quiet() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    cmd_with_server(&url)
        .args(["custom-fields", "delete", "cf1", "--force"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("Deleted custom field definition cf1"));
}

#[test]
fn quiet_custom_fields_delete_confirmation_suppressed() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    let output = cmd_with_server(&url)
        .args(["custom-fields", "delete", "cf1", "--force", "--quiet"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        !stdout.contains("Deleted custom field definition"),
        "--quiet must suppress the delete confirmation:\n{stdout}"
    );
}

/// Phase 7 SC-3 positive pin: the batch "N ok, M failed" summary is DATA,
/// not chatter — it must SURVIVE --quiet. The locked Phase 7 contract (see
/// 07-VERIFICATION.md re-probe) prints the summary on the failure path only
/// ("N/M entity deleted, M failed"), so the honest positive pin is a MIXED
/// batch under --quiet: exit 1, per-item failure line AND summary present.
#[test]
fn quiet_batch_delete_mixed_summary_survives_quiet() {
    let err_body = serde_json::json!({
        "type": "https://api.pipelite.app/errors/INTERNAL",
        "title": "Internal Server Error",
        "status": 500,
        "detail": "boom"
    })
    .to_string();
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, ""), (500, &err_body)]);

    let output = cmd_with_server(&url)
        .args(["deals", "delete", "id1", "id2", "--force", "--quiet"])
        .assert()
        .code(1)
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert_eq!(
        counter.load(Ordering::SeqCst),
        2,
        "both deletes must be attempted (continue-on-error)"
    );
    assert!(
        stderr.contains("[2/2] Failed"),
        "the per-item failure line is data and must survive --quiet:\n{stderr}"
    );
    assert!(
        stderr.contains("1/2") && stderr.contains("failed"),
        "the 'N ok, M failed'-shaped batch summary must survive --quiet (Phase 7 SC-3):\n{stderr}"
    );
}

/// Exit-code companion: all-ok batch under --quiet exits 0 (exit 0 only when
/// EVERY item succeeded — the all-ok path prints no summary by locked
/// Phase 7 design; the point pinned here is the exit code under --quiet).
#[test]
fn quiet_batch_delete_all_ok_exits_zero() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, ""), (204, "")]);

    cmd_with_server(&url)
        .args(["deals", "delete", "id1", "id2", "--force", "--quiet"])
        .assert()
        .code(0);

    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

// --------------------------------------------------- axis: no_color_zero_ansi

/// Scan helper: the escaped ANSI control prefix must not appear in `bytes`.
fn assert_no_ansi(bytes: &[u8], stream: &str, context: &str) {
    let text = String::from_utf8_lossy(bytes).to_string();
    assert!(
        !text.contains("\x1b["),
        "--no-color: zero ANSI escape bytes expected on {stream} ({context}):\n{text}"
    );
}

/// (a) Success-path table render with --no-color: zero ANSI on stdout.
#[test]
fn no_color_webhooks_list_table_zero_ansi() {
    let body = serde_json::json!({
        "data": [{
            "id": "wh1",
            "url": "https://example.com/hook",
            "events": ["deal.created"],
            "active": true,
            "created_at": "2026-09-01T10:00:00.000Z",
            "updated_at": "2026-09-01T10:00:00.000Z"
        }],
        "meta": {"total": 1, "offset": 0, "limit": 50}
    })
    .to_string();
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    let output = cmd_with_server(&url)
        .args(["webhooks", "list", "--format", "table", "--no-color"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    assert_no_ansi(&output.stdout, "stdout", "webhooks list table success path");
    assert_no_ansi(&output.stderr, "stderr", "webhooks list table success path");
}

/// (b) Error path with --no-color: error.rs renders detail/hint with color
/// when enabled — with --no-color the 404 stderr carries zero ANSI.
#[test]
fn no_color_webhooks_get_404_error_path_zero_ansi() {
    let body = serde_json::json!({
        "type": "https://api.pipelite.app/errors/NOT_FOUND",
        "title": "Not Found",
        "status": 404,
        "detail": "webhook not found"
    })
    .to_string();
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(404, &body)]);

    let output = cmd_with_server(&url)
        .args(["webhooks", "get", "wh1", "--no-color"])
        .assert()
        .code(1)
        .get_output()
        .clone();
    assert_no_ansi(&output.stderr, "stderr", "webhooks get 404 error path");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("webhook not found"),
        "error detail must still be present (colorless, not missing)"
    );
}

/// (c) Truncation-heavy table surface (trash) with --no-color: zero ANSI.
#[test]
fn no_color_trash_list_zero_ansi() {
    let body = serde_json::json!({
        "data": [{
            "id": "t1",
            "entity_type": "deal",
            "type": "deals",
            "name": "Acme deal",
            "secondary": null,
            "deleted_at": "2026-09-01T12:00:00.000Z",
            "linked_parents": [],
            "deleted_by": {"kind": "user", "name": "Jane", "email": "jane@x.com"}
        }],
        "meta": {"total": 1, "offset": 0, "limit": 50}
    })
    .to_string();
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    let output = cmd_with_server(&url)
        .args(["trash", "list", "--format", "table", "--no-color"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    assert_no_ansi(&output.stdout, "stdout", "trash list table");
}

/// (d) Batch path with --no-color: grammar note — `deals update` has no
/// --force (updates carry no confirmation gate; proven by the proceeds
/// control row), so the row is `--stdin --no-color`. Both streams scanned.
#[test]
fn no_color_batch_update_stdin_zero_ansi() {
    let deal = |id: &str, title: &str| {
        serde_json::json!({
            "data": {
                "id": id,
                "title": title,
                "value": 100.0,
                "stage_id": "stage_001",
                "organization_id": null,
                "person_id": null,
                "owner_id": "user_001",
                "position": null,
                "expected_close_date": null,
                "notes": null,
                "custom_fields": null,
                "created_at": "2026-01-15T10:30:00Z",
                "updated_at": "2026-03-20T14:22:00Z"
            }
        })
        .to_string()
    };
    let (url, _counter, _heads, _bodies) = common::spawn_head_capturing_stub_server(&[
        (200, &deal("deal_1", "New Title")),
        (200, &deal("deal_2", "Other")),
    ]);

    let output = cmd_with_server(&url)
        .args(["deals", "update", "--stdin", "--no-color"])
        .write_stdin(r#"[{"id":"deal_1","title":"New Title"},{"id":"deal_2","title":"Other"}]"#)
        .assert()
        .code(0)
        .get_output()
        .clone();
    assert_no_ansi(&output.stdout, "stdout", "batch update success render");
    assert_no_ansi(&output.stderr, "stderr", "batch update stderr");
}

// ------------------------------------------------------- axis: csv_and_plain

/// Shared csv runner: exit 0, line 1 is a comma-separated header carrying
/// the group's default_columns fragments, and a data line exists.
fn csv_row(args: &[&str], body: &str, header_fragments: &[&str], header_must_not_contain: &[&str]) {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, body)]);

    let output = cmd_with_server(&url)
        .args(args)
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let first = stdout
        .lines()
        .next()
        .expect("csv output must have a header line");

    assert!(
        first.contains(','),
        "csv line 1 must be comma-separated:\n{stdout}"
    );
    for fragment in header_fragments {
        assert!(
            first.contains(fragment),
            "csv header must derive from the group's default_columns ({fragment:?}):\n{stdout}"
        );
    }
    for forbidden in header_must_not_contain {
        assert!(
            !first.contains(forbidden),
            "csv header must NOT contain {forbidden:?} (this group has no such column):\n{stdout}"
        );
    }
    assert!(
        stdout.lines().count() >= 2,
        "csv must carry the 1-row fixture as a second line:\n{stdout}"
    );
}

/// Shared plain runner: exit 0, non-empty, contains the fixture's
/// first-column value (plain renders raw values, tab-separated, no header).
fn plain_row(args: &[&str], body: &str, first_column_value: &str) {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, body)]);

    let output = cmd_with_server(&url)
        .args(args)
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        !stdout.trim().is_empty(),
        "plain output must be non-empty:\n{stdout:?}"
    );
    assert!(
        stdout.contains(first_column_value),
        "plain output must contain the fixture's first-column value {first_column_value:?}:\n{stdout}"
    );
}

// -- per-group fixtures (1 row each, copied from the surface stub tests) --

fn runs_1row() -> String {
    serde_json::json!({
        "data": [{
            "id": "run1",
            "workflow_id": "wf1",
            "status": "completed",
            "trigger_data": {"dealId": "deal_001"},
            "error": null,
            "depth": 0,
            "dry_run": false,
            "current_node_id": null,
            "started_at": "2026-01-01T00:00:00Z",
            "completed_at": "2026-01-01T00:00:05Z",
            "created_at": "2026-01-01T00:00:00Z"
        }],
        "meta": {"total": 1, "offset": 0, "limit": 50}
    })
    .to_string()
}

fn templates_1row() -> String {
    serde_json::json!({
        "data": [{
            "id": "tpl_1",
            "name": "Deal Alert",
            "description": null,
            "category": "sales",
            "trigger": {"type": "crm_event", "entity": "deal"},
            "nodes": [],
            "created_at": "2026-01-01T00:00:00Z"
        }],
        "meta": {"total": 1, "offset": 0, "limit": 50}
    })
    .to_string()
}

fn notes_1row() -> String {
    serde_json::json!({
        "data": [{
            "id": "n1",
            "entity_type": "deal",
            "entity_id": "d1",
            "content": "hello",
            "author_id": "u1",
            "source": "user",
            "created_at": "2026-09-01T10:00:00.000Z",
            "updated_at": "2026-09-01T10:00:00.000Z"
        }],
        "meta": {"total": 1, "offset": 0, "limit": 50}
    })
    .to_string()
}

fn webhooks_1row() -> String {
    serde_json::json!({
        "data": [{
            "id": "wh1",
            "url": "https://example.com/hook",
            "events": ["deal.created"],
            "active": true,
            "created_at": "2026-09-01T10:00:00.000Z",
            "updated_at": "2026-09-01T10:00:00.000Z"
        }],
        "meta": {"total": 1, "offset": 0, "limit": 50}
    })
    .to_string()
}

fn trash_1row() -> String {
    serde_json::json!({
        "data": [{
            "id": "t1",
            "entity_type": "deal",
            "type": "deals",
            "name": "Acme deal",
            "secondary": null,
            "deleted_at": "2026-09-01T12:00:00.000Z",
            "linked_parents": [],
            "deleted_by": {"kind": "user", "name": "Jane", "email": "jane@x.com"}
        }],
        "meta": {"total": 1, "offset": 0, "limit": 50}
    })
    .to_string()
}

fn audit_1row() -> String {
    serde_json::json!({
        "data": [{
            "id": "a1",
            "entity_type": "deal",
            "entity_id": "d1",
            "action": "updated",
            "changes": {"value": {"from": 100, "to": 200}},
            "actor_kind": "user",
            "actor_user_id": "u1",
            "workflow_run_id": null,
            "import_session_id": null,
            "created_at": "2026-09-01T12:00:00.000Z"
        }],
        "meta": {"total": 1, "offset": 0, "limit": 50}
    })
    .to_string()
}

fn custom_fields_1row() -> String {
    serde_json::json!({
        "data": [{
            "id": "cf1",
            "entity_type": "deal",
            "name": "price",
            "type": "number",
            "config": null,
            "required": false,
            "position": 10000.5,
            "show_in_list": false,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }],
        "meta": {"total": 1, "offset": 0, "limit": 50}
    })
    .to_string()
}

// -- csv rows (7 groups; headers derive from each group's default_columns —
//    trash and audit have NO id column, models.rs trash/audit table configs) --

#[test]
fn csv_runs_list_header_and_data_row() {
    csv_row(
        &["workflows", "runs", "list", "--workflow", "wf1", "--format", "csv"],
        &runs_1row(),
        &["id", "status"],
        &[],
    );
}

#[test]
fn csv_templates_list_header_and_data_row() {
    csv_row(
        &["templates", "list", "--format", "csv"],
        &templates_1row(),
        &["id", "name"],
        &[],
    );
}

#[test]
fn csv_notes_list_header_and_data_row() {
    csv_row(
        &["notes", "list", "deals", "d1", "--format", "csv"],
        &notes_1row(),
        &["id", "created_at", "content"],
        &[],
    );
}

#[test]
fn csv_webhooks_list_header_and_data_row() {
    csv_row(
        &["webhooks", "list", "--format", "csv"],
        &webhooks_1row(),
        &["id", "url", "events"],
        &[],
    );
}

#[test]
fn csv_trash_list_header_exact_no_id_column() {
    csv_row(
        &["trash", "list", "--format", "csv"],
        &trash_1row(),
        &["name,type,deleted_at,deleted_by,linked_parents"],
        &["id"],
    );
}

#[test]
fn csv_audit_list_header_exact_no_id_column() {
    csv_row(
        &["audit", "list", "--format", "csv"],
        &audit_1row(),
        &["created_at,actor,action,entity"],
        &["id"],
    );
}

#[test]
fn csv_custom_fields_list_header_and_data_row() {
    csv_row(
        &["custom-fields", "list", "--entity-type", "deals", "--format", "csv"],
        &custom_fields_1row(),
        &["id", "entity_type", "name", "type"],
        &[],
    );
}

// -- plain rows (7 groups) --

#[test]
fn plain_runs_list_first_column_value() {
    plain_row(
        &["workflows", "runs", "list", "--workflow", "wf1", "--format", "plain"],
        &runs_1row(),
        "run1",
    );
}

#[test]
fn plain_templates_list_first_column_value() {
    plain_row(
        &["templates", "list", "--format", "plain"],
        &templates_1row(),
        "tpl_1",
    );
}

#[test]
fn plain_notes_list_first_column_value() {
    plain_row(
        &["notes", "list", "deals", "d1", "--format", "plain"],
        &notes_1row(),
        "n1",
    );
}

#[test]
fn plain_webhooks_list_first_column_value() {
    plain_row(
        &["webhooks", "list", "--format", "plain"],
        &webhooks_1row(),
        "wh1",
    );
}

#[test]
fn plain_trash_list_first_column_value() {
    plain_row(
        &["trash", "list", "--format", "plain"],
        &trash_1row(),
        "Acme deal",
    );
}

#[test]
fn plain_audit_list_first_column_value() {
    plain_row(
        &["audit", "list", "--format", "plain"],
        &audit_1row(),
        "2026-09-01T12:00:00.000Z",
    );
}

#[test]
fn plain_custom_fields_list_first_column_value() {
    plain_row(
        &["custom-fields", "list", "--entity-type", "deals", "--format", "plain"],
        &custom_fields_1row(),
        "cf1",
    );
}
