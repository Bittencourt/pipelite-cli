---
phase: 13-integration-hardening-docs-refresh
plan: 02
subsystem: documentation
tags: [docs-refresh, e2e, changelog, api-reference, deferred-items, checkpoint]
requires:
  - "Phases 7-12 surfaces (batch, runs, templates, notes, webhooks, trash, audit, custom-fields, docs, typed writing)"
  - "13-01 contract matrix (stub anchors: batch quiet exit codes, show-once secret)"
provides:
  - "docs/SKILL.md v1.1 extension (9 surfaces + exit-code contract + batch/typed-writing patterns)"
  - "docs/api-reference.md complete for all 9 new groups (--help-verified)"
  - "README command enumeration + CHANGELOG v1.1 released-ready"
  - "scripts/e2e-v1.1.sh (env-gated live E2E, 4 SC-2 scenarios) + docs/e2e-v1.1-report.md (FILLED — live run 21 PASS / 0 FAIL)"
  - "deferred-items disposition roll-up for the milestone audit"
  - "api/mod.rs size decision: KEEP SINGLE FILE (1805 < ~2000)"
affects:
  - "v1.1 milestone audit (reads this summary, the filled E2E report, and the annotated deferred-items files)"
tech-stack:
  added: []
  patterns:
    - "Docs authoring rule enforced: every documented flag/example verified against live --help or src at authoring time"
    - "E2E safety encoded in code, not comments: pl() invocation guard (refuses purge, exit 70), readonly FORBIDDEN_PATTERNS, assert_no_purge/assert_no_secrets with self-match-safe character-class grep"
key-files:
  created:
    - scripts/e2e-v1.1.sh
    - docs/e2e-v1.1-report.md
  modified:
    - docs/SKILL.md
    - docs/api-reference.md
    - README.md
    - CHANGELOG.md
    - src/api/mod.rs
    - src/api/models.rs
    - tests/deals_integration.rs
    - .planning/phases/07-batch-operations-for-all-entities/deferred-items.md
    - .planning/phases/08-foundations-error-layer-models-pagination/deferred-items.md
    - .planning/milestones/v1.0-phases/01-foundation/deferred-items.md
decisions:
  - "api/mod.rs is 1805 lines (< ~2000 threshold) — KEEP SINGLE FILE, no split; STATE.md watch item closed for this milestone"
  - "--continue-on-error documented as BUILT-IN batch semantics, not a flag (plan inventory said flag; --help/src prove unconditional continue + summary)"
  - "Batch stdin documented as JSON array only (no NDJSON — serde_json::from_str into Vec)"
  - "E2E all-ok-batch scenario asserts silent exit 0 (locked Phase 7 contract: summary prints on the failure path only), not the plan's 'exit 0 + summary present' shorthand"
  - "Env-dependent test trio marked RESOLVED via WR-03 hermetic env (PIPELITE_CONFIG override + unreachable endpoint), verified empirically with a real config present"
metrics:
  duration: 20 min (+ Task 4 live E2E executed by orchestrator)
  completed: 2026-09-05
  tasks: 4
  files: 9
---

# Phase 13 Plan 02: Docs Refresh + E2E Authoring Summary

**Docs brought current with the shipped v1.1 CLI (SKILL.md 9-surface extension, api-reference gap audit with --help-verified examples, README enumeration, CHANGELOG v1.1 released-ready), loose ends closed (api/mod.rs keep-single-file decision, evidence-based deferred-items audit), and the live-server E2E authored + safety-guarded + EXECUTED (21 PASS / 0 FAIL, 2 live-found bugs fixed with regression test).**

## What Was Built

### Task 1: docs/SKILL.md refresh (commit b0576ee)

Extended in the existing voice (249 → 350 lines, 11 → 15 `##` sections; existing accurate
sections untouched):

