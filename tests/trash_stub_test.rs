//! Stub-server integration tests for the trash surface (TRSH-01..03,
//! SC-3/SC-4, Phase 11).
//!
//! Test order follows task order: Task 1 = trash list (9-alias normalization
//! to plural tabs, pagination, bounded --all fan-out, table-cell truncation
//! with full json, deleted_by variants, empty hint, general 403 surface).
//! Task 2 appends restore + purge — the PINNED zero-HTTP purge refusal test
//! (exit 2, zero requests on an unreachable server) lives in the purge block.
//!
//! Uses tests/common/mod.rs (shared Phase 9 helpers, no new crates).

mod common;

use predicates::prelude::*;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

// -- fixtures (verified wire shapes from RESEARCH § Trash) --

/// Single trash row with the default `user` deleted_by and empty
/// linked_parents.
fn trash_row(id: &str, entity_type: &str, tab: &str, name: &str) -> serde_json::Value {
    trash_row_full(
        id,
        entity_type,
        tab,
        name,
        serde_json::json!([]),
        serde_json::json!({"kind": "user", "name": "Jane", "email": "jane@x.com"}),
    )
}

/// Full-control row builder (linked_parents + deleted_by variants).
fn trash_row_full(
    id: &str,
    entity_type: &str,
    tab: &str,
    name: &str,
    linked_parents: serde_json::Value,
    deleted_by: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "entity_type": entity_type,
        "type": tab,
        "name": name,
        "secondary": null,
        "deleted_at": "2026-09-01T12:00:00.000Z",
        "linked_parents": linked_parents,
        "deleted_by": deleted_by
    })
}

/// Verified list envelope `{data, meta:{total, offset, limit}}`.
fn list_body(rows: Vec<serde_json::Value>, total: u64) -> String {
    serde_json::json!({
        "data": rows,
        "meta": {"total": total, "offset": 0, "limit": 50}
    })
    .to_string()
}

/// Verified server 403 shape (used for the list unresolvable-actor edge and
/// the restore foreign-record probe — both must render the GENERAL hint).
fn forbidden_403() -> String {
    serde_json::json!({
        "type": "https://api.pipelite.app/errors/FORBIDDEN",
        "title": "Forbidden",
        "status": 403,
        "detail": "You don't have access to this resource"
    })
    .to_string()
}

/// Verified server 404 shape for a record not currently in the trash.
fn not_found_404() -> String {
    serde_json::json!({
        "type": "https://api.pipelite.app/errors/NOT_FOUND",
        "title": "Not Found",
        "status": 404,
        "detail": "Record not found in trash"
    })
    .to_string()
}

/// The captured (lowercased) head of request `idx`.
fn captured_head(heads: &Mutex<Vec<String>>, idx: usize) -> String {
    heads
        .lock()
        .expect("heads lock")
        .get(idx)
        .expect("captured head")
        .clone()
}

// -- trash list (TRSH-01, SC-3) --

/// Omitting --type sends NO type param (the server defaults to the deals
/// tab); the table renders the name, the PLURAL tab, and the deleted_by
/// label.
#[test]
fn list_default_no_type() {
    let body = list_body(vec![trash_row("t1", "deal", "deals", "Acme deal")], 1);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["trash", "list", "--format", "table"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("Acme deal"))
        .stdout(predicate::str::contains("deals"))
        .stdout(predicate::str::contains("user (Jane"));

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("get /api/v1/trash?"),
        "list must GET /api/v1/trash: {head}"
    );
    assert!(
        head.contains("limit=50&offset=0"),
        "default pagination must reach the wire: {head}"
    );
    assert!(
        !head.contains("type="),
        "no --type must send NO type param (server defaults to deals):\n{head}"
    );
}

/// The singular alias `deal` normalizes to the PLURAL tab in the query
/// string (P5 — the server 422s singular tokens).
#[test]
fn list_type_deal_alias() {
    let body = list_body(vec![trash_row("t1", "deal", "deals", "Acme deal")], 1);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["trash", "list", "--type", "deal", "--format", "json"])
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("type=deals"),
        "--type deal must normalize to type=deals: {head}"
    );
}

