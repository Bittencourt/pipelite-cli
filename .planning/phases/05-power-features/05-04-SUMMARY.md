---
phase: 05-power-features
plan: 04
subsystem: cache
tags: [cache, invalidation, mutations, crud]

requires:
  - phase: 05-power-features
    provides: "CacheStore with invalidate/invalidate_prefix, KEY constants, Option<CacheStore> on AppContext"
provides:
  - "Automatic cache invalidation on all 18 mutation commands (create/update/delete x 6 entities)"
  - "Cascading stages cache invalidation on pipeline mutations"
  - "Parent pipeline stage cache invalidation on stage mutations"
affects: []

tech-stack:
  added: []
  patterns: [post-mutation-cache-invalidation, cascading-prefix-invalidation]

key-files:
  created: []
  modified:
    - src/commands/deals/create.rs
    - src/commands/deals/update.rs
    - src/commands/deals/delete.rs
    - src/commands/orgs/create.rs
    - src/commands/orgs/update.rs
    - src/commands/orgs/delete.rs
    - src/commands/people/create.rs
    - src/commands/people/update.rs
    - src/commands/people/delete.rs
    - src/commands/activities/create.rs
    - src/commands/activities/update.rs
    - src/commands/activities/delete.rs
    - src/commands/pipelines/create.rs
    - src/commands/pipelines/update.rs
    - src/commands/pipelines/delete.rs
    - src/commands/stages/create.rs
    - src/commands/stages/update.rs
    - src/commands/stages/delete.rs

key-decisions:
  - "Stage delete uses invalidate_prefix('stages_') since pipeline_id is unavailable from delete response"
  - "Batch create with individual-create loop (activities/pipelines/stages) invalidates once after loop, not per-item"
  - "Pipeline mutations always invalidate all stages_* caches via invalidate_prefix for cascading freshness"

patterns-established:
  - "Post-mutation invalidation: if let Some(ref cache) = ctx.cache { cache.invalidate(KEY) } after API success"
  - "Cascading invalidation: pipeline changes invalidate stages_* prefix"

requirements-completed: [CACH-01]

duration: 5min
completed: 2026-03-25
---

# Phase 5 Plan 4: Cache Invalidation Wiring Summary

**All 18 mutation commands (create/update/delete x 6 entities) auto-invalidate relevant cache entries after successful API calls, with cascading pipeline-to-stages invalidation**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-26T00:03:24Z
- **Completed:** 2026-03-26T00:09:19Z
- **Tasks:** 1
- **Files modified:** 18

## Accomplishments
- Wired cache invalidation into all 18 create/update/delete command handlers across deals, orgs, people, activities, pipelines, and stages
- Pipeline mutations cascade to invalidate all stages_* caches via invalidate_prefix
- Stage create/update invalidate the specific parent pipeline's stage cache (stages_{pipeline_id})
- Batch create handlers for all entities also invalidate after successful creation

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire cache invalidation into all mutation commands** - `757917d` (feat)

**Plan metadata:** pending (docs: complete plan)

## Files Created/Modified
- `src/commands/deals/create.rs` - Cache invalidation after single and batch deal create
- `src/commands/deals/update.rs` - Cache invalidation after deal update
- `src/commands/deals/delete.rs` - Cache invalidation after deal delete
- `src/commands/orgs/create.rs` - Cache invalidation after single and batch org create
- `src/commands/orgs/update.rs` - Cache invalidation after org update
- `src/commands/orgs/delete.rs` - Cache invalidation after org delete
- `src/commands/people/create.rs` - Cache invalidation after single and batch person create
- `src/commands/people/update.rs` - Cache invalidation after person update
- `src/commands/people/delete.rs` - Cache invalidation after person delete
- `src/commands/activities/create.rs` - Cache invalidation after single and batch activity create
- `src/commands/activities/update.rs` - Cache invalidation after activity update (both regular and raw/mark-undone paths)
- `src/commands/activities/delete.rs` - Cache invalidation after activity delete
- `src/commands/pipelines/create.rs` - Cache invalidation after single and batch pipeline create + stages_* prefix
- `src/commands/pipelines/update.rs` - Cache invalidation after pipeline update + stages_* prefix
- `src/commands/pipelines/delete.rs` - Cache invalidation after pipeline delete + stages_* prefix
- `src/commands/stages/create.rs` - Cache invalidation after single stage create (KEY_STAGES + stages_{pipeline_id}) and batch (prefix)
- `src/commands/stages/update.rs` - Cache invalidation after stage update (KEY_STAGES + stages_{pipeline_id})
- `src/commands/stages/delete.rs` - Cache invalidation after stage delete (KEY_STAGES + stages_* prefix)

## Decisions Made
- Stage delete uses `invalidate_prefix("stages_")` broadly since pipeline_id is not available from the DELETE response -- slightly broader invalidation but always correct
- Batch create handlers that use individual-create loops (activities, pipelines, stages) invalidate once after the entire loop completes, not per-item
- Pipeline mutations always invalidate all stages_* caches via invalidate_prefix for cascading freshness

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Cache invalidation complete across all mutation paths
- Completions and prompts now see fresh data after any create/update/delete operation
- Pre-existing cli_skeleton test failures unrelated to cache changes

---
*Phase: 05-power-features*
*Completed: 2026-03-25*
