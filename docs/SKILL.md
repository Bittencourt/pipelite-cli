# Pipelite CLI -- AI Agent Skill File

> This file provides structured context for AI coding agents (Claude Code, Cursor, Copilot, Aider, etc.) to understand and work effectively with the Pipelite CLI codebase.

## Project Identity

- **Name**: Pipelite CLI (`pipelite`)
- **Language**: Rust (edition 2024)
- **Type**: CLI application for CRM management
- **Binary**: `pipelite`
- **Version**: 0.1.0
- **Runtime**: Tokio single-threaded async

## What This Project Does

Pipelite CLI is a terminal-based interface for the Pipelite CRM platform. It provides CRUD operations for 7 entity types (deals, organizations, people, activities, pipelines, stages, workflows) plus utility commands (dashboard, cache, config, completions). It communicates with the Pipelite REST API using Bearer token authentication.

## Quick Orientation

```
src/
  main.rs              # Entry point -- splash screen check, clap parse, run()
  cli/                 # Argument definitions only (clap derive)
    mod.rs             # Top-level Cli struct + Commands enum
    deals.rs           # DealsCommands enum + arg structs
    ...                # One file per entity
  commands/            # Business logic (handlers)
    deals/             # list.rs, get.rs, create.rs, update.rs, delete.rs
    ...                # One directory per entity
    init.rs, ping.rs, dashboard.rs, completions.rs
  api/
    mod.rs             # PipeliteClient -- all HTTP methods (~1078 lines)
    models.rs          # Entity structs: Base, Create, Update (~830 lines)
  output/
    mod.rs             # render_list(), render_single(), detect_format()
    json.rs, table.rs, csv.rs, plain.rs  # Format-specific renderers
    format.rs          # Value formatting (dates, currency)
    fields.rs          # Dynamic field extraction from JSON
  config.rs            # TOML config I/O with env var override
  context.rs           # AppContext (single source of truth for all handlers)
  cache.rs             # TTL-based JSON file cache
  error.rs             # CliError enum with detail + hint
  dry_run.rs           # Mutation preview rendering
  prompt.rs            # Interactive dialoguer prompts
  splash.rs            # ASCII art splash screen
tests/
  *_integration.rs     # CLI integration tests using assert_cmd
```

## Key Patterns

### Entity CRUD Pattern

Every entity follows the same structure. When adding or modifying entities, follow this pattern exactly:

1. **CLI args** (`src/cli/<entity>.rs`):
   - `<Entity>Commands` enum with `List`, `Get`, `Create`, `Update`, `Delete` variants
   - Each variant wraps an args struct
   - Shell completion candidates from cache for ID arguments
   - Required fields typed as `Option<T>` for interactive prompting

2. **Command handlers** (`src/commands/<entity>/`):
   - `mod.rs` with `pub async fn run(ctx, cmd)` that dispatches to submodule
   - `list.rs`, `get.rs`, `create.rs`, `update.rs`, `delete.rs`
   - Handlers check `ctx.dry_run` before API calls
   - Handlers check `ctx.no_input` to decide whether to prompt

3. **API models** (`src/api/models.rs`):
   - `Entity` struct (base, all fields, for deserialization)
   - `CreateEntity` struct (required + optional, for POST body)
   - `UpdateEntity` struct (all optional, for PUT body)

4. **API client** (`src/api/mod.rs`):
   - `list_<entities>()` -> `ApiListResponse<Entity>`
   - `get_<entity>()` -> `ApiSingleResponse<Entity>`
   - `create_<entity>()` -> `ApiSingleResponse<Entity>`
   - `update_<entity>()` -> `ApiSingleResponse<Entity>`
   - `delete_<entity>()` -> `()`

5. **Wiring** (`src/cli/mod.rs` + `src/main.rs`):
   - Add variant to `Commands` enum
   - Add match arm in `run()` function

### Output Pattern

All command handlers render output via:
```rust
output::render_list(&items, &ctx.output_format, &DEFAULT_COLUMNS, &args.fields, ctx.color, Some(&meta))
output::render_single(&item, &ctx.output_format, &DEFAULT_COLUMNS, &args.fields, ctx.color)
```

### Dry-Run Pattern

```rust
if ctx.dry_run {
    return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
}
```

### Error Pattern

```rust
use crate::error::CliError;

// In API client, map HTTP status to CliError variant
match status {
    401 | 403 => Err(CliError::Auth { detail, hint: "run `pipelite init`".into() }.into()),
    404 => Err(CliError::NotFound { detail, hint: format!("run `pipelite {} list`", entity) }.into()),
    422 => Err(CliError::Validation { detail, hint: "check field values".into() }.into()),
    _ => Err(CliError::Api { status, detail, hint }.into()),
}
```

## Configuration System

- **File**: `~/.pipelite/config.toml` (0600 permissions)
- **Env overrides**: `PIPELITE_API_KEY`, `PIPELITE_SERVER_URL`, `PIPELITE_CONFIG`, `NO_COLOR`
- **Precedence**: CLI flags > env vars > config file > defaults
- **Config editing**: Uses `toml_edit` to preserve comments when setting values

## Cache System

