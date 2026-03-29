# Phase 6: Update the CLI tools to include the new Workflow API - Research

**Researched:** 2026-03-29
**Domain:** Rust CLI - new entity CRUD + trigger command, dashboard extension, caching, completions
**Confidence:** HIGH

## Summary

Phase 6 adds workflow entity support to the Pipelite CLI. The codebase has mature, well-established patterns from Phases 1-5 that must be replicated exactly for workflows. Every prior entity (deals, orgs, people, activities, pipelines, stages) follows an identical architecture: models in `src/api/models.rs`, client methods in `src/api/mod.rs`, CLI args in `src/cli/<entity>.rs`, command handlers in `src/commands/<entity>/`, cache keys in `src/cache.rs`, completions via `ArgValueCandidates`, and prompt helpers in `src/prompt.rs`.

The workflow entity is unique in two ways: (1) it has deeply nested sub-objects (TriggerConfig, WorkflowNode) that are too complex for interactive prompts -- these must be accepted as raw JSON via flags or stdin, and (2) it has an additional `trigger` subcommand (POST `/workflows/:id/run`) that returns a fire-and-forget run_id/status, which is unlike any existing command pattern.

**Primary recommendation:** Follow existing entity patterns exactly for CRUD. Model TriggerConfig and WorkflowNode as serde structs for deserialization but accept them as raw JSON strings on create/update. Add the `trigger` subcommand as a new pattern that POSTs optional JSON data and renders the run response.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: Workflow fields: `id`, `name`, `description` (nullable), `triggers[]` (TriggerConfig), `nodes[]` (WorkflowNode), `active` (boolean), `created_by`, `created_at`, `updated_at`
- D-02: TriggerConfig is a discriminated union by `type`: crm_event, schedule, webhook, manual -- each with type-specific fields
- D-03: WorkflowNode has `id`, `type` (action/condition/delay), `label`, `config` (object), `nextNodeId`, `trueBranch`, `falseBranch`
- D-04: API uses standard CRUD pattern: GET/POST `/workflows`, GET/PUT/DELETE `/workflows/:id`, plus POST `/workflows/:id/run`
- D-05: Command: `pipelite workflows <action>` with alias `w`
- D-06: Subcommands: `list`, `get <id>`, `create`, `update <id>`, `delete <id>`, `trigger <id>`
- D-07: `trigger` verb (not `run`) for manual workflow execution
- D-08: Interactive prompts for basic fields: name, description, active toggle on TTY
- D-09: Triggers and nodes are too complex for interactive prompts -- accept via `--stdin` JSON or `--triggers`/`--nodes` JSON string flags
- D-10: All standard global flags apply: `--format`, `--fields`, `--no-color`, `--no-input`, `--dry-run`, `--limit`, `--offset`
- D-11: Add workflow summary section to existing `pipelite dashboard` output
- D-12: Summary shows active workflow count and recent trigger activity (e.g., "3 active workflows, 12 runs today")
- D-13: `pipelite workflows trigger <id>` with optional `--data` flag accepting arbitrary JSON
- D-14: No --entity-type/--entity-id flags -- entity context goes inside --data JSON blob
- D-15: Confirm-only output: show run_id and 'pending' status immediately, no polling/waiting for completion
- D-16: Cache workflows with TTL (same tiered approach as other entities -- slow-changing, hours TTL)
- D-17: Mutation invalidation on create/update/delete (follows Phase 5 pattern)
- D-18: Dynamic shell completions for workflow IDs with name hints (follows Phase 5 pattern)

### Claude's Discretion
- Exact table columns for list vs get display
- Workflow default table format (compact for list, full for get)
- How to render triggers/nodes in table output (summary line vs full JSON)
- Dashboard workflow summary formatting and placement
- Cache key structure for workflows
- --from-file flag design for loading workflow JSON from a file (if useful)
- Trigger --data flag format (inline JSON string vs @file path)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

## Standard Stack

### Core
No new dependencies required. The existing Cargo.toml has everything needed:

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.6 | CLI args, subcommands, derives | Already used for all commands |
| serde/serde_json | 1.0 | Model serialization, JSON parsing | Already used for all entities |
| reqwest | 0.13 | HTTP client | Already used via PipeliteClient |
| chrono | 0.4 | Timestamp handling | Already used in cache and output |
| dialoguer | 0.12 | Interactive prompts (Input, Confirm, FuzzySelect) | Already used for all create/update |
| clap_complete | 4.6 | ArgValueCandidates for completions | Already used for all entities |

