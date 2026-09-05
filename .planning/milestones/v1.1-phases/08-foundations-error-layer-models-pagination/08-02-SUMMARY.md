---
phase: 08-foundations-error-layer-models-pagination
plan: 02
subsystem: models-lists-cli-truth
tags: [models, serde-flatten, pagination, dead-flags, breaking-change, tdd]
requires:
  - Phase 8-01 error layer (Forbidden variant, parse_rfc7807, body-carrying stub pattern)
  - Phase 7 exit-code contract (InvalidInput → 2) + batch cmd() stub pattern
provides:
  - f64 position fields (Deal Option<f64>, Stage f64) — fractional positions deserialize
  - "#[serde(flatten)] `expanded` map on all 7 Base models — --expand payloads survive to output"
  - StagesListParams.pipeline_id: Option<String> — stages list all-mode (single unfiltered call)
  - WorkflowsListParams without `active`; workflows list --active client-side filter + locked stderr warning
  - Ceiling warning text locked in all 7 fetch_all loops (guard kept conditional)
  - 5 dead flags parse-then-error (hidden, InvalidInput exit 2, replacement hint, zero HTTP)
  - WorkflowCreate without `active` — no prompt, no active key on flag or stdin path
  - CHANGELOG.md v1.1 Breaking Changes section
  - Capturing TcpListener stub helper (request head + body) in tests/dead_flags_test.rs & stages_integration.rs
affects:
  - Phase 12 (CFLD-01: custom_field_definitions.position must be f64 from birth — doc note pinned in models.rs)
  - Phase 13 docs pass (docs/api-reference.md: stages list --pipeline now optional; removed flags)
tech-stack:
  added: []
  patterns:
    - "#[serde(flatten, default, skip_serializing_if)] Map passthrough for expand payloads"
    - parse-then-error guards as first handler statements (zero HTTP on structural rejection)
    - client-side filtering with rebuilt PaginationMeta + mandatory stderr warning
    - request-capturing TcpListener stub for asserting exact serialized bodies
key-files:
  created:
    - CHANGELOG.md
    - tests/dead_flags_test.rs
  modified:
    - src/api/models.rs
    - src/api/mod.rs
    - src/commands/stages/list.rs
    - src/commands/workflows/list.rs
    - src/commands/deals/list.rs
    - src/commands/orgs/list.rs
    - src/commands/people/list.rs
    - src/commands/activities/list.rs
    - src/commands/pipelines/list.rs
    - src/commands/workflows/create.rs
    - src/commands/pipelines/create.rs
    - src/commands/stages/create.rs
    - src/commands/cache/refresh.rs
    - src/commands/dashboard.rs
    - src/prompt.rs
    - src/cli/people.rs
    - src/cli/stages.rs
    - src/cli/pipelines.rs
    - src/cli/workflows.rs
    - tests/stages_integration.rs
    - tests/workflows_integration.rs
    - tests/deals_integration.rs
    - tests/people_integration.rs
