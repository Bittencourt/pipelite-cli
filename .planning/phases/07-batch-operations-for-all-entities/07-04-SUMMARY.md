---
phase: 07-batch-operations-for-all-entities
plan: 04
subsystem: cli-batch-tests
tags: [batch, integration-tests, dry-run, error-handling, cli]
requires:
  - All 7 entities batch-capable (Plans 07-01..07-03)
  - src/batch.rs (BatchOutcome finalize error format)
  - tests/dry_run_test.rs cmd() helper pattern
provides:
  - tests/batch_update_test.rs (26-test batch suite member: update coverage D-01..D-03)
  - tests/batch_delete_test.rs (delete coverage D-04, D-05 + input validation paths)
  - tests/batch_error_test.rs (continue-on-error/unreachable-server coverage D-06, D-07)
affects: []
tech-stack:
  added: []
  patterns:
    - assert_cmd write_stdin("") piped-stdin trick for non-TTY paths (prompt skip, clap rejection)
    - PIPELITE_SERVER_URL env override for hermetic unreachable-server error tests
key-files:
  created:
    - tests/batch_update_test.rs
    - tests/batch_delete_test.rs
    - tests/batch_error_test.rs
  modified: []
decisions:
  - cmd() helpers set PIPELITE_SERVER_URL (the env var the app actually reads) in addition to the suite-conventional PIPELITE_URL — error-path tests must not depend on the dead env var
  - Help-text assertions cover all 7 entities for both update and delete; behavioral tests anchor on deals (reference entity) plus orgs update dry-run
metrics:
  duration: 9 min
  completed: 2026-09-02T11:47:00Z
  tasks: 2
  files: 3
---

# Phase 7 Plan 4: Batch Integration Tests Summary

Integration test suite (26 tests across 3 files) verifying batch update, batch delete, and error handling for all 7 entities via real CLI invocation — dry-run rendering, help-text flag presence, invalid-input rejection, mutual exclusivity, and continue-on-error non-zero exit against an unreachable server.

## Tasks Completed

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1 | Batch update integration tests | dfa5ca5 | tests/batch_update_test.rs |
| 2 | Batch delete + error handling integration tests | a262c4d | tests/batch_delete_test.rs, tests/batch_error_test.rs |

## What Was Built

- **tests/batch_update_test.rs (11 tests)** — `--stdin` help-text presence for all 7 entities (deals, orgs, people, activities, pipelines, stages, workflows); deals batch update `--stdin --dry-run` renders per-item PUTs with URLs (`/api/v1/deals/deal_N`) and body content; orgs dry-run renders PUT against `/api/v1/organizations/org_1`; invalid JSON on stdin fails with "Invalid JSON input"; empty piped stdin fails cleanly (non-TTY + empty content → parse error). Covers D-01, D-02, D-03.
- **tests/batch_delete_test.rs (12 tests)** — `--stdin` help-text presence for all 7 entities (delete); multi-ID dry-run renders all 3 IDs; `--stdin` string-array dry-run renders both IDs; invalid JSON fails; `--stdin` + positional ID rejected ("mutually exclusive"); no IDs and no `--stdin` rejected by clap. Covers D-04 and the input-validation half of D-05.
- **tests/batch_error_test.rs (3 tests)** — batch update against unreachable `127.0.0.1:1` → per-item "Failed" on stderr + non-zero exit; batch delete (prompt skipped via piped stdin) → both fail, non-zero exit; item missing `id` reported as failure without crashing the batch. Covers D-06, D-07.

## Test Results

- `cargo test --test batch_update_test` — 11 passed
- `cargo test --test batch_delete_test` — 12 passed
- `cargo test --test batch_error_test` — 3 passed
- `cargo test batch` — all batch-related binaries pass (26 new + 3 BatchOutcome unit + 2 stub tests)
- **Full suite with scratch HOME** (real RUSTUP/CARGO_HOME — the suite's designed no-config environment): **238 passed, 0 failed** across 23 test binaries (212 baseline + 26 new)
- Full suite with real HOME: only the documented pre-existing environment-dependent failures appear (`config_set_parses_positional_args`, `global_flags_parse_without_error`) — the new tests pass in both environments, proving they are config-hermetic
- Hermeticity smoke: manual run confirmed the unreachable-server tests produce "Connection failed" (requests hit `127.0.0.1:1`), not a live-server response

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] cmd() helper env var: added PIPELITE_SERVER_URL (the live override)**
- **Found during:** Task 1 (read-first analysis of src/config.rs)
- **Issue:** The plan's `cmd()` helper sets `PIPELITE_URL`, but `src/config.rs` only reads `PIPELITE_SERVER_URL` — `PIPELITE_URL` is a dead env var (a latent issue in the existing suite's helpers too, left untouched as out of scope). Harmless for Task 1's help/dry-run tests (no HTTP), but Task 2's error-path tests make real connection attempts: without a working URL override they would hit the configured default server (`https://app.pipelite.io`) instead of the unreachable endpoint, making them network-dependent and non-hermetic (potentially mutating a real server if a valid config were present).
- **Fix:** All three new `cmd()` helpers set `PIPELITE_SERVER_URL=http://127.0.0.1:1` (the env var the app actually reads) alongside the suite-conventional `PIPELITE_URL`, with explanatory comments. Verified hermetic via "Connection failed" smoke test.
- **Files modified:** tests/batch_update_test.rs, tests/batch_delete_test.rs, tests/batch_error_test.rs
- **Commits:** dfa5ca5, a262c4d

No other deviations — test names, structure, and assertions follow the plan's code as written.

## Requirements Note

BATCH-01, BATCH-02, BATCH-03 (claimed in this plan's frontmatter) are complete — all were already checked off by Plan 07-03; this plan's re-mark is idempotent. **BATCH-04 remains open** (contract hardening: exit-code matrix, `--quiet` summary survival, stdin full validation before first HTTP, 429 Retry-After retry). Note: 07-03 anticipated BATCH-04 would land in Plan 07-04, but this plan's frontmatter claims only BATCH-01..03 and its scope is D-01..D-07 test coverage — BATCH-04 needs its own plan (or absorption into Phase 8). Partial overlap already delivered: exit 1 on any failure and exit 2 on structural input are now test-verified.

## Known Stubs

None — all tests are fully wired against real CLI behavior.

## Self-Check: PASSED

- tests/batch_update_test.rs exists — FOUND
- tests/batch_delete_test.rs exists — FOUND
- tests/batch_error_test.rs exists — FOUND
- Commits dfa5ca5, a262c4d — FOUND in git log
- All 26 new tests pass; full suite 238/0 with scratch HOME — VERIFIED
