---
phase: 07-batch-operations-for-all-entities
reviewed: 2026-09-02T13:22:45Z
depth: standard
iteration: re-review-2
files_reviewed: 26
files_reviewed_list:
  - src/batch.rs
  - src/cli/activities.rs
  - src/cli/deals.rs
  - src/cli/orgs.rs
  - src/cli/people.rs
  - src/cli/pipelines.rs
  - src/cli/stages.rs
  - src/cli/workflows.rs
  - src/commands/activities/create.rs
  - src/commands/activities/delete.rs
  - src/commands/activities/update.rs
  - src/commands/deals/create.rs
  - src/commands/deals/delete.rs
  - src/commands/deals/update.rs
  - src/commands/orgs/create.rs
  - src/commands/orgs/delete.rs
  - src/commands/orgs/update.rs
  - src/commands/people/create.rs
  - src/commands/people/delete.rs
  - src/commands/people/update.rs
  - src/commands/pipelines/delete.rs
  - src/commands/pipelines/update.rs
  - src/commands/stages/delete.rs
  - src/commands/stages/update.rs
  - src/commands/workflows/delete.rs
  - src/commands/workflows/update.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: clean
---

# Phase 07: Code Review Report (Re-review, Fix Loop Iteration 2 — Final Verification)

**Reviewed:** 2026-09-02T13:22:45Z
**Depth:** standard
**Files Reviewed:** 26
**Status:** clean (0 Critical / 0 Warning / 2 Info — Info items are pre-existing, out of fix scope, and do not block)

## Summary

Final verification pass over the 4 fix commits (`45c1cc9`..`4942a81`) resolving WR-06..WR-09. All four are **resolved correctly**, verified three ways: full reads of every changed file, `git show` inspection of each fix commit's diff, and live behavioral probes against the built binary (fake server `http://127.0.0.1:1`, per project test convention — 12 probes total).

No new Critical or Warning issues were introduced. `cargo check --all-targets` produces only the 4 pre-existing dead-code warnings in files untouched by this phase (`workflows/list.rs`, `api/models.rs`, `cache.rs`, `prompt.rs`) — notably, no unused-import residue from the WR-09 deletions. All batch test binaries pass (15 + 5 + 11 + 4 new unit tests in `batch.rs` covered by the 104-test bin target). The only 2 `cargo test` failures remain the pre-existing `tests/cli_skeleton.rs` environment-coupling cases documented last round (they assume no `~/.pipelite/config.toml` exists); that file is untouched by phase 07.

Key verifications:

- **WR-06**: the gate condition is now `ids.len() == 1 && !args.stdin` in all 6 non-workflow delete handlers, so piped `--stdin` input always reaches the batch `--force` gate regardless of item count, while the v1.0 gate-free positional single delete is deliberately preserved. Live probes: `echo '["deal_1"]' | pipelite deals delete --stdin` → refusal + exit 1 before any HTTP; same command with `--force` → attempts HTTP; `pipelite deals delete deal_1` positional → gate-free HTTP attempt; piped 1-ID `--dry-run` → previews without prompting (dry-run intercept precedes the gate).
- **WR-07**: all 7 delete `after_help` texts now advertise `--stdin --force` (grep count = 1 in each of deals/orgs/people/activities/pipelines/stages/workflows). The advertised commands now match actual behavior.
- **WR-08**: `run_batch_update` and `ensure_update_fields` take a dedicated `cli_entity` parameter; all 7 call sites pass runnable plural subcommand names (orgs correctly passes `"orgs"`, not its `"organizations"` REST path). Live probes: noop-item hint reads `check \`pipelite orgs update --help\`` (was singular `deal`); orgs "No data on stdin" hint (via pty) reads `echo '…' | pipelite orgs update --stdin` (was `organizations`). Both hints now name commands that exist.
- **WR-09**: the four private `parse_custom_fields` copies are deleted; all 8 call sites in `src/commands/` now invoke `batch::parse_custom_fields` (grep confirms zero remaining `fn parse_custom_fields` outside `batch.rs`). Behavior preserved: invalid pair still errors with the same detail + hint; valid pair still renders `custom_fields` in the dry-run body.

## Prior Findings — Resolution Verdicts

