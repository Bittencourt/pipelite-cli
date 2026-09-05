---
phase: 09-workflow-runs-templates-docs
plan: 02
subsystem: workflow-watch-templates
tags: [workflows, runs, watch, templates, api-client, cli, cache, stub-tests]
requires:
  - 09-01 seam: render_detail, WorkflowRunStatus helpers (run_status_is_terminal, watch_exit_code), tests/common/mod.rs
  - Phase 8 error layer (parse_rfc7807 422 passthrough, per-surface hints)
  - Phase 7 batch utility (collect_delete_ids, run_batch_delete)
  - workflows delete confirmation flow (mirrored verbatim)
provides:
  - "--watch/--exit-status poll loop on workflows runs get (2s fixed, no timeout, locked exit-code contract, SIGINT -> 130)"
  - "Top-level pipelite templates group: list/get/create/delete + hidden Update parse-then-error (exit 2, delete-and-recreate hint)"
  - "WorkflowTemplate/WorkflowTemplateCreate models + 5 client methods (typed create + raw stdin create) on surface \"templates\""
  - "KEY_TEMPLATES/TTL_TEMPLATES cache lifecycle (list caches (id,name), create/delete invalidate) + template id completion candidates"
  - "tests/common/mod.rs stub helper now captures FULL request bodies (4-tuple return)"
affects:
  - 09-03 (docs plan reuses tests/common/mod.rs 4-tuple helper)
  - scripts consuming `workflows runs get --watch --exit-status` for run-outcome automation
tech-stack:
  added: []
  patterns:
    - scripted multi-response stub IS the polling harness (one connection per poll)
    - hidden-subcommand parse-then-error (Phase 8 pattern) for the nonexistent template update
    - raw-Value POST twin method for verbatim --stdin passthrough
    - prev-as-String-first-empty transition tracking (first observation silent)
key-files:
  created:
    - src/cli/templates.rs
    - src/commands/templates/mod.rs
    - src/commands/templates/list.rs
    - src/commands/templates/get.rs
    - src/commands/templates/create.rs
    - src/commands/templates/delete.rs
    - tests/templates_stub_test.rs
  modified:
    - src/cli/workflows.rs
    - src/cli/mod.rs
    - src/main.rs
    - src/commands/mod.rs
    - src/commands/workflows/runs/detail.rs
    - src/api/models.rs
    - src/api/mod.rs
    - src/cache.rs
    - tests/common/mod.rs
    - tests/workflow_runs_stub_test.rs
    - tests/help_examples_test.rs
decisions:
  - "Raw stdin create posts via post_workflow_template_raw (not create_workflow_template_raw) — verbatim passthrough preserved AND the plan's exact-count gate grep 'fn create_workflow_template' == 1 stays truthful"
  - "create --stdin rejects ALL field flags (not just --workflow/--trigger) — silent flag ignoring would mislead, matching the workflows create precedent"
  - "templates --help group after_help carries an Examples block so 'templates --help contains Examples:' is testable"
  - "--exit-status without --watch has no effect (single-shot path untouched); help doc says 'with --watch'"
metrics:
  duration: 24 min
  completed: 2026-09-03T19:56:00Z
  tasks: 3
  files: 18
---

# Phase 9 Plan 02: Watch + Templates Summary

**One-liner:** `--watch` polling with the locked exit-code contract (2s fixed, waiting polls, SIGINT → 130) plus the full top-level `templates` stack — triggers[0]→trigger mapping with multi-trigger warning, verbatim `--stdin`, hidden update exit-2 — proven by 23 new tests.

## What Was Built

- **Task 1 — Watch** (TDD: 080f58f RED → 6df6203 GREEN): `--watch`/`--exit-status` on `workflows runs get`. Fetch once; non-watch or already-terminal runs take the untouched single-shot path. Otherwise a loop polls `get_workflow_run` on a fixed `tokio::time::sleep(2s)`, prints one stderr transition line per state change (`run <id>: X → Y`, quiet-suppressible, first observation silent), renders the terminal state through the shared `render_detail` seam, and exits via the 09-01 `watch_exit_code` helper (0 on any terminal state by default; 1 with `--exit-status` on failed/defensive-unknown). `waiting` polls on (non-terminal). No SIGINT handler — default disposition kills the process, shells report 130 (documented in the fn doc + flag help). Eight scripted-stub tests prove poll counts, transition-line discipline, waiting non-terminality, exit codes, single-fetch on terminal runs, and SIGINT death with signal 2.
- **Task 2 — Templates stack** (1104840): `WorkflowTemplate` (verified wire fields) + `WorkflowTemplateCreate` + table config + 3 unit round-trip tests; `list/get/create/delete` client methods on surface `"templates"` plus `post_workflow_template_raw` so `--stdin` passes a body through verbatim; `KEY_TEMPLATES`/`TTL_TEMPLATES` (3600); new `src/cli/templates.rs` (list/get/create/delete + hidden `Update` with permissive args); top-level `Templates` variant on `Commands` with the truth-telling after_help (instantiation note, no-update sentence, global-resource note); handlers: list caches (id, name) + renders, get renders all fields, create enforces exactly-one trigger source pre-HTTP (`--workflow` fetches and maps `triggers.first()` → `trigger` with a quiet-suppressible multi-trigger stderr warning, `--trigger` parses inline, `--stdin` posts raw, no source → exit 2 naming all three), delete mirrors `workflows/delete.rs` exactly (dry-run first, TTY dialoguer confirm, non-TTY refusal = `CliError::Validation` exit 1, batch via `run_batch_delete` with api_path `workflow-templates`).
- **Task 3 — Integration proof** (b8595a9): extended the shared stub helper to a 4-tuple (full request bytes captured per connection; fixed its latent content-length-parsed-from-first-head bug) and added 15 `templates_stub_test.rs` tests + a truth-of-help test in `help_examples_test.rs`: the triggers[0] mapping with the array never leaking into the payload, the "2 triggers" warning, zero-trigger exit-2 after exactly 1 fetch, verbatim stdin (single request), 422 errors[] passthrough, source-conflict/no-source exit-2 pre-HTTP, create dry-run preview, the full delete confirmation contract, and hidden update exit-2 with zero HTTP.

