# Roadmap: Pipelite CLI

## Milestones

- ✅ **v1.0 MVP** — Phases 1-6 (shipped 2026-03-29)
- 🚧 **v1.1 Server v2 Parity** — Phases 7-13 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-6) — SHIPPED 2026-03-29</summary>

- [x] Phase 1: Foundation (2/2 plans) — completed 2026-03-25
- [x] Phase 2: Core CRUD and Output (3/3 plans) — completed 2026-03-25
- [x] Phase 3: Full Entity Coverage (3/3 plans) — completed 2026-03-25
- [x] Phase 4: Developer Experience (3/3 plans) — completed 2026-03-25
- [x] Phase 5: Power Features (5/5 plans) — completed 2026-03-28
- [x] Phase 6: Workflow API Integration (2/2 plans) — completed 2026-03-29

Full details: `.planning/milestones/v1.0-ROADMAP.md`

</details>

### 🚧 v1.1 Server v2 Parity (In Progress)

**Milestone Goal:** Bring the CLI to full parity with the upgraded Pipelite CRM server — batch operations, 8 new API surfaces (notes, workflow runs, webhooks, trash, custom fields, templates, audit, docs), and fixes for every dead flag and model mismatch — without breaking the v1.0 conventions (4 output formats, `--dry-run` = zero HTTP, hint-bearing errors, `--no-input`/`--quiet`/`--no-color`).

- [x] **Phase 7: Batch Operations** - Batch update/delete across all 7 entities with a script-friendly exit-code/summary contract (completed 2026-09-02)
- [x] **Phase 8: Foundations** - Error layer (403/409/RFC 7807), model + pagination fixes, dead-flag removal — shared infrastructure every later phase consumes (completed 2026-09-03)
- [x] **Phase 9: Workflow Runs, Templates & Docs** - Run list/detail/watch, template CRUD (no update), OpenAPI spec fetch (completed 2026-09-03)
- [x] **Phase 10: Notes** - Notes CRUD on deals/orgs/people/activities (completed 2026-09-03)
- [x] **Phase 11: Webhooks, Trash & Audit** - Webhook CRUD with show-once secret, trash restore/purge, audit log viewer (completed 2026-09-04)
- [x] **Phase 12: Custom Fields** - Definitions CRUD + type-aware `--custom-field` writing (completed 2026-09-04)
- [ ] **Phase 13: Integration Hardening & Docs** - Cross-phase verification of global-flag/convention contracts + docs refresh

## Phase Details

### Phase 7: Batch Operations
**Goal**: Users can update or delete many records of any entity in one command, with results scripts can trust
**Depends on**: Nothing (first phase of v1.1 — plans already drafted and validated)
**Requirements**: BATCH-01, BATCH-02, BATCH-03, BATCH-04
**Success Criteria** (what must be TRUE):
  1. User can pipe JSON objects (array or NDJSON) to `<entity> update --stdin` and update every record in one invocation, for all 7 entity types
  2. User can batch-delete by passing multiple IDs as arguments or piping IDs via `--stdin`
  3. With `--continue-on-error`, one failing item doesn't abort the batch: user sees per-item results and a final `N ok, M failed` summary that survives `--quiet`; exit code is 0 only when every item succeeded, 1 if any failed
  4. Structurally broken input (malformed JSON, missing IDs) is fully rejected before the first HTTP call — zero mutations on invalid input
  5. A rate-limited (429) item retries once per `Retry-After`; if it still fails it is classified as a failed item with detail — never a crash or silent skip
**Notes**: Plans drafted as 01-01…01-04 in `.planning/phases/01-batch-operations-for-all-entities/` — **to be renumbered 07-01…07-04 after roadmap approval** (orchestrator handles the directory/plan rename). Research amendment folded into 07-01: extract `confirm_destructive` into the shared batch utility (trash purge in Phase 11 needs the identical dry-run→confirm ordering). Internal order: batch utility before per-entity batch plans.
**Plans**: 5 plans — 07-01 (batch utility + deals), 07-02 (orgs/people/activities), 07-03 (pipelines/stages/workflows), 07-04 (integration tests), 07-05 (gap closure: BATCH-04 hardening — missing-ID pre-validation, 429 Retry-After retry-once, exit-2 structural contract)

