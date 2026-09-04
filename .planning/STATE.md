---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Server v2 Parity
status: executing
stopped_at: Completed 11-02-PLAN.md (trash surface — 24 stub tests, 474 passing)
last_updated: "2026-09-04T02:58:25.468Z"
last_activity: 2026-09-04
progress:
  total_phases: 7
  completed_phases: 4
  total_plans: 14
  completed_plans: 13
  percent: 57
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-09-02)

**Core value:** Users can manage their entire Pipelite CRM from the terminal — fast, scriptable, and composable with other tools.
**Current focus:** Phase 11 — webhooks-trash-audit

## Current Position

Phase: 11 (webhooks-trash-audit) — EXECUTING
Plan: 3 of 3
Status: Ready to execute
Last activity: 2026-09-04

Progress: [█████████░] 93%

## Performance Metrics

**Velocity:**

- Total plans completed: 29 (v1.0)
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

### Pending Todos

None.

### Blockers/Concerns

- Orchestrator to rename `.planning/phases/01-batch-operations-for-all-entities/` → `07-*` (plans + context docs) after roadmap approval
- Research flags to resolve during planning: webhook PUT semantics + trash `linked_parents` rendering (Phase 11), definition soft-delete marker + `multi_select` shape (Phase 12)
- `api/mod.rs` size watch item: split into `api/client.rs` + `api/methods/` if it crosses ~2,000 lines
- Dead-flag removals in Phase 8 are breaking — changelog callout required

## Deferred Items

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| Differentiator | WRUN-04, AUDT-03, WHOK-04, CFLD-04 | Future requirements | v1.1 planning |
| Server-blocked | SRV-01 (global search), SRV-02 (file upload) | Request upstream | v1.1 planning |
| Maintenance | comfy-table 7→8 upgrade | Post-milestone | v1.1 planning |

## Session Continuity

Last session: 2026-09-04T02:58:25.437Z
Stopped at: Completed 11-02-PLAN.md (trash surface — 24 stub tests, 474 passing)
Resume file: None
