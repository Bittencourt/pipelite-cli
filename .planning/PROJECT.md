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
- ✓ Batch update (`--stdin`, JSON array) and batch delete (multi-ID / `--stdin`) across all 7 entities, with continue-on-error reporting, exit-code contract (0 ok / 1 item-failed / 2 structural), --force gating on batch deletes, and 429 Retry-After retry — Validated in Phase 7: Batch Operations
- ✓ Truthful error layer (RFC 7807 errors[]/detail parsing, Forbidden ≠ Auth with per-surface hints, 409 activation hint), honest models (fractional positions, --expand passthrough), honest lists (stages all-mode, --all ceiling warning), dead flags removed with exit-2 hints + v1.1 changelog — Validated in Phase 8: Foundations
- ✓ Notes CRUD on deals/orgs/people/activities (top-level `notes` group, flag/@file/stdin/prompt body sources, edit-by-ID with no-single-GET contract, confirm-delete) — Validated in Phase 10: Notes
- ✓ Workflow runs observation (list with --status + --include-dry-run, flattened step detail, --watch with --exit-status script contract), workflow templates CRUD (no-update-by-design, triggers[0] mapping), `docs` OpenAPI fetch (unauthenticated, --save) — Validated in Phase 9: Workflow Runs, Templates & Docs
- ✓ Shell completions (bash, zsh, fish) with dynamic entity ID completion — v1.0
- ✓ Local caching with TTL-based invalidation — v1.0
- ✓ ASCII art splash screen — v1.0
- ✓ Pipeline dashboard with deal counts/values per stage + workflow summary — v1.0
- ✓ Connection testing / server health check via `pipelite ping` — v1.0
- ✓ Workflow trigger execution (fire-and-forget with @filepath data support) — v1.0

### Active

**Milestone v1.1 — Server v2 Parity:**
- Batch operations for all entities (batch update/delete via --stdin, multi-ID, continue-on-error)
- Notes CRUD on deals/orgs/people/activities
- Workflow runs (list with status/dry-run filters, detail with steps)
- Webhooks CRUD (13 event names validated client-side, show-once secret)
- Trash (list/restore/purge with admin + confirmation warnings)
- Custom field definitions CRUD + type-aware `--custom-field` writing
- Workflow templates (list/get/create/delete)
- Audit log viewer (admin keys, clear 403 handling)
- `pipelite docs` (fetch public OpenAPI spec)
- Fixes: dead filters (people --org/--owner, workflows --active), --expand passthrough, Deal.position float, stages all-mode

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

## Current Milestone: v1.1 Server v2 Parity

**Goal:** Bring the CLI to full parity with the upgraded Pipelite CRM server — batch operations, new entity surfaces (notes, workflow runs, webhooks, trash, custom fields, templates, audit, docs), and fixes for all dead flags and model mismatches.

**Target features:**
- Batch operations (4 drafted plans: shared batch utility, per-entity coverage, integration tests)
- Notes CRUD on deals/orgs/people/activities
- Workflow runs (list + detail with steps)
- Webhooks CRUD (event-name validation, show-once secret)
- Trash (list/restore/purge)
- Custom field definitions CRUD + type-aware --custom-field writing
- Workflow templates (list/get/create/delete)
- Audit log viewer
- `pipelite docs` (OpenAPI fetch)
- Existing-CLI fixes: dead filters, --expand passthrough, position float, stages all-mode

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-09-04 after Phase 12 (Custom Fields) completion*
