# CLAUDE.md -- Pipelite CLI

## Project Overview

Pipelite CLI (`pipelite`) is a Rust CLI for managing the Pipelite CRM from the terminal. It provides CRUD operations for deals, organizations, people, activities, pipelines, stages, and workflows via REST API.

## Build & Test

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo test                     # All tests
cargo test <pattern>           # Tests matching pattern
```

## Project Structure

- `src/cli/` -- Clap argument definitions (one file per entity)
- `src/commands/` -- Command handlers (one directory per entity, files: list/get/create/update/delete)
- `src/api/mod.rs` -- PipeliteClient HTTP methods
- `src/api/models.rs` -- Entity data types (Base + Create + Update per entity)
- `src/output/` -- Format-specific renderers (json, table, csv, plain)
- `src/config.rs` -- TOML config I/O at `~/.pipelite/config.toml`
- `src/context.rs` -- AppContext (config + client + output settings)
- `src/cache.rs` -- TTL-based JSON file cache at `~/.pipelite/cache/`
- `src/error.rs` -- CliError enum with detail + hint fields
- `src/dry_run.rs` -- Mutation preview rendering
- `src/prompt.rs` -- Interactive prompts (dialoguer)

## Key Conventions

- Every entity follows the same CRUD pattern (see `docs/SKILL.md` for details)
- All errors must include actionable `hint` text
- Respect `--dry-run` (no HTTP calls), `--no-input` (no stdin prompts), `--quiet`, `--no-color`
- Config file written with 0600 permissions
- No `unwrap()` in production code -- use `?` with `anyhow::Result`
- Tests use fake credentials (`127.0.0.1:1` + `fake-test-key`)

## Detailed Documentation

- `docs/SKILL.md` -- Comprehensive agent skill file with patterns, endpoints, and task guides
- `docs/architecture.md` -- Internal architecture and data flow
- `docs/api-reference.md` -- Complete CLI command reference
