---
phase: 11-webhooks-trash-audit
reviewed: 2026-09-04T03:55:29Z
depth: standard
iteration: 2
files_reviewed: 7
files_reviewed_list:
  - src/commands/webhooks/update.rs
  - src/commands/webhooks/mod.rs
  - src/commands/webhooks/create.rs
  - src/commands/webhooks/delete.rs
  - src/dry_run.rs
  - src/output/mod.rs
  - tests/webhooks_stub_test.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: clean
---

# Phase 11: Code Review Report (Re-review, Iteration 2 — Final Verification)

**Reviewed:** 2026-09-04T03:55:29Z
**Depth:** standard
**Files Reviewed:** 7 (fix scope) + diff review of commits f3dba8e, 3978c34, 4cac5e5
**Status:** clean

## Summary

All three in-scope findings from iteration 1 (CR-01, WR-01, IN-03) are **fixed correctly and verified three ways**: code-level inspection of the fix commits, the full test suite (496 passed / 0 failed, including the 3 new pinning tests), and fresh empirical execution of the built binary against an unreachable server (`http://127.0.0.1:1`). No new Critical or Warning findings were introduced by the fixes.

**CR-01 regression questions answered explicitly:**
- *Does the flags path still work identically?* **Yes.** `git diff 0aa225a..HEAD -- src/commands/webhooks/update.rs` removes exactly one block (the IN-03 placeholder injection); every other change is additive. The flags path (mutual-exclusion check → flags-given check → flag validation → delta dry-run intercept with note line → get→merge→PUT) is byte-identical. Empirically: `webhooks update wh1 --url … --dry-run` still exits 0, previews the delta only, prints the "omitted keys are preserved" note, zero HTTP.
- *Does the dry-run preview render the merged body?* **Correct for both modes.** Stdin mode has no merge (the doc-locked contract is a verbatim PUT with no GET first), so the preview renders the verbatim stdin body — which is exactly what will be PUT (verified live: body echoed in the preview struct). Flags mode intentionally previews the delta only, with the merge deferred to execution — that is the pre-existing, documented (`update.rs:84-86`) and test-pinned (`update_dry_run_zero_http`) behavior, unchanged by the fix.

IN-01 and IN-02 remain **accepted** (documented, harmless, not in fix scope; both verified still present and unchanged).

Build: `cargo build` OK (3 pre-existing dead-code warnings, none phase-11). Suite: 32 test binaries, **496 passed / 0 failed** — matches the fix report's claim (493 baseline + 3 new).

## Fix Verification

### CR-01: `webhooks update --stdin` ignores `--dry-run` — **VERIFIED FIXED** (commit f3dba8e)

**Code:** `src/commands/webhooks/update.rs:47-54` — the stdin branch now intercepts `ctx.dry_run` after `read_stdin_body()` and before `execute()`, rendering `dry_run::render_dry_run("PUT", item_url, &body, …)` and returning. Zero HTTP; execution path intact.
**Tests:** new `update_stdin_dry_run_zero_http` pins exit 0, stub request counter == 0, no "Connection failed", `PUT` + target URL + verbatim body on stdout.
**Empirical (this re-review):**
- `… update wh1 --stdin --dry-run` (body piped) → exit 0, structured preview `{"dry_run":true,"method":"PUT","url":"http://127.0.0.1:1/api/v1/webhooks/wh1","body":{…verbatim…}}`, no connection attempt.
- Control (`--stdin` without `--dry-run`) → exit 1, "Could not connect" (real PUT still attempted — execution path unchanged).
- Validation ordering preserved: stdin body validation (`read_stdin_body` → `validate_stdin_body`) runs *before* the dry-run intercept, matching flags mode where flag validation also precedes the intercept — a bad body still exits 2 under `--dry-run`.
- Cache untouched on the dry-run path (invalidation remains solely after a successful PUT, `update.rs:121-123`) — correct.

### WR-01: Non-string `events` entries bypassed the allow-list — **VERIFIED FIXED** (commit 3978c34)

