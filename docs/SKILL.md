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

Pipelite CLI is a terminal-based interface for the Pipelite CRM platform. It provides CRUD operations for 7 entity types (deals, organizations, people, activities, pipelines, stages, workflows) plus v1.1 command groups: batch update/delete over stdin, workflow runs (list/get/watch), workflow templates, notes, webhooks, trash, audit log, custom-field definitions (the type source for typed `--custom-field` writing), and `docs` (OpenAPI spec fetch). Utility commands: dashboard, cache, config, completions. It communicates with the Pipelite REST API using Bearer token authentication (except `pipelite docs`, which is unauthenticated by design).

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
    mod.rs             # PipeliteClient -- all HTTP methods (~1805 lines)
    models.rs          # Entity structs: Base, Create, Update (~2344 lines)
  batch.rs             # Shared batch flows: stdin parsing, BatchOutcome, confirmation gate
  custom_fields.rs     # Shared typed-writing resolver (all 8 create/update handlers)
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

## Exit Code Contract

Every command honors this contract (pinned by `tests/exit_code_test.rs` + the Phase 13 contract matrix):

| Code | Meaning | Examples |
|------|---------|----------|
| `0` | Success | all-ok batch, dry-run preview, declined-but-safe flows where designed |
| `1` | Item failure or user refusal | batch `--continue-on-error` partial failure; destructive deletes refused in non-TTY without `--force`; run failed with `--watch --exit-status` |
| `2` | Structural rejection BEFORE any HTTP | InvalidInput/MissingInput: unknown enums, mutual exclusivity (`--custom-field` + `--stdin`), missing args, trash purge non-TTY refusal, batch malformed input |

**Exit 2 guarantees zero HTTP** — pre-HTTP rejections are validated before the first request. Hint conventions: every error carries an actionable `hint`; pre-HTTP rejections name the offending flag and the valid vocabulary (e.g. an unknown trash type lists all 9 aliases).

```
error: Unknown trash type 'organisation'
  hint: valid types: deal/deals, organization/orgs/organizations, person/people, activity/activities
```

The rendered error is three lines: category, server/client detail, hint. Nothing is sent when validation fails — the contract matrix pins zero-HTTP on an unreachable server (`http://127.0.0.1:1`).

## Batch Pattern

All 7 entities share the batch flows in `src/batch.rs`:

- **Batch update**: `<entity> update --stdin` reads a JSON array (or NDJSON) of objects, each with an `id` key; each item is a per-item PUT.
- **Batch delete**: `<entity> delete id1 id2 ...` (positional) or `--stdin` (JSON array of IDs).
- **Batch create**: `<entity> create --stdin` (POSTs the array to `/api/v1/{entity}/batch`).
- `--continue-on-error` keeps going past per-item failures and prints the `N ok, M failed` summary that survives `--quiet`; without it the first failure aborts. Exit 0 only when every item succeeded, else 1.
- Non-TTY deletes require explicit `--force` (refusal is exit 1, pre-HTTP).
- Pre-validation rejects structurally broken input (missing `id`, malformed JSON) before the first HTTP — exit 2.

## Custom Fields & Typed Writing Pattern

Custom-field definitions (`pipelite custom-fields`) are the type source for `--custom-field key=value` on deals/orgs/people/activities (all 8 create/update handlers route through ONE shared resolver, `src/custom_fields.rs`):

- **number** → i64-first JSON number (`price=4` stores `4`, never `"4"`); **boolean** → strict lowercase; **select/single_select** → option-validated; **multi_select** → comma-split JSON array; **formula** → refused, exit 2 (server strips formula writes); unknown names → raw strings + one quiet-suppressible warning.
- `--custom-field-json '<object>'` writes verbatim, bypassing inference.
- `--custom-field`, `--custom-field-json`, and `--stdin` are mutually exclusive — exit 2 pre-HTTP on all 8 handlers.
- `--dry-run` is cache-only: cold cache → raw strings + a visible note; warm cache → typed values. Never fetches definitions.
- Definition create/update/delete invalidate the `custom_fields_` cache prefix.