/// Three of the nine aliases across three runs: orgs → organizations,
/// person → people, activities → activities.
#[test]
fn list_type_aliases() {
    for (alias, tab) in [
        ("orgs", "organizations"),
        ("person", "people"),
        ("activities", "activities"),
    ] {
        let body = list_body(vec![trash_row("t1", "x", tab, "row")], 1);
        let (url, _counter, heads, _bodies) =
            common::spawn_head_capturing_stub_server(&[(200, &body)]);

        common::cmd_with_server(&url)
            .args(["trash", "list", "--type", alias, "--format", "json"])
            .assert()
            .code(0);

        let head = captured_head(&heads, 0);
        assert!(
            head.contains(&format!("type={tab}")),
            "--type {alias} must normalize to type={tab}: {head}"
        );
    }
}

/// Unknown types exit 2 pre-HTTP: no server exists in this test at all, so
/// any request would surface "Connection failed" — its absence proves zero
/// HTTP.
#[test]
fn list_unknown_type() {
    let output = common::cmd()
        .args(["trash", "list", "--type", "bogus"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Unknown trash type"))
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Connection failed"),
        "unknown type must be rejected BEFORE any HTTP:\n{stderr}"
    );
}

/// --limit/--offset pass through verbatim on a plain (non---all) list.
#[test]
fn list_pagination() {
    let body = list_body(vec![], 0);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["trash", "list", "--limit", "2", "--offset", "4", "--format", "json"])
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("limit=2&offset=4"),
        "explicit pagination must reach the wire: {head}"
    );
}

/// --all fans out at limit=100 and STOPS at the partial page (P3): total
/// says 103, but page 2 carries only 3 rows — exactly 2 GETs, and page-2
/// rows render.
#[test]
fn list_all_fanout_partial_break() {
    let page1: Vec<serde_json::Value> = (0..100)
        .map(|i| trash_row(&format!("d{i}"), "deal", "deals", &format!("Deal number {i}")))
        .collect();
    let body1 = serde_json::json!({
        "data": page1,
        "meta": {"total": 103, "offset": 0, "limit": 100}
    })
    .to_string();
    let body2 = list_body(
        vec![
            trash_row("dx1", "deal", "deals", "Deal number page-two-a"),
            trash_row("dx2", "deal", "deals", "Deal number page-two-b"),
            trash_row("dx3", "deal", "deals", "Deal number page-two-c"),
        ],
        103,
    );
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body1), (200, &body2)]);

    common::cmd_with_server(&url)
        .args(["trash", "list", "--all", "--format", "table"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("Deal number page-two-a"));

    assert_eq!(
        heads.lock().expect("heads lock").len(),
        2,
        "the partial page must break the fan-out: exactly 2 GETs"
    );
    let head0 = captured_head(&heads, 0);
    assert!(
        head0.contains("limit=100") && head0.contains("offset=0"),
        "fan-out page 1 must use limit=100 offset=0: {head0}"
    );
    let head1 = captured_head(&heads, 1);
    assert!(
        head1.contains("offset=100"),
        "fan-out page 2 must step the offset: {head1}"
    );
}

/// An empty FIRST page breaks the --all loop immediately: exactly 1 GET.
#[test]
fn list_all_empty_first_page() {
    let body = list_body(vec![], 0);
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body)]);

    common::cmd_with_server(&url)
        .args(["trash", "list", "--all", "--format", "json"])
        .assert()
        .code(0);

    assert_eq!(
        heads.lock().expect("heads lock").len(),
        1,
        "empty first page must break the fan-out: exactly 1 GET"
    );
}