**Code:** `src/commands/webhooks/mod.rs:86-99` — the silent `filter_map` is replaced by an explicit collect loop: the first non-string entry returns `CliError::InvalidInput` ("Webhook event entries must be strings (got {e})" + full 13-event hint) — the same exit-2 pre-HTTP tier as unknown names. String entries are still validated as a batch via `validate_events` (first unknown name reported); empty `events: []` remains valid.
**Tests:** new unit `validate_stdin_body_rejects_non_string_event_entries` (`[123]`, `[null]`, mixed `["deal.created", 42]`, plus empty-array acceptance) and new stub `create_stdin_non_string_event` (exit 2, stderr names the problem and lists the 13, request counter == 0).
**Empirical (this re-review):** `{"events":[123]}` → exit 2 via `create --stdin --dry-run` **and** via `update --stdin` (non-dry-run) with zero HTTP (no connect error in output); `[null]` → exit 2 with "got null". The tightening breaks no legitimate body: the server's zod requires string entries, so previously-silently-dropped entries would have been 422'd anyway.
**Note:** the fix also benefits update's stdin path (shared validator) — both create and update are covered.

### IN-03: Update JSON output injected a `secret` placeholder — **VERIFIED FIXED** (commit 4cac5e5)

**Code:** `src/commands/webhooks/update.rs:125-128` — the placeholder injection is removed; the PUT response is serialized and rendered verbatim, with a comment stating the rationale. Code and the `:28` doc ("No secret exists anywhere in this flow") now agree.
**Safety:** no test pinned the old update-placeholder behavior (iteration 1 confirmed unpinned; the `(shown once at creation)` assertions in `tests/webhooks_stub_test.rs:414,442,482` cover list/get only — untouched). `output::render_single` dispatches Json to `json::render_single(item, fields)` verbatim (columns affect only table/csv/plain), so removal is behaviorally clean. Wire-faithfulness restored; scripts keying on `secret` in update output now correctly see it absent.

## Warnings

_None. No new Critical or Warning findings introduced by the fix commits._

## Info

### IN-01: `KEY_WEBHOOKS` cache is write-only — no consumer exists (carried, ACCEPTED)

**File:** `src/cache.rs:36-41`, `src/commands/webhooks/list.rs:33-41`
**Status:** unchanged and accepted (same rationale as iteration 1: harmless id/url-only speculative I/O, invalidation correctly wired on all three mutations — re-verified `create.rs:100`, `delete.rs:62`, `update.rs:122`).

### IN-02: Fan-out loop and constants duplicated within the trash module (carried, ACCEPTED)

**File:** `src/commands/trash/list.rs:11-18,92-108` vs `src/commands/trash/purge.rs:10-17,145-166`
**Status:** unchanged and accepted (not in fix scope; re-verified both `FANOUT_LIMIT` copies still present and identical).

### IN-04: Non-array `events` value in a `--stdin` body is delegated to the server (pre-existing observation, OUT of locked-contract scope)

**File:** `src/commands/webhooks/mod.rs:86`
**Issue:** A body like `{"events": "deal.created"}` (key present but not an array) still passes `validate_stdin_body` — the `and_then(|v| v.as_array())` guard skips it — and relies on the server's 422 (runtime Validation tier, exit 1). This is the same tier-inconsistency shape WR-01 described, but for a wrong-typed key rather than array entries.
**Why Info, not Warning:** the documented contract explicitly scopes the allow-list defense to arrays ("if an `events` key is present (and an array)"), the iteration-1 contract table locked "flags and stdin **arrays**", and unknown/mismatched keys are documented as left for the server under the permissive parse-then-validate stance. It predates this phase's fixes and is not a regression.
**Fix (optional, future):** reject a present-but-non-array `events` key with the same InvalidInput tier, e.g. `match body.get("events") { Some(v) if v.as_array().is_none() => return Err(InvalidInput { … }), Some(v) => { …existing entry loop… }, None => {} }`.

## Re-review Evidence Log

| Check | Method | Result |
|---|---|---|
| Flags path unchanged | `git diff 0aa225a..HEAD -- src/commands/webhooks/update.rs` (removals) | only IN-03 block removed; all else additive |
| stdin + `--dry-run` zero HTTP | live binary vs `127.0.0.1:1` | exit 0, PUT preview w/ verbatim body, no connect error |
| stdin without `--dry-run` still PUTs | live binary | exit 1, "Could not connect" |
| flags `--dry-run` unchanged | live binary | exit 0, delta preview + note line, zero HTTP |
| non-string events exit 2 pre-HTTP | live binary (create + update, dry-run and not) | exit 2, "must be strings (got 123/null)", all-13 hint, zero HTTP |
| empty `events: []` still valid | unit test + doc | accepted |
| update JSON verbatim | code + `output/mod.rs:74-75` dispatch | no injected key; list/get placeholders untouched |
| suite health | `cargo build` + `cargo test` | build OK (3 pre-existing warnings); 496 passed / 0 failed |

---

_Reviewed: 2026-09-04T03:55:29Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
