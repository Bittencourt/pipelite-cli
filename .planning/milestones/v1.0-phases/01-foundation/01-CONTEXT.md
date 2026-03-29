# Phase 1: Foundation - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Auth, config management, HTTP client with timeouts/retries, error handling framework, and TTY-aware output abstraction. Delivers working `pipelite init`, `pipelite ping`, `pipelite config show/set/get`, `--version`, and `-q` quiet mode. No entity CRUD — that's Phase 2.

</domain>

<decisions>
## Implementation Decisions

### Init & Auth Flow
- Guided wizard: step-by-step prompts for server URL, then API key, then test connection, then confirm and save
- API key stored plain in config.toml (file created with 0600 permissions)
- `PIPELITE_API_KEY` env var always wins over config file value — standard for CI/scripts
- Single server in v1 — no profiles. Profiles deferred to v2
- `pipelite init --url <url> --key <key>` also supported for headless/scripted init

### Command Structure
- Noun-verb subcommands: `pipelite config show`, `pipelite config set <key> <value>` — like git/gh
- Entity names are plural: `pipelite deals list`, `pipelite orgs get` — like kubectl resources
- Short aliases for entities: `pipelite d ls` = `pipelite deals list`, `pipelite p ls` = `pipelite people list`
- Action aliases: `ls` = `list`, `rm` = `delete`
- Entity IDs passed as positional args: `pipelite deals get 123`
- Delete requires confirmation in interactive TTY unless `--force` flag is present; in headless mode (`--no-input`), `--force` is required or error
- `pipelite --version` shows rich info: `pipelite 0.1.0 (rustc 1.84, linux-x86_64)` with build metadata
- Clap default help + after_help examples section on every command

### Global Flags (available on every command)
- `--format json|table|csv|plain` — override output format
- `--no-color` — disable colored output
- `-q` / `--quiet` — suppress non-essential output
- `-v` / `--verbose` — show extra debug info (API calls, timing)

### Error Message Style
- Structured format: type + message + hint
  ```
  error: Not found
    deal 123 does not exist
    hint: run `pipelite deals list` to see available deals
  ```
- Red for errors, yellow for warnings, dim for hints — standard terminal conventions
- All non-data output goes to stderr (errors, warnings, progress, hints). Only data to stdout. Pipe-safe.
- Auth errors are detailed with recovery steps: show what was tried (env var? config file?), what failed, and how to fix

### Config File Layout
- Path: `~/.pipelite/config.toml` — simple and memorable, like ~/.docker/ or ~/.ssh/
- Support `PIPELITE_CONFIG` env var override for custom location
- Grouped TOML sections: `[server]`, `[output]`, `[display]` organized by concern
- Generated config is a commented template: all keys present with comments explaining each, defaults filled in
- `pipelite config set` accepts dotted paths: `pipelite config set output.format json`
- Config precedence: CLI flags > env vars > config file

### Claude's Discretion
- Exact progress spinner implementation for `pipelite ping`
- Internal module structure (cli/ vs commands/ split)
- Error type hierarchy (thiserror enums vs anyhow contexts)
- HTTP client configuration details (exact timeout values, retry counts)
- TTY detection implementation details

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project with only `fn main()` hello world

### Established Patterns
- Cargo.toml uses edition 2024 (Rust 2024 edition)
- Package name is `pipelite` (binary name)
- No dependencies added yet

### Integration Points
- `src/main.rs` is the entry point — will become the clap app dispatch
- `Cargo.toml` needs all dependencies added

</code_context>

<specifics>
## Specific Ideas

- Should feel like gh, kubectl, and stripe CLI — composable and predictable
- Config at ~/.pipelite/config.toml with grouped sections is a deliberate choice for discoverability
- Rich version info with build metadata for debugging user reports
- Structured errors with hints encourage self-service troubleshooting

</specifics>

<deferred>
## Deferred Ideas

- Config profiles for multiple servers ([profiles.production], [profiles.staging]) — v2 requirement (PROF-01, PROF-02)
- Activity logging shortcut (`pipelite log`) — v2 requirement (ALOG-01)

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-03-24*