/// linked_parents render truncated in TABLE cells (joined, ~80 chars) but
/// in FULL via --json (T-11-09). comfy-table may wrap a long cell across
/// lines, so table assertions compare with line breaks/spaces flattened.
#[test]
fn list_linked_parents_truncation() {
    let p1 = "P1".repeat(15); // 30 chars
    let p2 = "P2".repeat(15);
    let p3 = "P3".repeat(15);
    let row = trash_row_full(
        "t1",
        "deal",
        "deals",
        "Orphaned deal",
        serde_json::json!([p1.clone(), p2.clone(), p3.clone()]),
        serde_json::json!({"kind": "user", "name": "Jane", "email": "jane@x.com"}),
    );
    let body = list_body(vec![row], 1);
    // Two scripted pages: the table run and the json run each make one GET.
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body), (200, &body)]);

    let output = common::cmd_with_server(&url)
        .args(["trash", "list", "--format", "table"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let flat = stdout.replace(['\n', ' '], "");

    assert!(
        flat.contains(&p1) && flat.contains("..."),
        "truncated cell must keep the first parent name + ellipsis:\n{stdout}"
    );
    assert!(
        !flat.contains(&p3),
        "truncated cell must drop the last parent name:\n{stdout}"
    );

    // --json keeps the FULL array.
    let output = common::cmd_with_server(&url)
        .args(["trash", "list", "--format", "json"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json list parses");
    assert_eq!(
        parsed[0]["linked_parents"],
        serde_json::json!([p1, p2, p3]),
        "json must expose the FULL linked_parents array:\n{stdout}"
    );
}

/// deleted_by renders as a kind label (plus name/workflow details when
/// present) in table cells; --json shows the full objects — and the api_key
/// object carries NO name field, by server design (S6).
#[test]
fn list_deleted_by_variants() {
    let rows = vec![
        trash_row_full(
            "t1",
            "deal",
            "deals",
            "By api key",
            serde_json::json!([]),
            serde_json::json!({"kind": "api_key"}),
        ),
        trash_row_full(
            "t2",
            "deal",
            "deals",
            "By workflow",
            serde_json::json!([]),
            serde_json::json!({"kind": "workflow_run", "workflow_name": "Nightly"}),
        ),
        trash_row_full(
            "t3",
            "deal",
            "deals",
            "Not recorded",
            serde_json::json!([]),
            serde_json::json!({"kind": "not_recorded"}),
        ),
    ];
    let body = list_body(rows, 3);
    // Two scripted pages: the table run and the json run each make one GET.
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body), (200, &body)]);

    common::cmd_with_server(&url)
        .args(["trash", "list", "--format", "table"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("api_key"))
        .stdout(predicate::str::contains("workflow_run (Nightly)"))
        .stdout(predicate::str::contains("not_recorded"));

    let output = common::cmd_with_server(&url)
        .args(["trash", "list", "--format", "json"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json list parses");
    assert_eq!(
        parsed[0]["deleted_by"],
        serde_json::json!({"kind": "api_key"}),
        "api_key deleted_by must carry NO name field (S6):\n{stdout}"
    );
    assert_eq!(parsed[1]["deleted_by"]["workflow_name"], "Nightly");
    assert_eq!(parsed[2]["deleted_by"], serde_json::json!({"kind": "not_recorded"}));
}

/// An empty trash prints one stderr hint (with a restore round-trip
/// pointer), suppressed under --quiet, exit 0 either way.
#[test]
fn list_empty_hint() {
    let body = list_body(vec![], 0);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &body), (200, &body)]);

    common::cmd_with_server(&url)
        .args(["trash", "list", "--format", "json"])
        .assert()
        .code(0)
        .stderr(predicate::str::contains("No trashed"));

    common::cmd_with_server(&url)
        .args(["trash", "list", "--quiet", "--format", "json"])
        .assert()
        .code(0)
        .stderr(predicate::str::contains("No trashed").not());
}

/// The list 403 edge (unresolvable actor) renders the GENERAL permission
/// wording — never the purge hint (T-11-08).
#[test]
fn list_unscoped_403() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(403, &forbidden_403())]);

    let output = common::cmd_with_server(&url)
        .args(["trash", "list"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "Your API key doesn't have permission",
        ))
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("purge"),
        "list 403 must NOT render the purge hint:\n{stderr}"
    );
}

// -- trash restore (TRSH-02, SC-3) --

/// The singular alias normalizes to the PLURAL tab in the POST URL (P5);
/// 204 → success line, no confirmation.
#[test]
fn restore_deal_alias() {
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    common::cmd_with_server(&url)
        .args(["trash", "restore", "deal", "t1"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("Restored"));

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("post /api/v1/trash/deals/t1/restore"),
        "restore must POST the PLURAL-tab URL: {head}"
    );
}