Plans:
- [x] 07-01: (drafted as 01-01) Batch utility module + deals reference implementation
- [x] 07-02: (drafted as 01-02) Batch ops for orgs, people, activities
- [x] 07-03: (drafted as 01-03) Batch ops for pipelines, stages, workflows
- [x] 07-04: (drafted as 01-04) Integration tests for all batch operations

### Phase 8: Foundations — Error Layer, Models & Pagination
**Goal**: The CLI tells the truth — errors carry the server's real reason with actionable hints, permission tiers are distinguishable, models match server output, and no list truncates silently
**Depends on**: Nothing from Phase 7 (independent; ordered second because Phase 7's plans are pre-drafted). Hard gate for Phases 9-12 — research is emphatic this lands before any 403-heavy surface ships
**Requirements**: FIX-01, FIX-02, FIX-03, FIX-04, FIX-05, FIX-06
**Success Criteria** (what must be TRUE):
  1. A 403 on any admin-gated surface reports Forbidden with a surface-specific hint (e.g. "audit log requires an admin key") instead of "check your API key / run `pipelite init`"
  2. Server validation errors display the server's `detail`/`errors[]` message; 409 on an inactive workflow trigger maps to an actionable hint
  3. `deals list` succeeds against records with fractional `position` values (deserialization no longer breaks), and `stages list` without `--pipeline` lists all stages across pipelines
  4. `--expand` payloads appear in command output instead of being silently discarded
  5. Dead flags are gone: `people list --org/--owner` removed with hint, `workflows create --active` and dead `--custom-field` flags on pipelines/stages removed (breaking — changelog callout), `workflows list --active` filters client-side with a visible warning; `--all` pagination warns loudly on stderr at the record ceiling
**Plans**: 2 plans — 08-01 (error layer: RFC 7807 parser, Forbidden variant, 409 hint), 08-02 (models/pagination/dead-flags: f64 positions, expand passthrough, stages all-mode, flag removals + changelog)

Plans:
- [x] 08-01-PLAN.md — Error layer: RFC 7807 parsing, Forbidden variant with per-surface hints at all three 401/403 sites, 409 inactive-trigger hint
- [x] 08-02-PLAN.md — Models/pagination/dead-flags: f64 positions, --expand passthrough, stages all-mode, dead-flag removals + CHANGELOG, --all ceiling warning

### Phase 9: Workflow Runs, Templates & Docs
**Goal**: Users can observe workflow executions, reuse workflow templates, and fetch the server's API contract — the observe-and-react half of automation
**Depends on**: Phase 8 (error layer: RFC 7807 parsing, 409 inactive-trigger hint)
**Requirements**: WRUN-01, WRUN-02, WRUN-03, TPL-01, DOCS-01
**Success Criteria** (what must be TRUE):
  1. User can list a workflow's runs filtered by `--status`; test runs appear only with `--dry-run` opt-in, with a hint when empty results hide dry-run runs
  2. User can view a run's detail with every step flattened to readable rows (node, status, input, output, error, timing)
  3. User can `--watch` a run until completion; with `--exit-status`, a failed run yields exit code 1 so scripts can react to outcomes
  4. User can list/get/create/delete workflow templates — and correctly finds no `update` subcommand (the server has none)
  5. `pipelite docs [--save FILE]` fetches the OpenAPI 3.1 spec without authentication
**Notes**: Parallelizable with Phase 10 (mutually independent read surfaces).
**Plans**: 3 plans — 09-01 (runs list/detail), 09-02 (watch + templates), 09-03 (docs)

Plans:
- [x] 09-01-PLAN.md — Workflow runs: models + API client (list/detail, status/dry_run params), `workflows runs list/get` with required `--workflow`, steps table + JSON passthrough, empty-result hints (WRUN-01, WRUN-02)
- [x] 09-02-PLAN.md — `--watch`/`--exit-status` poll loop on run detail + top-level `templates` group (list/get/create with triggers[0] mapping/hidden update/delete with confirmation) (WRUN-03, TPL-01)
- [x] 09-03-PLAN.md — `docs` command: unauthenticated OpenAPI fetch (headerless client), `--save` with overwrite refusal, error-layer hints (DOCS-01)

### Phase 10: Notes
**Goal**: Users can attach, revise, and remove notes on deals, organizations, people, and activities — the first parent-scoped sub-resource
**Depends on**: Phase 8 (author-or-admin 403 → Forbidden hint)
**Requirements**: NOTE-01, NOTE-02, NOTE-03, NOTE-04
**Success Criteria** (what must be TRUE):
  1. User can list notes on any deal, organization, person, or activity
  2. User can add a note via flag, `@file`, stdin, or interactive prompt on any of the four note-capable entities
  3. User can edit a note by ID, with `notes list --json` documented as the view-before-edit path (no single-note GET exists — prompt text explains why existing content can't be shown)
  4. User can delete a note by ID with confirmation, bypassed by `--force`
  5. Notes on pipelines/stages/workflows parents (and a `notes get` attempt) are rejected with an actionable hint
**Notes**: Parallelizable with Phase 9. Top-level `notes` group with entity-type positional — NOT nested ×4 under entities.
**Plans**: 1 plan — 10-01 (single plan: 16 files, under the 17-file 09-02 precedent; models + client + group wiring + list/rejections + body resolver + add/edit/delete + tests + docs)

Plans:
- [x] 10-01-PLAN.md — Top-level `notes` group: Note model (serializer-exact), 4 client methods, list/add/edit/delete with body-source precedence (--body > @file > --stdin > prompt), pre-HTTP rejection surfaces (non-capable types, hidden get, multiple sources), templates delete contract, docs + 25 stub tests (NOTE-01, NOTE-02, NOTE-03, NOTE-04)

### Phase 11: Webhooks, Trash & Audit
**Goal**: Users can manage automation integrations and recover from mistakes — with admin-gated and irreversible operations failing safe
**Depends on**: Phase 7 (`confirm_destructive` from the batch utility), Phase 8 (Forbidden hints, RFC 7807 parsing)
**Requirements**: WHOK-01, WHOK-02, WHOK-03, TRSH-01, TRSH-02, TRSH-03, AUDT-01, AUDT-02
**Success Criteria** (what must be TRUE):
  1. User can list/get/create/update (PUT)/delete webhooks; an unknown event name is rejected client-side with an actionable error listing the 13 valid events
  2. The signing secret is displayed exactly once on create — full, untruncated, on its own line with a save-it-now warning — and never appears in list/get output or the cache
  3. User can list trashed records with `--type` filter and restore by type + ID; piped round-trips (`trash list | jq | trash restore`) work via singular/plural type normalization
  4. Purge is permanent and admin-only with the strongest confirmation in the codebase (`--force` bypass); under `--no-input` without `--force` it refuses with exit 2 and zero HTTP calls
  5. User can list audit entries with all four filters (`--entity-type`, `--entity-id`, `--actor-kind`, `--workflow-run-id`); non-admin keys get a first-class Forbidden hint on every admin-gated surface
**Notes**: Webhooks ∥ trash ∥ audit are internally parallelizable. Write the 64-char-secret truncation test FIRST. Research flags to resolve during planning: webhook PUT semantics (full object vs omitted-keys-as-no-op), trash `linked_parents` table rendering.
**Plans**: 3 plans — 11-01 (webhooks), 11-02 (trash), 11-03 (audit); sequential waves (shared wiring files). Amendments applied from research: `--description` dropped (no server field), https mirrored client-side, purge = CLI fan-out with exit-2 zero-HTTP refusal, `--all` on trash only.

Plans:
- [x] 11-01-PLAN.md — `webhooks` group: serializer-exact models (no secret in list/get — create-response carries it), 6 client methods, 13-event + https pre-HTTP validation (exit 2), show-once secret rendering (truncation test FIRST), KEY_WEBHOOKS (id/url only) + invalidation, get→merge→PUT update, standard delete contract, foreign-403 hint probes (WHOK-01, WHOK-02, WHOK-03)
- [x] 11-02-PLAN.md — `trash` group: TrashRow/DeletedBy models, 3 client methods, singular/plural type normalization, list (--type + bounded --all fan-out, offset cap 10,000), restore (no confirm, 404 re-wrap, general 403 hint), purge (scope-validated pre-HTTP fan-out, strongest confirm, exit-2 zero-HTTP refusal pinned, continue-on-error N ok / M failed) (TRSH-01, TRSH-02, TRSH-03)
- [x] 11-03-PLAN.md — `audit` group: AuditEntry model, 1 client method (non-empty filters only, limit clamped ≤100), 4 passthrough filters + --limit/--offset (no --all), default ts/actor/action/entity columns, --json changes payload, admin-403 hint probes incl. gate-before-validation ordering (AUDT-01, AUDT-02)

### Phase 12: Custom Fields — Definitions & Typed Writing
**Goal**: Users can define custom fields per entity and write correctly-typed values to them (fixes the "stores `\"4\"` not `4`" data-correctness bug)
**Depends on**: Phase 7 (confirm helper for destructive definition deletes), Phase 8 (foundations). Internal order: definitions CRUD before the typed parser (definitions are the type source)
**Requirements**: CFLD-01, CFLD-02, CFLD-03
**Success Criteria** (what must be TRUE):
  1. User can list/get/create/update/delete custom field definitions, with `custom-fields list --entity-type` filtering
  2. `--custom-field price=4` on a number definition stores JSON number `4`, not string `"4"` — type-correct writes for number/boolean/date/array/select, resolved from cached definitions
  3. User can bypass type inference entirely with `--custom-field-json '{"key": ...}'`
  4. `--dry-run` never triggers a definitions fetch — cache-only fallback with a visible note
**Notes**: Touches 8 existing handlers (create+update × 4 entities) — widest blast radius of the milestone, hence last feature phase. Coordinate with FIX-01 (dead `--custom-field` flags), which lands in Phase 8. Research flag: verify definition soft-delete marker and `multi_select` config shape against server code during planning.
**Plans**: 2 plans, sequential waves — 12-01 (definitions CRUD group, CFLD-01) → 12-02 (typed-writing resolver rewiring the 8 handlers + --custom-field-json, CFLD-02/03)

### Phase 13: Integration Hardening & Docs Refresh
**Goal**: The milestone holds together — every new surface honors the v1.0 global contract, cross-cutting behaviors are verified end-to-end, and documentation is current
**Depends on**: Phases 7-12 (all prior phases)
**Requirements**: None exclusively — cross-cutting verification of every requirement delivered in Phases 7-12
**Success Criteria** (what must be TRUE):
  1. Every command added in Phases 7-12 honors `--dry-run` (zero HTTP), `--no-input`, `--quiet`, `--no-color`, and renders in table/csv/json/plain
  2. Cross-cutting contracts pass end-to-end: batch exit codes under `--quiet`, untruncated webhook secret, trash list→restore round-trip, Forbidden hints on all admin-gated surfaces
  3. `docs/SKILL.md` and `docs/api-reference.md` document every new command group with endpoints and conventions
  4. Full test suite passes with no regressions against the v1.0 baseline (98 unit + 13 integration tests)
**Plans**: TBD

## Progress

**Execution Order:** 7 → 8 → 9 → 10 → 11 → 12 → 13
(Phases 9 and 10 are mutually parallelizable; Phase 8 is the hard gate for Phases 9-12.)

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 2/2 | Complete | 2026-03-25 |
| 2. Core CRUD and Output | v1.0 | 3/3 | Complete | 2026-03-25 |
| 3. Full Entity Coverage | v1.0 | 3/3 | Complete | 2026-03-25 |
| 4. Developer Experience | v1.0 | 3/3 | Complete | 2026-03-25 |
| 5. Power Features | v1.0 | 5/5 | Complete | 2026-03-28 |
| 6. Workflow API Integration | v1.0 | 2/2 | Complete | 2026-03-29 |
| 7. Batch Operations | v1.1 | 5/5 | Complete    | 2026-09-03 |
| 8. Foundations — Error Layer, Models & Pagination | v1.1 | 2/2 | Complete    | 2026-09-03 |
| 9. Workflow Runs, Templates & Docs | v1.1 | 3/3 | Complete    | 2026-09-03 |
| 10. Notes | v1.1 | 1/1 | Complete    | 2026-09-04 |
| 11. Webhooks, Trash & Audit | v1.1 | 3/3 | Complete    | 2026-09-04 |
| 12. Custom Fields — Definitions & Typed Writing | v1.1 | 2/2 | Complete    | 2026-09-04 |
| 13. Integration Hardening & Docs | v1.1 | 0/TBD | Not started | - |
