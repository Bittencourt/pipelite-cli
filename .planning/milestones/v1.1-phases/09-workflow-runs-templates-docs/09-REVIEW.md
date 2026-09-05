---
phase: 09-workflow-runs-templates-docs
reviewed: 2026-09-03T20:39:03Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - src/api/models.rs
  - src/cli/templates.rs
  - src/cli/workflows.rs
  - src/commands/templates/create.rs
  - src/commands/workflows/runs/detail.rs
  - src/error.rs
  - tests/common/mod.rs
  - tests/docs_stub_test.rs
  - tests/templates_stub_test.rs
  - tests/workflow_runs_stub_test.rs
findings:
  critical: 0
  warning: 0
  info: 8
  total: 8
status: clean
---

# Phase 9: Code Review Report (Final Re-review, Iteration 2)

**Reviewed:** 2026-09-03T20:39:03Z
**Depth:** standard
**Files Reviewed:** 10 (fix hunks + their tests + `src/error.rs` for exit-code contract verification)
**Status:** clean

## Summary

Final verification re-review of the iteration-1 fixes (commits `691bd0b`, `de11961`, `ce23b08`). Both warnings (WR-01, WR-02) and both actionable info items (IN-01, IN-02) are **resolved correctly**, with no new Critical or Warning issues introduced. Full test suite passes (0 failures across all 28 test binaries, including the 3 new WR-02 tests and the new WR-01 test); `cargo build` emits only the 3 pre-existing dead-code warnings, none introduced by the fixer.

### Per-finding verdicts

| Finding | Verdict | Evidence |
|---|---|---|
| WR-01 (`--workflow` silently ignores `--nodes`) | **RESOLVED** | `create.rs:59-68` rejects the combination pre-HTTP with `InvalidInput` (exit 2) + actionable hint; flag help and doc comment updated; test `create_workflow_with_nodes_rejected_pre_http` asserts exit 2, both messages, and zero HTTP (unreachable server, "Connection failed" absent) — passing |
| WR-02 (watch dies on single transient poll error) | **RESOLVED** | `detail.rs:88-120` tolerates 3 consecutive failed polls, gives up on the 4th with `Validation` (exit 1) + hint; 3 new tests pass; terminal contract untouched (existing watch tests pass) |
| IN-01 (unused `assert_cmd::Command` imports) | **RESOLVED** | `ce23b08` removes both imports; no `unused` warnings in test builds |
| IN-02 (stale `#[allow(dead_code)]` on watch helpers) | **RESOLVED** | All 3 attributes removed (`models.rs`); build confirms no dead-code warnings — `as_str`, `run_status_is_terminal`, `watch_exit_code` are genuinely used by `detail.rs:75,66/82,84` |
| IN-03 (docs `--save` TOCTOU) | open (non-blocking) | unchanged from iteration 1 |
| IN-04 (missing `#[serde(default)]` on nullable run/step fields) | open (non-blocking) | unchanged from iteration 1 |
| IN-05 (non-UTF-8 stdin hintless error) | open (non-blocking) | unchanged from iteration 1 |
| IN-06 (hintless invariant fallback after `check_missing`) | open (non-blocking) | unchanged from iteration 1 |
| IN-07 (confirmation-decline UX single vs batch) | open (non-blocking) | unchanged from iteration 1 |
| IN-08 (docs 429 re-wrap hint) | open (non-blocking) | unchanged from iteration 1 |
| IN-09 (dead `PIPELITE_URL` env in test helper) | open (non-blocking) | unchanged from iteration 1 (still present at `tests/common/mod.rs:22`) |

### Adversarial verification of WR-02 semantics (the risky change)

- **Counter reset:** `Ok(d) => { consecutive_failures = 0; d }` (`detail.rs:89-92`) — any successful poll resets unconditionally, so scattered single blips can never accumulate into an abort. Matches the specified semantics exactly.
- **Terminal precedence vs failure counter:** the terminal check runs at the top of the loop (`detail.rs:82`) *before* any refetch; the give-up arm is reachable only from the refetch `match`, which executes only when the last-known state was non-terminal. A terminal observation therefore always wins, and `--exit-status` mapping is untouched (`watch_exit_status_maps_failed_to_exit_1` still passes).
- **Quiet handling:** the per-failure warning is gated on `!ctx.quiet` (`detail.rs:95`), matching the transition-line contract; `watch_quiet_suppresses_poll_failure_warnings` proves suppression on the wire.
- **No hot-spin:** every tolerated failure iterates through the 2s sleep before the next fetch (`continue` re-enters the loop above the `sleep`), so retries stay on the locked 2s interval.
- **No duplicate transition lines:** after a tolerated failure `prev` already equals `current`, so the re-observation prints nothing — the "exactly one stderr line per change" contract holds.
- **Give-up arithmetic:** the message's `MAX + 1` = 4 is accurate at abort time (3 tolerated + 1 fatal); exit 1 confirmed via `error.rs:87-99` (`Validation` falls to the else arm).
- **Initial fetch still fails fast** via `?` (`detail.rs:62`) — correctly documented and unchanged.
- **Stub-server double-count check:** the status-0 transport-failure path (`tests/common/mod.rs:136-140`) increments the counter once and `continue`s *before* the normal-path increment (line 158) — no double-count; proven empirically by the counter assertions (4 and 5) in the passing new tests.
- **No internal client retry desync:** the passing `watch_survives_a_transient_poll_failure_then_completes` (scripted drop, then normal responses, counter == 4) proves reqwest does not transparently retry the dropped connection and desync the script.

### WR-01 interaction matrix (all pre-HTTP, all exit 2)

- `--stdin` + any flag (incl. `--nodes`) → `create.rs:38-44`
- `--workflow` + `--trigger` → `create.rs:46-54`
- `--workflow` + `--nodes` → `create.rs:59-68` (new)
- `--trigger` + `--nodes` → still the only valid nodes pairing (`create.rs:91-97`)
- no trigger source → `create.rs:99-105`

First HTTP in any accepted path occurs at `create.rs:76` (workflow fetch) or later — the rejection cannot fire a request. The pre-HTTP property is pinned by the unreachable-server test.

## Info

### IN-10: Pre-existing dead-code warnings in phase-adjacent code (NOT introduced by the fixer)

**File:** `src/api/models.rs:451`, `src/cache.rs:30`, `src/prompt.rs:318`
**Issue:** `cargo build` emits 3 dead-code warnings: `WorkflowRunTrigger` is never constructed anywhere (only its definition matches), and `TTL_WORKFLOWS`/`get_workflows_cached` are dead in the binary target (used only within `prompt.rs`, whose cached-workflow prompt path is unreached). Verified pre-existing: the fix commits touched only 5 src files (`git diff a30e120..HEAD --stat`), none of them these items, and removing an `#[allow]` cannot create warnings on other items. `WorkflowRunTrigger` sits inside the phase-09 models surface and was missed by iteration 1.
**Fix:** Either wire up or delete `WorkflowRunTrigger` (if the server emits a `trigger` object on runs, it may deserve `#[serde(default)]` + usage; otherwise remove it). For `prompt.rs`, remove the unreached cached-workflow helper or call it. Out of phase 09 scope — track as backlog.

---

_Reviewed: 2026-09-03T20:39:03Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