- **Stale facts corrected against src**: `api/mod.rs ~1078` → **1805 lines**; models.rs →
  **2344 lines**; cache keys from `KEY_*` consts (now include `users`, `templates`,
  `webhooks`, `custom_fields_{deal,organization,person,activity}`); TTL list updated
  (templates/webhooks/custom-fields = 1h, users = 2h); Quick Orientation tree now includes
  `batch.rs` + `custom_fields.rs`.
- **Endpoint table** rebuilt from `src/api/mod.rs` doc comments: runs
  (`/workflows/{id}/runs[/runId]`), templates (`/workflow-templates` CRUD minus update),
  notes (collection routes carry the parent; item routes only the note ID), webhooks,
  trash (restore + admin-only delete), audit, custom-field-definitions, public `/docs`,
  batch create `{entity}/batch` (deals/organizations/people), and the `run` trigger path.
- **New sections**: *Exit Code Contract* (0/1/2 with the zero-HTTP guarantee for exit 2 +
  hint conventions + rendered-error example), *Batch Pattern*, *Custom Fields & Typed
  Writing Pattern*, *New Surfaces (v1.1)* (9 compact subsections pointing at api-reference).
- **Agent task guides added**: "Adding a new command group (v1.1 style)" and the
  body-source precedence pattern.

### Task 2: api-reference gap audit + README + CHANGELOG (commit b61abcb)

- **api-reference.md**: added the 4 missing sections — *Batch operations* (update/delete/
  create via stdin, exit-code contract, `--force` non-TTY rule, per-entity batch-create
  mechanics: true `/batch` POSTs for deals/orgs/people vs per-item loops for activities/
  pipelines/stages; workflows create --stdin is single-object), *workflows runs*,
  *templates* (no update), *docs* (unauthenticated, `--save` overwrite refusal) — plus
  corrected drift found by the mandatory `--help` validation pass: all 7 entity delete
  sections now document multi-ID/`--stdin`/`--force`, all 7 update sections note `--stdin`
  batch update, and the trash list jq example fixed (`.data[].id` → `.[].id` — list JSON is
  a bare array; that fix landed in commit b408a61).
- **README.md**: new command blocks (Workflow Runs, Templates, Notes, Webhooks, Trash,
  Audit, Custom Fields with a typed `price=4` example, Docs, Batch Operations), workflows
  block extended with runs, Features bullets added, project-structure listing extended with
  all new `cli/` + `commands/` entries and `batch.rs`/`custom_fields.rs`.
- **CHANGELOG.md**: header → `## v1.1 — Server v2 Parity (released 2026-09-04)` (zero
  "unreleased" remaining); Added now carries the full Phase 7-12 feature list (batch,
  runs/watch, templates, docs, notes, webhooks, trash, audit, custom-fields + typed
  writing cross-referencing the existing data-correctness headline in Changed/Fixed);
  Breaking Changes preserved verbatim.

### Task 3: loose ends + E2E authoring (commit b408a61)

- **api/mod.rs size decision**: re-measured at execution — **1805 lines < ~2000 threshold →
  KEEP SINGLE FILE, no split**. Closes the STATE.md watch item for the milestone audit.