## New Surfaces (v1.1)

Nine command groups beyond the original 7-entity CRUD (full flags: `docs/api-reference.md`):

- **Batch** — stdin-driven update/delete/create across all 7 entities; see Batch Pattern above.
- **workflows runs** — `list --workflow <id>` (required; `--status`, `--include-dry-run`, `--limit/--offset`), `get <run-id> --workflow <id>` with `--watch` (2s poll, no timeout) and `--exit-status` (exit 1 on failed run). Statuses hint: pending/running/completed/failed/waiting.
- **templates** — list/get/create/delete; NO update (the server has none). `create --workflow <id>` snapshots triggers[0] + nodes (multi-trigger warning); `--trigger '<json>'` inline; `--stdin` verbatim.
- **notes** — top-level group with entity-type positional (`deals|orgs|people|activities`): list/add/edit/delete. Body precedence `--body` > `@file` > `--stdin` > prompt. No single-note GET (hidden `notes get` rejects with a hint, exit 2).
- **webhooks** — CRUD; 13-event validation pre-HTTP; the signing secret is shown exactly once at create (never on list/get); https enforced; another user's webhook 403s even with an admin key.
- **trash** — list/restore/purge with dual type vocabulary (9 singular/plural aliases → 4 plural URL tabs). Restore needs no confirmation; purge is admin-only with the strongest confirmation, refusing non-TTY without `--force` at exit 2.
- **audit** — read-only, admin-gated; 4 verbatim filters (`--entity-type/--entity-id/--actor-kind/--workflow-run-id`); `--limit` clamped ≤100; no `--all`.
- **custom-fields** — definitions CRUD; see the Typed Writing Pattern above.
- **docs** — fetches the server's OpenAPI 3.1 spec from the PUBLIC `/api/v1/docs` route (a local headerless client — never sends the API key); `--save <file>` refuses overwrites unless `--force`; `--format` accepted but ignored.

## Configuration System

- **File**: `~/.pipelite/config.toml` (0600 permissions)
- **Env overrides**: `PIPELITE_API_KEY`, `PIPELITE_SERVER_URL`, `PIPELITE_CONFIG`, `NO_COLOR`
- **Precedence**: CLI flags > env vars > config file > defaults
- **Config editing**: Uses `toml_edit` to preserve comments when setting values

## Cache System

- **Location**: `~/.pipelite/cache/`
- **Format**: JSON files with embedded `CacheEntry<T>` (data + cached_at + ttl_seconds)
- **TTL**: Pipelines/Stages/Workflows/Templates/Webhooks/Custom-field definitions = 1 hour, Users = 2 hours, Entity lists (deals/orgs/people/activities) = 5 minutes
- **Writes**: Atomic (temp file + rename)
- **Corruption**: Silently deleted on read failure
- **Keys**: `pipelines`, `stages`, `users`, `deals`, `orgs`, `people`, `activities`, `workflows`, `templates`, `webhooks`, `custom_fields_deal`, `custom_fields_organization`, `custom_fields_person`, `custom_fields_activity` (see `KEY_*` consts in `src/cache.rs`)
- **Prefix invalidation**: mutations clear related prefixes (e.g. `stages_` on stage/pipeline changes; `custom_fields_` on definition mutations; trash is never cached)

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
- `tests/contract_matrix_test.rs` is the table-driven v1.0-contract matrix (61 tests) over all v1.1 surfaces — run it after any behavior change
- `tests/common/mod.rs` provides hermetic `cmd()` helpers (temp HOME, unreachable server) and head+body-capturing stub servers

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

### Adding a new command group (v1.1 style)

The 9 v1.1 groups (notes, webhooks, trash, audit, custom-fields, templates, docs, runs, batch) extend the entity pattern:

