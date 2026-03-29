# Architecture Guide

This document describes the internal architecture of Pipelite CLI for contributors and AI agents working on the codebase.

## High-Level Architecture

```
User Input (CLI args / stdin)
        |
   [clap Parser]  (src/cli/)
        |
   [AppContext]    (src/context.rs)
   /    |    \
Config Client Cache
        |
   [Command Handler]  (src/commands/)
        |
   [PipeliteClient]   (src/api/mod.rs)
        |
   [HTTP Request]     (reqwest)
        |
   [Response]
        |
   [Output Renderer]  (src/output/)
        |
   stdout / stderr
```

## Module Responsibilities

### `src/cli/` -- Argument Definitions

Pure data definitions using clap derive macros. No business logic. Each entity has its own file defining subcommands and their arguments.

**Pattern**: Every entity module exports a `*Commands` enum with variants: `List`, `Get`, `Create`, `Update`, `Delete`. Each variant wraps a `*Args` struct.

**Key design choices**:
- Global flags (format, color, quiet, verbose, no-input, dry-run) live on the top-level `Cli` struct
- Aliases allow short forms: `deals` -> `d`, `orgs` -> `o`, etc.
- `ArgValueCandidates` provide shell completion candidates from the local cache
- Required fields for `create` are typed as `Option<T>` to enable interactive prompting

### `src/commands/` -- Command Handlers

Business logic for each command. Handlers receive `&AppContext` and the parsed args, then:

1. Validate / prompt for missing inputs
2. Call `PipeliteClient` methods (or render dry-run preview)
3. Format and render output

**Pattern**: Each entity directory has `list.rs`, `get.rs`, `create.rs`, `update.rs`, `delete.rs`. The entity `mod.rs` has a `run()` function that dispatches to the right handler.

### `src/api/mod.rs` -- HTTP Client

`PipeliteClient` wraps `reqwest::Client` with:
- Bearer token authentication
- Base URL construction
- Standard query parameters (limit, offset, expand, filters)
- Response parsing with error mapping
- Timeout configuration (5s connect, 30s total)

**API Pattern**: Methods follow naming convention `list_*`, `get_*`, `create_*`, `update_*`, `delete_*`. Each returns `Result<ApiSingleResponse<T>>` or `Result<ApiListResponse<T>>`.

### `src/api/models.rs` -- Data Types

Every API entity has three structs:
- **Base model** (e.g., `Deal`) -- full entity with all fields, for deserialization
- **Create struct** (e.g., `CreateDeal`) -- required + optional fields for creation
- **Update struct** (e.g., `UpdateDeal`) -- all-optional fields for partial updates

**Response wrappers**:
- `ApiSingleResponse<T>` -- `{ data: T }`
- `ApiListResponse<T>` -- `{ data: Vec<T>, meta: PaginationMeta }`
- `PaginationMeta` -- `{ total, offset, limit }`

### `src/context.rs` -- Application Context

`AppContext` is the central state object passed to all handlers. Built once per command from CLI flags + config + env vars. Contains:
- `config` -- merged AppConfig
- `client` -- PipeliteClient instance
- `output_format` -- resolved OutputFormat
- `quiet`, `verbose`, `color`, `no_input`, `dry_run` -- flags
- `cache` -- optional CacheStore

### `src/config.rs` -- Configuration

Handles TOML config file at `~/.pipelite/config.toml`:
- `load_config()` -- reads file + merges env var overrides
- `write_config()` -- writes with 0600 permissions (Unix)
- `set_config_value()` -- uses `toml_edit` to preserve comments
- `generate_commented_config()` -- creates human-readable template

### `src/output/` -- Rendering

Format-agnostic rendering system:
- `render_list()` / `render_single()` dispatch to format-specific modules
- `detect_format()` -- auto-detects JSON for pipes, Table for TTY
- `format.rs` -- smart value formatting (dates -> relative, values -> currency)
- `fields.rs` -- dynamic field extraction from JSON values

### `src/cache.rs` -- Caching Layer

File-based cache at `~/.pipelite/cache/`:
- Each key = one JSON file containing `CacheEntry<T>` with embedded timestamp + TTL
- Atomic writes (temp file + rename)
- Corrupted files silently deleted and re-fetched
- Prefix-based invalidation for related entries

### `src/error.rs` -- Error Handling

`CliError` enum with variants: Auth, Connection, Config, NotFound, Validation, Api, MissingInput. Each variant carries `detail` and `hint` strings.

- HTTP status codes mapped to appropriate variants (401->Auth, 404->NotFound, 422->Validation)
- Exit code 1 for runtime errors, 2 for usage errors
- Color-aware error display on stderr

### `src/dry_run.rs` -- Dry-Run Preview

Renders mutation previews showing HTTP method, URL, and request body without making actual API calls.

### `src/prompt.rs` -- Interactive Prompts

Uses `dialoguer` for interactive input:
- `FuzzySelect` for entity selection (searchable dropdowns)
- `Password` for API key entry
- `Confirm` for destructive operations
- Automatic skip when `--no-input` or stdin is not a TTY

## Data Flow: Create Command Example

```
1. User runs: pipelite deals create --title "Big Deal" --stage stg_001
2. clap parses into Cli { command: Commands::Deals(DealsCommands::Create(args)) }
3. main.rs calls AppContext::build(&cli) -> loads config, creates client
4. commands::deals::run(&ctx, &cmd) dispatches to create::run(&ctx, &args)
5. create::run checks --dry-run flag
   - If dry-run: render_dry_run("POST", url, body) -> stdout
   - If real: client.create_deal(payload).await -> render_single(deal)
6. Output rendered via output::render_single() in the resolved format
```

## Adding a New Entity

1. **Models**: Add `Entity`, `CreateEntity`, `UpdateEntity` structs to `src/api/models.rs`
2. **API client**: Add `list_*`, `get_*`, `create_*`, `update_*`, `delete_*` methods to `src/api/mod.rs`
3. **CLI args**: Create `src/cli/entity.rs` with `EntityCommands` enum and arg structs
4. **Commands**: Create `src/commands/entity/` with `mod.rs`, `list.rs`, `get.rs`, `create.rs`, `update.rs`, `delete.rs`
5. **Wire up**: Add variant to `Commands` enum in `src/cli/mod.rs` and match arm in `main.rs`
6. **Cache**: Add key constant and TTL to `src/cache.rs`
7. **Completions**: Add ID candidate function to CLI args

## Dependencies

| Crate | Purpose |
|-------|---------|
| clap 4.6 | CLI parsing (derive macros + shell completions) |
| tokio 1 | Async runtime (single-threaded) |
| reqwest 0.13 | HTTP client (JSON, rustls TLS) |
| serde / serde_json | Serialization |
| toml / toml_edit | Config file I/O |
| dialoguer 0.12 | Interactive prompts (fuzzy select) |
| colored 3.1 | Terminal colors |
| comfy-table 7.2 | Table rendering |
| csv 1.3 | CSV output |
| chrono / chrono-humanize | Date/time formatting |
| anyhow / thiserror | Error handling |
| indicatif 0.18 | Progress indicators |

## Testing Strategy

- **Integration tests** (`tests/`): Run the compiled binary via `assert_cmd`, verify output and exit codes
- **Unit tests** (inline `#[cfg(test)]`): Test formatting, config I/O, error handling, cache TTL
- **Dry-run tests**: Verify `--dry-run` never makes HTTP requests
- **Headless tests**: Verify `--no-input` mode works without TTY
- Tests use fake credentials pointing to `127.0.0.1:1` (no real server needed)
