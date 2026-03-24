# Feature Landscape

**Domain:** CRM CLI Tool (Rust, REST API client)
**Researched:** 2026-03-23

## Table Stakes

Features users expect from a CRM CLI tool modeled after gh/kubectl/stripe. Missing any of these and the tool feels broken or incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **API key authentication** | Every CLI tool that talks to a service needs auth. Users expect `pipelite auth login` or `pipelite init` flow similar to `gh auth login` or `hs init`. | Low | Store in `~/.pipelite/config.toml`. Never accept keys as flags (leaks to `ps` and shell history). Support `PIPELITE_API_KEY` env var for CI. |
| **Full CRUD on all entities** | The entire point of the tool. `pipelite deals list`, `pipelite deals create`, `pipelite deals get <id>`, `pipelite deals update <id>`, `pipelite deals delete <id>`. Same for orgs, people, activities, pipelines, stages. | High | Largest surface area. Each entity needs list/get/create/update/delete subcommands. This is the bulk of the work. |
| **JSON output (`--json`)** | Every modern CLI (gh, aws, az, stripe, kubectl) supports JSON output. Required for scripting, piping to `jq`, and agent consumption. | Low | Should be the default when stdout is not a TTY (piped). Stable contract -- human output can change, JSON must not. |
| **Table output (default for TTY)** | Humans expect readable tabular output when running commands interactively. gh, kubectl, aws all do this. | Medium | Auto-detect TTY vs pipe. Use column-aligned tables. Truncate long fields with `...`. |
| **`--help` on every command** | Universal CLI expectation. Three tiers: no-args shows brief usage, `--help` shows full details, `pipelite help <command>` works too. | Low | Use clap's built-in derive macros for help generation. Lead with examples in help text. |
| **`--version` flag** | Standard. Every CLI has it. | Low | `pipelite --version` returns `pipelite 0.1.0`. |
| **Non-zero exit codes on failure** | Scripts need to detect failures. 0 = success, 1 = runtime error, 2 = user misuse. | Low | Map API errors (401, 404, 500) to meaningful exit codes. |
| **Actionable error messages** | "Connection refused" is useless. "Cannot connect to server at https://crm.example.com -- check your server URL with `pipelite config show`" is useful. clig.dev emphasizes this as critical. | Medium | Catch known error classes (auth failure, network error, not found, validation error) and provide specific guidance with suggested next commands. |
| **Configuration file** | Users need persistent config: server URL, API key, default output format, default pipeline. `~/.pipelite/config.toml`. | Low | Precedence: flags > env vars > project config > user config. Support `pipelite config set/get/show` subcommands. |
| **Connection test / health check** | Users need to verify their setup works before doing anything else. `pipelite ping` or `pipelite status`. | Low | First thing users run after `pipelite init`. Shows server version, authenticated user, connection latency. |
| **Listing with filtering** | `pipelite deals list --stage="Negotiation" --owner="me"` -- users expect to filter lists, not fetch everything and grep. | Medium | Map common CRM filters to flags. Support `--limit` and `--offset` for pagination. |
| **CSV output (`--csv`)** | CRM users live in spreadsheets. CSV export from CLI is table stakes for data that feeds into Excel, Google Sheets, or other tools. | Low | Simple format, easy to implement alongside JSON and table. |
| **Quiet mode (`-q`)** | Scripts need to suppress non-essential output. Standard flag in every well-designed CLI. | Low | Suppress status messages, only output data or errors. |

## Differentiators