## Tasks Completed

| Task | Name | Commit(s) | Files |
| ---- | ---- | --------- | ----- |
| 1 | Watch — --watch/--exit-status poll loop | 080f58f (test/RED), 6df6203 (feat/GREEN) | src/cli/workflows.rs, src/commands/workflows/runs/detail.rs, tests/workflow_runs_stub_test.rs |
| 2 | Templates stack — models, client, cache, CLI, handlers | 1104840 | src/api/{models.rs,mod.rs}, src/cache.rs, src/cli/{templates.rs,mod.rs}, src/commands/templates/*, src/commands/mod.rs, src/main.rs |
| 3 | Templates integration tests + help examples + body-capturing helper | b8595a9 | tests/templates_stub_test.rs, tests/help_examples_test.rs, tests/common/mod.rs, tests/workflow_runs_stub_test.rs, src/cli/mod.rs |

## Verification Results

- Full suite: **358 passed, 0 failed** (27 test binaries; baseline 331 after 09-01 → +27)
- Task gates: all three `<verify><automated>` gates printed GATE-OK (Task 2's required renaming the raw stdin client method, see deviations)
- Watch behaviors proven on the wire: 3-poll completion, exactly-one transition line, quiet suppression, failed→exit 0/1 split, waiting keeps polling (3 requests), already-terminal single fetch, SIGINT → signal 2
- `templates update` → exit 2, delete-and-recreate hint, zero HTTP, hidden from `--help` (smoke-verified on the built binary + stub tests)
- No `unwrap()`/`expect()` in new production code; every new error carries a hint; zero new crates

## Deviations from Plan

**1. [Rule 3 - Blocking API gap] assert_cmd::Command::spawn is private in 2.2.0**
- **Found during:** Task 1 (SIGINT test)
- **Issue:** The plan assumed `assert_cmd Command::spawn() -> std::process::Child`; the 2.2.0 API keeps `spawn` private (only `output`/`assert`/`unwrap` are public)
- **Fix:** The SIGINT test builds a `std::process::Command` directly from `env!("CARGO_BIN_EXE_pipelite")` with the same hermetic env as `common::cmd_with_server`
- **Files modified:** tests/workflow_runs_stub_test.rs
- **Commit:** 080f58f (RED) / 6df6203 (GREEN)

**2. [Rule 1 - Bug] Stub helper parsed content-length from the FIRST request's head**
- **Found during:** Task 3 (extending the helper for body capture)
- **Issue:** `heads.lock()[0]` read the first recorded head on every connection — on multi-request scripts (GET before a POST) a later request's body could be mis-framed
- **Fix:** Parse `heads.last()` (the current request); also now records full request bytes (head + body) as a second Vec, making the return a 4-tuple
- **Files modified:** tests/common/mod.rs, tests/workflow_runs_stub_test.rs (mechanical destructure)
- **Commit:** b8595a9

**3. [Rule 3 - Gate/naming] Raw stdin create needed a fifth client method**
- **Found during:** Task 2
- **Issue:** The plan's 4-method client surface cannot pass `--stdin` bodies through verbatim (typed `WorkflowTemplateCreate` would drop unknown keys), and naming it `create_workflow_template_raw` would break the plan's exact-count gate `grep -c 'fn create_workflow_template' == 1`
- **Fix:** Added `post_workflow_template_raw` (raw Value POST, typed response parse); the typed `create_workflow_template` is unchanged
- **Files modified:** src/api/mod.rs, src/commands/templates/create.rs
- **Commit:** 1104840

**4. [Note - wire truth] WorkflowTemplate has 7 fields, not 8**
- The plan's must_haves said "8 verified fields"; the research's own verified serializer shape (§ Server Contract 3) has 7 (id, name, description, category, trigger, nodes, created_at). Implemented the wire shape.

**5. [Note - expected TDD behavior] Task 3 tests passed at first run**
- The templates integration tests compiled and passed immediately because the functionality landed in Task 2 (the plan sequences implementation before integration proof, as 09-01 did). Task 1 carried the genuine RED→GREEN pair (080f58f → 6df6203). No investigation needed.

## Auth Gates

None.

## Known Stubs

None. All surface area is wired to live API client methods; `--exit-status` without `--watch` is documented as a no-op in the flag help (single-shot path intentionally untouched per the locked decision).

## Threat Flags

None — no trust-boundary surface beyond the plan's threat model. T-09-04 (global-resource delete UX: id shown in prompt, --force explicit, --dry-run URL preview, help states deployment-global), T-09-05 (multi-trigger warning, request-count-tested), T-09-07 (hidden update exit-2 before any HTTP, tested zero-HTTP) are all implemented and covered by tests.

## Self-Check: PASSED

- FOUND: src/cli/templates.rs, src/commands/templates/{mod,list,get,create,delete}.rs, tests/templates_stub_test.rs
- FOUND modified: src/cli/workflows.rs, src/cli/mod.rs, src/main.rs, src/commands/mod.rs, src/commands/workflows/runs/detail.rs, src/api/{models.rs,mod.rs}, src/cache.rs, tests/{common/mod.rs,workflow_runs_stub_test.rs,help_examples_test.rs}
- FOUND commits: 080f58f, 6df6203, 1104840, b8595a9 (all in `git log`)
