# Phase 1: Batch Operations for All Entities - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Extend batch capabilities across all 7 entity types (deals, orgs, people, activities, pipelines, stages, workflows). Batch create already exists via `--stdin` for all entities. This phase adds batch update and batch delete, plus standardizes error handling across all batch operations.

</domain>

<decisions>
## Implementation Decisions

### Input Formats
- **D-01:** JSON only via `--stdin`. No CSV or NDJSON support. Consistent with the existing batch create pattern that all entities already use.

### Batch Update
- **D-02:** Reuse the existing `update` subcommand with `--stdin` flag (same pattern as create). No new subcommands.
- **D-03:** Each JSON object in the stdin array includes an `"id"` field plus the fields to patch. Example: `[{"id": "deal_1", "title": "New Title"}, ...]`. Self-contained, mirrors single-update pattern.

### Batch Delete
- **D-04:** Accept multiple positional IDs: `pipelite deals delete id1 id2 id3`. Also support `--stdin` for large ID lists (JSON array of strings).
- **D-05:** Batch delete shows confirmation prompt: "Delete N [entity]s? [y/N]". Respects `--no-input` (skips prompt, proceeds) and `--dry-run` (shows what would be deleted without executing).

### Error Handling
- **D-06:** Continue-on-error for all batch operations. Process all items, collect errors, then report summary: "N/M succeeded, K failed".
- **D-07:** Successful items rendered to stdout normally (respecting --format flag). Error summary printed to stderr. Exit code 0 if all succeed, non-zero if any fail.

### Claude's Discretion
- Specific exit code value for partial failure (e.g., exit 1 vs a custom code)
- Whether to apply continue-on-error retroactively to existing batch create operations
- Internal implementation patterns (trait-based batch handling, shared batch utilities, etc.)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### CLI Patterns
- `docs/SKILL.md` — Comprehensive agent skill file with CRUD patterns, endpoints, and task guides
- `docs/architecture.md` — Internal architecture and data flow

### Existing Batch Implementation
- `src/commands/deals/create.rs` — Reference batch create implementation (stdin JSON → batch API call)
- `src/commands/pipelines/create.rs` — Reference batch create with individual-create loop (no batch API endpoint)
- `src/cli/deals.rs` — CLI arg definitions with `--stdin` flag pattern

### API Client
- `src/api/mod.rs` — PipeliteClient with existing `batch_create_*` methods for deals, orgs, people

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `--stdin` flag pattern already defined in all 7 entity CLI modules (`src/cli/*.rs`)
- `batch_create` functions in all 7 entity create commands — can serve as templates
- `dry_run::render_dry_run` and `dry_run::render_dry_run_delete` — dry-run rendering utilities
- `prompt::check_missing` — validation pattern for required fields
- `output::render_list` and `output::render_single` — output rendering for batch results

### Established Patterns
- Stdin detection: `io::stdin().is_terminal()` check before reading
- Mutual exclusivity: `--stdin` and individual flags are mutually exclusive (enforced in create)
- Cache invalidation: `cache.invalidate(KEY_*)` after mutations
- Dry-run intercept: check `ctx.dry_run` before HTTP calls

### Integration Points
- `src/cli/*.rs` — Add `--stdin` flag to update subcommands, change delete `id` arg to accept multiple values
- `src/api/mod.rs` — May need batch update/delete API methods (depends on server API support)
- `src/commands/*/update.rs` — Add batch_update pathway parallel to existing batch_create
- `src/commands/*/delete.rs` — Extend to handle multiple IDs with confirmation

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-batch-operations-for-all-entities*
*Context gathered: 2026-03-29*
