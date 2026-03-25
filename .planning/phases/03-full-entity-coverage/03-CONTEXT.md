# Phase 3: Full Entity Coverage - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Replicate the proven deals CRUD pattern across the remaining five entities: organizations, people, activities, pipelines, and stages. Each entity gets list, get, create, update, and delete subcommands with the same output formats, field selection, filtering, and pagination as deals. No interactive prompts (Phase 4), no caching (Phase 5).

</domain>

<decisions>
## Implementation Decisions

### Stages & pipelines relationship
- Both `pipelite pipelines` and `pipelite stages` are top-level commands (flat, consistent with all other entities)
- `stages list` requires `--pipeline <id>` — error with hint if omitted
- `stages create` requires `--pipeline <id>` as a named flag (not positional)
- `stages get/update/delete` take stage ID as positional arg (no --pipeline needed — ID is unique)
- Short alias: `s` for stages, `pl` or similar for pipelines (Claude's discretion on pipeline alias to avoid collisions)

### Default table columns
- **Organizations**: id, name, owner_id, updated_at (4 columns)
- **People**: id, name, email, organization_id, updated_at (5 columns)
- **Activities**: id, type, subject, deal_id, done, due_date (6 columns)
- **Pipelines**: id, name, updated_at (3 columns)
- **Stages**: id, name, pipeline_id, position (4 columns)
- All entities: `get <id>` shows full key-value pairs (vertical layout), same as deals

### Entity-specific filter flags
- **Organizations**: `--owner <id>`
- **People**: `--org <id>`, `--owner <id>`
- **Activities**: `--deal <id>`, `--type <type>`, `--done` (boolean flag)
- **Pipelines**: no entity-specific filters (typically few pipelines)
- **Stages**: `--pipeline <id>` (required for list, no additional filters)
- All entities: `--limit`, `--offset`, `--expand`, `--fields` carried from global/deals pattern

### Batch/stdin support
- All entities support `--stdin` batch create
- If API has a /batch endpoint, use it; otherwise loop individual creates
- Individual-create loop shows progress: "Creating 3/10..."
- Partial failures continue processing — report summary at end: "Created 8/10. 2 failed (see errors above)."
- Non-zero exit code if any item fails

### Claude's Discretion
- Exact data model fields per entity (researcher confirms against API spec)
- Required vs optional fields for create payloads per entity
- Pipeline short alias choice (avoid collisions with `p` for people)
- Activity type values (call, email, meeting, etc. — from API)
- How `--done` flag works (toggle vs explicit true/false)
- Internal code organization (one module per entity vs shared generic)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `PipeliteClient` (src/api/mod.rs): HTTP client with `handle_response<T>`, `handle_delete_response`, `map_request_error` — add CRUD methods per entity
- `ApiSingleResponse<T>` / `ApiListResponse<T>` (src/api/models.rs): Generic response wrappers — reuse for all entities
- `DealsListParams` pattern (src/api/mod.rs): Query param builder with `to_query_pairs()` — replicate per entity
- `TableConfig` / `deals_table_config()` (src/api/models.rs): Default column config — add per entity
- Output modules (src/output/): Fully generic (serde_json::Value) — table, csv, json, plain all work without entity-specific code
- `detect_format()` / `AppContext` (src/context.rs): TTY detection, format selection, color, quiet, verbose — shared across all commands
- `CliError` (src/error.rs): Auth, NotFound, Validation, Api, Connection variants — no new variants needed

### Established Patterns
- Clap derive API: `DealsCommands` enum with `List(args)`, `Get(args)`, `Create(args)`, `Update(args)`, `Delete(args)`
- Command dispatch: `commands/deals/mod.rs` with `run(ctx, cmd)` matching subcommands to handlers
- Each CRUD operation is a separate file: `list.rs`, `get.rs`, `create.rs`, `update.rs`, `delete.rs`
- Create/update use named `--flags` per field with `Option` for runtime validation
- List uses entity-specific `ListParams` struct with `to_query_pairs()`
- Get single item: unwraps `ApiSingleResponse<T>` to return entity directly
- Delete: confirmation in TTY unless `--force`

### Integration Points
- `Commands` enum in src/cli/mod.rs — add variants for Orgs, People, Activities, Pipelines, Stages
- `src/main.rs` dispatch — add match arms for each new entity
- `src/api/mod.rs` — add CRUD methods per entity on `PipeliteClient`
- `src/api/models.rs` — add entity structs, create/update payloads, table configs

</code_context>

<specifics>
## Specific Ideas

- Pattern is proven end-to-end with deals — replicate, don't reinvent
- Entity data models need API spec confirmation (researcher reads OpenAPI docs at localhost:3001/api/v1/docs)
- Deals batch create used dedicated /batch endpoint — new entities may need individual-create loop fallback
- Should feel mechanical: same UX, same flags, same output behavior — users learn one entity, they know them all

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-full-entity-coverage*
*Context gathered: 2026-03-24*