- **Deferred-items audit** (all 3 files annotated in place, evidence-based):
  - Env-dependent trio (07 #1, 08 #1): **RESOLVED 2026-09-04** — the WR-03 hermetic env is
    in place (`PIPELITE_CONFIG` → nonexistent path + `PIPELITE_SERVER_URL=http://127.0.0.1:1`
    in both `cmd()` helpers and the inline test env); empirically verified with the real
    `~/.pipelite/config.toml` present: cli_skeleton 7/7, deals_integration 12/12 green.
  - Build warnings (08 #2): **CARRIED** — exactly 3 warnings remain (WorkflowRunTrigger
    never constructed, TTL_WORKFLOWS never used, get_workflows_cached never used); the
    workflows/list.rs `total` warning stays resolved. Audit should decide delete-vs-silence.
  - Config unit-test flake (08 #3 + v1.0 #1): **CARRIED with live evidence** — reproduced
    during this plan: 4 failing runs of 20 (`env_var_precedence_server_url` ×2, sibling
    `env_var_precedence_api_key` ×2, once both together) with the real config present.
    Candidate fix unchanged (hermetic temp config / mutex around env mutation).
- **scripts/e2e-v1.1.sh** (executable, `bash -n` clean): env-var credentials required with
  an unset-guard (exit 2 + usage; smoke-tested); release binary resolution; 4 scenario
  functions matching the SC-2 checks; per-scenario PASS/FAIL lines to stdout AND a scratch
  results file (`$E2E_RESULTS_FILE`, default `./e2e-v1.1-results.txt`, never committed);
  trap-based best-effort cleanup; every assertion prints observed vs expected. Safety rules
  ENCODED as code: `pl()` wrapper refuses the purge subcommand (exit 70);
  `readonly FORBIDDEN_PATTERNS`; `assert_no_purge` + `assert_no_secrets` re-run after the
  scenarios (grep-proven, self-match-safe character-class patterns). Structural dry-run
  against an unreachable server confirmed: guards PASS, scenarios fail honestly with
  observed-vs-expected, no false PASSes.
- **docs/e2e-v1.1-report.md** template: per-scenario `Result: PASS/FAIL/SKIPPED` +
  evidence placeholders, `## Results` table, secret recorded as shown-once yes/no (64
  chars, REDACTED), documented leftover trash entry, cleanup accounting, and the
  Forbidden (non-admin) section pre-filled: SKIPPED live — no non-admin key available;
  covered by `tests/error_layer_stub_test.rs` stubs per 13-CONTEXT. Header states the
  filled report is the SC-2 evidence for the milestone audit.

### Task 4: live E2E execution — EXECUTED (2026-09-05)

**Status: executed — the orchestrator ran
`export PIPELITE_SERVER_URL=... && export PIPELITE_API_KEY=<ADMIN key> && bash scripts/e2e-v1.1.sh`
with the user's admin credentials (session env): 4 scenarios, **21 PASS / 0 FAIL**.
`docs/e2e-v1.1-report.md` is finalized from the live output (commit b4bcf51).**

The live run surfaced 2 real bugs, both fixed immediately and covered by a regression
test (commit 30939ec):

1. **`batch_create_{deals,organizations,people}` unwrapped the `{data}` envelope twice** —
   the server's batch-create response nests items under `{data}`, but the helpers already
   returned the envelope-shaped payload, so items were silently dropped/misparsed.
   Fixed in `src/api/mod.rs` (single unwrap at the correct layer).
2. **`PaginationMeta` required all fields** — batch endpoints return partial meta
   (no `total`/`total_pages` on `{entity}/batch`), which failed strict deserialization.
   Fixed in `src/api/models.rs` with `#[serde(default)]` on the optional meta fields.

Regression test added in `tests/deals_integration.rs` (partial-meta batch-create
envelope shape); full suite after the fixes: **651 passed / 0 failed**.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Plan bug] `--continue-on-error` is not a flag**
- **Found during:** Task 2 (mandatory --help validation pass)
- **Issue:** The plan's interface inventory documented `--continue-on-error` as a batch
  flag. No such flag exists anywhere in `src/` or `--help` — continuing past per-item
  failures + the `N ok, M failed` summary is the UNCONDITIONAL batch behavior
  (`src/batch.rs` BatchOutcome). Documenting the flag would have been the exact
  dead-flag lie class this milestone removed (threat T-13-02-03).
- **Fix:** api-reference + SKILL.md document built-in continue-on-error semantics with an
  explicit "there is deliberately NO --continue-on-error flag" note; the E2E mixed-batch
  scenario asserts exit 1 + summary WITHOUT the flag.
- **Files modified:** docs/api-reference.md, docs/SKILL.md, scripts/e2e-v1.1.md (script)
- **Commit:** b61abcb, b408a61

**2. [Rule 1 - Plan bug] Batch stdin is JSON array only — no NDJSON**
- **Found during:** Task 2 (--help/src verification)
- **Issue:** Plan inventory said "array/NDJSON". `read_stdin_json` uses
  `serde_json::from_str` into `Vec<T>` — NDJSON would be rejected exit 2.
- **Fix:** Docs say JSON array (with the exit-2 structural rejection).
- **Files modified:** docs/api-reference.md
- **Commit:** b61abcb

**3. [Rule 1 - Plan bug] E2E all-ok batch summary assertion contradicted the locked contract**
- **Found during:** Task 3 (script authoring)
- **Issue:** The plan's scenario (a) said "update-batch under --quiet → exit 0 + summary
  PRESENT". The locked Phase 7 contract (pinned by 13-01's contract matrix, 61/61 green,
  ROADMAP SC-3 wording) prints the summary on the FAILURE path only — an all-ok batch
  under --quiet is silent. The script would have failed live against correct behavior.