### Supporting
All existing supporting libraries (colored, comfy-table, csv, anyhow, thiserror) are already present and sufficient.

**Installation:** No new packages needed.

## Architecture Patterns

### Recommended Project Structure
```
src/
  api/
    mod.rs          # Add workflow CRUD + trigger methods
    models.rs       # Add Workflow, WorkflowCreate, WorkflowUpdate, TriggerConfig, WorkflowNode, WorkflowRunTrigger, WorkflowRunResponse
  cli/
    mod.rs          # Add Workflows variant to Commands enum
    workflows.rs    # NEW: WorkflowsCommands enum, all arg structs
  commands/
    mod.rs          # Add workflows module
    workflows/      # NEW directory
      mod.rs        # Dispatch subcommands
      list.rs       # List workflows
      get.rs        # Get single workflow
      create.rs     # Create workflow
      update.rs     # Update workflow
      delete.rs     # Delete workflow
      trigger.rs    # Trigger workflow run (NEW pattern)
  cache.rs          # Add KEY_WORKFLOWS, TTL_WORKFLOWS constants
  prompt.rs         # Add get_workflows_cached() helper
  commands/
    dashboard.rs    # Extend with workflow summary section
tests/
  workflows_integration.rs  # NEW: help/arg tests
```

### Pattern 1: Entity CRUD (replicate from deals)

**What:** Every entity follows the same structure: models -> client methods -> CLI args -> command handlers -> cache integration -> completions.

**When to use:** For all 5 CRUD subcommands (list, get, create, update, delete).

**Model pattern (from models.rs):**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub triggers: Option<Vec<serde_json::Value>>,  // Keep as Value for flexibility
    #[serde(default)]
    pub nodes: Option<Vec<serde_json::Value>>,      // Keep as Value for flexibility
    pub active: bool,
    #[serde(default)]
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

**Client method pattern (from api/mod.rs):**
```rust
pub async fn list_workflows(&self, params: &WorkflowsListParams) -> Result<ApiListResponse<Workflow>> {
    let url = format!("{}/api/v1/workflows", self.base_url);
    let query_pairs = params.to_query_pairs();
    let response = self.client.get(&url).query(&query_pairs).send().await.map_err(|e| self.map_request_error(e))?;
    self.handle_response(response).await
}
```

### Pattern 2: Trigger Command (new pattern)

**What:** POST to `/workflows/:id/run` with optional JSON body, return run_id + status.

**When to use:** Only for the `trigger` subcommand.

