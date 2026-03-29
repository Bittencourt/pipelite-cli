# Milestones

## v1.0 MVP (Shipped: 2026-03-29)

**Phases completed:** 6 phases, 18 plans, 34 tasks

**Key accomplishments:**

- Clap derive CLI skeleton with rich --version, TOML config with 0600 permissions and env var merge, and thiserror structured error display on stderr
- PipeliteClient with auth headers and timeouts, init wizard with headless mode, ping with spinner/latency, config show/set/get with comfy-table, plus 10 integration tests
- Generic output rendering (table/JSON/CSV/plain) with dot-notation field selection, relative dates, currency formatting, and Deal data models with serde skip_serializing_if
- Full deal CRUD command tree with filtering, auto-pagination, batch stdin create, custom fields, and typed API error handling
- Fixed inverted field-selection conditional in deals get and removed stale dead_code allows from output module
- Full CRUD for organizations (4 fields) and people (7 fields) with batch create, auto-pagination, and 16 integration tests
- Full activities CRUD with individual-create loop, client-side done filter, and mark-done/undone via chrono timestamps
- Models (src/api/models.rs):
- Global --no-input/--dry-run flags, shared prompt and dry-run modules, shell completions, and deals as proof-of-pattern for interactive/headless/dry-run support
- Interactive prompts, headless validation, and dry-run support replicated from deals to all 5 remaining entities (orgs, people, activities, pipelines, stages)
- 21 integration tests validating headless validation (exit code 2, batch missing), shell completions (bash/zsh/fish), and dry-run request preview without HTTP calls
- TTL-based JSON file cache at ~/.pipelite/cache/ with get/set/clear/invalidate/invalidate_prefix, CLI commands, and AppContext wiring
- Pipeline dashboard with deal aggregation per stage and ASCII art splash screen with TTY-aware display
- Cache-through helpers for FuzzySelect prompts: pipelines, stages, and orgs load from local cache first with auto-paginating API fallback on miss
- All 18 mutation commands (create/update/delete x 6 entities) auto-invalidate relevant cache entries after successful API calls, with cascading pipeline-to-stages invalidation
- Cache-backed dynamic shell completions for all entity ID args using clap_complete unstable-dynamic ArgValueCandidates
- Commit:
- Commit:

---