- **Location**: `~/.pipelite/cache/`
- **Format**: JSON files with embedded `CacheEntry<T>` (data + cached_at + ttl_seconds)
- **TTL**: Pipelines/Stages/Workflows = 1 hour, Entities = 5 minutes
- **Writes**: Atomic (temp file + rename)
- **Corruption**: Silently deleted on read failure
- **Keys**: `pipelines`, `stages`, `deals`, `orgs`, `people`, `activities`, `workflows`

## Build & Test

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo test                     # All tests
cargo test deals               # Tests matching "deals"
```

- `build.rs` embeds rustc version + platform into binary version string
- Tests use fake credentials (`127.0.0.1:1` + `fake-test-key`)
- Integration tests use `assert_cmd` to run the binary

## Common Tasks for Agents

### Adding a new CLI flag to an entity

1. Add the field to the args struct in `src/cli/<entity>.rs`
2. Use the field in the command handler in `src/commands/<entity>/<action>.rs`
3. Pass it to the API client method if it affects the request

### Adding a new output column

1. Update `DEFAULT_COLUMNS` constant in the relevant command handler
2. Ensure the API response includes the field (check `src/api/models.rs`)

### Modifying API endpoints

All API calls go through `src/api/mod.rs`. The `PipeliteClient` has:
- `base_url: String` -- server URL
- `client: reqwest::Client` -- HTTP client
- `api_key: String` -- Bearer token

URL construction: `format!("{}/api/v1/{}", self.base_url, path)`

### Working with interactive prompts

`src/prompt.rs` provides helpers. Always gate behind `ctx.no_input`:
```rust
if ctx.no_input {
    return Err(CliError::MissingInput { detail, hint }.into());
}
let value = prompt::text("Enter value")?;
```

## Constraints & Conventions

- **No unwrap() in production code** -- use `?` operator with `anyhow::Result`
- **All errors must have hints** -- every `CliError` variant includes actionable guidance
- **Respect --dry-run** -- never make HTTP requests when dry_run is true
- **Respect --no-input** -- never block on stdin when no_input is true or stdin is not a TTY
- **Respect --quiet** -- suppress informational messages (but still output data)
- **Color opt-out** -- check `ctx.color` before using `.red()`, `.bold()`, etc.
- **No panics** -- handle all error cases gracefully
- **Config file security** -- always write config with 0600 permissions on Unix

## API Base URL & Endpoints

Default: `https://app.pipelite.io`

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/deals` | List deals |
| GET | `/api/v1/deals/:id` | Get deal |
| POST | `/api/v1/deals` | Create deal |
| PUT | `/api/v1/deals/:id` | Update deal |
| DELETE | `/api/v1/deals/:id` | Delete deal |
| POST | `/api/v1/deals/batch` | Batch create deals |
| GET | `/api/v1/organizations` | List organizations |
| GET | `/api/v1/organizations/:id` | Get organization |
| POST | `/api/v1/organizations` | Create organization |
| PUT | `/api/v1/organizations/:id` | Update organization |
| DELETE | `/api/v1/organizations/:id` | Delete organization |
| GET | `/api/v1/people` | List people |
| GET | `/api/v1/people/:id` | Get person |
| POST | `/api/v1/people` | Create person |
| PUT | `/api/v1/people/:id` | Update person |
| DELETE | `/api/v1/people/:id` | Delete person |
| GET | `/api/v1/activities` | List activities |
| GET | `/api/v1/activities/:id` | Get activity |
| POST | `/api/v1/activities` | Create activity |
| PUT | `/api/v1/activities/:id` | Update activity |
| DELETE | `/api/v1/activities/:id` | Delete activity |
| GET | `/api/v1/pipelines` | List pipelines |
| GET | `/api/v1/pipelines/:id` | Get pipeline |
| POST | `/api/v1/pipelines` | Create pipeline |
| PUT | `/api/v1/pipelines/:id` | Update pipeline |
| DELETE | `/api/v1/pipelines/:id` | Delete pipeline |
| GET | `/api/v1/stages` | List stages |
| GET | `/api/v1/stages/:id` | Get stage |
| POST | `/api/v1/stages` | Create stage |
| PUT | `/api/v1/stages/:id` | Update stage |
| DELETE | `/api/v1/stages/:id` | Delete stage |
| GET | `/api/v1/workflows` | List workflows |
| GET | `/api/v1/workflows/:id` | Get workflow |
| POST | `/api/v1/workflows` | Create workflow |
| PUT | `/api/v1/workflows/:id` | Update workflow |
| DELETE | `/api/v1/workflows/:id` | Delete workflow |
| POST | `/api/v1/workflows/:id/trigger` | Trigger workflow |
| GET | `/api/v1/ping` | Health check |

## Query Parameters

Standard across list endpoints:

| Parameter | Type | Description |
|-----------|------|-------------|
| `limit` | u64 | Max results per page (default 50) |
| `offset` | u64 | Pagination offset |
| `expand` | string | Comma-separated relation names |

Entity-specific filters:

| Endpoint | Parameter | Description |
|----------|-----------|-------------|
| `/deals` | `stage` | Filter by stage ID |
| `/deals` | `org` | Filter by organization ID |
| `/deals` | `owner` | Filter by owner ID |
| `/stages` | `pipeline` | Filter by pipeline ID |
| `/workflows` | `active` | Filter by active status |
