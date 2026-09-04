---
phase: 11-webhooks-trash-audit
plan: 03
subsystem: api
tags: [audit, admin-gated, forbidden-hints, passthrough-filters, pagination-clamp, rust, clap, reqwest, stub-server-tests, tdd]

# Dependency graph
requires:
  - phase: 08-foundations-error-layer-models-pagination
    provides: forbidden_hint table ("audit" hint pre-registered — zero registration work), Phase 8 422 errors[] parser, ApiListResponse envelope
  - phase: 09-workflow-runs-templates-docs
    provides: tests/common/mod.rs head+body-capturing stub server, cmd()/cmd_with_server() hermetic helpers
  - phase: 10-notes
    provides: no---all help precedent (notes list), table-only truncation pattern
  - phase: 11-01
    provides: RFC 7807 403 fixture family, top-level group wiring pattern, sequential shared-wiring discipline
  - phase: 11-02
    provides: sequential shared wiring (cli/commands/main), empty-hint + multi-invocation stub scripting precedent
provides:
  - top-level `audit` command group (single `list` subcommand) against GET /api/v1/audit
  - AuditEntry (verbatim changes Value, plain-String action, serde-defaulted actor id Options) + audit_table_config (created_at/actor/action/entity)
  - PipeliteClient::list_audit — snake_case query pairs appended ONLY for Some(non-empty) filters (P4), limit/offset always sent, surface "audit"
  - client-side limit clamp 1..=100 (wire-proven at both bounds: 0→1, 150→100); deliberate NO --all
  - gate-before-validation 403 probes pinned end-to-end (hint renders with NO filters AND with invalid filters)
affects: [future AUDT-03 diff rendering (changes payload already verbatim via --json), SC-5 phase verification]

# Tech tracking
tech-stack:
  added: [] # zero new crates — clap/reqwest/serde already pinned (T-11-SC holds)
  patterns: [client-side clamp to mirror server parsePagination, Some(non-empty) query-pair omission, table-only composed cells (actor/entity) with full json passthrough, gate-before-validation ordering probes, plain-String over enum for server-owned closed unions]

key-files:
  created:
    - src/cli/audit.rs
    - src/commands/audit/mod.rs
    - src/commands/audit/list.rs
    - tests/audit_stub_test.rs
  modified:
    - src/api/models.rs
    - src/api/mod.rs
    - src/cli/mod.rs
    - src/commands/mod.rs
    - src/main.rs
    - tests/help_examples_test.rs
    - docs/api-reference.md

key-decisions:
  - "Wire keys are the server's snake_case query params (entity_type/entity_id/actor_kind/workflow_run_id) — the plan's test-2 wording said kebab-case, corrected to the plan's own RESEARCH route citation; verbatim passthrough = values unvalidated, keys server-shaped"
  - "Empty-flag omission lives in list_audit (client) — a query pair is pushed ONLY when the value is Some AND non-empty (P4); the handler passes Options straight through"
  - "--limit clamps client-side into 1..=100 so the wire always carries the effective value (T-11-11); no --all — the server's only offset bound is the global 1e6 clamp, after_help says iterate --offset"
  - "changes stays a verbatim serde_json::Value, action is a plain String (all 4 union values incl. merged render, future values too) — the CLI never interprets entry payloads (AUDT-03 deferred)"
  - "Table cells compose actor (kind + first-present id) and entity (type/id) truncated at 40 chars; changes NEVER renders in the table — --format json only; audit is never cached"
  - "Forbidden probes pin the gate-before-validation ordering: the registered audit hint renders identically with no filters and with an invalid filter value (the bogus value provably reaches the wire)"

requirements-completed: [AUDT-01, AUDT-02]

# Metrics
duration: 16min
completed: 2026-09-04
---

# Phase 11 Plan 03: Audit Log Viewer Summary

**Read-only admin-gated `audit list` with four verbatim passthrough filters (empty flags omitted from the wire), client-clamped 1..=100 pagination with deliberate NO --all, timestamp/actor/action/entity table cells with json-only verbatim changes payload, and gate-before-validation 403 probes — 493 tests green.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-09-04T03:01:43Z
- **Completed:** 2026-09-04T03:17:45Z
- **Tasks:** 2 (task 1 TDD: RED f862f52 → GREEN 65f52eb)
- **Files modified:** 11

## Accomplishments