- **Fix:** Scenario (a) asserts silent exit 0 for the all-ok case and pins the summary on
  the mixed batch (exit 1, "1/2" + "1 failed" present); the deviation is documented in
  the script header and the report template.
- **Files modified:** scripts/e2e-v1.1.sh, docs/e2e-v1.1-report.md
- **Commit:** b408a61

**4. [Rule 1 - Docs bug] trash list jq example used a nonexistent `.data` wrapper**
- **Found during:** Task 3 (verifying JSON shapes for the E2E script)
- **Issue:** api-reference's trash list example piped through `jq -r '.data[].id'`, but
  list JSON renders a bare array (`render_list` of unwrapped items) — the example would
  fail for every user.
- **Fix:** Example corrected to `jq -r '.[].id'`.
- **Files modified:** docs/api-reference.md
- **Commit:** b408a61

**5. [Rule 3 - Blocking] assert_no_purge grep self-match**
- **Found during:** Task 3 (structural dry-run of the E2E script)
- **Issue:** The guard's regex matched its own source line in `$0`, producing a false
  "script references trash purge" FAIL.
- **Fix:** Character-class patterns (`pl[[:space:]]+trash[[:space:]]+purge`) that cannot
  self-match; re-run structural check → guards PASS.
- **Files modified:** scripts/e2e-v1.1.sh
- **Commit:** b408a61

**6. [Rule 1 - Bug, live E2E] batch_create_{deals,organizations,people} double-unwrapped the `{data}` envelope**
- **Found during:** Task 4 (orchestrator-run live E2E, 2026-09-05)
- **Issue:** The batch-create helpers deserialized the server response as if items were
  top-level, but the `{entity}/batch` response nests them under `{data}` — created items
  were lost to the caller (live scenario caught the mismatch).
- **Fix:** Single envelope unwrap at the correct layer in `src/api/mod.rs`.
- **Files modified:** src/api/mod.rs
- **Commit:** 30939ec

**7. [Rule 1 - Bug, live E2E] PaginationMeta failed deserialization on partial batch meta**
- **Found during:** Task 4 (same live run)
- **Issue:** Batch endpoints return meta without `total`/`total_pages`; strict
  deserialization rejected valid partial responses.
- **Fix:** `#[serde(default)]` on optional meta fields in `src/api/models.rs` so partial
  batch meta deserializes with sensible defaults.
- **Files modified:** src/api/models.rs, tests/deals_integration.rs (regression test)
- **Commit:** 30939ec

### --help Validation Checklist (T-13-02-03 mitigation)

