# Roadmap: Pipelite CLI

## Milestones

- ✅ **v1.0 MVP** — Phases 1-6 (shipped 2026-03-29)

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

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 2/2 | Complete | 2026-03-25 |
| 2. Core CRUD and Output | v1.0 | 3/3 | Complete | 2026-03-25 |
| 3. Full Entity Coverage | v1.0 | 3/3 | Complete | 2026-03-25 |
| 4. Developer Experience | v1.0 | 3/3 | Complete | 2026-03-25 |
| 5. Power Features | v1.0 | 5/5 | Complete | 2026-03-28 |
| 6. Workflow API Integration | v1.0 | 2/2 | Complete | 2026-03-29 |

### Phase 1: Batch operations for all entities

**Goal:** Add batch update (--stdin) and batch delete (multi-ID + --stdin) with continue-on-error to all 7 entity types, plus confirmation prompts for batch delete.
**Requirements:** BATCH-01 (batch update via --stdin), BATCH-02 (batch delete via multi-ID + --stdin), BATCH-03 (continue-on-error with summary), BATCH-04 (shared batch utility)
**Depends on:** Phase 0
**Plans:** 4 plans

Plans:
- [ ] 01-01-PLAN.md — Batch utility module + deals reference implementation
- [ ] 01-02-PLAN.md — Batch ops for orgs, people, activities
- [ ] 01-03-PLAN.md — Batch ops for pipelines, stages, workflows
- [ ] 01-04-PLAN.md — Integration tests for all batch operations
