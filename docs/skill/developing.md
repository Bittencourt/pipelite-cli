# Developing the CLI

Load this file when modifying the Pipelite CLI codebase — not when operating it.

## Source orientation

```
src/
  main.rs              # Entry point -- splash screen check, clap parse, run()
  cli/                 # Argument definitions only (clap derive)
    mod.rs             # Top-level Cli struct + Commands enum
    deals.rs           # DealsCommands enum + arg structs
    ...                # One file per entity/group
  commands/            # Business logic (handlers)
    deals/             # list.rs, get.rs, create.rs, update.rs, delete.rs
    ...                # One directory per entity/group
    init.rs, ping.rs, dashboard.rs, completions.rs
  api/
    mod.rs             # PipeliteClient -- all HTTP methods (~1805 lines; keep single-file until ~2000)
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
tests/
  *_integration.rs     # CLI integration tests using assert_cmd
  common/mod.rs        # Hermetic cmd() helpers + head/body-capturing stub servers
  contract_matrix_test.rs  # 61-test v1.0-contract matrix over all v1.1 surfaces
```

## Entity CRUD pattern

Every entity follows the same structure exactly:

1. **CLI args** (`src/cli/<entity>.rs`): `<Entity>Commands` enum (List/Get/Create/Update/Delete), each variant wrapping an args struct; completion candidates from cache for ID args; required fields typed `Option<T>` for prompting.
2. **Handlers** (`src/commands/<entity>/`): `mod.rs` dispatches via `pub async fn run(ctx, cmd)`; submodules per action; check `ctx.dry_run` before HTTP and `ctx.no_input` before prompting.
3. **Models** (`src/api/models.rs`): `<Entity>` (Base), `<Entity>Create`, `<Entity>Update`.
4. **Client** (`src/api/mod.rs`): `list_<entities>() -> ApiListResponse<E>`, `get_/create_/update_<entity>() -> ApiSingleResponse<E>`, `delete_<entity>() -> ()`.
5. **Wiring**: `Commands` enum + `run()` match arm in `main.rs`; group in `cli/mod.rs` + `commands/mod.rs`.

## Output, dry-run, error patterns

```rust
// Output
output::render_list(&items, &ctx.output_format, &DEFAULT_COLUMNS, &args.fields, ctx.color, Some(&meta));
output::render_single(&item, &ctx.output_format, &DEFAULT_COLUMNS, &args.fields, ctx.color);

// Dry-run — always before any HTTP
if ctx.dry_run {
    return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
}

// Error mapping (in the API client)
match status {
    401 => Err(CliError::Auth { detail, hint }.into()),
    403 => Err(CliError::Forbidden { detail, hint: forbidden_hint(surface) }.into()),
    404 => Err(CliError::NotFound { detail, hint }.into()),
    422 => Err(CliError::Validation { detail, hint }.into()),
    _   => Err(CliError::Api { status, detail, hint }.into()),
}
```

## Adding a new command group (v1.1 style)

The v1.1 groups (notes, webhooks, trash, audit, custom-fields, templates, docs, runs) extend the entity pattern:

1. **CLI args** in `src/cli/<group>.rs` — subcommand enum + per-subcommand arg structs + `--help` examples block (after_help).
2. **Handlers** in `src/commands/<group>/` — `ctx.dry_run` BEFORE any HTTP; `ctx.no_input` before prompting.
3. **Client methods** in `src/api/mod.rs` — doc comment names the exact endpoint path.
4. **Exit-code discipline**: validate enums/vocabulary pre-HTTP → exit 2 (`CliError::InvalidInput`) with valid values in the hint; destructive actions require `--force` in non-TTY → exit 1 (`CliError::Validation`); trash purge deliberately uses exit 2 instead.
5. **Empty-page hint** on lists: one stderr line when empty (and `meta.total == 0`), suppressed under `--quiet`.
6. **Output** via `render_list`/`render_single` with a `DEFAULT_COLUMNS` const.
7. **Stub tests** in `tests/<group>_stub_test.rs` using `tests/common/mod.rs` helpers; extend `tests/contract_matrix_test.rs` if the surface adds user-facing commands.
8. **Help-truthfulness test**: assert `--help` mentions real flags only (the help text referencing a nonexistent flag is a shipped-bug class — see the `--json` incident).

## Small changes

- **New CLI flag**: args struct (`src/cli/<entity>.rs`) → handler → client method if it affects the request.
- **New output column**: `DEFAULT_COLUMNS` in the handler (JSON always carries the full model).
- **API change**: all calls go through `PipeliteClient` (`base_url`, `client`, `api_key`); URL = `format!("{}/api/v1/{}", self.base_url, path)`.

## Data-source precedence pattern (body inputs)

Commands accepting content resolve EXACTLY ONE source with an XOR guard
before any read: explicit flag (`--body`, including `@file`/`@-`) > `--stdin` >
interactive prompt. Two explicit sources → exit 2 before any I/O.

## Cache keys

`KEY_*` consts in `src/cache.rs`: `pipelines`, `stages`, `users`, `deals`,
`orgs`, `people`, `activities`, `workflows`, `templates`, `webhooks`,
`custom_fields_{deal,organization,person,activity}`. TTL: 1h (pipelines/
stages/workflows/templates/webhooks/definitions), 2h (users), 5m (entity
lists). Mutations invalidate related keys/prefixes; trash and audit are never
cached. Writes are atomic; corrupt files silently deleted on read.

## Build & test

```bash
cargo build                 # debug
cargo test                  # all tests (hermetic: fake creds 127.0.0.1:1 + fake-test-key)
cargo test deals            # filter by name
cargo test --test contract_matrix_test   # the v1.0-contract matrix (61 tests)
```

- `build.rs` embeds rustc version + platform into `--version`.
- Integration tests drive the real binary via `assert_cmd`; `tests/common/mod.rs` provides env-isolated `cmd()` helpers and TcpListener stub servers that capture request heads+bodies.
- Run the contract matrix after any behavior change touching global flags, exit codes, or output formats.

## Constraints & conventions

- **No unwrap()/expect() in production code** — `?` with `anyhow::Result`
- **Every error carries an actionable `hint`** — name the recovery command where one exists
- **Respect `--dry-run`** (zero HTTP), **`--no-input`** (never block on stdin), **`--quiet`** (suppress info, keep data + the batch summary contract), **`--no-color`** (check `ctx.color` before styling)
- **No panics** — handle all error paths
- **Config writes are 0600** on Unix
- **Pre-HTTP validation**: vocabulary/enum checks exit 2 with the valid values in the hint