| Group | Sections verified | Drift found/fixed |
|-------|-------------------|-------------------|
| batch (via entities) | deals/orgs/people/activities/pipelines/stages/workflows update+delete+create --help | delete/update sections lacked multi-ID/--stdin/--force → fixed (7+7) |
| workflows runs | runs list/get --help (flags: --workflow required, --status, --include-dry-run, --watch, --exit-status) | new section, written from --help verbatim |
| templates | list/get/create/delete --help (trigger XOR sources, --nodes requires --trigger, delete batch grammar) | new section, written from --help verbatim |
| docs | docs --help (--save, --force, --format-ignored note) | new section, written from --help verbatim |
| notes | list/add/edit/delete --help (body precedence, no --all, --force) | existing section accurate — no drift |
| webhooks | group/list/get/create/update/delete --help (13 events, --active/--inactive tri-state, https, show-once) | existing section accurate — no drift |
| trash | group/list/restore/purge --help (9→4 aliases, --all 10k cap, purge exit-2 note) | jq example path fixed |
| audit | list --help (4 verbatim filters, clamp 1..=100, no --all) | existing section accurate — no drift |
| custom-fields | group/list/create/update/delete --help (--key=name, select alias, --options rules, --position, tri-state flags) | existing section accurate — no drift |
| typed writing | deals/orgs create+update --help (--custom-field/--custom-field-json/--stdin exclusivity wording) | existing section accurate — no drift |

## Verification Results

- Task 1 gate: 15 `##` sections (≥14), Exit Code Contract + Batch Pattern + typed-writing
  + all 9 surface keywords present, zero "1078" — GATE-OK (350 lines, meets min_lines 350)
- Task 2 gate: 21 `pipelite` sections (≥15), templates/docs/batch/runs present, CHANGELOG
  zero "unreleased", README enumerates custom-fields/trash/runs — GATE-OK
- Task 3 gate: script executable + `bash -n` clean, env-gated, zero credential-shaped
  literals in script AND report (grep-proven), zero `pipelite … trash purge` invocations,
  guard functions present, report has `## Results` + SKIPPED, all 3 deferred-items files
  annotated RESOLVED/CARRIED — GATE-OK
- Credential guard smoke test: unset env → exit 2 with usage message
- Structural dry run (unreachable server): guards PASS; scenarios fail honestly with
  observed-vs-expected; exit 1 — no false PASSes
- **Task 4 live E2E (orchestrator-run, 2026-09-05): 21 PASS / 0 FAIL** across all 4
  SC-2 scenarios; report finalized (docs/e2e-v1.1-report.md, commit b4bcf51)
- Full `cargo test` after the E2E-found bug fixes: **651 passed, 0 failed**
  (650 baseline + 1 regression test for the batch-create envelope/partial-meta fix)

## Known Stubs

None blocking. `docs/e2e-v1.1-report.md` was a TEMPLATE at authoring time by design and
is now FILLED from the live run (Task 4, commit b4bcf51). The Forbidden (non-admin) live
checks remain SKIPPED by design (13-CONTEXT: no non-admin key available; stub-covered by
`tests/error_layer_stub_test.rs`) — recorded in the report.

## TDD Gate Compliance

Not applicable — this plan is type `execute` (docs/tooling, no `tdd="true"` tasks).

## Self-Check: PASSED

- docs/SKILL.md (350 lines, 15 sections, zero "1078") — FOUND
- docs/api-reference.md (903 lines, 21 pipelite sections incl. the 4 added) — FOUND
- README.md (659 lines, all new groups enumerated + structure listing) — FOUND
- CHANGELOG.md (121 lines, released header, zero "unreleased") — FOUND
- scripts/e2e-v1.1.sh (executable, env-gated, guards, 4 scenarios) — FOUND
- docs/e2e-v1.1-report.md (FILLED from live run — 4 scenarios, Result: PASS) — FOUND
- 3 deferred-items files annotated (RESOLVED 2026-09-04 / CARRIED) — FOUND
- Commits b0576ee, b61abcb, b408a61, 30939ec (E2E bug fixes + regression test), b4bcf51
  (filled report) in git log — FOUND
- cargo test 651/651 green (650 baseline + 1 E2E regression test) — VERIFIED
- Live E2E 21 PASS / 0 FAIL (orchestrator-run, 2026-09-05) — VERIFIED via finalized report
