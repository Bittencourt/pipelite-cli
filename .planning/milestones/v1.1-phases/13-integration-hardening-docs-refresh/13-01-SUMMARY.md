---
phase: 13-integration-hardening-docs-refresh
plan: 01
subsystem: testing
tags: [contract-matrix, integration-tests, hardening, v1.0-contract]
requires:
  - tests/common/mod.rs (cmd / cmd_with_server / spawn_head_capturing_stub_server)
  - "Phases 7-12 surfaces (batch, runs, templates, notes, webhooks, trash, audit, custom-fields, docs, typed writing)"
provides:
  - tests/contract_matrix_test.rs (61-test table-driven contract matrix over 5 flag axes + v1.0 smoke)
  - "Verified contract baseline for the milestone audit (ROADMAP SC-1/SC-2/SC-4)"
affects:
  - 13-02 (live E2E rides the stub anchors pinned here: batch quiet exit codes, show-once secret)
tech-stack:
  added: []
  patterns:
    - "Table-driven Case table (Kind: Mutation/ReadOnly/Destructive), ONE #[test] per row delegating to shared axis runners"
    - "Unreachable-server zero-HTTP proof (assert 'Connection failed' absent) + stub-backed counter==0 variant"
key-files:
  created:
    - tests/contract_matrix_test.rs
  modified: []
decisions:
  - "No violations found — the Phase 7-12 surfaces already honor the full v1.0 global contract; zero src changes needed"
  - "templates create dry-run row authored with --trigger (local resolution); the --workflow source GET is a read exempt from the zero-mutation contract (Phase 9 pinned)"
  - "Batch summary positive pin re-authored to the locked Phase 7 failure-path shape (mixed batch under --quiet), not the plan's '2 ok, 0 failed' shorthand"
metrics:
  duration: 13 min
  completed: 2026-09-05
  tasks: 3
  files: 1
---

# Phase 13 Plan 01: Contract Matrix — Integration Hardening Summary

**Table-driven contract matrix (61 tests) proving every Phase 7-12 surface honors the v1.0 global contract — --dry-run zero-HTTP, --no-input locked refusals, --quiet data-survives, --no-color zero-ANSI, csv/plain per group — plus v1.0-surface smoke; full suite 650/650 green with zero violations found.**

## What Was Built

### Task 1: Matrix scaffold + behavioral axes (commit bec55a3)

`tests/contract_matrix_test.rs` with a `Case` table (`Kind::Mutation | ReadOnly | Destructive`,
per-row `args`/`stdin`/`expect`/`code`), one `#[test]` per row delegating to shared axis runners:

- **dry_run_zero_http** (13 rows): every new mutation (batch update/delete via stdin, templates
  create/delete, notes add/edit/delete, webhooks create/update, trash restore, custom-fields
  create/update/delete) against the unreachable `http://127.0.0.1:1` server → exit 0, stderr lacks
  "Connection failed", stdout carries the method + endpoint preview. ReadOnly rows documented as
  exempt at the type level (v1.0 precedent: reads are side-effect-free and fetch under --dry-run).
- **dry_run_typed_writing_cold_cache** (2 rows): `deals create --custom-field price=4 --dry-run`
  with HOME=tempdir → zero HTTP, cache-miss note visible on stderr; suppressed under --quiet
  (12-02 SC-4 verbatim).
- **no_input_refusals** (7 rows + 1 control): batch delete stdin → exit 1 "Re-run with --force";
  webhooks/templates/notes/custom-fields delete → exit 1 "--force"; trash purge → exit 2 (the
  locked stricter outlier, pre-fan-out); notes add without body → exit 2 "--body" (no hang); all
  refusals proven pre-HTTP on the unreachable server. Control row: batch update --stdin PROCEEDS
  (reaches HTTP, never refuses).
- Stub-backed `counter == 0` proof for one representative dry-run row.
- All row grammar `--help`-verified at authoring time. 24 tests green on first run.

### Task 2: Presentation axes (commit 7b27ebc)

- **quiet_suppression** (20 rows): empty-page hints for runs (via `--status`), notes, webhooks,
  trash, audit, custom-fields — present without --quiet, absent with it; templates covered via its
  only chatter (create multi-trigger warning). SC-2 anchors: the show-once webhook secret is DATA
  (present on stdout under --quiet; only the save-it-now warning suppresses); the batch
  "N ok, M failed" summary pinned POSITIVELY on a mixed batch under --quiet (exit 1, per-item
  failure line + `1/2` summary survive); all-ok batch exits 0 under --quiet.
- **no_color_zero_ansi** (4 rows): zero `\x1b[` bytes on stdout AND stderr — webhooks list table
  (success), webhooks get 404 (error path, detail still present), trash list (truncation-heavy
  table), batch update stdin path.
- **csv_and_plain** (14 rows): all 7 list-bearing groups. csv line 1 = header derived from each
  group's `default_columns` (runs/templates/notes/webhooks/custom-fields contain "id"; trash =
  exact `name,type,deleted_at,deleted_by,linked_parents`; audit = exact
  `created_at,actor,action,entity` — both NO id) + a data line; plain non-empty with the fixture's
  first-column value. batch + docs exempt with comments (outcome-summary text / spec passthrough).
- 31 new tests, 55 total, green on first run.

### Task 3: Full matrix run, v1.0 smoke, suite gate (commit 6575745)

- Full matrix: **61/61 green on the first complete run — no contract violations discovered** (see
  Deviation Log).