**Key differences from CRUD:**
- Response is `WorkflowRunResponse` (run_id, status), not the Workflow entity itself
- Request body is optional (WorkflowRunTrigger)
- No cache invalidation needed (triggering doesn't change the workflow)
- Output is a simple confirmation, not a full entity display

```rust
// In api/mod.rs
pub async fn trigger_workflow(&self, id: &str, data: Option<&serde_json::Value>) -> Result<WorkflowRunResponse> {
    let url = format!("{}/api/v1/workflows/{}/run", self.base_url, id);
    let mut req = self.client.post(&url);
    if let Some(body) = data {
        req = req.json(body);
    }
    let response = req.send().await.map_err(|e| self.map_request_error(e))?;
    let wrapper: ApiSingleResponse<WorkflowRunResponse> = self.handle_response(response).await?;
    Ok(wrapper.data)
}
```

### Pattern 3: Complex JSON Fields via Flags

**What:** Accept triggers and nodes as JSON strings via --triggers and --nodes flags.

**When to use:** For create and update commands where triggers/nodes cannot be prompted interactively.

```rust
// In CLI args
#[arg(long)]
pub triggers: Option<String>,  // Raw JSON string

#[arg(long)]
pub nodes: Option<String>,     // Raw JSON string

// In command handler - parse JSON string to Value
fn parse_json_flag(flag: &Option<String>, name: &str) -> Result<Option<Vec<serde_json::Value>>> {
    match flag {
        Some(s) => {
            let val: Vec<serde_json::Value> = serde_json::from_str(s)
                .map_err(|e| CliError::Validation {
                    detail: format!("Invalid JSON for --{}: {}", name, e),
                    hint: format!("--{} must be a valid JSON array.", name),
                })?;
            Ok(Some(val))
        }
        None => Ok(None),
    }
}
```

### Pattern 4: Dashboard Extension

**What:** Add a workflow summary section after the existing pipeline overview.

**When to use:** In the dashboard command's render functions.

The dashboard currently fetches pipelines/stages/deals and renders per-pipeline sections. The workflow summary should be a compact addition:

```
Workflows: 3 active, 12 runs today
```

This requires fetching workflows (auto-paginate) and counting `active: true`. The "runs today" count is NOT available from the workflow list API -- it would require a separate runs/stats endpoint that does not exist in the OpenAPI spec. The dashboard should show what is available: active count and total workflow count.

### Anti-Patterns to Avoid
- **Do NOT model TriggerConfig/WorkflowNode as full Rust enums for CLI input:** The discriminated union with type-specific fields is complex. Use `serde_json::Value` for flexibility and validate server-side.
- **Do NOT add interactive prompts for triggers/nodes:** Per D-09, these are JSON-only.
- **Do NOT poll for workflow run completion:** Per D-15, show run_id and pending status only.
- **Do NOT change existing entity patterns:** Follow the exact same structure used by deals/orgs/etc.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON parsing for --triggers/--nodes | Custom parser | `serde_json::from_str` | Handles all edge cases, good error messages |
| Active toggle prompt | Custom bool prompt | `dialoguer::Confirm` | Already used for pipeline is_default (Phase 04-02 decision) |
| Table column rendering | Custom workflow table | Existing `output::render_list`/`render_single` | Generic over serde_json::Value, works automatically |
| Completion candidates | Custom completion logic | `ArgValueCandidates` + `CacheStore::get` | Exact pattern from Phase 5 |

## Common Pitfalls

### Pitfall 1: Triggers/Nodes Serialization with skip_serializing_if
**What goes wrong:** Using `#[serde(skip_serializing_if = "Option::is_none")]` on triggers/nodes in WorkflowCreate will omit empty arrays, but the API default is `[]`.
**Why it happens:** The OpenAPI spec shows `default: []` for triggers and nodes on create.
**How to avoid:** Use `#[serde(skip_serializing_if = "Option::is_none")]` consistently -- if the user doesn't provide triggers/nodes, don't send them and let the server apply its default.
**Warning signs:** Server returns error about missing fields.

### Pitfall 2: WorkflowNode type Field Naming
**What goes wrong:** `type` is a reserved word in Rust.
**Why it happens:** The API uses `type` as a field name on both TriggerConfig and WorkflowNode.
**How to avoid:** Use `#[serde(rename = "type")]` with a Rust field named `node_type` or `trigger_type`. This exact pattern is already used for Stage's `stage_type` field (Phase 03-03 decision).
**Warning signs:** Compilation error on `type` field name.

### Pitfall 3: Dashboard "Runs Today" Data Unavailability
**What goes wrong:** D-12 mentions "12 runs today" but the workflow API has no runs/stats endpoint.
**Why it happens:** The OpenAPI spec only has CRUD + trigger run, no runs listing endpoint.
**How to avoid:** Show only what the API provides: active workflow count and total count. Omit "runs today" or show a simplified summary. The planner should note this limitation.
**Warning signs:** Trying to call a non-existent API endpoint.

### Pitfall 4: Trigger Data Flag as JSON String
**What goes wrong:** Users pass malformed JSON to --data flag.
**Why it happens:** Inline JSON on the command line requires careful quoting.
**How to avoid:** Parse with `serde_json::from_str` and give clear validation errors. Consider also accepting `@filepath` syntax for reading JSON from a file (Claude's Discretion area).
**Warning signs:** Confusing error messages about JSON syntax.

### Pitfall 5: Cache Key Collision
**What goes wrong:** Using "workflows" cache key that might collide with future entities.
**Why it happens:** Cache keys are simple strings in a flat namespace.
**How to avoid:** Follow existing convention: `KEY_WORKFLOWS = "workflows"` matches KEY_PIPELINES = "pipelines", KEY_DEALS = "deals" etc. No collision risk with current keys.

## Code Examples

Verified patterns from existing codebase:

### Entity Model Registration (models.rs)
```rust
// Source: src/api/models.rs - follow exact pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub triggers: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub nodes: Option<Vec<serde_json::Value>>,
    pub active: bool,
    #[serde(default)]
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub fn workflows_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "name", "active", "updated_at"],
    }
}
```

### CLI Command Registration (cli/mod.rs)
```rust
// Source: src/cli/mod.rs - add to Commands enum
/// Manage workflows
#[command(
    subcommand,
    alias = "w",
    after_help = "Examples:\n  pipelite workflows list\n  pipelite workflows get wf_abc123\n  pipelite workflows trigger wf_abc123"
)]
Workflows(WorkflowsCommands),
```

### Cache Constants (cache.rs)
```rust
// Source: src/cache.rs - add alongside existing constants
pub const TTL_WORKFLOWS: u64 = 3600;  // Workflows change rarely, same as pipelines
pub const KEY_WORKFLOWS: &str = "workflows";
```

### Cache-Through Helper (prompt.rs)
```rust
// Source: src/prompt.rs - follow get_pipelines_cached pattern
pub async fn get_workflows_cached(
    cache: Option<&CacheStore>,
    client: &PipeliteClient,
) -> Result<Vec<(String, String)>> {
    if let Some(store) = cache {
        if let Some(cached) = store.get::<Vec<(String, String)>>(KEY_WORKFLOWS) {
            return Ok(cached);
        }
    }
    // Auto-paginate workflow list, extract (id, name) pairs
    // Cache on success with TTL_WORKFLOWS
    // ...
}
```

### Completion Candidates (cli/workflows.rs)
```rust
// Source: src/cli/deals.rs - follow deal_id_candidates pattern
fn workflow_id_candidates() -> Vec<CompletionCandidate> {
    let cache = match CacheStore::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let items: Vec<(String, String)> = cache
        .get(crate::cache::KEY_WORKFLOWS)
        .unwrap_or_default();
    items
        .into_iter()
        .map(|(id, name)| CompletionCandidate::new(id).help(Some(name.into())))
        .collect()
}
```

### Main Dispatch (main.rs)
```rust
// Source: src/main.rs - add to run() match
Commands::Workflows(ref cmd) => {
    let ctx = AppContext::build(&cli)?;
    commands::workflows::run(&ctx, cmd).await
}
```

## Discretion Recommendations

These are areas marked as Claude's Discretion in CONTEXT.md:

### Table Columns
- **List view:** `id`, `name`, `active`, `updated_at` -- compact 4-column default matching pipelines pattern
- **Get view:** All fields. For triggers/nodes, show count summary in table mode (e.g., "2 triggers, 5 nodes"), full JSON in JSON mode

### Trigger/Node Rendering in Tables
- **Table format:** Show summary counts: "2 triggers" and "5 nodes" columns, not full JSON
- **JSON format:** Full nested objects rendered as-is via serde

### Dashboard Placement
- Add workflow summary AFTER the pipeline sections, as a compact single line
- Format: `Workflows: {active_count} active of {total_count} total`
- Note: "runs today" (D-12) cannot be implemented without a runs API endpoint; use what the API provides

### Cache Key Structure
- `KEY_WORKFLOWS = "workflows"` with `TTL_WORKFLOWS = 3600` (1 hour, same as pipelines -- workflows are slow-changing config)

### --data Flag Format
- Accept inline JSON string: `--data '{"dealId": "deal_123"}'`
- Also support `@filepath` syntax: `--data @payload.json` -- reads file contents as JSON
- This pattern is common in CLI tools (curl uses it) and helps with complex JSON payloads

### --from-file Flag
- Not needed as separate flag. The `--stdin` pattern already handles full workflow JSON input for create/update. The `@filepath` syntax on `--data` covers trigger payloads.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | assert_cmd 2.0 + predicates 3.0 |
| Config file | Cargo.toml [dev-dependencies] |
| Quick run command | `cargo test --test workflows_integration` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WF-01 | workflows help shows subcommands | integration | `cargo test --test workflows_integration` | No - Wave 0 |
| WF-02 | workflows list help shows flags | integration | `cargo test --test workflows_integration` | No - Wave 0 |
| WF-03 | workflows create help shows flags | integration | `cargo test --test workflows_integration` | No - Wave 0 |
| WF-04 | workflows trigger help shows flags | integration | `cargo test --test workflows_integration` | No - Wave 0 |
| WF-05 | workflows get without id fails exit 2 | integration | `cargo test --test workflows_integration` | No - Wave 0 |
| WF-06 | dry-run on create/update/delete/trigger | integration | `cargo test --test dry_run_test` | Extend existing |
| WF-07 | model serialization (Workflow, WorkflowCreate) | unit | `cargo test --lib api::models` | No - Wave 0 |
| WF-08 | headless create without required flags fails | integration | `cargo test --test headless_test` | Extend existing |

### Sampling Rate
- **Per task commit:** `cargo test --test workflows_integration`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/workflows_integration.rs` -- help/arg validation tests (follow deals_integration.rs pattern)
- [ ] Model unit tests in `src/api/models.rs` -- Workflow deserialization, WorkflowCreate serialization

## Integration Points Summary

All files that need modification or creation:

| File | Action | Purpose |
|------|--------|---------|
| `src/api/models.rs` | MODIFY | Add Workflow, WorkflowCreate, WorkflowUpdate, WorkflowRunTrigger, WorkflowRunResponse models + table config |
| `src/api/mod.rs` | MODIFY | Add workflow CRUD methods + trigger_workflow + WorkflowsListParams to PipeliteClient, add Workflow imports |
| `src/cli/mod.rs` | MODIFY | Add `workflows` module, import WorkflowsCommands, add Workflows variant to Commands enum |
| `src/cli/workflows.rs` | CREATE | WorkflowsCommands enum, all arg structs, completion candidate functions |
| `src/commands/mod.rs` | MODIFY | Add `pub mod workflows;` |
| `src/commands/workflows/mod.rs` | CREATE | Dispatch subcommands |
| `src/commands/workflows/list.rs` | CREATE | List handler |
| `src/commands/workflows/get.rs` | CREATE | Get handler |
| `src/commands/workflows/create.rs` | CREATE | Create handler with JSON flag parsing |
| `src/commands/workflows/update.rs` | CREATE | Update handler with JSON flag parsing |
| `src/commands/workflows/delete.rs` | CREATE | Delete handler |
| `src/commands/workflows/trigger.rs` | CREATE | Trigger handler (new pattern) |
| `src/commands/dashboard.rs` | MODIFY | Add workflow summary section |
| `src/cache.rs` | MODIFY | Add KEY_WORKFLOWS, TTL_WORKFLOWS constants |
| `src/prompt.rs` | MODIFY | Add get_workflows_cached() helper, add WorkflowsListParams import |
| `src/main.rs` | MODIFY | Add Workflows match arm |
| `tests/workflows_integration.rs` | CREATE | Integration tests |

## Sources

### Primary (HIGH confidence)
- `../pipelite/public/openapi.yaml` lines 833-1006 -- Workflow schemas (Workflow, WorkflowCreate, WorkflowUpdate, TriggerConfig, WorkflowNode, WorkflowRunTrigger, WorkflowRunResponse)
- `../pipelite/public/openapi.yaml` lines 2236-2394 -- Workflow API endpoints
- `src/api/mod.rs` -- Existing PipeliteClient CRUD patterns
- `src/api/models.rs` -- Existing entity model patterns with serde attributes
- `src/cli/deals.rs` -- Existing CLI arg/completion pattern
- `src/commands/deals/` -- Existing command handler patterns (all 5 CRUD files)
- `src/cache.rs` -- Existing cache TTL/key constants and CacheStore API
- `src/prompt.rs` -- Existing cache-through and prompt helpers
- `src/commands/dashboard.rs` -- Existing dashboard rendering to extend

### Secondary (MEDIUM confidence)
- Dashboard "runs today" (D-12) -- Cannot be fully implemented; OpenAPI spec has no workflow runs listing endpoint. This is verified against the spec.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, reusing existing stack entirely
- Architecture: HIGH -- following established patterns from 5 completed phases
- Pitfalls: HIGH -- identified from direct code inspection and OpenAPI spec review

**Research date:** 2026-03-29
**Valid until:** 2026-04-28 (stable -- internal project patterns, no external API changes expected)
