# Phase 3: Full Entity Coverage - Research

**Researched:** 2026-03-24
**Domain:** Rust CLI CRUD replication across 5 CRM entities (orgs, people, activities, pipelines, stages)
**Confidence:** HIGH

## Summary

Phase 3 is a mechanical replication phase. The deals implementation (Phase 2) established a proven pattern: CLI args (clap derive) -> command handler -> API client method -> generic output layer. Each of the 5 new entities (organizations, people, activities, pipelines, stages) needs the same 5 operations (list, get, create, update, delete) following the identical code structure.

The Pipelite CRM API has been confirmed live at localhost:3001 with full OpenAPI documentation. All entity schemas, required/optional fields, filter parameters, and batch endpoint availability have been verified against the API spec. Three entities have batch endpoints (deals, organizations, people); activities, pipelines, and stages do not -- requiring individual-create loop fallback for `--stdin` on those entities.

The output layer (table, json, csv, plain) is already fully generic (`serde_json::Value`), so no output code changes are needed. The work is entirely: (1) data model structs, (2) API client methods, (3) CLI arg definitions, (4) command handlers, and (5) integration tests per entity.

**Primary recommendation:** Implement one entity at a time in order of complexity (orgs first as simplest, then people, pipelines, stages, activities last as most complex). Each entity follows the exact deals pattern -- copy-adapt, do not reinvent.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Both `pipelite pipelines` and `pipelite stages` are top-level commands (flat, consistent with all other entities)
- `stages list` requires `--pipeline <id>` -- error with hint if omitted
- `stages create` requires `--pipeline <id>` as a named flag (not positional)
- `stages get/update/delete` take stage ID as positional arg (no --pipeline needed -- ID is unique)
- Short alias: `s` for stages, `pl` or similar for pipelines (Claude's discretion on pipeline alias to avoid collisions)
- Default table columns: Organizations (id, name, owner_id, updated_at), People (id, name, email, organization_id, updated_at), Activities (id, type, subject, deal_id, done, due_date), Pipelines (id, name, updated_at), Stages (id, name, pipeline_id, position)
- All entities: `get <id>` shows full key-value pairs (vertical layout), same as deals
- Entity-specific filter flags: Organizations (`--owner`), People (`--org`, `--owner`), Activities (`--deal`, `--type`, `--done`), Pipelines (none), Stages (`--pipeline` required for list)
- All entities support `--stdin` batch create
- If API has a /batch endpoint, use it; otherwise loop individual creates
- Individual-create loop shows progress: "Creating 3/10..."
- Partial failures continue processing -- report summary at end: "Created 8/10. 2 failed (see errors above)."
- Non-zero exit code if any item fails

### Claude's Discretion
- Exact data model fields per entity (researcher confirms against API spec)
- Required vs optional fields for create payloads per entity
- Pipeline short alias choice (avoid collisions with `p` for people)
- Activity type values (call, email, meeting, etc. -- from API)
- How `--done` flag works (toggle vs explicit true/false)
- Internal code organization (one module per entity vs shared generic)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| ORG-01 | List organizations with `pipelite orgs list` | API GET /organizations confirmed with offset/limit/expand params |
| ORG-02 | Get org by ID with `pipelite orgs get <id>` | API GET /organizations/{id} confirmed |
| ORG-03 | Create org with `pipelite orgs create` | API POST /organizations, required: name only. Batch: /organizations/batch exists |
| ORG-04 | Update org with `pipelite orgs update <id>` | API PUT /organizations/{id}, all fields optional |
| ORG-05 | Delete org with `pipelite orgs delete <id>` | API DELETE /organizations/{id} confirmed |
| PEOP-01 | List people with `pipelite people list` | API GET /people confirmed with offset/limit/expand params |
| PEOP-02 | Get person by ID with `pipelite people get <id>` | API GET /people/{id} confirmed |
| PEOP-03 | Create person with `pipelite people create` | API POST /people, required: first_name + last_name. Batch: /people/batch exists |
| PEOP-04 | Update person with `pipelite people update <id>` | API PUT /people/{id}, all fields optional |
| PEOP-05 | Delete person with `pipelite people delete <id>` | API DELETE /people/{id} confirmed |
| ACTV-01 | List activities with `pipelite activities list` | API GET /activities with type_id, deal_id, owner_id filters |
| ACTV-02 | Get activity by ID with `pipelite activities get <id>` | API GET /activities/{id} confirmed |
| ACTV-03 | Create activity with `pipelite activities create` | API POST /activities, required: title + type_id. NO batch endpoint -- use loop |
| ACTV-04 | Update activity with `pipelite activities update <id>` | API PUT /activities/{id}, all fields optional including completed_at |
| ACTV-05 | Delete activity with `pipelite activities delete <id>` | API DELETE /activities/{id} confirmed |
| PIPE-01 | List pipelines with `pipelite pipelines list` | API GET /pipelines with offset/limit/expand (owner, stages) |
| PIPE-02 | Get pipeline by ID with `pipelite pipelines get <id>` | API GET /pipelines/{id} confirmed |
| PIPE-03 | Create pipeline with `pipelite pipelines create` | API POST /pipelines, required: name. NO batch endpoint -- use loop |
| PIPE-04 | Update pipeline with `pipelite pipelines update <id>` | API PUT /pipelines/{id}, optional: name, is_default |
| PIPE-05 | Delete pipeline with `pipelite pipelines delete <id>` | API DELETE /pipelines/{id} confirmed |
| STAG-01 | List stages with `pipelite stages list` | API GET /stages with pipeline_id filter (required per user decision) |
| STAG-02 | Get stage by ID with `pipelite stages get <id>` | API GET /stages/{id} confirmed |
| STAG-03 | Create stage with `pipelite stages create` | API POST /stages, required: name + pipeline_id. NO batch endpoint -- use loop |
| STAG-04 | Update stage with `pipelite stages update <id>` | API PUT /stages/{id}, optional: name, color, type, description |
| STAG-05 | Delete stage with `pipelite stages delete <id>` | API DELETE /stages/{id} confirmed |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.6 | CLI arg parsing with derive | Already in use, derive API for subcommands |
| reqwest | 0.13 | HTTP client | Already in use, handles all API calls |
| serde / serde_json | 1.0 | Serialization | Already in use for all entity models |
| tokio | 1 | Async runtime | Already in use (current_thread) |
| anyhow | 1.0 | Error propagation | Already in use throughout |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| indicatif | 0.18 | Progress bars | Batch create loop "Creating 3/10..." |
| comfy-table | 7.2 | Table output | Already used by output layer (no changes needed) |
| assert_cmd | 2.0 | Integration tests | CLI help/arg validation tests per entity |
| predicates | 3.0 | Test assertions | String matching in integration tests |

No new dependencies are needed. Everything required is already in Cargo.toml.

## Architecture Patterns

### Recommended Project Structure
```
src/
├── api/
│   ├── mod.rs              # PipeliteClient -- add CRUD methods per entity
│   └── models.rs           # Entity structs, Create/Update payloads, table configs
├── cli/
│   ├── mod.rs              # Commands enum -- add 5 new variants
│   ├── deals.rs            # (existing) -- template for new entities
│   ├── orgs.rs             # OrgsCommands, list/get/create/update/delete args
│   ├── people.rs           # PeopleCommands, list/get/create/update/delete args
│   ├── activities.rs       # ActivitiesCommands, list/get/create/update/delete args
│   ├── pipelines.rs        # PipelinesCommands, list/get/create/update/delete args
│   └── stages.rs           # StagesCommands, list/get/create/update/delete args
├── commands/
│   ├── mod.rs              # Add entity modules
│   ├── deals/              # (existing) -- template for new entities
│   ├── orgs/               # list.rs, get.rs, create.rs, update.rs, delete.rs
│   ├── people/             # list.rs, get.rs, create.rs, update.rs, delete.rs
│   ├── activities/         # list.rs, get.rs, create.rs, update.rs, delete.rs
│   ├── pipelines/          # list.rs, get.rs, create.rs, update.rs, delete.rs
│   └── stages/             # list.rs, get.rs, create.rs, update.rs, delete.rs
└── main.rs                 # Add match arms for 5 new commands
```

### Pattern: Entity CRUD Module (copy from deals)
**What:** Each entity gets a CLI definition file, a commands directory with 5 handler files, API client methods, and model structs.
**When to use:** Every entity in this phase.

**CLI definition template (e.g., `src/cli/orgs.rs`):**
```rust
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum OrgsCommands {
    /// List organizations
    #[command(after_help = "Examples:\n  pipelite orgs list\n  ...")]
    List(OrgsListArgs),
    Get(OrgsGetArgs),
    Create(OrgsCreateArgs),
    Update(OrgsUpdateArgs),
    Delete(OrgsDeleteArgs),
}

#[derive(Args)]
pub struct OrgsListArgs {
    #[arg(long)]
    pub owner: Option<String>,  // entity-specific filter
    #[arg(long, default_value = "50")]
    pub limit: u64,
    #[arg(long, default_value = "0")]
    pub offset: u64,
    #[arg(long)]
    pub all: bool,
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
    #[arg(long, value_delimiter = ',')]
    pub expand: Option<Vec<String>>,
}
// ... Get, Create, Update, Delete args follow same pattern as deals
```

**Command enum registration (in `src/cli/mod.rs`):**
```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing variants
    #[command(subcommand, alias = "o")]
    Orgs(OrgsCommands),
    #[command(subcommand, alias = "p")]
    People(PeopleCommands),
    #[command(subcommand, alias = "a")]
    Activities(ActivitiesCommands),
    #[command(subcommand, alias = "pl")]
    Pipelines(PipelinesCommands),
    #[command(subcommand, alias = "s")]
    Stages(StagesCommands),
}
```

### Pattern: Batch Create with Loop Fallback
**What:** For entities without a /batch API endpoint (activities, pipelines, stages), `--stdin` reads JSON array and creates items one-by-one with progress and partial failure handling.
**When to use:** Activities, pipelines, stages `--stdin` create.

```rust
async fn batch_create_loop(ctx: &AppContext, items: Vec<EntityCreate>) -> Result<()> {
    let total = items.len();
    let mut created = Vec::new();
    let mut failed = 0u32;

    for (i, item) in items.iter().enumerate() {
        if !ctx.quiet {
            eprint!("\rCreating {}/{}...", i + 1, total);
        }
        match ctx.client.create_entity(item).await {
            Ok(entity) => created.push(entity),
            Err(e) => {
                eprintln!("\nFailed item {}: {}", i + 1, e);
                failed += 1;
            }
        }
    }
    if !ctx.quiet {
        eprintln!(); // newline after progress
    }

    // Render successfully created items
    // ...

    if failed > 0 {
        eprintln!("Created {}/{}. {} failed (see errors above).",
            created.len(), total, failed);
        std::process::exit(1);
    }
    Ok(())
}
```

### Pattern: Required Pipeline Filter on Stages List
**What:** `stages list` validates that `--pipeline` is provided before making the API call, erroring with a hint if omitted.
**When to use:** Stages list command only.

```rust
pub async fn run(ctx: &AppContext, args: &StagesListArgs) -> Result<()> {
    let pipeline_id = args.pipeline.as_ref().ok_or_else(|| CliError::Validation {
        detail: "Missing required flag: --pipeline".to_string(),
        hint: "Usage: pipelite stages list --pipeline <pipeline_id>".to_string(),
    })?;
    // proceed with API call using pipeline_id filter
}
```

### Anti-Patterns to Avoid
- **Do NOT create a generic/macro entity system:** Copy-paste from deals and adapt. Each entity has unique fields, filters, and required params. Abstractions would save little code and make each entity harder to modify independently.
- **Do NOT share `parse_custom_fields()` between modules yet:** It exists in deals create.rs and update.rs. Extracting to a util module is fine but not required -- focus on correctness per entity first.
- **Do NOT add `--done` as a boolean toggle for activities:** The API has `completed_at` (datetime), not a boolean `done` field. The `--done` filter flag from CONTEXT.md should filter on whether `completed_at` is set. For the create/update command, use `--completed-at <datetime>` instead.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON serialization | Custom field serializers | `#[serde(skip_serializing_if = "Option::is_none")]` | Proven pattern from deals, handles null omission |
| Query param building | String concatenation | `to_query_pairs()` + reqwest `.query()` | Handles URL encoding, proven pattern |
| Progress output | Custom counters | `eprint!("\rCreating {}/{}...", i, total)` | Simple, no dependency needed for basic progress |
| Table column config | Dynamic column detection | `TableConfig { default_columns: vec![...] }` | Static config per entity, proven pattern |

## Common Pitfalls

### Pitfall 1: API Response Wrapper Inconsistency
**What goes wrong:** Single-item responses are wrapped in `ApiSingleResponse<T>` (with `.data` field). Forgetting to unwrap causes deserialization failures.
**Why it happens:** Phase 2 hit this exact bug (see 02-03 gap closure fix in STATE.md).
**How to avoid:** Always use `let wrapper: ApiSingleResponse<T> = self.handle_response(response).await?; Ok(wrapper.data)` for get/create/update. List uses `ApiListResponse<T>` directly.
**Warning signs:** "Failed to parse response" errors at runtime.

### Pitfall 2: CONTEXT.md Column Names vs API Field Names
**What goes wrong:** CONTEXT.md says Activities default columns include "type", "subject", "done", "due_date". The actual API fields are `type_id`, `title`, `completed_at`, `due_at`.
**Why it happens:** User decisions were made before API schema verification.
**How to avoid:** Use actual API field names in table configs and code. Map the user intent: "type" -> `type_id`, "subject" -> `title`, "done" -> presence of `completed_at`, "due_date" -> `due_at`. For the table config, use the real field names: `["id", "title", "type_id", "deal_id", "due_at", "completed_at"]`.
**Warning signs:** Empty columns in table output.

### Pitfall 3: People Have first_name/last_name, Not "name"
**What goes wrong:** CONTEXT.md says People default columns include "name". The API has `first_name`, `last_name`, and a computed `full_name` (read-only).
**Why it happens:** Natural assumption that "name" is a single field.
**How to avoid:** Default table columns for people should use `full_name` (or `first_name` + `last_name`). Create/update flags should be `--first-name` and `--last-name`. The `full_name` field is read-only (computed by server).
**Warning signs:** Missing name in output, or create failing because "name" is not a valid field.

### Pitfall 4: Filter Flags Without API Support
**What goes wrong:** CONTEXT.md specifies `--owner` filter for orgs and `--org`/`--owner` for people, but the API's OpenAPI spec does not list these as query parameters on those endpoints.
**Why it happens:** Assumed API supports common filters.
**How to avoid:** Still add the CLI flags (the API may accept undocumented params, or filtering can be done client-side for small datasets). But: send the param as a query parameter anyway -- many REST APIs accept filter params not in the OpenAPI spec. If the API ignores them, the results will just be unfiltered. Test against the real API.
**Warning signs:** Filter flags having no effect on results.

### Pitfall 5: Stages List Without --pipeline Returns All Stages
**What goes wrong:** The user decision says `--pipeline` is required for `stages list`, but the API accepts it as optional.
**Why it happens:** UX decision to enforce context (stages without a pipeline are meaningless to browse).
**How to avoid:** Validate `--pipeline` is present in the CLI handler before calling the API (runtime validation, same pattern as deals create validates `--title`).

### Pitfall 6: Batch Endpoint Availability
**What goes wrong:** Assuming all entities have batch endpoints like deals.
**Why it happens:** Deals has `/deals/batch`. Only orgs and people also have batch endpoints.
**How to avoid:** Use batch endpoint for deals, orgs, people. Use individual-create loop for activities, pipelines, stages. The CONTEXT.md already specifies the fallback behavior.

## Entity Data Models (from API spec)

### Organization
| Field | Type | Required (Create) | Notes |
|-------|------|-------------------|-------|
| id | String | - | Read-only |
| name | String | YES | maxLength: 100 |
| website | Option<String> | no | URI format |
| industry | Option<String> | no | maxLength: 100 |
| notes | Option<String> | no | |
| owner_id | String | - | Read-only (set by server) |
| custom_fields | Option<Value> | no | |
| created_at | String | - | Read-only |
| updated_at | String | - | Read-only |

**API endpoints:** GET/POST /organizations, GET/PUT/DELETE /organizations/{id}, POST /organizations/batch
**List filters:** offset, limit, expand (owner)
**CLI flags for create:** `--name` (required), `--website`, `--industry`, `--notes`, `--custom-field`

### Person
| Field | Type | Required (Create) | Notes |
|-------|------|-------------------|-------|
| id | String | - | Read-only |
| first_name | String | YES | maxLength: 100 |
| last_name | String | YES | maxLength: 100 |
| full_name | Option<String> | - | Computed read-only |
| email | Option<String> | no | Email format |
| phone | Option<String> | no | maxLength: 50 |
| notes | Option<String> | no | |
| organization_id | Option<String> | no | |
| owner_id | String | - | Read-only |
| custom_fields | Option<Value> | no | |
| created_at | String | - | Read-only |
| updated_at | String | - | Read-only |

**API endpoints:** GET/POST /people, GET/PUT/DELETE /people/{id}, POST /people/batch
**List filters:** offset, limit, expand (owner, organization)
**CLI flags for create:** `--first-name` (required), `--last-name` (required), `--email`, `--phone`, `--notes`, `--org`, `--custom-field`

### Activity
| Field | Type | Required (Create) | Notes |
|-------|------|-------------------|-------|
| id | String | - | Read-only |
| title | String | YES | |
| type_id | String | YES | Activity type ID (call, meeting, task, email) |
| deal_id | Option<String> | no | |
| owner_id | String | no (defaults to auth user) | |
| due_at | Option<String> | no | ISO datetime |
| completed_at | Option<String> | no | ISO datetime, marks done |
| notes | Option<String> | no | |
| custom_fields | Option<Value> | no | |
| created_at | String | - | Read-only |
| updated_at | String | - | Read-only |

**API endpoints:** GET/POST /activities, GET/PUT/DELETE /activities/{id}. NO batch endpoint.
**List filters:** type_id, deal_id, owner_id, offset, limit, expand (type, deal, owner)
**CLI flags for create:** `--title` (required), `--type` (required, maps to type_id), `--deal`, `--due-at`, `--notes`, `--custom-field`
**CLI flags for update:** all create flags plus `--completed-at`
**ActivityType schema:** id, name, icon (nullable), color (nullable) -- these are reference data

### Pipeline
| Field | Type | Required (Create) | Notes |
|-------|------|-------------------|-------|
| id | String | - | Read-only |
| name | String | YES | |
| is_default | bool | no | Default: false |
| owner_id | String | - | Read-only |
| created_at | String | - | Read-only |
| updated_at | String | - | Read-only |

**API endpoints:** GET/POST /pipelines, GET/PUT/DELETE /pipelines/{id}. NO batch endpoint.
**List filters:** offset, limit, expand (owner, stages)
**CLI flags for create:** `--name` (required), `--default` (boolean flag for is_default)
**CLI flags for update:** `--name`, `--default`

### Stage
| Field | Type | Required (Create) | Notes |
|-------|------|-------------------|-------|
| id | String | - | Read-only |
| pipeline_id | String | YES | |
| name | String | YES | |
| description | Option<String> | no | |
| color | Option<String> | no | Default: "blue" |
| type | String | no | Enum: open, won, lost. Default: open |
| position | i64 | - | Read-only (set by server) |
| created_at | String | - | Read-only |
| updated_at | String | - | Read-only |

**API endpoints:** GET/POST /stages, GET/PUT/DELETE /stages/{id}. NO batch endpoint.
**List filters:** pipeline_id, offset, limit, expand (pipeline)
**CLI flags for create:** `--name` (required), `--pipeline` (required), `--color`, `--type` (open/won/lost), `--description`
**CLI flags for update:** `--name`, `--color`, `--type`, `--description`

## Code Examples

### API Client Method (Organizations example)
```rust
// Source: Existing deals pattern in src/api/mod.rs
pub async fn list_orgs(
    &self,
    params: &OrgsListParams,
) -> Result<ApiListResponse<Organization>> {
    let url = format!("{}/api/v1/organizations", self.base_url);
    let query_pairs = params.to_query_pairs();
    let response = self
        .client
        .get(&url)
        .query(&query_pairs)
        .send()
        .await
        .map_err(|e| self.map_request_error(e))?;
    self.handle_response(response).await
}

pub async fn get_org(&self, id: &str, expand: Option<&[String]>) -> Result<Organization> {
    let url = format!("{}/api/v1/organizations/{}", self.base_url, id);
    let mut req = self.client.get(&url);
    if let Some(expand) = expand {
        req = req.query(&[("expand", expand.join(","))]);
    }
    let response = req.send().await.map_err(|e| self.map_request_error(e))?;
    let wrapper: ApiSingleResponse<Organization> = self.handle_response(response).await?;
    Ok(wrapper.data)
}
```

### Individual-Create Loop (for entities without batch endpoint)
```rust
// Source: CONTEXT.md batch/stdin decision
async fn batch_create_loop(ctx: &AppContext, items: Vec<ActivityCreate>) -> Result<()> {
    let total = items.len();
    let mut created_items: Vec<serde_json::Value> = Vec::new();
    let mut failed: u32 = 0;

    for (i, item) in items.iter().enumerate() {
        if !ctx.quiet {
            eprint!("\rCreating {}/{}...", i + 1, total);
        }
        match ctx.client.create_activity(item).await {
            Ok(entity) => {
                if let Ok(val) = serde_json::to_value(&entity) {
                    created_items.push(val);
                }
            }
            Err(e) => {
                eprintln!("\nFailed item {}: {}", i + 1, e);
                failed += 1;
            }
        }
    }
    if !ctx.quiet && total > 0 {
        eprintln!(); // clear progress line
    }

    // Render created items
    let config = activities_table_config();
    let columns: Vec<String> = config.default_columns.iter().map(|s| s.to_string()).collect();
    output::render_list(&created_items, &ctx.output_format, &columns, &None, ctx.color, None)?;

    if failed > 0 {
        eprintln!("Created {}/{}. {} failed (see errors above).", created_items.len(), total, failed);
        // Return error to trigger non-zero exit
        return Err(CliError::Api {
            status: 0,
            detail: format!("{} of {} items failed", failed, total),
            hint: "Review errors above and retry failed items.".to_string(),
        }.into());
    }
    Ok(())
}
```

## Discretion Recommendations

### Pipeline Short Alias
**Recommendation:** Use `pl` for pipelines. The alias `p` would collide with `people` (which should get `p`). Using `pi` reads awkwardly. `pl` is unambiguous.

Entity alias mapping:
- `deals` -> `d`
- `orgs` -> `o`
- `people` -> `p`
- `activities` -> `a`
- `pipelines` -> `pl`
- `stages` -> `s`

### Activity --done Flag
**Recommendation:** The `--done` filter on `activities list` should map to a boolean-style filter. Since the API uses `completed_at` (a datetime), implement `--done` as a convenience flag that the CLI sends as a query param (the API may not support it directly). If the API doesn't support a "done" filter, apply client-side: filter results where `completed_at` is not null. For `activities update`, provide `--mark-done` to set `completed_at` to current time and `--mark-undone` to clear it (set to null).

### Internal Code Organization
**Recommendation:** One module per entity (matching the deals pattern). Do NOT create a generic/macro system. The code duplication across 5 entities is manageable (~30 files total) and each entity has enough unique fields, filters, and validation that a generic approach would add complexity without significant benefit.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| N/A | Copy-adapt deals pattern | Phase 2 established | All entities follow same structure |

No deprecated features or recent changes affect this phase. The stack is stable and already proven.

## Open Questions

1. **Organizations/People filter params not in OpenAPI spec**
   - What we know: The OpenAPI spec for GET /organizations and GET /people only lists offset, limit, expand -- no owner_id or organization_id filter params.
   - What's unclear: Whether the API accepts these params anyway (many APIs accept undocumented query params).
   - Recommendation: Add the CLI flags as decided. Send the params as query parameters. If the API ignores them, the behavior is "returns all" which is acceptable. Test against real API during implementation.

2. **Activity "done" filtering**
   - What we know: API has `completed_at` field (datetime, nullable), not a boolean `done`.
   - What's unclear: Whether the API supports filtering by "completed" status.
   - Recommendation: Add `--done` flag to CLI. Try sending `completed=true` as query param. If API doesn't support it, filter client-side (small performance cost, acceptable for v1).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + assert_cmd 2.0 for integration |
| Config file | Cargo.toml `[dev-dependencies]` |
| Quick run command | `cargo test --test <test_file> -- --nocapture` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ORG-01 | orgs list subcommand parses | integration | `cargo test --test orgs_integration -- --nocapture` | No - Wave 0 |
| ORG-02 | orgs get requires ID arg | integration | `cargo test --test orgs_integration -- --nocapture` | No - Wave 0 |
| ORG-03 | orgs create shows required flags in help | integration | `cargo test --test orgs_integration -- --nocapture` | No - Wave 0 |
| ORG-04 | orgs update shows flags in help | integration | `cargo test --test orgs_integration -- --nocapture` | No - Wave 0 |
| ORG-05 | orgs delete requires ID arg | integration | `cargo test --test orgs_integration -- --nocapture` | No - Wave 0 |
| PEOP-01 | people list subcommand parses | integration | `cargo test --test people_integration -- --nocapture` | No - Wave 0 |
| PEOP-02 | people get requires ID arg | integration | `cargo test --test people_integration -- --nocapture` | No - Wave 0 |
| PEOP-03 | people create shows required flags | integration | `cargo test --test people_integration -- --nocapture` | No - Wave 0 |
| PEOP-04 | people update shows flags in help | integration | `cargo test --test people_integration -- --nocapture` | No - Wave 0 |
| PEOP-05 | people delete requires ID arg | integration | `cargo test --test people_integration -- --nocapture` | No - Wave 0 |
| ACTV-01 | activities list subcommand parses | integration | `cargo test --test activities_integration -- --nocapture` | No - Wave 0 |
| ACTV-02 | activities get requires ID arg | integration | `cargo test --test activities_integration -- --nocapture` | No - Wave 0 |
| ACTV-03 | activities create shows required flags | integration | `cargo test --test activities_integration -- --nocapture` | No - Wave 0 |
| ACTV-04 | activities update shows flags in help | integration | `cargo test --test activities_integration -- --nocapture` | No - Wave 0 |
| ACTV-05 | activities delete requires ID arg | integration | `cargo test --test activities_integration -- --nocapture` | No - Wave 0 |
| PIPE-01 | pipelines list subcommand parses | integration | `cargo test --test pipelines_integration -- --nocapture` | No - Wave 0 |
| PIPE-02 | pipelines get requires ID arg | integration | `cargo test --test pipelines_integration -- --nocapture` | No - Wave 0 |
| PIPE-03 | pipelines create shows required flags | integration | `cargo test --test pipelines_integration -- --nocapture` | No - Wave 0 |
| PIPE-04 | pipelines update shows flags in help | integration | `cargo test --test pipelines_integration -- --nocapture` | No - Wave 0 |
| PIPE-05 | pipelines delete requires ID arg | integration | `cargo test --test pipelines_integration -- --nocapture` | No - Wave 0 |
| STAG-01 | stages list subcommand parses | integration | `cargo test --test stages_integration -- --nocapture` | No - Wave 0 |
| STAG-02 | stages get requires ID arg | integration | `cargo test --test stages_integration -- --nocapture` | No - Wave 0 |
| STAG-03 | stages create shows required flags | integration | `cargo test --test stages_integration -- --nocapture` | No - Wave 0 |
| STAG-04 | stages update shows flags in help | integration | `cargo test --test stages_integration -- --nocapture` | No - Wave 0 |
| STAG-05 | stages delete requires ID arg | integration | `cargo test --test stages_integration -- --nocapture` | No - Wave 0 |

Additionally, unit tests in `src/api/models.rs` for each entity struct deserialization (same pattern as existing Deal tests).

### Sampling Rate
- **Per task commit:** `cargo test --test <entity>_integration -- --nocapture`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full `cargo test` green before verify

### Wave 0 Gaps
- [ ] `tests/orgs_integration.rs` -- covers ORG-01 through ORG-05
- [ ] `tests/people_integration.rs` -- covers PEOP-01 through PEOP-05
- [ ] `tests/activities_integration.rs` -- covers ACTV-01 through ACTV-05
- [ ] `tests/pipelines_integration.rs` -- covers PIPE-01 through PIPE-05
- [ ] `tests/stages_integration.rs` -- covers STAG-01 through STAG-05
- [ ] Unit tests in `src/api/models.rs` for Organization, Person, Activity, Pipeline, Stage deserialization

## Sources

### Primary (HIGH confidence)
- Pipelite CRM API OpenAPI spec at localhost:3001/api/v1/docs -- all entity schemas, endpoints, required fields, query parameters confirmed
- Existing codebase (src/api/mod.rs, src/api/models.rs, src/cli/deals.rs, src/commands/deals/*) -- proven patterns

### Secondary (MEDIUM confidence)
- CONTEXT.md user decisions -- locked design choices for filters, columns, batch behavior

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, using proven existing stack
- Architecture: HIGH - direct replication of proven deals pattern
- Entity data models: HIGH - verified against live API OpenAPI spec
- Pitfalls: HIGH - based on actual Phase 2 bugs (response wrapper) and API spec mismatches
- Filter support: MEDIUM - some filters (orgs/people owner) not confirmed in API spec

**Research date:** 2026-03-24
**Valid until:** 2026-04-24 (stable domain, unlikely to change)