decisions:
  - expanded map skips serialization when empty — exactly-typed payloads render byte-identical keys (no synthetic key)
  - workflows --active filter rebuilds PaginationMeta from filtered rows — footer never claims more than shown
  - ceiling warning fires only under the pre-existing `if total > max_records` guard — negative test pins the conditional
  - dead flags stay defined with hide=true (parse-then-error) so the handler names the replacement — clap rejection can't carry hints
  - macro-generated serde attribute rejected (derive can't see through macro invocations) — inline field ×7
metrics:
  duration: 27 min
  completed: 2026-09-03
  tasks: 3
  files: 23
---

# Phase 8 Plan 2: Models Truth, Honest Lists, Dead-Flag Removals Summary

**Position fields f64 + `--expand` flatten passthrough on all 7 Base models, stages all-mode, workflows `--active` as a warned client-side filter, the `--all` ceiling warning in all 7 list loops, and 5 dead flags now rejecting loudly (exit 2 + replacement hint, zero HTTP, hidden from help) — recorded in a new CHANGELOG.md v1.1 Breaking Changes section.**

## What Was Built

### Task 1 — Models truth: f64 positions + expand passthrough (TDD: e26c415 RED → d84f119 GREEN)
- **FIX-03**: `Deal.position: Option<f64>`, `Stage.position: f64`. Fixture tests pin the FIX-03 break (10010.5), int-token losslessness (10000 → 10000.0), the `serde_json` `"position":10000.0` rendering nuance (Pitfall 5 — accepted and documented, not hand-rolled away), and stage 2.0/3.0. Doc note pins that Phase 12's `custom_field_definitions.position` (`numeric(20,10)` + parseFloat) must be f64 from birth.
- **FIX-02**: `#[serde(flatten, default, skip_serializing_if = "Map::is_empty")] pub expanded: Map<String, serde_json::Value>` on all 7 Base models (Deal, Organization, Person, Activity, Pipeline, Stage, Workflow). Tests prove: `owner` sibling payload lands in `expanded` and re-serializes top-level; exactly-typed payloads emit NO `expanded` key; Workflow/Stage/Person unknown keys survive. `#[serde(default)]` required (Pitfall 6).
- **Deviation note (Rule 1, intra-task):** first implementation used a `macro_rules!` field-expander — serde derive can't see through macro invocations (109 errors, `Deserialize` not satisfied); replaced with the inlined field ×7, which is also what the plan's grep gate counts.
- Renderers untouched: JSON shows expand keys automatically; table/csv/plain via `--fields` (`format_value` compact-JSON fallback).
- Full bin suite 128 passed at GREEN.

### Task 2 — Honest lists (TDD: 8f35653 RED → 3c478af GREEN)
- **FIX-04**: `StagesListParams.pipeline_id: Option<String>`; pair pushed only when `Some`. `stages list` without `--pipeline` makes one unfiltered call (stub asserts request line has NO `pipeline_id=` and counter == 1); rows distinguishable via the `pipeline_id` column (asserted in stdout). `--pipeline pl_1` still filters (pair asserted present). Required-flag `Validation` deleted; `Option<String>` threaded through `fetch_page`/`fetch_all`; `cli/stages.rs` after_help + arg doc updated to all-mode.
- **FIX-01 (list side)**: `WorkflowsListParams` lost the dead `active` field and its query push entirely. `workflows list --active` now: emits exactly one locked stderr line (`warning: --active filters client-side after fetching all records`), filters in BOTH paths, and rebuilds `PaginationMeta` from filtered rows so the footer never claims more than displayed. Stub test: mixed true/false rows → only active rows on stdout, exact warning on stderr, exactly 1 request. Without `--active`: unfiltered and silent (regression).
- **FIX-06**: all 7 `fetch_all` loops — the `if total > max_records` guard kept verbatim, only the message replaced with `warning: --all stopped at 1000 records (server ceiling); results may be incomplete` (unconditional stderr inside the guard). Stub tests: total=1243 → warning + exit 0 + exactly 10 requests; total=450 → NO warning + exactly 5 requests (negative case pinning the conditional guard).
- **Rule 3 fixes**: the param type changes broke 5 more construction sites (cache/refresh.rs, dashboard.rs ×2, prompt.rs ×2) — `Some()`-wrapped and `active: None` removed; mechanical, verified by build + full suite.

### Task 3 — Dead flags, workflows create truth, CHANGELOG (TDD: 296ea8f RED → a1e0975 GREEN)
- **CLI defs**: all 5 sites get `hide = true` + `/// [REMOVED v1.1]` docs — people list `--org`/`--owner`, `WorkflowsCreateArgs.active` (list's `--active` kept, line 63-65 untouched), pipelines/stages create `--custom-field`. People list after_help no longer advertises the removed flags.
- **Parse-then-error guards** as the FIRST statements of each handler (before table config, cache, stdin branch, any HTTP — Pitfall 7), all `CliError::InvalidInput` (exit 2):
  - people list: `--org/--owner were removed…` + "filter client-side, e.g. with jq" hint
  - pipelines/stages create: `--custom-field was removed…` + unaffected-surfaces hint
  - workflows create: `--active was removed on create…` + `pipelite workflows update <id> --active true` activation hint
- **Pitfall 4 closed in one change**: `dialoguer::Confirm` "Set workflow active?" block deleted; `args.active` removed from `has_flags`; `WorkflowCreate` built without `active`; `WorkflowCreate.active` field removed from models.rs (server ignores it — A3: stdin JSON carrying `active` still deserializes, new unit test proves it and that the key is never re-serialized). `Workflow.active` and `WorkflowUpdate.active` untouched (both live; grep gate pins exactly one of each).
- **tests/dead_flags_test.rs** (11 tests): 5 removal tests assert exit 2 + hint fragments + NO "Connection failed" against the unreachable 127.0.0.1:1 server (zero-HTTP proof, T-08-04); 4 help-hiding assertions; 2 stub success tests (flag path non-TTY: success + no `"active"` in captured request body; stdin path with `"active":true` in the payload: dropped). The stub helper now captures full request head + body.
- **tests/people_integration.rs**: `people_list_help_shows_filter_flags` flipped to `.not().contains` for `--org`/`--owner` (checker-flagged blocker).
- **CHANGELOG.md** (new): `## v1.1 (unreleased)` → `### Breaking Changes` covering all 5 removed flags with replacements, `--active` client-side semantics + exact warning line, ceiling warning (exit unchanged), f64 `10000.0` rendering nuance, and the orgs `--owner` known-dead-but-out-of-scope limitation; plus `### Added` (RFC 7807, Forbidden hints, 409 hint) and `### Fixed` (stages all-mode, expand rendering) one-liners.

## Commits

| Commit | Type | Description |
|--------|------|-------------|
| e26c415 | test(08-02) | RED: failing tests for f64 positions + expand passthrough |
| d84f119 | feat(08-02) | GREEN: f64 positions + flatten `expanded` on all 7 Base models |
| 8f35653 | test(08-02) | RED: stages all-mode, --active client filter, ceiling warning tests |
| 3c478af | feat(08-02) | GREEN: stages all-mode, workflows --active filter, ceiling warning ×7 |
| 296ea8f | test(08-02) | RED: dead-flag removal + workflows create truth tests |
| a1e0975 | feat(08-02) | GREEN: 5 dead flags removed, prompt deleted, WorkflowCreate truth, CHANGELOG |

## TDD Gate Compliance

All three tasks followed RED → GREEN: each RED commit was run and observed failing before implementation (Task 1: 11 compile errors; Task 2: 3 new-behavior tests failing; Task 3: 10/11 dead-flags tests + flipped people test failing). GREEN commits made each suite fully pass. No REFACTOR commits needed.

## Deviations from Plan

**1. [Rule 1 - Bug] Macro-generated serde attribute broke the derive**
- **Found during:** Task 1 GREEN
- **Issue:** DRY-ing the flatten field via `macro_rules!` produced 109 errors — serde's derive cannot see through macro invocations in field position.
- **Fix:** inlined the attribute + field in all 7 structs (also required by the plan's own grep gate).
- **Files modified:** src/api/models.rs
- **Commit:** d84f119

**2. [Rule 3 - Blocking] 5 additional param-construction sites**
- **Found during:** Task 2 GREEN
- **Issue:** `StagesListParams`/`WorkflowsListParams` type changes broke src/commands/cache/refresh.rs, src/commands/dashboard.rs (×2), src/prompt.rs (×2).
- **Fix:** `Some(pipeline_id.to_string())` wraps; `active: None,` lines removed. Mechanical, no behavior change.
- **Files modified:** the 3 files listed
- **Commit:** 3c478af

**3. [Rule 1 - Bug] Unfiltered `--all` meta regression caught before commit**
- **Found during:** Task 2 GREEN
- **Issue:** first version of `apply_active_filter` in `fetch_all` passed `limit: 0` for the unfiltered path, silently changing footer behavior.
- **Fix:** build `PaginationMeta` with `limit: all_workflows.len()` before filtering; pass through unchanged when unfiltered.
- **Files modified:** src/commands/workflows/list.rs
- **Commit:** 3c478af

### Out-of-Scope Discoveries (logged to deferred-items.md, not fixed)

**[Rule-scope] Intermittent config unit-test flake — proven pre-existing.** `config::tests::env_var_precedence_server_url` / `config::tests::load_and_save_roundtrip` fail ~1-in-20 full-suite runs (which one varies), including at the pre-08-02 commit `7627c15` (verified via throwaway worktree: 19 pass / 1 fail over 20 runs). Same env/config-race family as the three failures documented in 08-01. NOT a regression of this plan.

## Verification Results

- Full `cargo test --no-fail-fast`: **23/25 test binaries green**; the only failures are the documented pre-existing environment-dependent tests (`cli_skeleton::config_set_parses_positional_args`, `cli_skeleton::global_flags_parse_without_error`, `deals_integration::deals_list_limit_zero_is_accepted`) — identical at pre-plan commits, plus the intermittent config flake above (also pre-existing, proven).
- `cargo test --bin pipelite`: 129 passed (10 new unit tests included).
- Task gates: `GATE-OK` on all three plan verify commands (f64/flatten greps ×7, ceiling-message ×7 with `:1` per file, zero `pub active` in api/mod.rs params, exactly one `pub active: bool` + one `pub active: Option<bool>` in models.rs, dialoguer-free workflows create, ≥5 `hide = true`, CHANGELOG body mention).
- Build: 4 warnings, all pre-existing (one fewer than before — the workflows list `total` warning was fixed in the rewritten file).

## Known Stubs

None — every behavior is wired to real HTTP semantics through the stub servers and asserted by request count and captured request bodies.

## Threat Model Compliance

- **T-08-04** (guards before HTTP): all 5 guards are the first handler statements; every removal test asserts stderr contains no "Connection failed" against an unreachable server.
- **T-08-05** (loud behavior change): locked stderr warning + meta rebuilt from filtered rows + CHANGELOG entry.
- **T-08-06** (DoS): 1000-record cap and batch-100 loop retained verbatim; negative test proves the warning is conditional.
- **T-08-07** (expand payloads): renders only keys already present in the authenticated server response; no new endpoints.
- **T-08-SC**: no new crates; stubs are hand-rolled `std::net::TcpListener`.

No security-relevant surface beyond the plan's threat model.

## Notes for Downstream Phases

- **Phase 12 (CFLD-01)**: `custom_field_definitions.position` must be `f64` — the doc comment at `Deal.position` in models.rs pins this.
- **Phase 13 docs pass**: `stages list --pipeline` is now optional; the 5 removed flags and `--active` client-side semantics need reflecting in docs/api-reference.md; CHANGELOG.md now exists as the breaking-changes record.
- The capturing stub (`spawn_capturing_stub_server`) in tests/dead_flags_test.rs returns full request head+body — reference pattern for asserting exact serialized payloads.

## Self-Check: PASSED

All created files exist (CHANGELOG.md, tests/dead_flags_test.rs); all 6 task commits verified in git log (e26c415, d84f119, 8f35653, 3c478af, 296ea8f, a1e0975). Full suite green except documented pre-existing environment failures.
