# Retrospective: Pipelite CLI

## Milestone: v1.0 — MVP

**Shipped:** 2026-03-29
**Phases:** 6 | **Plans:** 18 | **Tasks:** 34

### What Was Built
- Full CRM CLI with CRUD on 7 entities (deals, orgs, people, activities, pipelines, stages, workflows)
- 4 output formats (table/JSON/CSV/plain) with auto TTY detection and field selection
- Interactive prompts with cache-backed dropdowns, headless mode, dry-run previews
- Shell completions (bash/zsh/fish) with dynamic entity ID suggestions from cache
- TTL-based local cache with mutation invalidation across all 18 mutation commands
- Pipeline dashboard with deal aggregation per stage + workflow summary
- Workflow automation trigger (fire-and-forget with @filepath data support)
- 10,779 LOC Rust + 1,447 LOC tests, 98 unit tests, 13 integration tests

### What Worked
- Entity replication pattern: deals proved the full stack, then 5 entities replicated cleanly
- Generic output rendering (serde_json::Value) meant zero output code per entity
- Cache-through pattern: one helper function per entity, consistent API fallback
- Plan verification loop caught requirement tracking gaps before execution
- Worktree isolation for parallel executor agents prevented merge conflicts

### What Was Inefficient
- Phases 2 and 5 roadmap checkboxes never got marked `[x]` during execution (cosmetic tracking gap)
- Some plans had 11+ files — borderline for a single task, though pattern replication made it manageable
- Dashboard JSON format changed from bare array to wrapped object in Phase 6 — minor breaking change

### Patterns Established
- Noun-verb CLI command structure: `pipelite <entity> <action>` with short aliases
- Entity module structure: `src/commands/<entity>/` with mod.rs dispatch + per-action files
- Cache key convention: `KEY_<ENTITY>` constants, `TTL_<ENTITY>` for per-type TTL
- Prompt cascade: pipeline → stage FuzzySelect with cache-through helpers
- Dry-run renders JSON preview of exact API request without executing
- Delete confirmation: TTY dialoguer::Confirm + --force for headless

### Key Lessons
- Auto-paginate with 1000 cap prevents runaway API calls while covering most use cases
- serde_json::Value for complex nested types (triggers/nodes) is the right tradeoff when the server validates
- @filepath syntax on --data flags (curl convention) is a natural CLI pattern for JSON payloads
- Integration tests with unreachable server + dry-run prove CLI wiring without API dependency

### Cost Observations
- Model mix: 100% opus for execution, sonnet for verification
- Sessions: ~10 sessions across 7 days
- Notable: Phase 6 (workflow API) executed in a single session end-to-end: discuss → plan → execute → verify → UAT

## Cross-Milestone Trends

| Metric | v1.0 |
|--------|------|
| Phases | 6 |
| Plans | 18 |
| Tasks | 34 |
| LOC (src) | 10,779 |
| LOC (tests) | 1,447 |
| Duration | 7 days |
| Commits | 101 |
