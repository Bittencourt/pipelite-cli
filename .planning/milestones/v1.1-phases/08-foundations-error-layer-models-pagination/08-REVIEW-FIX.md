---
phase: 08-foundations-error-layer-models-pagination
fixed_at: 2026-09-03T17:20:00Z
review_path: .planning/phases/08-foundations-error-layer-models-pagination/08-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-09-03T17:20:00Z
**Source review:** .planning/phases/08-foundations-error-layer-models-pagination/08-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (CR-01, WR-01, WR-02, WR-03)
- Fixed: 4
- Skipped: 0

**Full suite after fixes:** `cargo test --no-fail-fast` — **306 passed, 0 failed** (129 unit + 177 integration across 25 test binaries). The 3 previously environment-dependent failures (`deals_list_limit_zero_is_accepted`, `config_set_parses_positional_args`, `global_flags_parse_without_error`) now pass hermetically.

## Fixed Issues

### CR-01: `workflows list --active` on the page path silently returns an incomplete filtered subset while claiming completeness

**Files modified:** `src/commands/workflows/list.rs`, `tests/workflows_integration.rs`
**Commit:** 9bd37be
**Status:** fixed: requires human verification (control-flow/behavior change — see note)
**Applied fix:** Chose the review's primary option (truthful completeness over perf tradeoff). `run()` now routes every `--active` invocation through the auto-paginating `fetch_all` (batches of 100, 1000-ceiling warning intact) before printing the existing stderr warning; the single-page `fetch_page` path no longer filters or rewrites `meta.total` (`apply_active_filter` is now only called from `fetch_all`). The code now matches CHANGELOG.md lines 26–32 ("fetches all records and filters them locally") — no CHANGELOG change needed. Module doc comment updated to describe the new routing.

**Tests:** Updated the stale counter assertion in `workflows_list_active_true_filters_client_side_with_warning` (message now reflects the fetch_all route; still 1 request because the fixture total fits one 100-batch). Added two regression tests:
- `workflows_list_active_covers_all_pages_before_filtering` — 150 records across 2 pages; the active row living on page 2 ("Page Two Active") must appear and both fetches must happen (counter == 2). This is the exact scenario CR-01 described.
- `workflows_list_active_false_fetches_all_pages_before_filtering` — same 2-page fixture for `--active false`.

Note: marked "requires human verification" because the fix changes runtime routing logic; the new 2-page integration tests exercise the behavior end-to-end, but a human should confirm the routing intent before the phase verifier runs.

### WR-01: Dead `PeopleListParams.org`/`owner` fields left behind by the flag removal

**Files modified:** `src/api/mod.rs`, `src/commands/people/list.rs`, `src/commands/cache/refresh.rs`
**Commit:** 7a3c945
**Applied fix:** Deleted the `org`/`owner` fields from `PeopleListParams` and the two unreachable `pairs.push` branches in `to_query_pairs`; updated all three construction sites (`people/list.rs` fetch_page + fetch_all, `cache/refresh.rs` fetch_all_people). Struct doc comment now documents why there are no filters. Same treatment the phase already applied to `WorkflowsListParams.active`. Verified via grep that these were the only construction sites.

### WR-02: Orphaned `///` doc block attached to `PingResponse`

**Files modified:** `src/api/models.rs`
**Commit:** c99702f
**Applied fix:** Converted the module-top flatten-pattern block from `///` to plain `//` comments (plus a trailing note explaining why plain comments are used), so rustdoc no longer renders the convention note as `PingResponse` documentation.

### WR-03: Integration tests without env isolation make real authenticated API calls on machines with a live config

**Files modified:** `tests/deals_integration.rs`, `tests/cli_skeleton.rs`
**Commit:** 3d8233e
**Applied fix:** Minimal per-test env isolation following the batch-suite helper pattern:
- `deals_integration.rs::deals_list_limit_zero_is_accepted` — inline env: `PIPELITE_CONFIG` → nonexistent path, `PIPELITE_URL`/`PIPELITE_SERVER_URL` → `http://127.0.0.1:1`, `PIPELITE_API_KEY` → `fake-test-key`. The remaining bare commands in the file are help/clap-rejection paths (no HTTP, no config I/O) and were left untouched.
- `cli_skeleton.rs` — added a hermetic `cmd()` helper with the same four overrides and used it in `config_set_parses_positional_args` and `global_flags_parse_without_error`. Note: `config set` was the worst offender — with a live config it would have MODIFIED the user's real `~/.pipelite/config.toml` (exit 0, test fail); the nonexistent `PIPELITE_CONFIG` path makes it bail with exit 1 deterministically.

Verified exit-code semantics before relying on them: `error.rs::exit_code` returns 1 for all non-clap, non-structural errors (confirmed by unit test and source read), so unreachable-endpoint/connection and missing-config failures are exit 1, not clap's 2.

## Skipped Issues

None — all findings were skipped nowhere; every in-scope finding was fixed and committed.

Out-of-scope observations from the review (IN-01 indentation, IN-02 empty-page guard, IN-03 collapsible_if) were NOT addressed: fix scope for this run was CR-01 + WR-01..03 only.

---

_Fixed: 2026-09-03T17:20:00Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