| ID | Verdict | Evidence |
|----|---------|----------|
| WR-06 | **Resolved** | `ids.len() == 1 && !args.stdin` at `src/commands/deals/delete.rs:23`, and identically in `orgs/delete.rs:23`, `people/delete.rs:23`, `activities/delete.rs:23`, `pipelines/delete.rs:23`, `stages/delete.rs:23`. Doc comments rewritten to match ("even a 1-ID list … must pass --force"; "only a single positional ID keeps the v1.0 gate-free flow") — the doc/behavior contradiction is gone. Workflows unchanged (its `single_delete` already gates: `workflows/delete.rs:57-76`). Probes: piped 1-ID without `--force` refuses pre-HTTP (deals + orgs); with `--force` proceeds; positional single stays gate-free; piped 1-ID dry-run previews without prompting. |
| WR-07 | **Resolved** | `src/cli/deals.rs:80`, `orgs.rs:50`, `people.rs:65`, `activities.rs:65`, `pipelines.rs:50`, `stages.rs:65` — each delete `after_help` now ends `echo '[…]' \| pipelite <entity> delete --stdin --force`; `workflows.rs:50` already had it. Live grep over `--help` output: 1 match per entity, 7/7. |
| WR-08 | **Resolved** | `run_batch_update` gained `cli_entity: &str` (`src/batch.rs:259`), used in the stdin hint (`batch.rs:275`); `ensure_update_fields` gained `cli_entity` (`batch.rs:231`) used in the help hint (`batch.rs:234`); both documented (batch.rs:228-230, 248-251). Call sites: orgs `("organization", "orgs", …, "organizations")` at `orgs/update.rs:129-134`; deals/people/activities/workflows/pipelines/stages pass their plural names at the matching positions — all verified by reading each call site. Probes: `pipelite orgs update --help` and `pipelite orgs update --stdin` hints confirmed live; `pipelite deals update --help` hint confirmed live. |
| WR-09 | **Resolved** | Commit `4942a81` deletes the 19-line private copy from each of `deals/create.rs`, `orgs/create.rs`, `people/create.rs`, `activities/create.rs` (diff-verified: only `+use crate::batch;`, call-site swap, and the deleted fn). `grep 'parse_custom_fields' src/commands/` → 8 call sites, all `batch::parse_custom_fields`; zero private definitions remain. Behavior probes: `--custom-field oops` → same "Invalid custom field format" error + hint, exit 1; `--custom-field industry=Tech --dry-run` → `"custom_fields":{"industry":"Tech"}` in body, exit 0. |
| CR-01..CR-02, WR-01..WR-05 | **Still resolved** | Untouched by this iteration's diffs (commit inspection: the 4 fix commits do not regress the earlier fixes). The WR-06 gate change composes correctly with CR-01's batch gate — probe 1 exercises both (stdin source → batch path → refusal). |
| IN-03 | **Still open** (Info, out of scope) | Dry-run still renders the raw stdin item incl. `"id"` (`src/batch.rs:294-300`). Unchanged this iteration; non-blocking. |
| IN-04 | **Still open** (Info, out of scope) | `tests/batch_cli_stub_test.rs` dead `PIPELITE_URL` + duplicated `--help` coverage; `batch_error_test.rs` weak predicate. Unchanged this iteration; non-blocking. |

## Regression Scan (new issues introduced by the 4 fix commits)

None found. Specifics checked:

- **WR-06 control flow on destructive ops**: gate placement is after `collect_delete_ids` (so mutual-exclusivity and empty-list validation still run first) and before `run_batch_delete` (whose own dry-run intercept precedes the `--force` refusal, so piped single-ID dry-runs still preview — probed). No path reaches `single_delete` with stdin-sourced IDs anymore. The 6 edits are byte-identical in structure (diff-verified), and workflows' different (correct) structure was left alone.
- **WR-08 signature change**: compiler-enforced — all 7 `run_batch_update` call sites and all 7 `ensure_update_fields` call sites updated; `cargo check --all-targets` clean. Argument order (`entity`, `cli_entity`, `example`, `api_path`, …) matches the definition at every site; `api_path` is no longer interpolated into any hint.
- **WR-09 mechanical refactor**: no leftover `CliError` unused imports (each create file still uses `CliError` for its own validation errors); removed fns were verbatim copies, so no hint-text drift (the shared `batch.rs:375` hint matches the deleted copies' text).
- **Test suite**: 104 bin/unit + 8 + 2 + 15 + 5 + 11 + 4 pass; 2 `cli_skeleton.rs` failures are the documented pre-existing environment coupling, unchanged from the prior review.

## Info

### IN-03: Batch update dry-run still shows `"id"` (and unknown keys) that the real request omits — OPEN (carried, out of fix scope)

**File:** `src/batch.rs:294-300`

**Issue:** Unchanged: the dry-run loop renders the raw stdin item while the executed request deserializes into the Update model and strips unknown keys. Non-blocking preview fidelity gap; explicitly not part of the WR-06..WR-09 fix scope.

**Fix:** In the dry-run loop, deserialize into `T` first, run `ensure_update_fields`, and render `serde_json::to_value(&data)?`.

### IN-04: Test suite nits — OPEN (carried, out of fix scope)

**File:** `tests/batch_cli_stub_test.rs:5-10, 12-28`; `tests/batch_error_test.rs:64`

**Issue:** Unchanged: `batch_cli_stub_test.rs` duplicates `--help` coverage and sets the dead `PIPELITE_URL` env var (app reads `PIPELITE_SERVER_URL`); the `batch_error_test.rs:64` assertion `contains("missing") || contains("id")` is nearly vacuous. Hygiene only; all batch tests pass.

**Fix:** Delete the stub file; drop `PIPELITE_URL` from `cmd()` helpers; tighten the predicate to `contains("missing 'id' field")`.

---

_Not flagged, verified fine:_ all 7 delete arg structs keep the `--force` flag with consistent help text ("required in non-interactive mode"); `collect_delete_ids` still enforces `--stdin`/positional exclusivity and empty-list rejection before the WR-06 branch; cache invalidation (`KEY_*` + `stages_` prefixes) identical pre/post refactor in both `single_delete` and batch paths; pipelines/stages batch deletes preserve `Some("stages_")` prefix invalidation; CR-02's `completed_at: null` routing and its ordering vs the no-op guard untouched; workflows delete/update paths untouched this iteration.

_Out of scope but observed (unchanged from prior review):_ 2 pre-existing `tests/cli_skeleton.rs` failures from `~/.pipelite/config.toml` environment coupling; 4 pre-existing dead-code warnings in files outside phase 07 scope.

_Reviewed: 2026-09-02T13:22:45Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard (re-review, fix loop iteration 2 — final verification)_
