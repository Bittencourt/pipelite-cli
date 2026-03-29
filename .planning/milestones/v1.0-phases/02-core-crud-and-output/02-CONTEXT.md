# Phase 2: Core CRUD and Output - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Full CRUD on deals (list, get, create, update, delete) with all output formats (table/csv/json/plain), field selection (`--fields`), entity-specific filtering, pagination, and colored output with NO_COLOR support. Deals are the first entity — patterns proven here will be replicated for remaining entities in Phase 3. Interactive prompts are Phase 4; this phase handles flag/arg-based input only.

</domain>

<decisions>
## Implementation Decisions

### Deal data model
- Typed `Deal` struct modeled from the API OpenAPI spec (confirmed at localhost:3001/api/v1/docs)
- Fields: id, title, value (nullable), stage_id, organization_id (nullable), person_id (nullable), owner_id, position (nullable), expected_close_date (nullable), notes (nullable), custom_fields (nullable object), created_at, updated_at
- `DealCreate` requires: title, stage_id. Optional: value, organization_id, person_id, expected_close_date, notes, custom_fields
- `DealUpdate`: all fields optional
- Custom fields are first-class — dynamic column support in table output, selectable via `--fields`, filterable
- `--expand <relations>` flag on `deals list` and `deals get` to inline related entities (maps to API `?expand=` param). Expanded fields accessible via dot notation in `--fields` (e.g., `--fields=id,title,owner.name`)

### Table & field output
- Default table columns for `deals list`: id, title, value, stage_id, owner_id, updated_at — compact, enough to identify and act on deals
- Long values truncated with ellipsis to fit terminal width (e.g., titles > 30 chars become "Long deal titl...")
- `deals get <id>` displays as key-value pairs (vertical layout like `gh issue view`), not a single-row table
- `--format plain` outputs tab-separated values, no headers, no borders — one row per item, ideal for awk/cut piping
- `--format csv` includes header row, standard CSV escaping
- `--format json` outputs full JSON (array for list, object for get) — the complete API response
- `--fields=id,title,value` selects specific fields across all output formats
- Custom fields accessible in --fields via dot notation: `--fields=id,title,custom_fields.industry`

### Filtering & pagination
- Named flags for deal-specific filters: `--stage <id>`, `--org <id>`, `--owner <id>` — maps 1:1 to API query params
- `--limit` and `--offset` pass through to API (default 50, max 100 per request)
- Table footer shows pagination meta: "Showing 1-50 of 234"
- `--all` flag auto-paginates in batches of 100, capped at 1000 records with warning if more exist ("Showing 1000 of 5234. Use --limit/--offset for more.")

### Create/update input
- Named flags per field: `--title "Big Deal" --stage abc123 --value 50000`
- Custom fields via repeatable flag: `--custom-field industry=Tech --custom-field priority=high`
- Missing required fields on create = error with hint listing required fields
- Successful create/update: echo the full deal in current output format (same as `deals get`)
- Successful delete: print confirmation message
- Basic batch support via `--stdin`: `echo '[{...}]' | pipelite deals create --stdin` calls `POST /deals/batch`

### Colored output
- Table borders and headers in dim/white, values in default color
- `NO_COLOR` env var and `--no-color` flag respected (already in AppContext)
- Value formatting: currency values right-aligned, dates formatted as relative ("2h ago") in table, ISO in JSON

### Claude's Discretion
- Table rendering library choice (comfy-table, tabled, custom)
- Column width calculation algorithm
- CSV library choice
- Exact error messages for validation failures
- Batch endpoint error handling (partial success reporting)
- How expanded relations render in table vs key-value view

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `PipeliteClient` (src/api/mod.rs): HTTP client with auth headers, timeouts — needs new methods for CRUD endpoints
- `OutputFormat` enum (src/output/mod.rs): Already has Json, Table, Csv, Plain variants
- `detect_format()` (src/output/mod.rs): TTY detection for auto-format selection
- `AppContext` (src/context.rs): Carries config, client, format, color, quiet, verbose — pass to all command handlers
- `CliError` (src/error.rs): Structured errors with detail + hint — add new variants for validation, API errors
- `output/table.rs`: Empty, reserved for entity table formatting

### Established Patterns
- Clap derive API with `#[command(subcommand)]` for noun-verb structure
- Global flags defined on `Cli` struct with `global = true`
- Commands in `src/commands/` as modules with public execute functions
- `after_help` with examples on every command
- Directory modules for commands with helpers (e.g., commands/config/mod.rs + table.rs)

### Integration Points
- `Commands` enum in src/cli/mod.rs — add `Deals(DealsCommands)` variant
- `src/main.rs` dispatch — add deals match arm
- `src/api/mod.rs` — add CRUD methods to `PipeliteClient`
- `src/api/models.rs` — add Deal, DealCreate, DealUpdate structs

</code_context>

<specifics>
## Specific Ideas

- API docs confirmed at http://localhost:3001/api/v1/docs — OpenAPI 3.1.0 spec
- API uses string IDs (not numeric), pagination via offset/limit with meta object: `{ "data": [...], "meta": { "total": N, "offset": N, "limit": N } }`
- API supports relation expansion via `?expand=owner,organization` query param
- Deal position field is for kanban ordering — not relevant for CLI display by default
- Batch endpoint exists at `POST /deals/batch` — include basic stdin-based batch create
- Should feel like gh, kubectl — composable and predictable (carried from Phase 1)

</specifics>

<deferred>
## Deferred Ideas

- Interactive prompts with dropdowns for stage/pipeline selection — Phase 4 (INTR-01, INTR-02)
- Bulk operations beyond basic batch create (BULK-01, BULK-02) — v2
- JSONL streaming output (OUTP-08) — v2

</deferred>

---

*Phase: 02-core-crud-and-output*
*Context gathered: 2026-03-24*
