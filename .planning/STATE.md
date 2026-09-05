---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Server v2 Parity
status: Awaiting next milestone
stopped_at: Phase 13 complete (13-01 + 13-02) — live E2E executed 2026-09-05 (21 PASS / 0 FAIL); suite 651/651 green; ready for milestone verification
last_updated: "2026-09-05T02:51:33.987Z"
last_activity: 2026-09-05 — Milestone v1.1 completed and archived
progress:
  total_phases: 7
  completed_phases: 7
  total_plans: 18
  completed_plans: 18
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-09-02)

**Core value:** Users can manage their entire Pipelite CRM from the terminal — fast, scriptable, and composable with other tools.
**Current focus:** Milestone complete

## Current Position

Phase: Milestone v1.1 complete
Plan: —
Status: Awaiting next milestone
Last activity: 2026-09-05 — Milestone v1.1 completed and archived

## Performance Metrics

**Velocity:**

- Total plans completed: 36 (v1.0)
- Average duration: — (not tracked)
- Total execution time: —

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| v1.0 (Phases 1-6) | 18 | 34 tasks | — |
| Phase 07 P03 | 10 min | 2 tasks | 9 files |
| Phase 7 P04 | 9 min | 2 tasks | 3 files |
| 7 | 5 | - | - |
| Phase 8 P02 | 27 min | 3 tasks | 23 files |
| 8 | 2 | - | - |
| Phase 09 P01 | 13 min | 3 tasks | 10 files |
| Phase 09 P02 | 24 min | 3 tasks | 18 files |
| Phase 09 P03 | 12 min | 2 tasks | 8 files |
| 9 | 3 | - | - |
| Phase 10 P01 | 23min | 3 tasks | 16 files |
| 10 | 1 | - | - |
| Phase 11 P02 | 15min | 3 tasks | 13 files |
| 11 | 3 | - | - |
| Phase 12 P01 | 25min | 3 tasks | 16 files |
| Phase 12 P02 | 32min | 3 tasks | 19 files |
| 12 | 2 | - | - |
| Phase 13 P01 | 13min | 3 tasks | 1 files |
| Phase 13 P02 | 20min | 3 tasks | 7 files |
| 13 | 2 | - | - |

## Accumulated Context

### Decisions

Full log in PROJECT.md Key Decisions table. Recent decisions affecting v1.1:

