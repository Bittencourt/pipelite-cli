# Phase 5: Power Features - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Local caching layer for speed (pipelines, stages, users, entity lists), pipeline dashboard showing deal counts and values per stage, ASCII splash screen when running `pipelite` with no subcommand on a TTY, and cache-powered dynamic shell completions. No new entity CRUD, no TUI, no new output formats.

</domain>

<decisions>
## Implementation Decisions

### Cache storage & location
- Cache lives at `~/.pipelite/cache/` — alongside config.toml, everything in one place
- All entity types are cached: pipelines, stages, users/owners, and entity lists (deals, orgs, people, activities)
- Tiered TTLs: long TTL for slow-changing data (pipelines, stages, users — hours), short TTL for fast-changing data (entity lists — minutes)
- Mutations (create/update/delete) auto-invalidate the relevant entity cache immediately
- `pipelite cache clear` clears all cached data
- `pipelite cache refresh` pre-populates cache by fetching all cacheable data at once

### Dashboard
- `pipelite dashboard` shows ALL pipelines — no pipeline selection, full overview in one command
- Each pipeline has a summary header (total deals, total value) plus a per-stage vertical table breakdown
- Per-stage table columns: Stage | Deals | Value
- Dashboard always fetches fresh data (no cache) — accuracy matters for overview
- Respects `--format` flag (table default, json/csv/plain supported for scripting)

### Splash screen
- Triggered when running `pipelite` with no subcommand on a TTY
- Bold block letter ASCII art logo ("PIPELITE" in figlet-style chunky font)
- Logo rendered in a single branded accent color (e.g., cyan or green), help hints in default color
- Below logo: 3-4 quick-start command hints (e.g., "Try: pipelite deals list, pipelite dashboard")
- If not configured (no API key): show logo + "Get started: pipelite init" instead of normal hints
- Non-TTY (piped): show standard clap help text, no splash, no ASCII art
- Respects `--no-color` and `NO_COLOR` env var

### Cache-powered completions
- Dynamic shell completions suggest entity IDs with name hints: `abc123 -- Big Deal Corp`
- Completions read from cache first, fall back to API if cache is empty/expired
- Interactive FuzzySelect prompts (from Phase 4) also use cache first, API fallback on miss — prompts feel instant
- `pipelite cache refresh` is the recommended post-setup step to prime completions

### Claude's Discretion
- Cache file format (JSON, bincode, SQLite, etc.)
- Exact TTL values per entity type
- Cache key structure and file naming
- ASCII art font choice and exact layout
- Branded color choice (cyan, green, etc.)
- How dynamic completions integrate with clap_complete (custom completer vs shell script generation)
- Dashboard value formatting (currency symbols, abbreviations like $45k)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AppConfig` (src/config.rs): TOML config with `[server]`, `[output]`, `[display]` sections — add `[cache]` section for TTL config
- `PipeliteClient` (src/api/mod.rs): All CRUD methods exist for all entities — cache layer wraps these calls
- `OutputFormat` + output modules (src/output/): Fully generic table/csv/json/plain — dashboard output uses these directly
- `detect_format()` (src/output/mod.rs): TTY detection for auto-format selection
- `AppContext` (src/context.rs): Carries config, client, format, color, quiet, verbose, no_input, dry_run — pass to cache and dashboard
- `dialoguer::FuzzySelect` (src/prompt.rs): Already used for interactive prompts — wire to cache for faster option loading
- Shell completions (src/commands/completions.rs): Static clap_complete — extend with dynamic entity completions

### Established Patterns
- Noun-verb command structure: `pipelite <noun> <action>` — cache and dashboard follow this
- Global flags on `Cli` struct with `global = true`
- Commands in `src/commands/` as modules with public execute functions
- `CliError` with detail + hint pattern for all error reporting
- `IsTerminal` checks throughout codebase for TTY-aware behavior

### Integration Points
- `Commands` enum in src/cli/mod.rs — add `Cache(CacheCommands)` and `Dashboard` variants
- `src/main.rs` dispatch — add cache, dashboard match arms + splash screen check before command dispatch
- `src/api/mod.rs` — cache layer wraps or sits alongside PipeliteClient methods
- `src/commands/completions.rs` — extend with dynamic completion generation from cache
- `src/prompt.rs` — modify FuzzySelect helpers to read cache before API calls

</code_context>

<specifics>
## Specific Ideas

- Cache should feel invisible — user doesn't think about it, things are just faster
- Dashboard should feel like a quick glance, not a deep analysis tool — one command, full picture
- Splash screen should feel like `gh` or `docker` with no args — branded, helpful, not cluttered
- `pipelite cache refresh` after `pipelite init` is the natural flow for new users

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-power-features*
*Context gathered: 2026-03-25*