1. **CLI args** in `src/cli/<group>.rs` — subcommand enum with per-subcommand arg structs; `--help` examples block at the bottom
2. **Handlers** in `src/commands/<group>/` — check `ctx.dry_run` BEFORE any HTTP (previews render the method + endpoint), check `ctx.no_input` before prompting
3. **Client methods** in `src/api/mod.rs` with a doc comment naming the exact endpoint path (the endpoint table in this file is derived from those comments)
4. **Exit-code discipline**: validate enums/vocabulary pre-HTTP → exit 2 with the valid values in the hint; destructive actions require `--force` in non-TTY → exit 1 refusal
5. **Empty-page hint** on list subcommands: one stderr line when a page is empty, suppressed under `--quiet`
6. **Output** via `output::render_list`/`render_single` with a `DEFAULT_COLUMNS` const; `--fields` selection for free
7. **Stub tests** in `tests/<group>_stub_test.rs` using `tests/common/mod.rs` helpers

### Data-source precedence pattern (body-style inputs)

Commands accepting content (notes add/edit) resolve EXACTLY ONE source with an XOR guard before reading: explicit flag (`--body`, including `@file` / `@-`) > `--stdin` > interactive prompt. Two explicit sources → exit 2 before any I/O. Template/definition bodies follow the same shape (`--workflow`/`--trigger`/`--stdin`).

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
| POST | `/api/v1/{entity}/batch` | Batch create (`{entity}` = deals, organizations, people, activities — 4 server-supported) |
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
| POST | `/api/v1/workflows/:id/run` | Trigger (run) endpoint used by `workflows trigger` |
| GET | `/api/v1/workflows/:id/runs` | List workflow runs (`?status&dry_run&limit&offset`) |
| GET | `/api/v1/workflows/:id/runs/:runId` | Get one run + its steps |
| GET | `/api/v1/workflow-templates` | List workflow templates |
| GET | `/api/v1/workflow-templates/:id` | Get template |
| POST | `/api/v1/workflow-templates` | Create template |
| DELETE | `/api/v1/workflow-templates/:id` | Delete template (no update exists) |
| GET | `/api/v1/{parent}/{id}/notes` | List notes (parent = deals/organizations/people/activities) |
| POST | `/api/v1/{parent}/{id}/notes` | Add note |
| PATCH | `/api/v1/notes/:noteId` | Edit note (URL carries ONLY the note ID) |
| DELETE | `/api/v1/notes/:noteId` | Delete note (soft delete) |
| GET | `/api/v1/webhooks` | List webhooks (owner-scoped) |
| GET | `/api/v1/webhooks/:id` | Get webhook (no secret) |
| POST | `/api/v1/webhooks` | Create webhook (201 carries the show-once secret) |
| PUT | `/api/v1/webhooks/:id` | Update webhook (merged body) |
| DELETE | `/api/v1/webhooks/:id` | Delete webhook (hard) |
| GET | `/api/v1/trash?type&offset&limit` | List trashed records |
| POST | `/api/v1/trash/:type/:id/restore` | Restore one record |
| DELETE | `/api/v1/trash/:type/:id` | Permanently destroy one record (ADMIN-ONLY) |
| GET | `/api/v1/audit?entity_type&entity_id&actor_kind&workflow_run_id&offset&limit` | Audit entries (admin-gated) |
| GET | `/api/v1/custom-field-definitions` | List definitions (`?entity_type&offset&limit`) |
| GET | `/api/v1/custom-field-definitions/:id` | Get definition |
| POST | `/api/v1/custom-field-definitions` | Create definition |
| PUT | `/api/v1/custom-field-definitions/:id` | Partial update |
| DELETE | `/api/v1/custom-field-definitions/:id` | Soft delete |
| GET | `/api/v1/docs` | OpenAPI 3.1 spec (PUBLIC — no API key sent) |
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