- Batch operations lead the milestone (Phase 7): drafted plans reused; establishes `batch.rs` (BatchOutcome, stdin parsing, `confirm_destructive`) consumed by Phases 11-12
- Foundations (Phase 8) promoted to the front per research — error layer/models/pagination are prerequisites for 403-heavy surfaces, not end-of-milestone cleanup
- `--force` (not `--yes`) is the confirmation-bypass flag — codebase consistency with `workflows delete`
- Notes is a top-level command group with entity-type positional, not nested ×4 under entities
- All v1.1 fixes land in Phase 8 (research ordering), not last — dead flags actively lie to users
- [Phase 7]: Batch pattern replicated to pipelines/stages/workflows verbatim from deals/orgs reference; workflows batch delete skips confirmation on force/no-input/non-TTY while single-delete refusal behavior preserved
- [Phase 7]: Pipeline mutations invalidate KEY_PIPELINES + stages_ cache prefix; batch stage updates use stages_ prefix invalidation (multi-pipeline safe)
- [Phase 7]: Batch integration suite (26 tests/3 files) verifies all 7 entities; cmd() helpers set PIPELITE_SERVER_URL (live override) so unreachable-server error tests stay config-hermetic
- [Phase 8]: Dead flags stay defined with hide=true (parse-then-error: exit 2 + replacement hint before any HTTP) instead of clap removal — hints must survive
- [Phase 8]: workflows --active is a client-side filter with one locked stderr warning and meta rebuilt from filtered rows; dead WorkflowsListParams.active removed
- [Phase 8]: expanded Map uses flatten+default+skip_serializing_if on all 7 Base models — expand payloads render in JSON automatically, no synthetic key when empty
- [Phase 8]: Deal/Stage position are f64 (server numeric column) — 10000.0 rendering documented in CHANGELOG; Phase 12 CFLD-01 must be f64 from birth
- [Phase 9]: Runs list probe discipline locked by tests: exactly one limit-1 dry_run=true probe only on an empty page without the flag; statuses hint takes precedence (no probe when --status set)
- [Phase 9]: step_duration uses to_text_en(Rough, Present): HumanTime Display renders positive deltas as 'in N minutes' (wrong tense for durations); sub-minute renders 'now'
- [Phase 9]: runs detail requires --workflow (server path /workflows/{id}/runs/{runId}, no run-to-workflow lookup); render_detail seam exposed for 09-02 watch reuse
- [Phase 9]: watch is a fixed 2s poll, no timeout — default exit 0 on ANY terminal state, --exit-status maps failed/unknown to 1, waiting keeps polling, Ctrl-C = default SIGINT (shell 130)
- [Phase 9]: templates create resolves the trigger from exactly one source pre-HTTP; --workflow maps triggers[0]->trigger with a quiet-suppressible multi-trigger warning; --stdin posts raw via post_workflow_template_raw (verbatim)
- [Phase 9]: stub helper is a 4-tuple (heads + full bodies); content-length parses the current request's head — hidden templates update stays parse-then-error exit 2
- [Phase 9]: docs get_docs builds a local headerless reqwest client — the shared authenticated client leaks the API key via default_headers onto every request; the no-auth contract is wire-tested on the raw request head
- [Phase 9]: docs --save refuses overwrites pre-HTTP (zero requests, exit 2) unless --force; 404/Api docs errors re-wrap at the command layer preserving server detail with the locked server-version hint; --format accepted but ignored per CONTEXT
- [Phase 10]: notes is the first parent-scoped sub-resource — collection routes carry the parent segment, item routes (PATCH/DELETE) carry ONLY the note ID; parent args are grammar-locked (validated then unused)
- [Phase 10]: resolve_body locks the input precedence (--body > @file/@- > --stdin > prompt); XOR explicit-source guard BEFORE any read — @- + --stdin is two sources exit 2; prompting suppressed under --dry-run
- [Phase 10]: table-only truncation (flatten newlines + ~80-char truncate_with_ellipsis in the table value builder); json/plain/csv keep full raw text; mutation success output keys on resolved format (json → render_single, else confirmation line)
- [Phase 11]: Trash type tokens are dual-vocabulary — entity_type (singular, display) vs type (plural, URL round-trip); only normalized plural tabs ever reach a URL; trash is never cached
- [Phase 11]: Purge non-TTY no-force refusal is InvalidInput exit 2 BEFORE the fan-out (zero HTTP, pinned on unreachable server) — stricter than webhooks delete's Validation exit-1; restore 403 keeps surface 'general' (P6); 404 re-wrap preserves server detail + not-in-trash hint
- [Phase 11]: --all and purge victim fan-out break on all four conditions (empty page, partial page, accumulated>=total, offset>10000) — trust page emptiness, never total reachability (P3/S6)
- [Phase 12]: definitions create maps --key to the wire 'name' (blob keys are definition names); POST body is exactly {name, entity_type, type, required, show_in_list}(+config.options) — no position key ever (server auto-assigns max+10000, PUT-only)
- [Phase 12]: select is a CLI alias normalized to single_select; --options required for the select family, rejected for all other types; 9 entity aliases normalize to the 4 server tokens pre-HTTP (exit 2)
- [Phase 12]: definition create/update/delete all invalidate the custom_fields_ cache prefix (two-run stub-proven); filtered lists warm KEY_CUSTOM_FIELDS_<token> — the 12-02 resolver interface
- [Phase 12]: typed writing runs through ONE shared resolver (src/custom_fields.rs) consumed by all 8 create/update handlers — number i64-first (price=4 stores JSON number 4, wire-pinned), strict boolean, option-validated select/multi_select, formula writes refused exit 2, unknown names sent as strings with one aggregated quiet-suppressible warning
- [Phase 12]: --custom-field-json gives verbatim raw-object passthrough; it and --custom-field are mutually exclusive with each other and --stdin at exit 2 pre-HTTP across all 8 handlers (create exclusivity normalized Validation->InvalidInput); batch::parse_custom_fields deleted; --dry-run is cache-only (zero HTTP, strings + note when cold, typed when warm)
- [Phase 13]: Contract matrix (61 tests) proves all Phase 7-12 surfaces honor the v1.0 global contract - zero violations found, zero src changes; first fully-green hardening run
- [Phase 13]: templates create --workflow dry-run GET is a read exempt from zero-mutation (Phase 9 pinned) - matrix row authored with --trigger for pure zero-HTTP; batch summary pinned on the locked Phase 7 failure-path shape (mixed batch under --quiet)
- [Phase 13]: api/mod.rs is 1805 lines (< ~2000 threshold) — KEEP SINGLE FILE, no split; watch item closed for the milestone audit
- [Phase 13]: live E2E v1.1 EXECUTED by the orchestrator 2026-09-05 — 21 PASS / 0 FAIL across 4 SC-2 scenarios; report finalized (docs/e2e-v1.1-report.md, b4bcf51); Forbidden live checks SKIPPED by design (no non-admin key; stub-covered)
- [Phase 13]: live E2E surfaced 2 real bugs, fixed in 30939ec + regression test — batch_create_{deals,orgs,people} double-unwrapped the {data} envelope (src/api/mod.rs); PaginationMeta rejected partial batch meta (#[serde(default)] in src/api/models.rs); suite 651/651 green
- [Phase 13]: deferred-items dispositioned — env-dependent test trio RESOLVED via WR-03 hermetic env (verified with real config present); 3 build warnings CARRIED; config unit-test flake CARRIED (reproduced 4/20 runs live)
- [Phase 13]: --continue-on-error documented as BUILT-IN batch semantics, not a flag (plan inventory said flag; --help/src prove unconditional continue + summary); batch stdin is JSON array only (no NDJSON)

### Pending Todos

None.

### Blockers/Concerns

- ~~ACTIVE (13-02 Task 4):~~ RESOLVED 2026-09-05 — orchestrator ran the live E2E with session credentials (21 PASS / 0 FAIL), report finalized, 2 E2E-found bugs fixed (30939ec)
- Orchestrator to rename `.planning/phases/01-batch-operations-for-all-entities/` → `07-*` (plans + context docs) after roadmap approval
- Research flags to resolve during planning: webhook PUT semantics + trash `linked_parents` rendering (Phase 11), definition soft-delete marker + `multi_select` shape (Phase 12)
- ~~`api/mod.rs` size watch item~~ DECIDED 2026-09-04: 1805 lines < ~2000 → keep single file
- 3 build warnings carried to milestone audit (WorkflowRunTrigger, TTL_WORKFLOWS, get_workflows_cached — deletion candidates); config unit-test flake carried (4/20 live reproduction)
- Dead-flag removals in Phase 8 are breaking — changelog callout required

## Deferred Items

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| Differentiator | WRUN-04, AUDT-03, WHOK-04, CFLD-04 | Future requirements | v1.1 planning |
| Server-blocked | SRV-01 (global search), SRV-02 (file upload) | Request upstream | v1.1 planning |
| Maintenance | comfy-table 7→8 upgrade | Post-milestone | v1.1 planning |

## Session Continuity

Last session: 2026-09-05
Stopped at: Phase 13 complete (13-01 + 13-02) — live E2E executed 2026-09-05 (21 PASS / 0 FAIL); suite 651/651 green; ready for milestone verification
Resume file: .planning/phases/13-integration-hardening-docs-refresh/13-02-SUMMARY.md

## Operator Next Steps

- Start the next milestone with /gsd-new-milestone
