---
phase: 07-batch-operations-for-all-entities
plan: 02
subsystem: cli-batch-operations
tags: [batch, orgs, people, activities, stdin, continue-on-error, cli]
requires:
  - src/batch.rs (BatchOutcome, read_stdin_json — from Plan 07-01)
  - deals batch implementation as canonical reference pattern
  - src/api/mod.rs (update_org, delete_org, update_person, delete_person, update_activity, delete_activity)
provides:
  - orgs batch update (--stdin, id-in-payload) + batch delete (multi-ID + --stdin + confirmation)
  - people batch update + batch delete (same pattern)
  - activities batch update + batch delete (same pattern)
affects:
  - Plan 07-03 (pipelines/stages/workflows replicate this pattern)
  - Plan 07-04 (integration tests build on these handlers)
tech-stack:
  added: []
  patterns:
    - BatchOutcome continue-on-error loop (verbatim from deals reference)
    - required_unless_present clap pattern for positional-or---stdin args
    - dry-run-before-confirmation ordering (D-05) in all batch delete paths
key-files:
  created: []
  modified:
    - src/cli/orgs.rs
    - src/cli/people.rs
    - src/cli/activities.rs
    - src/commands/orgs/update.rs
    - src/commands/orgs/delete.rs
    - src/commands/people/update.rs
    - src/commands/people/delete.rs
    - src/commands/activities/update.rs
    - src/commands/activities/delete.rs
decisions:
  - Replicated the deals reference implementation verbatim including the --stdin vs field-flags mutual-exclusivity check (07-01 Rule 2 fix is part of the canonical pattern)
  - activities update_with_null_completed now takes extracted `id: &str` parameter instead of reading args.id (consequence of Option<String> id extraction)
  - BATCH-01..03 intentionally not marked complete — shared with Plans 07-03/07-04 (same deferral as 07-01)
metrics:
  duration: 7 min
  completed: 2026-09-02T11:17:00Z
  tasks: 2
  files: 9
---

# Phase 7 Plan 2: Batch Operations for Orgs, People, Activities Summary

Deals batch pattern replicated to orgs, people, and activities: batch update via `--stdin` (JSON array with per-item `id`, continue-on-error) and batch delete via multi-ID positionals or `--stdin` with dry-run-first TTY confirmation — all four primary entities now batch-capable.

## Tasks Completed

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1 | Batch update + delete for orgs and people | 6f67b36 | src/cli/orgs.rs, src/cli/people.rs, src/commands/orgs/update.rs, src/commands/orgs/delete.rs, src/commands/people/update.rs, src/commands/people/delete.rs |
| 2 | Batch update + delete for activities | 76a9db4 | src/cli/activities.rs, src/commands/activities/update.rs, src/commands/activities/delete.rs |

## What Was Built

- **CLI args (all 3 entities)** — `UpdateArgs.id` is now `Option<String>` with `required_unless_present = "stdin"` plus a `--stdin` flag; `DeleteArgs.id` became `ids: Vec<String>` (multi-ID positional, same `required_unless_present`) plus `--stdin`. `ArgValueCandidates` completion annotations preserved on id/ids fields. after_help batch examples added to all six subcommands.
- **Batch update (orgs/people/activities)** — `<entity> update --stdin` reads a JSON array, rejects field flags alongside `--stdin` (mutual exclusivity, per deals reference), extracts `id` per object, deserializes the rest as `*Update`, calls the entity's `update_*` client method per item with continue-on-error via `BatchOutcome`; successes rendered via `render_list` after the loop; cache invalidated once if anything succeeded; dry-run renders one PUT per item.
- **Batch delete (orgs/people/activities)** — `<entity> delete id1 id2 id3` or `--stdin` with a JSON string array (mutually exclusive, empty-list rejected); single ID keeps the original single-delete flow; batch path checks dry-run FIRST (never prompts), then `dialoguer::Confirm` ("Delete N organization(s)? / person(s)? / activity(ies)?", default false) only on TTY without `--no-input`; per-item errors to stderr, cache invalidated once if any succeeded, `finalize` summary with exit 1 on any failure.
- **Activities specifics** — `update_with_null_completed` (--mark-undone raw-JSON path) signature changed to accept the extracted `id: &str`; both call sites inside the single-update path updated.

## Test Results

- `cargo check` — clean (only the 4 pre-existing warnings documented in 07-01)
- Full suite `cargo test` (scratch HOME — the suite's designed no-config environment) — **212 passed, 0 failed** across all 14 test binaries, including `activities_integration` (8/8) and `batch_cli_stub_test`
- CLI smoke: `orgs update --help` / `activities update --help` show `--stdin`; `people delete --help` shows `[IDS]...` + `--stdin`
- Functional smoke (dry-run, no HTTP): `orgs delete org_1 org_2 org_3 --dry-run` renders 3 DELETEs; `echo '["per_1","per_2"]' | people delete --stdin --dry-run` renders 2 DELETEs; stdin activity update array renders per-item PUTs; empty stdin list → "Empty ID list" + hint
- Note: with a real `~/.pipelite/config.toml` present, the 2 known pre-existing environment-dependent tests fail (`config_set_parses_positional_args`, `global_flags_parse_without_error`) — documented in 07-01's Deferred Issues, unrelated to this plan

## Deviations from Plan

None — plan executed exactly as written. The `--stdin`/field-flags mutual-exclusivity check replicated in all three update handlers is part of the deals reference implementation the plan mandates following (it was a Rule 2 fix in 07-01, now canonical). The `update_with_null_completed` signature change is a direct mechanical consequence of the plan's own instruction to extract `args.id` via `ok_or_else` in the activities single-update path.

## Requirements Note

BATCH-01..BATCH-03 are phase-level requirements shared across Plans 07-02/07-03/07-04. Entities covered so far: deals (07-01), orgs, people, activities (this plan). Pipelines, stages, and workflows remain (07-03), plus integration tests (07-04) — requirements stay open until those land (same deferral rationale as 07-01).

## Known Stubs

None — all handlers fully implemented and wired.

## Self-Check: PASSED

- src/cli/orgs.rs, src/cli/people.rs, src/cli/activities.rs modified — FOUND (grep: `pub id: Option<String>`, `pub ids: Vec<String>`, `pub stdin: bool` present)
- 6 handler files contain `async fn batch_update`/`async fn batch_delete` + `BatchOutcome::new` + `dialoguer::Confirm` (delete) — FOUND
- No `.unwrap()` in production paths of the 6 handlers — VERIFIED via grep
- Commits 6f67b36, 76a9db4 — FOUND in git log
- cargo check clean; full suite 212/0 with scratch HOME — VERIFIED
