# Phase 4: Developer Experience - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Interactive prompts for human-friendly create/update workflows, explicit headless mode for automation, shell completions for bash/zsh/fish, and --dry-run safety for previewing mutations. No caching (Phase 5), no new entity commands — this phase enhances existing commands with better input/output modes.

</domain>

<decisions>
## Implementation Decisions

### Interactive prompts
- Running create/update with no flags on a TTY opens interactive prompts for ALL fields (required first, then optional)
- Fields with known values (stage, pipeline, org, person) use dialoguer FuzzySelect — type to filter, arrow keys to browse, options fetched from API on prompt
- Partial flags are honored: if some flags provided, pre-fill those and prompt for remaining fields
- Optional fields show "(optional, press Enter to skip)" — empty input skips the field
- Prompts only activate when stdin is a TTY (consistent with existing init behavior)
- dialoguer already in use for init — extend usage, no library switch needed

### Headless mode
- `--no-input` flag guarantees no interactive prompts
- Non-TTY stdin automatically implies --no-input (auto-detect + explicit flag for control)
- Missing required fields in headless mode: fail with exit code 2, listing ALL missing fields at once (not one at a time)
- Delete in headless mode requires --force — error without it (consistent with Phase 1 decision)
- `--stdin` and individual field flags are mutually exclusive — error if both provided
- All mutations can be performed entirely via flags (HEAD-03 requirement)

### Shell completions
- `pipelite completions bash|zsh|fish` subcommand — outputs completion script to stdout
- Static completions only (clap_complete from clap definitions) — commands, subcommands, flags
- Dynamic entity ID completion deferred to Phase 5 (needs caching layer)
- Show shell-specific install instructions on stderr after generating script

### Dry-run
- `--dry-run` flag on all mutation commands: create, update, and delete
- Shows formatted output: HTTP method, endpoint URL, and JSON body that would be sent
- Delete dry-run shows "Would delete [entity] [id]"
- Dry-run works with interactive prompts: user fills prompts, then sees preview instead of executing
- Respects --format flag (e.g., --format json for machine-readable dry-run output)

### Claude's Discretion
- Exact prompt ordering and grouping for each entity's fields
- FuzzySelect display format for entity options (e.g., "name (id)" vs "id - name")
- Dry-run output styling (colors, separators)
- clap_complete shell integration details
- How --dry-run interacts with --stdin batch mode (show all payloads or summary)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `dialoguer` crate: Already used in `src/commands/init.rs` for Input, Password, Confirm — extend with FuzzySelect
- `IsTerminal` (std::io): Already imported and used throughout codebase for TTY detection
- `AppContext` (src/context.rs): Carries format, color, quiet, verbose — add no_input and dry_run flags
- `CliError::Validation`: Already handles missing field errors with detail + hint pattern
- `PipeliteClient` (src/api/mod.rs): CRUD methods exist for all entities — prompt code can call list endpoints to fetch dropdown options

### Established Patterns
- Create commands validate required flags at runtime with `Option` + `ok_or_else(CliError::Validation)` — refactor to prompt-or-error based on TTY
- Delete commands already have TTY confirmation + --force pattern (src/commands/deals/delete.rs)
- Init command already detects non-TTY and requires --url/--key in headless mode — same pattern extends to --no-input
- Clap derive API with global flags on Cli struct — add --no-input, --dry-run as globals

### Integration Points
- Every entity's create.rs and update.rs: Add prompt fallback logic before API call
- `src/cli/mod.rs`: Add --no-input, --dry-run global flags to Cli struct
- `src/context.rs`: Add no_input and dry_run to AppContext
- `src/commands/mod.rs`: Add completions subcommand
- `src/main.rs`: Add completions dispatch

</code_context>

<specifics>
## Specific Ideas

- Interactive prompts should feel like `gh issue create` — guided, fast, skippable
- Headless mode must be CI/script-safe: deterministic, no prompts, clear errors, proper exit codes
- Completions follow `gh completion` pattern: subcommand with shell argument, script to stdout
- Dry-run is a teaching tool: "fill it out, see what happens, then run for real"

</specifics>

<deferred>
## Deferred Ideas

- Dynamic shell completions for entity IDs/names — Phase 5 (needs caching, CACH-03)
- Config profiles for --no-input defaults per environment — v2 (PROF-01, PROF-02)

</deferred>

---

*Phase: 04-developer-experience*
*Context gathered: 2026-03-25*