Features that set pipelite CLI apart. Not expected, but create real value -- especially for the scripting/agent use case.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Interactive prompts for create/update** | Running `pipelite deals create` with no flags opens an interactive wizard that walks through required fields with validation. Massively lowers the barrier for humans. Similar to `gh pr create` interactive flow. | Medium | Use `dialoguer` or `inquire` crate. Only activate when stdin is a TTY. Skip entirely in headless mode (`--no-input`). Present dropdowns for stages/pipelines from cached data. |
| **Headless mode (`--no-input`)** | Explicit flag that guarantees no prompts, making the tool safe for CI/CD pipelines and AI agent workflows. All input via flags or stdin. Fails with clear error if required input is missing. | Low | Essential for the "agent-friendly" positioning. Many CLIs detect TTY implicitly but an explicit flag is clearer for automation. |
| **Stdin piping for bulk operations** | `cat deals.json \| pipelite deals create --stdin` or `pipelite deals list --json \| jq '.[] \| select(.value > 10000)' \| pipelite deals update --stdin --stage="Won"`. Unix composability is a killer feature for power users. | Medium | Accept JSON or JSONL from stdin. Process line-by-line for JSONL. This enables batch operations without a dedicated batch API. |
| **Shell completions (bash, zsh, fish)** | Tab-completion of commands, flags, and even entity IDs/names. `pipelite deals get <TAB>` showing recent deal names. gh and kubectl both provide this and it dramatically improves discoverability. | Medium | Use `clap_complete` for static completions. Dynamic completions (entity names) are harder -- requires local cache. |
| **Local caching for lookups** | Cache pipeline names, stage names, user names, and recently accessed entity IDs locally. Makes interactive prompts fast (dropdown of stages without API call) and enables dynamic shell completions. | Medium | Cache in `~/.pipelite/cache/`. TTL-based invalidation (e.g., 5 minutes for pipelines, 1 hour for users). `pipelite cache clear` to force refresh. |
| **Pipeline dashboard (`pipelite dashboard`)** | ASCII-rendered overview of deals by pipeline stage with counts and total values. A quick "how's my pipeline doing?" view without leaving the terminal. | Medium | Not a TUI -- just a well-formatted ASCII output. Show stage names as columns, deal counts and total values per stage. Color-code stages. |
| **Multiple output formats (`--format`)** | Support `--format=table,json,csv,plain,jsonl` via a single unified flag. `plain` outputs values only (no headers), useful for `xargs`. JSONL outputs one JSON object per line for streaming. | Low | `--json` as shorthand for `--format=json`. Plain format enables: `pipelite deals list --format=plain --fields=id \| xargs -I{} pipelite deals delete {}`. |
| **Field selection (`--fields`)** | `pipelite deals list --fields=id,title,value,stage` -- output only specific fields. Reduces noise and makes piping cleaner. gh does this well with `--json id,title`. | Low | Works with all output formats. In table mode, controls which columns appear. In JSON mode, filters the output object. |
| **ASCII splash screen** | Branded startup experience when running `pipelite` with no arguments. Shows logo, version, and quick-start hints. | Low | Fun, memorable, costs nothing in complexity. Only show on interactive TTY with no subcommand. |
| **`--dry-run` for mutations** | `pipelite deals create --title="Big Deal" --value=50000 --dry-run` shows what would be sent to the API without actually creating anything. Safety net for scripted operations. | Low | Print the request body that would be sent. No API call. Invaluable for debugging scripts. |
| **Colored output with `NO_COLOR` support** | Color-coded output: green for success, red for errors, yellow for warnings, cyan for IDs/links. Respect `NO_COLOR` env var and `--no-color` flag per the standard. | Low | Use `owo-colors` or `colored` crate. Auto-disable when not TTY. |
| **Config profiles for multiple servers** | `pipelite --profile=staging deals list` -- switch between CRM instances (dev, staging, production). Similar to AWS CLI profiles. | Low | Store in config.toml as `[profiles.staging]` sections. Default profile is used when no `--profile` flag. |
| **Activity logging shortcut** | `pipelite log --deal=123 --type=call --note="Discussed pricing"` -- quick activity logging is the most common CRM task for sales reps. Making it fast from terminal is a real workflow win. | Low | Sugar over `pipelite activities create` with sensible defaults (timestamp=now, type defaults to "note"). |

## Anti-Features

Features to explicitly NOT build. These are traps that waste time or make the tool worse.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **TUI / full-screen interface** | The project scope explicitly excludes TUI. A TUI is a different product (like `lazygit` vs `git`). It adds massive complexity (ratatui, event loop, state management), kills pipeability, and requires a fundamentally different testing approach. | Keep it as a pure CLI. If users want a TUI, it can be a separate project that wraps the CLI or uses the same API client library. |
| **Webhook management** | Server-side concern. The CLI is a client -- it should not manage server configuration. Webhooks require a running listener process, which is a fundamentally different concern. | Document how to use webhooks with the API directly. Out of scope per PROJECT.md. |
| **User/permission management** | Admin features that few users need. Adds complexity to auth model and increases security surface. Most CRM users are not admins. | Defer to web UI for admin tasks. The CLI serves individual users managing their CRM data. |
| **Offline mode with sync** | Conflict resolution in a multi-user CRM is extremely hard. Read-only caching is fine; bidirectional sync is a project unto itself. Would need queue management, eventual consistency, and merge conflict strategies. | Cache for read performance only. Always require network for mutations. Show clear error when offline. |
| **Built-in email/notification sending** | CRM CLIs should read/write CRM data, not become email clients. Email sending has deliverability, template rendering, and compliance concerns (CAN-SPAM, GDPR) that belong in the CRM server. | Use activities to log communications. Let the server handle actual sending. |
| **Custom scripting language / DSL** | Tempting to build `pipelite script run my-workflow.pipelite`. But the scripting language already exists: it is bash/sh/zsh. The CLI should be composable with shell, not replace it. | Provide excellent JSON output and stdin support so users compose workflows in their shell of choice. |
| **Plugin/extension system** | gh has extensions because GitHub's domain is enormous. A CRM CLI has a bounded domain (6 entities). An extension system adds architecture complexity (loading, versioning, API surface) for minimal value at this stage. | Keep the command set focused. If the API surface grows, add commands directly. Revisit only if the user base explicitly demands it. |
| **Real-time streaming / watching** | `pipelite deals watch` that streams changes. Requires websocket support, server-side events, or polling. Fundamentally different interaction model than request-response CLI. | Users can use system `watch` command: `watch -n 30 pipelite deals list`. Support `--watch` only if the server provides a changes endpoint. |
| **Import/export wizards** | Complex multi-step import flows (mapping CSV columns to CRM fields, handling duplicates, dry-run then commit, rollback on partial failure) are better handled by the web UI or a dedicated ETL tool. | Support basic `--stdin` for bulk creates from JSON. For complex imports, recommend the web UI or a dedicated script. |
| **Markdown/rich text rendering in terminal** | CRM notes may contain markdown or HTML. Rendering this in the terminal adds a dependency and rarely looks good. | Output raw text. Users can pipe to `glow` or `bat` if they want rendered markdown. |
| **Auto-update mechanism** | Distribution concern, not CLI concern. Adds complexity (checking for updates, downloading binaries, permissions) and security risk (code execution from network). | Use cargo-binstall, package managers, or GitHub releases for updates. |