#[test]
fn restore_people_alias() {
    let (url, _counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    common::cmd_with_server(&url)
        .args(["trash", "restore", "people", "p1"])
        .assert()
        .code(0);

    let head = captured_head(&heads, 0);
    assert!(
        head.contains("post /api/v1/trash/people/p1/restore"),
        "restore URL must carry the people tab: {head}"
    );
}

/// Unknown restore types exit 2 pre-HTTP — zero requests, no connection
/// attempt (the unreachable-server pattern).
#[test]
fn restore_unknown_type() {
    let output = common::cmd()
        .args(["trash", "restore", "bogus", "x"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Unknown trash type"))
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Connection failed"),
        "unknown restore type must be rejected BEFORE any HTTP:\n{stderr}"
    );
}

/// A 404 re-wraps: the server's detail is preserved AND the command layer
/// appends the not-in-trash hint (Phase 9 docs re-wrap precedent).
#[test]
fn restore_404_rewrap() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(404, &not_found_404())]);

    common::cmd_with_server(&url)
        .args(["trash", "restore", "deals", "t1"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Record not found in trash"))
        .stderr(predicate::str::contains("not in the trash"));
}

/// A foreign-record restore 403 renders the GENERAL wording — NEVER the
/// purge hint (P6 pin; restore is owner-or-admin, not admin-only).
#[test]
fn restore_403_general_hint() {
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(403, &forbidden_403())]);

    let output = common::cmd_with_server(&url)
        .args(["trash", "restore", "deals", "t_foreign"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "Your API key doesn't have permission",
        ))
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("purge"),
        "restore 403 must NOT render the purge hint:\n{stderr}"
    );
}

/// --dry-run previews the POST with zero requests.
#[test]
fn restore_dry_run() {
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(204, "")]);

    let output = common::cmd_with_server(&url)
        .args(["trash", "restore", "deal", "t1", "--dry-run"])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert_eq!(counter.load(Ordering::SeqCst), 0, "dry-run must be zero HTTP");
    assert!(stdout.contains("POST"), "preview must name the method:\n{stdout}");
    assert!(
        stdout.contains("/api/v1/trash/deals/t1/restore"),
        "preview must carry the restore URL:\n{stdout}"
    );
}

// -- trash purge (TRSH-03, SC-4) --

/// PINNED: non-TTY without --force refuses at exit 2 with ZERO HTTP — the
/// unreachable server proves no request of any kind is made (no "Connection
/// failed" can only mean no HTTP attempt). Deliberately InvalidInput
/// (exit 2), stricter than the standard delete's exit-1 refusal (P2).
#[test]
fn purge_refusal_zero_http() {
    let output = common::cmd()
        .args(["trash", "purge"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--force"))
        .stderr(predicate::str::contains("permanently"))
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Connection failed"),
        "the refusal must fire BEFORE the fan-out list — zero HTTP:\n{stderr}"
    );
}

/// Scope validation precedes BOTH the refusal and any HTTP: an unknown
/// --type exits 2 with the unknown-type hint even in non-interactive mode.
#[test]
fn purge_scope_validation_first() {
    let output = common::cmd()
        .args(["trash", "purge", "--type", "bogus"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Unknown trash type"))
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Connection failed"),
        "normalization must precede the refusal and any HTTP:\n{stderr}"
    );
}

/// --force fans out: one scoped list GET then one DELETE per victim, with
/// the "N permanently destroyed" summary.
#[test]
fn purge_force_fanout() {
    let page = list_body(
        vec![
            trash_row("d1", "deal", "deals", "First"),
            trash_row("d2", "deal", "deals", "Second"),
        ],
        2,
    );
    let (url, counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &page), (204, ""), (204, "")]);

    common::cmd_with_server(&url)
        .args(["trash", "purge", "--type", "deals", "--force"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("2 permanently destroyed"));

    assert_eq!(counter.load(Ordering::SeqCst), 3, "1 list + 2 deletes");
    let head0 = captured_head(&heads, 0);
    assert!(
        head0.contains("get /api/v1/trash") && head0.contains("type=deals"),
        "the fan-out list must be scoped: {head0}"
    );
    assert!(
        captured_head(&heads, 1).contains("delete /api/v1/trash/deals/d1"),
        "victim 1 DELETE: {head0}"
    );
    assert!(
        captured_head(&heads, 2).contains("delete /api/v1/trash/deals/d2"),
        "victim 2 DELETE"
    );
}

/// Without --type the fan-out list carries NO type param (all tabs) and
/// each DELETE uses the row's own plural tab.
#[test]
fn purge_all_tabs_no_type() {
    let page = list_body(
        vec![trash_row("a1", "activity", "activities", "Solo")],
        1,
    );
    let (url, counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &page), (204, "")]);

    common::cmd_with_server(&url)
        .args(["trash", "purge", "--force"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("1 permanently destroyed"));

    assert_eq!(counter.load(Ordering::SeqCst), 2);
    let head0 = captured_head(&heads, 0);
    assert!(
        head0.contains("get /api/v1/trash") && !head0.contains("type="),
        "no --type must send NO type param on the fan-out list:\n{head0}"
    );
    assert!(
        captured_head(&heads, 1).contains("delete /api/v1/trash/activities/a1"),
        "the DELETE must use the row's plural tab"
    );
}

