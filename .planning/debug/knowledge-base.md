# GSD Debug Knowledge Base

Resolved debug sessions. Used by `gsd-debugger` to surface known-pattern hypotheses at the start of new investigations.

---

## dashboard-deal-limit — Dashboard totals truncated by hardcoded 1000-record pagination cap
- **Date:** 2026-09-02
- **Error patterns:** dashboard, deals, totals lower than CRM, truncated, pagination, offset 1000, no errors shown
- **Root cause:** Dashboard's fetch_all_deals/pipelines/stages functions use `offset >= 1000` hard cap instead of response.meta.total for pagination. CRMs with >1000 deals silently show truncated aggregates.
- **Fix:** Replaced offset-based hard cap (1000) with meta.total-driven pagination in all three fetch functions (fetch_all_deals, fetch_all_pipelines, fetch_all_stages). Also increased batch size from 100 to 500 for efficiency.
- **Files changed:** src/commands/dashboard.rs
---
