---
phase: 07-batch-operations-for-all-entities
plan: 05
subsystem: cli-batch-error-contract
tags: [batch, exit-codes, error-handling, retry, rate-limit, 429, gap-closure]
requires:
  - src/error.rs (CliError enum, exit_code())
  - src/batch.rs (run_batch_update, read_stdin_json, collect_delete_ids)
  - src/api/mod.rs (PipeliteClient, handle_response, handle_delete_response, map_request_error)
  - tokio 1 (rt, macros) — gained the `time` feature
provides:
  - CliError::InvalidInput variant → exit code 2 for structural batch input failures
  - batch::prevalidate_item_ids — zero-HTTP rejection of missing/non-string ids (mixed batches included)
  - PipeliteClient::send_with_retry — 429 Retry-After sleep (clamped 0..=60s) + exactly one retry on all 40 entity send sites
  - 429 rate-limit detail arms in handle_response / handle_delete_response
  - Hand-rolled TcpListener stub server test harness (no new crates) in tests/batch_error_test.rs
affects:
  - Phase 8 (error layer foundations build on the CliError variant set)
  - Phase 13 (end-to-end batch exit-code verification under --quiet relies on this 0/1/2 matrix)
  - REQUIREMENTS.md BATCH-04 (now fully satisfiable)
tech-stack:
  added: []
  patterns:
    - "Exit-code matrix: 0 all-ok, 1 per-item failures (BatchOutcome::finalize → Validation), 2 structural input failure (InvalidInput)"
    - "Pre-loop structural validation before the dry-run block (dry-run validates structure too)"
    - "Client-layer retry via RequestBuilder::try_clone — one 429 retry, Retry-After u64-seconds clamped 0..=60, default 1s"
    - "thiserror title interpolation ({status}) so per-item failure lines carry the HTTP code"
key-files:
  created:
    - tests/batch_error_test.rs stub-server harness (spawn_stub_server, cmd_with_server, DEAL_SUCCESS_BODY)
  modified:
    - src/error.rs
    - src/batch.rs
    - src/api/mod.rs
    - Cargo.toml
    - src/commands/deals/update.rs
    - src/commands/orgs/update.rs
    - src/commands/people/update.rs
    - src/commands/activities/update.rs
    - src/commands/pipelines/update.rs
    - src/commands/stages/update.rs
    - src/commands/workflows/update.rs
    - tests/batch_error_test.rs
decisions:
  - Dedicated CliError::InvalidInput variant (not MissingInput overload) — keeps the structural-input distinction explicit in exit_code() and display
  - prevalidate_item_ids runs BEFORE the ctx.dry_run block — --dry-run is the input-validation path, so structurally broken input must be rejected there too
  - Retry lives in the shared client (send_with_retry), so single and batch operations both get 429 handling; batch classification stays automatic via record_failure
  - Api Display interpolates status ("API error (HTTP 429)") because per-item batch failure lines render only the error title, never the detail
  - Absent and non-string ids both count as missing in the pre-scan, matching the in-loop as_str() extraction semantics
metrics:
  duration: 18 min
  completed: 2026-09-03T10:53:00Z
  tasks: 3
  commits: 6
  files_modified: 12
---

# Phase 7 Plan 5: Gap Closure — exit-2 contract, pre-HTTP id validation, 429 retry Summary

Exit-code 0/1/2 matrix for batch ops, zero-mutation pre-scan of item ids, and a 429 Retry-After retry-once in the shared API client — closing all three verification gaps blocking BATCH-04.

## What Was Built

### Task 1 — Exit code 2 for structural batch input failures (gap 3)
- `CliError::InvalidInput { detail, hint }` added; `exit_code()` maps `MissingInput | InvalidInput` → 2; clap → 2; everything else → 1.
- All 7 pre-HTTP structural sites in `src/batch.rs` converted (read_stdin_json ×2, collect_delete_ids ×3, run_batch_update ×2) — detail/hint text unchanged.
- `--stdin` vs field-flag exclusivity error converted to InvalidInput in all 7 entity `update.rs` handlers.
- Untouched by design: `BatchOutcome::finalize` (per-item failures stay Validation → exit 1), the CR-01 `--force` refusal (Validation), `parse_custom_fields`.
- Tests: 5 exit-code contract tests (.code(2) tightenings + 4 new), 2 unreachable-server tests pinned at .code(1); unit test pins exit_code(InvalidInput)==2 / exit_code(Validation)==1.

### Task 2 — Missing-ID pre-validation before any HTTP call (gap 1)
- `batch::prevalidate_item_ids` scans all parsed items for a string `id`; any violation rejects the whole input with InvalidInput: "Item(s) missing a string 'id' field: 2, 5" + actionable hint.
- Called in `run_batch_update` after the empty-list check, BEFORE the dry-run block — zero HTTP calls, zero mutations, and `--dry-run` validates structure too.
- In-loop id extraction kept as unreachable defense-in-depth (no unwrap, per CLAUDE.md).
- Tests: rewritten weak-predicate missing-id test (carried IN-04 finding), mixed-batch zero-HTTP test ("Connection failed" and "[1/2] Failed" asserted absent), non-string-id test, dry-run pre-scan test.

