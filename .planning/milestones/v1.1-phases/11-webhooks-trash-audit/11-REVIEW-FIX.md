---
phase: 11-webhooks-trash-audit
fixed_at: 2026-09-04T00:00:00Z
review_path: .planning/phases/11-webhooks-trash-audit/11-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 11: Code Review Fix Report

**Fixed at:** 2026-09-04
**Source review:** `.planning/phases/11-webhooks-trash-audit/11-REVIEW.md`
**Iteration:** 1
**Scope:** CR-01, WR-01, IN-03 (per fix instructions; IN-01 and IN-02 were not in scope)

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0
- Final suite: `cargo test --no-fail-fast` — **496 passed, 0 failed** (493 baseline + 3 new tests), `cargo check --all-targets` clean (3 pre-existing dead-code warnings, none phase-11)

## Fixed Issues

### CR-01: `webhooks update --stdin` ignores `--dry-run` and executes the real PUT

**Files modified:** `src/commands/webhooks/update.rs`, `tests/webhooks_stub_test.rs`
**Commit:** f3dba8e
**Applied fix:** Added a `ctx.dry_run` intercept in the stdin branch of `update::run` (before the `execute()` call) that renders the verbatim-PUT preview via `dry_run::render_dry_run("PUT", item_url, &body, ...)` and returns — zero HTTP, exit 0, identical contract to the flags path. Extended the module doc to state `--dry-run` is honored in stdin mode too. New stub test `update_stdin_dry_run_zero_http` pins the behavior: piped stdin + `--dry-run` → exit 0, stub request counter == 0, no "Connection failed", stdout carries `PUT`, the target URL, and the verbatim body (mirrors the existing `update_dry_run_zero_http`).

### WR-01: Non-string entries in `--stdin` `events` arrays silently bypass the 13-event allow-list

**Files modified:** `src/commands/webhooks/mod.rs`, `tests/webhooks_stub_test.rs`
**Commit:** 3978c34
**Applied fix:** Replaced the silent `filter_map(|e| e.as_str())` in `validate_stdin_body` with an explicit collect loop: the first non-string entry now returns `CliError::InvalidInput` ("Webhook event entries must be strings (got {e})") with the 13-events hint — same exit-2 pre-HTTP tier as unknown names. String entries are still validated together via `validate_events` so the first unknown name is reported across the whole array; an empty `events` array remains valid. Updated the function doc. Added unit test `validate_stdin_body_rejects_non_string_event_entries` (covers `123`, `null`, and mixed arrays + empty-array acceptance) and stub test `create_stdin_non_string_event` (`{"events":[123]}` → exit 2, stderr names the problem and lists the 13, counter == 0).

### IN-03: Update JSON output injects a `secret` placeholder its own doc says never exists in this flow

**Files modified:** `src/commands/webhooks/update.rs`
**Commit:** 4cac5e5
**Applied fix:** Removed the `"secret": "(shown once at creation)"` injection from `execute()`'s JSON branch — the PUT response is now rendered verbatim, matching the module doc ("No secret exists anywhere in this flow"). Safe by construction: `render_single` only uses the placeholder for JSON output, and no test pinned the placeholder on update (the placeholder contract is locked on list/get, which are untouched).

## Skipped Issues

None — all in-scope findings were fixed.

## Notes

- IN-01 (write-only `KEY_WEBHOOKS` cache) and IN-02 (duplicated trash fan-out loop) were explicitly out of scope for this fix pass and remain open per the review.
- Each fix was verified per-finding: `cargo check --all-targets` + targeted tests before its atomic commit; the full suite ran green after all commits. All three fixes are behavior-pinned by new automated tests (exit codes, stub request counters, stdout assertions), not just syntax-checked.
- Empirical repro from the review (stdin + `--dry-run` → live PUT, exit 1) now fails closed: the same invocation renders the preview and exits 0 with zero connection attempts.

---

_Fixed: 2026-09-04_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