## Feature Dependencies

```
Authentication (init/login) --> ALL other features
  |
  v
Config file management --> Profiles, cached data
  |
  v
HTTP client + API layer --> CRUD operations on all entities
  |
  v
Entity CRUD (deals, orgs, people, activities, pipelines, stages)
  |                |                    |
  v                v                    v
Output formatting  Interactive prompts  Filtering/pagination
(json/csv/table)   (create/update)     (--stage, --limit)
  |
  v
Shell completions (depends on cached entity data)
  |
Pipeline dashboard (depends on deals + pipelines CRUD)
  |
Stdin piping (depends on output format + create/update commands)
```

**Critical path:** Auth -> Config -> HTTP Client -> Entity CRUD -> Output Formats

Everything else layers on incrementally after CRUD works with at least JSON output.

## MVP Recommendation

**Phase 1 -- Foundation (must ship first):**
1. Authentication (`pipelite init`, API key storage in config, env var support)
2. Configuration file (`~/.pipelite/config.toml`, `pipelite config set/get/show`)
3. Connection test (`pipelite ping`)
4. Core HTTP client with structured error handling

**Phase 2 -- CRUD + Output (the product becomes useful):**
1. Full CRUD on all 6 entities (deals, orgs, people, activities, pipelines, stages)
2. JSON, table, and CSV output formats with TTY auto-detection
3. List filtering (`--stage`, `--owner`, `--limit`)
4. Field selection (`--fields`)
5. Non-zero exit codes, actionable error messages
6. Colored output with `NO_COLOR` support

**Phase 3 -- Developer Experience (the product becomes pleasant):**
1. Interactive prompts for create/update
2. Headless mode (`--no-input`)
3. Shell completions (bash, zsh, fish)
4. `--dry-run` for mutations
5. ASCII splash screen
6. Quiet mode (`-q`)

**Phase 4 -- Power User Features (the product becomes loved):**
1. Local caching for lookups
2. Stdin piping for bulk operations
3. Pipeline dashboard
4. Config profiles for multiple servers
5. Activity logging shortcut
6. Plain and JSONL output formats

**Defer indefinitely:** TUI, webhooks, admin features, offline sync, plugin system, DSL, auto-update.

## Sources

- [Command Line Interface Guidelines (clig.dev)](https://clig.dev/) -- Comprehensive CLI design principles covering output, errors, flags, interactivity, configuration. HIGH confidence.
- [The 12 Rules of Great CLI UX](https://dev.to/chengyixu/the-12-rules-of-great-cli-ux-lessons-from-building-30-developer-tools-39o6) -- Practical rules from building 30+ developer tools. MEDIUM confidence.
- [GitHub CLI Manual](https://cli.github.com/manual/) -- Reference implementation for modern CLI design (aliases, extensions, JSON output, interactive flows). HIGH confidence.
- [Stripe CLI Documentation](https://docs.stripe.com/stripe-cli) -- Patterns for API-client CLIs (resource commands, webhook forwarding, log tailing). HIGH confidence.
- [kubectl Command Reference](https://kubernetes.io/docs/reference/kubectl/) -- CRUD-on-resources CLI patterns (`verb type name` structure). HIGH confidence.
- [HubSpot CLI](https://developers.hubspot.com/docs/developer-tooling/local-development/hubspot-cli/install-the-cli) -- CRM-adjacent CLI (auth init, file management, config). MEDIUM confidence.
- [AWS CLI Output Formats](https://docs.aws.amazon.com/cli/latest/userguide/cli-usage-output-format.html) -- Multi-format output patterns (json, table, text, yaml). HIGH confidence.
- [gh alias documentation](https://cli.github.com/manual/gh_alias) -- Extension and alias patterns for CLI customization. HIGH confidence.