- v1.0 smoke (6 tests): deals list table render, deals get --format json parse, orgs create
  --dry-run zero-HTTP POST preview, ping stub `{"status":"ok"}`, dashboard over stubbed
  pipelines/stages/deals/workflows pages (HOME redirected), completions bash.
- Full `cargo test`: **650 passed, 0 failed** — no regressions against the v1.0 baseline.
- `unwrap()` gate clean in all fix-scope surfaces (src/commands/{webhooks,trash,notes,templates,
  custom_fields}, src/batch.rs).

## Deviation Log

### Violations

**1. No violations found** — the matrix passed clean on its first complete run (61/61 after the
Task 1/Task 2 partial runs were also fully green). Every Phase 7-12 surface already honors the
locked contract: zero-HTTP dry-run previews, locked refusal codes (1 batch/standard deletes,
2 trash purge + missing input), quiet data-survival (batch summary, show-once secret), zero ANSI
under --no-color, and csv/plain rendering per list-bearing group. No src changes were made, so the
conditional surface test files listed in the plan frontmatter were correctly untouched.

### Row-authoring adjustments (grammar/contract alignment — NOT violations; each re-verified against `--help` or the locked surface contract before adjusting the ROW, never the expectation's contract substance)

1. **templates create dry-run row authored with `--trigger` instead of `--workflow`**
   - Surface: templates / axis: dry-run
   - The plan's interface grammar listed `templates create --name T --workflow wf1`. The
     `--workflow` source resolves by GETting the workflow BEFORE the dry-run intercept — a READ
     needed to build the preview, explicitly pinned by Phase 9
     (`templates_stub_test.rs::create_dry_run_previews_the_mapped_post_without_template_post`:
     exactly 1 request, the GET; the POST is previewed). The zero-HTTP invariant applies to the
     MUTATION. Row re-authored with `--trigger '{"type":"schedule"}'` (resolves locally) so the
     row honestly proves zero-HTTP on the unreachable server; the --workflow GET exemption is
     documented in a source comment and stays pinned at surface level.
2. **Batch quiet row pinned on the locked failure-path summary shape, not "2 ok, 0 failed"**
   - Surface: batch / axis: quiet
   - The plan encoded "2 deletes succeed under --quiet → exit 0 AND '2 ok, 0 failed'-shaped
     summary PRESENT". The locked Phase 7 contract (ROADMAP SC-3: "final `N ok, M failed` summary
     that survives `--quiet`; exit code 0 only when every item succeeded";
     `07-VERIFICATION.md` re-probe; `src/batch.rs::finalize`) prints the summary on the FAILURE
     path only — an all-ok batch is silent at exit 0. The honest positive pin is the mixed batch:
     `deals delete id1 id2 --force --quiet` against stub (204, 500) → exit 1, `[2/2] Failed` +
     `1/2 ... failed` summary PRESENT on stderr; plus the all-ok exit-0 companion. Contract-level
     justification: this is exactly the ROADMAP SC-3 wording; asserting a summary on the all-ok
     path would have required a behavior CHANGE to satisfy the matrix.
3. **templates quiet row uses the create multi-trigger warning**
   - Surface: templates / axis: quiet
   - `templates list` has NO empty-list hint by design (silent data-only render). The surface's
     only quiet-suppressible chatter is the create multi-trigger warning (`warning: workflow has N
     triggers...`), pinned present-without/suppressed-with --quiet.
4. **runs quiet row uses `--status failed` on an empty page**
   - Surface: workflow runs / axis: quiet
   - The group's empty-hint fires only with --status; an empty page without --status instead fires
     the hidden-test-runs probe (separately pinned in `workflow_runs_stub_test.rs`). Using
     --status exercises the hint without the probe.
5. **no-color batch row drops `--force`**
   - Surface: batch / axis: no-color
   - `deals update` has no --force flag (updates carry no confirmation gate — proven by the
     proceeds control row). Grammar fix to the row only; the zero-ANSI expectation unchanged.

### Pre-existing tests updated by fixes

None — no violations were found, so no existing test pinned old (wrong) behavior.

### Tooling note (non-code)

The plan's automated verify gates use `grep -c '#[test]'`; in grep regex `[test]` is a character
class, which never matches the literal token. Gates were evaluated with `grep -cF '#[test]'`
(fixed-string — the intended test-count semantic): 62 occurrences, 61 real test functions
(24 → 55 → 61 across the three tasks; every count gate satisfied with margin).

## Verification Results

- `cargo test --test contract_matrix_test`: **61 passed, 0 failed** (5 axes + v1.0 smoke)
- `cargo test` (full suite): **650 passed, 0 failed** across all 34 test binaries — no regressions
- Exit-code contract held: 0 dry-run/all-ok, 1 batch/standard-delete refusals + per-item failures,
  2 trash purge + structural/missing input
- Zero-HTTP proofs: unreachable-server "Connection failed" absence on all dry-run/refusal rows,
  stub-backed `counter == 0` variant included
- Quiet contract: batch summary + webhook show-once secret PRESENT under --quiet; empty-hints,
  confirmations, cache-miss note ABSENT
- csv headers derived per group from `default_columns` (trash/audit exact, NO id); plain
  first-column values verified for all 7 groups

## Known Stubs

None — the matrix exercises the real binary end-to-end against stub servers; no placeholder data
paths were introduced.

## Self-Check: PASSED

- tests/contract_matrix_test.rs exists (1,684 lines, `struct Case` present, `mod common` once) — FOUND
- Commits bec55a3, 7b27ebc, 6575745 present in git log — FOUND
- 61 matrix tests green, full suite 650/650 green — VERIFIED
- Deviation Log present with the explicit no-violations entry — PRESENT
