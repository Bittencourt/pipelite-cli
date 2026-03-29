# Phase 6: Update the CLI tools to include the new Workflow API - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Add full workflow entity support to the CLI: CRUD commands (list, get, create, update, delete), manual trigger execution, dashboard integration with workflow summary, caching, interactive prompts for basic fields, and shell completions. Follows all established patterns from Phases 1-5. No new output formats, no TUI, no workflow visual editor.

</domain>

<decisions>
## Implementation Decisions

### Workflow entity model
- **D-01:** Workflow fields: `id`, `name`, `description` (nullable), `triggers[]` (TriggerConfig), `nodes[]` (WorkflowNode), `active` (boolean), `created_by`, `created_at`, `updated_at`
- **D-02:** TriggerConfig is a discriminated union by `type`: crm_event, schedule, webhook, manual — each with type-specific fields
- **D-03:** WorkflowNode has `id`, `type` (action/condition/delay), `label`, `config` (object), `nextNodeId`, `trueBranch`, `falseBranch`
- **D-04:** API uses standard CRUD pattern: GET/POST `/workflows`, GET/PUT/DELETE `/workflows/:id`, plus POST `/workflows/:id/run`

### Command structure
- **D-05:** Command: `pipelite workflows <action>` with alias `w`
- **D-06:** Subcommands: `list`, `get <id>`, `create`, `update <id>`, `delete <id>`, `trigger <id>`
- **D-07:** `trigger` verb (not `run`) for manual workflow execution
- **D-08:** Interactive prompts for basic fields: name, description, active toggle on TTY
- **D-09:** Triggers and nodes are too complex for interactive prompts — accept via `--stdin` JSON or `--triggers`/`--nodes` JSON string flags
- **D-10:** All standard global flags apply: `--format`, `--fields`, `--no-color`, `--no-input`, `--dry-run`, `--limit`, `--offset`

### Dashboard integration
- **D-11:** Add workflow summary section to existing `pipelite dashboard` output
- **D-12:** Summary shows active workflow count and recent trigger activity (e.g., "3 active workflows, 12 runs today")

### Workflow execution (trigger)
- **D-13:** `pipelite workflows trigger <id>` with optional `--data` flag accepting arbitrary JSON
- **D-14:** No --entity-type/--entity-id flags — entity context goes inside --data JSON blob
- **D-15:** Confirm-only output: show run_id and 'pending' status immediately, no polling/waiting for completion

### Cache & completions
- **D-16:** Cache workflows with TTL (same tiered approach as other entities — slow-changing, hours TTL)
- **D-17:** Mutation invalidation on create/update/delete (follows Phase 5 pattern)
- **D-18:** Dynamic shell completions for workflow IDs with name hints (follows Phase 5 pattern)

### Claude's Discretion
- Exact table columns for list vs get display
- Workflow default table format (compact for list, full for get)
- How to render triggers/nodes in table output (summary line vs full JSON)
- Dashboard workflow summary formatting and placement
- Cache key structure for workflows
- --from-file flag design for loading workflow JSON from a file (if useful)
- Trigger --data flag format (inline JSON string vs @file path)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Workflow API specification
- `../pipelite/public/openapi.yaml` §833-1006 — Workflow, WorkflowCreate, WorkflowUpdate, TriggerConfig, WorkflowNode, WorkflowRunTrigger, WorkflowRunResponse schemas
- `../pipelite/public/openapi.yaml` §2236-2394 — Workflow API endpoints (list, get, create, update, delete, trigger run)
- `../pipelite/docs/user/reference/workflows.md` — Complete workflow feature reference (triggers, node types, variable picker, visual editor)

### Existing CLI patterns to follow
- `src/api/mod.rs` — PipeliteClient structure and CRUD method patterns
- `src/api/models.rs` — Entity model definitions (derive patterns, serde attributes)
- `src/commands/deals/` — Reference CRUD command structure (list, get, create, update, delete)
- `src/commands/dashboard.rs` — Dashboard command to extend with workflow summary
- `src/cache.rs` — CacheStore for workflow caching integration
- `src/prompt.rs` — Interactive prompt patterns (FuzzySelect, cache-through)
- `src/cli/mod.rs` — Command registration and global flags

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `PipeliteClient` (src/api/mod.rs): All CRUD methods follow the same pattern — add workflow methods following identical structure
- `ApiListResponse<T>` / `ApiSingleResponse<T>` (src/api/models.rs): Generic response wrappers — reuse for Workflow type
- Output modules (src/output/): Fully generic table/csv/json/plain rendering — works with any serde_json::Value
- `AppContext` (src/context.rs): Carries all global flags — no changes needed
- `CacheStore` (src/cache.rs): TTL-based caching with set/get/invalidate — add workflow cache keys
- `prompt.rs`: Cache-through FuzzySelect helpers — add workflow name selector
- `ArgValueCandidates` completions: Dynamic completion pattern from Phase 5 — add workflow completions

### Established Patterns
- Noun-verb command structure: `pipelite <noun> <action>` with aliases
- Entity modules in `src/commands/<entity>/` with mod.rs defining subcommands enum
- `handle_response<T>` for typed HTTP error mapping
- Auto-paginate with 100-batch, 1000 cap, stderr warning
- Create/update: Option fields validated at runtime, prompt fallback on TTY
- Delete: TTY confirmation + --force, headless requires --force
- Cache invalidation on all mutations (create/update/delete)

### Integration Points
- `Commands` enum in src/cli/mod.rs — add `Workflows(WorkflowsCommands)` variant
- `src/main.rs` dispatch — add workflows match arm
- `src/api/mod.rs` — add workflow CRUD + trigger methods to PipeliteClient
- `src/api/models.rs` — add Workflow, WorkflowCreate, WorkflowUpdate, TriggerConfig, WorkflowNode models
- `src/commands/dashboard.rs` — add workflow summary section
- `src/cache.rs` — add workflow cache key constants and invalidation
- Completions wiring in relevant command files

</code_context>

<specifics>
## Specific Ideas

- Workflow create/update should feel like editing a config file — basic metadata interactively, complex structure via JSON
- Trigger command should be fire-and-forget — quick confirmation, no waiting
- Dashboard workflow summary should be a compact addition, not dominate the pipeline overview
- List table should show workflow essentials at a glance: name, active status, trigger types, node count

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-update-the-cli-tools-to-include-the-new-workflow-api*
*Context gathered: 2026-03-29*