- AuditEntry matches the verified serializer: `changes` kept as a verbatim serde_json::Value (AUDT-03 diff rendering deliberately NOT built), `action` a plain String tolerating all four union values including `merged`, and the three actor id fields serde-defaulted (an omitted key still parses; exactly one is populated per actor_kind) — model unit tests pin the fixtures
- `list_audit` builds the query with snake_case server keys, appending each filter ONLY when Some AND non-empty (P4 — the server 422s `entity_id=`), always sends limit/offset, and passes surface "audit" so a non-admin 403 renders the pre-registered "The audit log requires an admin API key." hint — zero hint registration work
- `audit list` clamps `--limit` client-side into 1..=100 (wire-proven: 0→1, 150→100, unclamped values never reach the wire), passes the four filters through with NO client-side enum validation (server 422s flow through the Phase 8 errors[] parser untouched), and has NO --all — after_help documents all 6 entity types, 5 actor kinds, 4 actions, newest-first server sort, and "iterate --offset (no --all)"
- Display contract: table composes truncated (40-char) actor and entity cells — created_at renders via the pre-existing relative-time formatter — while `--format json` keeps the FULL entry: verbatim changes payload, every actor id, full 100-char entity ids (truncation test pins both directions); empty list prints one quiet-suppressible stderr hint; audit never cached
- The two PINNED 403 probes prove the gate-before-validation ordering end-to-end: with NO filters and with `--entity-type bogus` (asserted to reach the wire), the identical audit hint renders — the CLI adds no validation layer that could reorder or mask the gate (T-11-10)
- Help pages are truthful (admin gating at group + list level, no --all advertised anywhere in the group help) and docs/api-reference.md gained a complete `pipelite audit` section before Error Codes

## Task Commits

Each task was committed atomically (TDD: RED then GREEN for task 1):

1. **Task 1: AuditEntry model, list_audit client (empty-omit + clamp), audit list handler + Forbidden probes + wiring** — `f862f52` (test, RED: 13/13 failing, none vacuous) + `65f52eb` (feat, GREEN: 13 stub tests + 3 model unit tests)
2. **Task 2: Help truthfulness tests + docs/api-reference.md Audit section + full-suite gate** — `14f32c0` (feat: 3 help tests + docs, 493/493 across 32 suites)
3. **Task 2 follow-up:** `ff6d7be` (fix: staged the missed list_default test fix the committed suite requires)

## Files Created/Modified

- `src/api/models.rs` — AuditEntry (verbatim changes Value, plain action String, defaulted actor id Options), audit_table_config + 3 unit tests
- `src/api/mod.rs` — list_audit with Some(non-empty) query-pair omission, contract doc comments (admin gate before validation, server-fixed sort), surface "audit"
- `src/cli/audit.rs` — AuditCommands::List; 4 filter flags + clamped --limit/--offset/--fields; after_help with full enums, paging note, json pointer
- `src/commands/audit/mod.rs` — single-arm dispatch, zero validation (server owns filters — Responsibility Map)
- `src/commands/audit/list.rs` — 1..=100 clamp, composed+truncated actor/entity cells, quiet-suppressible empty hint, no cache, no --all, no changes column
- `src/cli/mod.rs`, `src/commands/mod.rs`, `src/main.rs` — Audit wiring after Trash with admin-gating group after_help
- `tests/audit_stub_test.rs` — 13 stub tests (wire honesty ×5, json contract ×2, PINNED 403 probes ×2, 422 passthrough, merged tolerance, truncation, empty hint)
- `tests/help_examples_test.rs` — 3 new truthfulness tests (19 total in file)
- `docs/api-reference.md` — `## \`pipelite audit\`` section (filters + enums, clamp, no---all rationale, json-only changes, admin gating + hint)

## Decisions Made

- Wire query keys are snake_case (`entity_type=deal`) per the plan's own RESEARCH route citation — the behavior-test wording said kebab-case (`entity-type=deal`), which would have been a real wire bug (the server reads snake_case); tests assert the server-shaped keys
- `list_default` asserts the table timestamp via the relative-time rendering ("ago") and the exact ISO value via a second --json invocation — the table layer's `*_at` → relative-time formatting is a pre-existing codebase convention (format.rs), and the multi-invocation stub needs one scripted page per invocation
- The table-config gate (`grep -c 'fn audit_table_config' == 1`) is kept satisfiable by naming the unit test `table_config_pins_default_display_columns` (the call site has no `fn ` prefix)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test 2's asserted wire keys corrected from kebab-case to snake_case**
- **Found during:** Task 1 (RED authoring)
- **Issue:** The behavior spec said the head contains `entity-type=deal` etc., but the plan's own server contract (RESEARCH § Audit route citation: `GET /api/v1/audit?entity_type&entity_id&actor_kind&workflow_run_id&offset&limit`) pins snake_case keys — kebab keys would be ignored by the server (silent filter loss)
- **Fix:** Client sends snake_case keys; tests assert `entity_type=deal`, `entity_id=d1`, `actor_kind=workflow_run`, `workflow_run_id=wr9`
- **Files modified:** tests/audit_stub_test.rs, src/api/mod.rs
- **Committed in:** f862f52 / 65f52eb