### Task 3 — 429 Retry-After retry-once in the API client (gap 2)
- `send_with_retry`: first send → if 429, sleep `Retry-After` seconds (u64 parse; missing/non-numeric → 1s; clamped 0..=60 — T-07G-01/02 mitigations), resend the SAME request once via `try_clone`, return the second response regardless of status.
- All 40 entity send sites (list/get/create/update/delete/batch_create ×7 entities, update_activity_raw, trigger_workflow) routed through the wrapper; `send_ping_request` intentionally untouched.
- 429 arms in `handle_response` + `handle_delete_response`: detail "Rate limited: server returned 429 again after one retry (Retry-After applied)" + reduce-batch-size hint.
- `CliError::Api` title now interpolates status → per-item failure lines read "Failed deal_1: API error (HTTP 429)".
- tokio gained the `time` feature only — no new crates (T-07-SC). Stub server is hand-rolled on `std::net::TcpListener`.
- Tests: 429→200 exits 0 with no failure line and request count == 2; 429-twice exits 1 with rate-limit detail and count == 2 (one retry, never a storm).

## Verification Results

- `cargo test` full suite: **all green** (246 tests, 0 failures) in the no-config environment the suite is designed for. On the developer's real `HOME` (a live `~/.pipelite/config.toml` exists), 2 pre-existing environment-dependent `cli_skeleton` failures remain — already logged in deferred-items.md, unrelated to this plan (see Deferred Issues).
- Gap 1 probe: `echo '[{"id":"deal_1","title":"X"},{"title":"No ID"}]' | PIPELITE_SERVER_URL=http://127.0.0.1:1 pipelite deals update --stdin` → exit 2, "Item(s) missing a string 'id' field: 2", no "Connection failed".
- Gap 3 probe: `echo 'not json' | pipelite deals update --stdin` → exit 2.
- 429 stub tests: request count == 2 in both scenarios.
- No `unwrap()`/`expect()` added to production code (diff-verified). All new errors carry detail + hint.
- Accepted overrides untouched: NDJSON still rejected (D-01), no `--continue-on-error` flag (D-06), `--force` gate intact (CR-01), `finalize` still Validation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Adjusted plan grep-gate numbers for Task 3**
- **Found during:** Task 3
- **Issue:** Plan verify gate expected `grep -c 'send_with_retry('` == 39 and `grep -c 'map_request_error(e)'` == 4, computed from an assumed 38 entity send sites. The actual codebase has **40** sites (plan undercounted: activities has `update_activity_raw`, and the per-entity tally differs from the estimate). After routing all sites: `send_with_retry(` == 41 (40 sites + 1 definition), `map_request_error(e)` == 2 (both sends inside the wrapper).
- **Fix:** Routed all 40 real sites; ran an adjusted gate (41/2 + asserted `send_ping_request` untouched and no entity site retains a direct `.send()`). The gate's intent — every entity send goes through the retry wrapper — is fully satisfied.
- **Files modified:** src/api/mod.rs
- **Commit:** e70bfcb

**2. [Rule 1 - Bug] `CliError::Api` Display now interpolates the status code**
- **Found during:** Task 3
- **Issue:** The plan's behavior requires the still-429 failure line to contain "429" or "Rate limited", but `BatchOutcome::record_failure` renders only the error title (thiserror Display) — never the detail field. The plan's fixed detail text alone could never reach stderr for batch items, so the test as specified was unsatisfiable without a title change.
- **Fix:** `#[error("API error")]` → `#[error("API error (HTTP {status})")]`. Failure lines now read "API error (HTTP 429)"; top-level Api errors also show the code in their title. No test asserted the old title (grep-verified).
- **Files modified:** src/error.rs
- **Commit:** e70bfcb

**3. [Rule 1 - Bug] Retry uses `try_clone()` instead of builder reuse**
- **Found during:** Task 3
- **Issue:** Plan stated "`send` takes `&self`, so the builder is reusable" — in reqwest, `RequestBuilder::send` **consumes** the builder.
- **Fix:** Clone the builder up front via `try_clone()` (all bodies here are buffered JSON/query/empty, so it always succeeds in practice); degenerate `None` case surfaces the 429 rather than skipping silently. Same semantics: the same request is resent exactly once.
- **Files modified:** src/api/mod.rs
- **Commit:** e70bfcb

## Deferred Issues

- The 2 `cli_skeleton` environment-dependent failures (`config_set_parses_positional_args`, `global_flags_parse_without_error`) pre-date this plan and are already logged in `deferred-items.md` (entry 1, verified pre-existing at an earlier commit). Out of scope per the scope-boundary rule; owner: Phase 8 / test-hygiene task.

## TDD Gate Compliance

Each task followed RED → GREEN with separate commits:
- Task 1: RED 3816235 → GREEN 1f0840a
- Task 2: RED 41d0475 → GREEN cd3e29c
- Task 3: RED 6b1d8c1 → GREEN e70bfcb

All RED runs failed for the expected reasons (verified in test output: exit-code mismatches and count==1 vs count==2). No refactor commits needed.

## Self-Check: PASSED

- Files: src/error.rs ✓, src/batch.rs ✓, src/api/mod.rs ✓, Cargo.toml ✓, 7× update.rs ✓, tests/batch_error_test.rs ✓ (all on disk, committed)
- Commits: 3816235 ✓, 1f0840a ✓, 41d0475 ✓, cd3e29c ✓, 6b1d8c1 ✓, e70bfcb ✓ (in git log)