/// Per-item failures are continue-on-error: the surviving delete still
/// counts, the summary reports N ok / M failed, and the whole command
/// exits 1 (item-failure tier).
#[test]
fn purge_continue_on_error() {
    let page = list_body(
        vec![
            trash_row("d1", "deal", "deals", "Survives"),
            trash_row("d2", "deal", "deals", "Fails"),
        ],
        2,
    );
    let server_error_500 = serde_json::json!({
        "type": "https://api.pipelite.app/errors/INTERNAL",
        "title": "Internal Server Error",
        "status": 500,
        "detail": "boom"
    })
    .to_string();
    let (url, counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &page), (204, ""), (500, &server_error_500)]);

    let output = common::cmd_with_server(&url)
        .args(["trash", "purge", "--type", "deals", "--force"])
        .env("NO_COLOR", "1")
        .assert()
        .code(1)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert_eq!(counter.load(Ordering::SeqCst), 3, "continue-on-error: all victims attempted");
    assert!(
        stdout.contains("1 permanently destroyed") && stdout.contains("1 failed"),
        "summary must report N ok / M failed:\n{stdout}"
    );
    assert!(
        stderr.contains("boom"),
        "the per-item failure detail must render on stderr:\n{stderr}"
    );
}

/// A 403 mid-fan-out renders the admin purge hint and counts as a failure
/// (the loop continues per the orchestrator contract — no abort).
#[test]
fn purge_403_admin_hint() {
    let page = list_body(vec![trash_row("d1", "deal", "deals", "Victim")], 1);
    let (url, _counter, _heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &page), (403, &forbidden_403())]);

    common::cmd_with_server(&url)
        .args(["trash", "purge", "--type", "deals", "--force"])
        .env("NO_COLOR", "1")
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "Permanent purge requires an admin API key.",
        ));
}

/// --dry-run previews victims with list GETs ONLY — zero DELETEs (counter
/// == 1 for a single page) — and names the would-be destruction count and
/// both victim ids.
#[test]
fn purge_dry_run_victims() {
    let page = list_body(
        vec![
            trash_row("d1", "deal", "deals", "Victim one"),
            trash_row("d2", "deal", "deals", "Victim two"),
        ],
        2,
    );
    let (url, counter, heads, _bodies) =
        common::spawn_head_capturing_stub_server(&[(200, &page)]);

    let output = common::cmd_with_server(&url)
        .args(["trash", "purge", "--type", "deals", "--dry-run"])
        .env("NO_COLOR", "1")
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert_eq!(counter.load(Ordering::SeqCst), 1, "dry-run issues ONE list GET");
    let heads_guard = heads.lock().expect("heads lock");
    assert!(
        heads_guard.iter().all(|h| !h.contains("delete")),
        "dry-run must issue zero DELETEs: {heads_guard:?}"
    );
    drop(heads_guard);
    assert!(
        stdout.contains("Would permanently destroy 2"),
        "preview must name the victim count:\n{stdout}"
    );
    assert!(
        stdout.contains("d1") && stdout.contains("d2"),
        "preview must list both victim ids:\n{stdout}"
    );
}