**2. [Rule 3 - Blocking] list_default asserted the raw ISO timestamp against a relative-time table cell**
- **Found during:** Task 1 (GREEN — 12/13, list_default failed)
- **Issue:** The output layer renders every `*_at` field as relative time ("2 days ago") — a pre-existing convention; the plan's "table contains ... the created_at value" expectation cannot hold in table mode
- **Fix:** Table asserts "ago" (timestamp column renders); exact ISO value asserted via a second --json invocation with a second scripted page
- **Files modified:** tests/audit_stub_test.rs
- **Committed in:** ff6d7be

**3. [Rule 3 - Blocking] Task 1 gate `grep -c 'fn audit_table_config' == 1` counted the unit test name**
- **Found during:** Task 1 gate run
- **Issue:** `fn audit_table_config_has_display_columns` contains the substring, making the count 2
- **Fix:** Renamed the test to `table_config_pins_default_display_columns` (same class of fix as 11-01 deviation #2 / 11-02 deviation #3)
- **Files modified:** src/api/models.rs (test name only)
- **Committed in:** 65f52eb

**4. [Rule 3 - Blocking] list_default test fix missed its GREEN-commit staging**
- **Found during:** Task 2 close-out `git status` review
- **Issue:** The GREEN commit staged only production files; the fixed test file stayed in the working tree — a clean checkout would fail the committed audit_stub_test
- **Fix:** Dedicated fix commit staging tests/audit_stub_test.rs (same pattern as 11-01's 9dac80b / 11-02's 5225ae8)
- **Files modified:** tests/audit_stub_test.rs
- **Verification:** suite re-run from the clean committed tree: 32 suites ok / 0 failed
- **Committed in:** ff6d7be

---

**Total deviations:** 4 auto-fixed (1 wire-contract correction, 3 test/gate-mechanics corrections)
**Impact on plan:** All four were mechanical corrections — no behavioral contract changed. Every must_have, pinned gate, and success criterion met as written (with test 2's keys corrected TO the server contract).

## Issues Encountered

None beyond the deviations above. Full suite green: 493 passed / 0 failed across 32 suites (474 pre-existing + 13 audit stub + 3 model units + 3 help tests). A shell backtick in one Task 2 commit message was amended within the same breath (14f32c0) — content unaffected.

## Threat Surface Scan

No new trust boundaries beyond the plan's threat model: the only endpoint is the modeled GET /api/v1/audit, no auth paths added, no schema changes, no new crates (T-11-SC holds). T-11-10 (admin-probing oracle) mitigated and stub-proven: the hint renders identically for absent AND invalid filters, and the invalid value provably reaches the wire — no response distinguishes gate state from filter state. T-11-11 (input tampering) mitigated and wire-proven: empty flags never reach the query, limit clamps at both bounds, all other values pass through reqwest's query encoder for the server to 422. T-11-12 (changes payload disclosure) accepted per plan: never rendered in table cells, json-only on explicit request, never cached.

## User Setup Required

None — no external service configuration required. (Exercising `audit list` against a live server requires an admin API key — server-side provisioning, not CLI setup.)

## Known Stubs

None — no placeholder data paths; every rendered value comes from the wire or the locked display contract. AUDT-03 diff rendering is a DEFERRED requirement (not a stub): the changes payload already reaches users verbatim via `--format json`.

## Next Phase Readiness

- Phase 11 complete: all three plans (11-01 webhooks, 11-02 trash, 11-03 audit) executed; SC-5's three admin-gated hint surfaces each stub-verified (webhook ownership hint, purge hint, audit hint)
- The confirmation hierarchy reference set is complete: purge (exit-2 zero-HTTP refusal) > webhook delete (exit-1) > restore/audit list (none — read-only)
- `api/mod.rs` at ~1,710 lines — the ~2,000-line split watch item remains open but untriggered
- AUDT-03 (changes diff rendering) is the natural seed for a future phase — the verbatim payload is already flowing through --json

---
*Phase: 11-webhooks-trash-audit*
*Completed: 2026-09-04*
