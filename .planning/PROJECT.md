# Pipelite CLI

## What This Is

A Rust command-line tool that connects to any Pipelite CRM server via API key, providing full CRUD operations on all CRM entities (deals, orgs, people, activities, pipelines, stages, workflows) plus workflow automation triggers. Designed for both interactive human use and headless scripting/agent workflows, with pipeable output in multiple formats and local caching for speed.

## Core Value

Users can manage their entire Pipelite CRM from the terminal — fast, scriptable, and composable with other tools.

## Requirements

### Validated

- ✓ API key-based authentication to any Pipelite CRM server — v1.0
- ✓ Full CRUD on deals, orgs, people, activities, pipelines, stages, and workflows — v1.0
- ✓ Pipeable output with format flags (table/csv/json/plain) — v1.0
- ✓ Interactive prompts for create/update operations — v1.0
- ✓ Headless mode for scripts and agents (no prompts, stdin/flags only) — v1.0
- ✓ Configuration via ~/.pipelite/config.toml — v1.0
- ✓ Shell completions (bash, zsh, fish) with dynamic entity ID completion — v1.0
- ✓ Local caching with TTL-based invalidation — v1.0
- ✓ ASCII art splash screen — v1.0
- ✓ Pipeline dashboard with deal counts/values per stage + workflow summary — v1.0
- ✓ Connection testing / server health check via `pipelite ping` — v1.0
- ✓ Workflow trigger execution (fire-and-forget with @filepath data support) — v1.0

### Active

(None — next milestone requirements to be defined via `/gsd:new-milestone`)

### Out of Scope

- GUI or TUI framework — this is a CLI tool, not a terminal UI app
- Webhook management — server-side concern, not CLI
- User/permission management — admin features deferred
- Offline mode with sync — too complex for v1, caching is read-only
- Custom scripting language / DSL — Shell is the scripting language
- Plugin/extension system — bounded domain doesn't warrant extensibility overhead

## Context

Shipped v1.0 with 10,779 LOC Rust (src/) + 1,447 LOC tests.
Tech stack: Rust, clap 4.6, reqwest 0.13, serde, dialoguer 0.12, comfy-table, clap_complete.
Config: ~/.pipelite/config.toml. Cache: ~/.pipelite/cache/ (JSON files with TTL).
7 entity types with full CRUD. 98 unit tests, 13 integration tests passing.

## Constraints

- **Tech stack**: Rust — single binary distribution, non-negotiable
- **API dependency**: Requires a running Pipelite CRM server with API access
- **Output formats**: Must support table, csv, json, and plain — pipeable to other tools
- **Config location**: ~/.pipelite/config.toml — standard for CLI tools

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust as language | Already initialized, performance + single binary distribution | ✓ Good |
| ~/.pipelite/config.toml | Standard CLI config location, TOML is human-readable | ✓ Good |
| Multiple output formats | Composability with other tools (jq, awk, csv tools) | ✓ Good |
| Interactive + headless modes | Serves both humans and automation | ✓ Good |
| serde_json::Value for workflow triggers/nodes | Server validates complex types, CLI stays flexible | ✓ Good |
| TTL-based JSON file cache | Simple, no external deps, fast enough for CLI use | ✓ Good |
| clap_complete unstable-dynamic for completions | Enables cache-backed entity ID suggestions | ✓ Good |
| Fire-and-forget workflow trigger | No polling needed, matches async workflow execution model | ✓ Good |

---
*Last updated: 2026-03-29 after v1.0 milestone*
