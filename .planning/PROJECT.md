# Pipelite CLI

## What This Is

A lightweight Rust command-line tool that connects to any Pipelite CRM server via API key, providing full CRUD operations on all CRM entities (deals, orgs, people, activities, pipelines, stages, workflows). Designed for both interactive human use and headless scripting/agent workflows, with pipeable output in multiple formats.

## Core Value

Users can manage their entire Pipelite CRM from the terminal — fast, scriptable, and composable with other tools.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] API key-based authentication to any Pipelite CRM server
- [ ] Full CRUD on deals, orgs, people, activities, pipelines, stages, and workflows
- [ ] Pipeable output with format flags (table/csv/json/plain)
- [ ] Interactive prompts for create/update operations
- [ ] Headless mode for scripts and agents (no prompts, stdin/flags only)
- [ ] Configuration via ~/.pipelite/config.toml
- [ ] Shell completions (bash, zsh, fish)
- [ ] Local caching for frequently accessed data
- [ ] ASCII art splash screen
- [ ] Status dashboard showing pipeline overview
- [ ] Connection testing / server health check

### Out of Scope

- GUI or TUI framework — this is a CLI tool, not a terminal UI app
- Webhook management — server-side concern, not CLI
- User/permission management — admin features deferred
- Offline mode with sync — too complex for v1, caching is read-only

## Context

- The project is a Rust binary (already initialized with Cargo)
- Targets the Pipelite CRM API (REST, JSON)
- Should feel like well-known CLI tools (gh, jq, kubectl) — composable, predictable
- Interactive prompts for create/update make it friendly for humans
- Headless mode makes it useful for CI/CD, scripts, and AI agents
- Config lives in ~/.pipelite/config.toml following XDG-like conventions

## Constraints

- **Tech stack**: Rust — already initialized, non-negotiable
- **API dependency**: Requires a running Pipelite CRM server with API access
- **Output formats**: Must support table, csv, json, and plain — pipeable to other tools
- **Config location**: ~/.pipelite/config.toml — standard for CLI tools

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust as language | Already initialized, performance + single binary distribution | — Pending |
| ~/.pipelite/config.toml | Standard CLI config location, TOML is human-readable | — Pending |
| Multiple output formats | Composability with other tools (jq, awk, csv tools) | — Pending |
| Interactive + headless modes | Serves both humans and automation | — Pending |

---
*Last updated: 2026-03-29 after Phase 6 completion (workflow API integration)*
