---
status: resolved
trigger: "Dashboard command shows fewer deals than expected — totals are lower than what's actually in the CRM."
created: 2026-03-25T00:00:00Z
updated: 2026-09-02T01:36:19Z
---

## Current Focus

hypothesis: CONFIRMED - Dashboard pagination hard-caps at 1000 deals via `offset >= 1000` check
test: Fix verified by automated checks and confirmed by user in real workflow
expecting: N/A - session resolved
next_action: None — user confirmed fix works; session archived to resolved/

## Symptoms

expected: Dashboard shows accurate deal counts and total values per stage, reflecting all deals in the CRM
actual: Dashboard shows some deals but totals are clearly lower than what's in the CRM — deals are being cut off
errors: None — command runs without errors
reproduction: Run `pipelite dashboard` against a CRM with many deals
started: Since Phase 5 implementation (first version of dashboard)

## Eliminated

## Evidence

- timestamp: 2026-03-25T00:01:00Z
  checked: src/commands/dashboard.rs fetch_all_deals (lines 108-135)
  found: Pagination loop uses `offset >= 1000` as hard cap, stops fetching after 1000 deals max
  implication: CRMs with >1000 deals silently show truncated totals

- timestamp: 2026-03-25T00:01:30Z
  checked: src/commands/deals/list.rs fetch_all (lines 53-105) for comparison
  found: Uses response.meta.total to know exact total, paginating until all records fetched (up to max_records)
  implication: The proper pattern exists in the codebase but dashboard doesn't use it

- timestamp: 2026-03-25T00:02:00Z
  checked: src/api/models.rs ApiListResponse and PaginationMeta
  found: API returns meta.total with true total count on every response
  implication: Dashboard can use meta.total to paginate without arbitrary cap

## Resolution

root_cause: Dashboard's fetch_all_deals/pipelines/stages functions use `offset >= 1000` hard cap instead of response.meta.total for pagination. CRMs with >1000 deals silently show truncated aggregates.
fix: Replaced offset-based hard cap (1000) with meta.total-driven pagination in all three fetch functions (fetch_all_deals, fetch_all_pipelines, fetch_all_stages). Also increased batch size from 100 to 500 for efficiency.
verification: cargo check passes, all 94 unit tests pass, all 3 dashboard integration tests pass. 2 pre-existing cli_skeleton test failures confirmed unrelated. Human verification (2026-09-02): user confirmed `pipelite dashboard` now shows correct deal totals matching the CRM.
files_changed: [src/commands/dashboard.rs]
