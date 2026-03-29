# Pipelite CLI

A fast, scriptable command-line interface for managing your [Pipelite](https://app.pipelite.io) CRM from the terminal. Built in Rust for speed and reliability.

```
 ____  _            _ _ _
|  _ \(_)_ __   ___| (_) |_ ___
| |_) | | '_ \ / _ \ | | __/ _ \
|  __/| | |_) |  __/ | | ||  __/
|_|   |_| .__/ \___|_|_|\__\___|
        |_|
```

## Features

- **Full CRM management** -- Deals, Organizations, People, Activities, Pipelines, Stages, and Workflows
- **Multiple output formats** -- JSON, Table, CSV, and Plain text
- **Smart shell completions** -- Fuzzy ID + name suggestions for Bash, Zsh, and Fish
- **Interactive & headless modes** -- Prompts for missing fields or run fully automated
- **Dry-run support** -- Preview any mutation before executing
- **Local caching** -- TTL-based cache for fast completions and repeated lookups
- **Dashboard view** -- Pipeline overview with deal counts and values per stage
- **Scriptable** -- Pipe-friendly JSON output, composable with `jq`, `xargs`, etc.

## Quick Start

### Install

```bash
# Build from source
cargo build --release

# The binary is at target/release/pipelite
cp target/release/pipelite ~/.local/bin/
```

### Configure

```bash
# Interactive setup (prompts for URL and API key)
pipelite init

# Or headless setup
pipelite init --url https://crm.example.com --key pk_live_abc123

# Test connectivity
pipelite ping
```

### First Commands

```bash
# View your pipeline overview
pipelite dashboard

# List all deals
pipelite deals list

# Create a deal
pipelite deals create --title "Acme Contract" --stage stg_001 --value 50000

# Get a specific deal as JSON
pipelite deals get deal_abc123 --format json
```

## Commands

### Global Flags

These flags apply to all commands:

| Flag | Short | Description |
|------|-------|-------------|
| `--format <fmt>` | | Output format: `json`, `table`, `csv`, `plain` |
| `--no-color` | | Disable colored output |
| `--quiet` | `-q` | Suppress non-essential output |
| `--verbose` | `-v` | Show debug information |
| `--no-input` | | Disable interactive prompts |
| `--dry-run` | | Preview mutations without executing |

### Entity Management

All entities follow a consistent CRUD pattern:

```
pipelite <entity> list [filters...] [--limit N] [--offset N] [--all] [--fields f1,f2]
pipelite <entity> get <id> [--fields f1,f2] [--expand rel1,rel2]
pipelite <entity> create [--field value...] [--custom-field key=value]
pipelite <entity> update <id> [--field value...]
pipelite <entity> delete <id>
```

#### Deals (`deals` | `d`)

```bash
# List with filters
pipelite deals list --stage stg_001 --org org_abc --limit 10
pipelite deals list --all --format json

# Create with custom fields
pipelite deals create --title "Big Deal" --stage stg_001 --value 75000 \
  --org org_abc --custom-field priority=high

# Batch create from stdin
echo '[{"title":"Deal A","stage_id":"stg_001"},{"title":"Deal B","stage_id":"stg_001"}]' \
  | pipelite deals create --stdin

# Update
pipelite deals update deal_abc123 --title "Renamed Deal" --value 100000

# Select specific fields
pipelite deals list --fields id,title,value,stage_id
```

#### Organizations (`orgs` | `o`)

```bash
pipelite orgs list
pipelite orgs create --name "Acme Corp"
pipelite orgs update org_abc123 --name "Acme Corporation" --website "https://acme.com"
pipelite orgs get org_abc123 --format json
pipelite orgs delete org_abc123
```

#### People (`people` | `p`)

```bash
pipelite people list
pipelite people create --first-name John --last-name Doe --email john@acme.com
pipelite people update per_abc123 --phone "+1-555-0100"
pipelite people get per_abc123 --expand organization
pipelite people delete per_abc123
```

#### Activities (`activities` | `a`)

```bash
pipelite activities list
pipelite activities create --title "Follow up call" --type type_call --deal deal_abc123
pipelite activities update act_abc123 --completed-at "2024-01-15T10:00:00Z"
pipelite activities get act_abc123
pipelite activities delete act_abc123
```

#### Pipelines (`pipelines` | `pl`)

```bash
pipelite pipelines list
pipelite pipelines create --name "Sales Pipeline"
pipelite pipelines update pl_abc123 --name "Enterprise Sales"
pipelite pipelines get pl_abc123
pipelite pipelines delete pl_abc123
```

#### Stages (`stages` | `s`)

```bash
pipelite stages list --pipeline pl_abc123
pipelite stages create --name "Qualified" --pipeline pl_abc123 --position 2
pipelite stages update stg_abc123 --name "Proposal Sent" --color "#4CAF50"
pipelite stages get stg_abc123
pipelite stages delete stg_abc123
```

#### Workflows (`workflows` | `w`)

```bash
# List and filter
pipelite workflows list
pipelite workflows list --active true

# Create
pipelite workflows create --name "New Deal Alert"
pipelite workflows create --name "Auto-assign" \
  --triggers '[{"type":"crm_event"}]' --nodes '[{"type":"action"}]'

# Create from stdin
echo '{"name":"WF","triggers":[]}' | pipelite workflows create --stdin

# Update
pipelite workflows update wf_abc123 --name "Updated Name" --active false

# Trigger a workflow run
pipelite workflows trigger wf_abc123
pipelite workflows trigger wf_abc123 --data '{"dealId":"deal_001"}'
pipelite workflows trigger wf_abc123 --data @payload.json

# Delete (with force to skip confirmation)
pipelite workflows delete wf_abc123 --force
```

### Utility Commands

#### Dashboard

Pipeline overview with deal counts and total values per stage:

```bash
pipelite dashboard
pipelite dashboard --format json
pipelite dashboard --format csv
```

#### Configuration

```bash
# Show current configuration
pipelite config show

# Set a configuration value
pipelite config set output.format json
pipelite config set display.no_color true
```

#### Cache Management

```bash
# Clear all cached data
pipelite cache clear

# Refresh pipeline and stage caches
pipelite cache refresh
```

#### Shell Completions

```bash
# Bash
source <(pipelite completions bash)

# Zsh
source <(pipelite completions zsh)

# Fish
pipelite completions fish > ~/.config/fish/completions/pipelite.fish
```

#### Connectivity Test

```bash
pipelite ping
```

## Configuration

### Config File

Located at `~/.pipelite/config.toml` (override with `PIPELITE_CONFIG` env var):

```toml
# Pipelite CLI Configuration

[server]
# Pipelite CRM server URL
url = "https://app.pipelite.io"
# API key for authentication (keep this secret!)
api_key = "pk_live_abc123"

[output]
# Default output format: json, table, csv, plain
# format = "table"

[display]
# Disable colored output (also: NO_COLOR env var or --no-color flag)
# no_color = false
```

The config file is created with `0600` permissions (owner read/write only) to protect the API key.

### Configuration Precedence

Settings are resolved in this order (highest priority first):

1. **CLI flags** (`--format json`, `--no-color`)
2. **Environment variables** (`PIPELITE_API_KEY`, `PIPELITE_SERVER_URL`, `NO_COLOR`)
3. **Config file** (`~/.pipelite/config.toml`)
4. **Defaults** (table format for TTY, JSON for pipes)

### Environment Variables

| Variable | Description |
|----------|-------------|
| `PIPELITE_API_KEY` | API key (overrides config file) |
| `PIPELITE_SERVER_URL` | Server URL (overrides config file) |
| `PIPELITE_CONFIG` | Custom config file path |
| `NO_COLOR` | Disable colored output (any value) |

## Output Formats

### Table (default in terminal)

```
+------------+-------------+--------+
| id         | title       | value  |
+------------+-------------+--------+
| deal_001   | Acme Deal   | 50,000 |
| deal_002   | Beta Corp   | 75,000 |
+------------+-------------+--------+
Showing 2 of 2 results
```

### JSON (default when piped)

```bash
pipelite deals list --format json
# or just pipe it
pipelite deals list | jq '.[].title'
```

### CSV

```bash
pipelite deals list --format csv > deals.csv
```

### Plain

```bash
# Values only, no headers -- useful for scripting
pipelite deals list --format plain --fields id | xargs -I {} pipelite deals get {}
```

### Field Selection

Select specific columns with `--fields`:

```bash
pipelite deals list --fields id,title,value
pipelite orgs list --fields id,name,website
```

### Smart Formatting

- Date/time fields (ending in `_at` or `_date`): displayed as relative time ("3 hours ago")
- Value fields: formatted with commas ("50,000.00")
- Null values: displayed as empty strings

## Dry-Run Mode

Preview any mutation without actually executing it:

```bash
pipelite deals create --title "Test" --stage stg_001 --dry-run
```

Output:

```
POST https://app.pipelite.io/api/v1/deals
{
  "title": "Test",
  "stage_id": "stg_001"
}
```

With `--format json`:

```json
{
  "dry_run": true,
  "method": "POST",
  "url": "https://app.pipelite.io/api/v1/deals",
  "body": {
    "title": "Test",
    "stage_id": "stg_001"
  }
}
```

## Caching

Pipelite CLI maintains a local cache at `~/.pipelite/cache/` to speed up shell completions and reduce API calls.

### TTL Values

| Entity | TTL | Reason |
|--------|-----|--------|
| Pipelines | 1 hour | Rarely change |
| Stages | 1 hour | Rarely change |
| Workflows | 1 hour | Rarely change |
| Deals, Orgs, People, Activities | 5 minutes | Change frequently |

### Cache Behavior

- **Atomic writes**: Uses temp file + rename to prevent corruption
- **Auto-recovery**: Corrupted cache files are silently deleted and re-fetched
- **Prefix invalidation**: Related cache entries are cleared together (e.g., all stage caches when a pipeline changes)
- **Shell completions**: Cache provides ID + name suggestions for Tab completion

## Error Handling

Errors are displayed with category, detail, and actionable hint:

```
error: Authentication failed
  invalid API key format
  hint: run `pipelite init` to reconfigure
```

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Runtime error (connection, API, validation) |
| `2` | Usage error (missing arguments, invalid flags) |

## Scripting Examples

### Export all deals to CSV

```bash
pipelite deals list --all --format csv > all_deals.csv
```

### Move all deals from one stage to another

```bash
pipelite deals list --stage stg_old --all --format plain --fields id \
  | xargs -I {} pipelite deals update {} --stage stg_new
```

### Count deals per pipeline

```bash
pipelite dashboard --format json | jq '.[] | "\(.pipeline): \(.total_deals) deals"'
```

### Create deals from a JSON file

```bash
cat deals.json | pipelite deals create --stdin
```

### Check if configured (for scripts)

```bash
if pipelite ping --quiet 2>/dev/null; then
  echo "Pipelite is configured and reachable"
fi
```

### Workflow automation

```bash
# Trigger a workflow with data
pipelite workflows trigger wf_notify --data '{"event":"new_quarter","region":"EMEA"}'

# Trigger with data from file
pipelite workflows trigger wf_report --data @quarterly_params.json
```

## Development

### Prerequisites

- Rust 1.84+ (edition 2024)
- Cargo

### Build

```bash
cargo build           # Debug build
cargo build --release # Release build
```

### Test

```bash
cargo test            # Run all tests
cargo test -- --nocapture  # With output
```

### Project Structure

```
src/
  main.rs           # Entry point, command routing
  cli/              # CLI argument definitions (clap)
    mod.rs          # Top-level CLI struct and Commands enum
    deals.rs        # Deal subcommands and args
    orgs.rs         # Organization subcommands
    people.rs       # People subcommands
    activities.rs   # Activity subcommands
    pipelines.rs    # Pipeline subcommands
    stages.rs       # Stage subcommands
    workflows.rs    # Workflow subcommands
    config.rs       # Config subcommands
    cache.rs        # Cache subcommands
    dashboard.rs    # Dashboard args
    init.rs         # Init args
    completions.rs  # Shell completion args
  commands/         # Command handler implementations
    deals/          # list.rs, get.rs, create.rs, update.rs, delete.rs
    orgs/           # (same pattern)
    people/
    activities/
    pipelines/
    stages/
    workflows/      # + trigger.rs
    config/
    cache/
    init.rs
    ping.rs
    dashboard.rs
    completions.rs
  api/
    mod.rs          # PipeliteClient (HTTP client)
    models.rs       # Data types (Deal, Org, Person, etc.)
  output/
    mod.rs          # Format dispatch
    json.rs         # JSON renderer
    table.rs        # Table renderer (comfy-table)
    csv.rs          # CSV renderer
    plain.rs        # Plain text renderer
    format.rs       # Value formatting (dates, currency)
    fields.rs       # Field extraction
  config.rs         # Config file I/O
  context.rs        # AppContext (config + client + settings)
  cache.rs          # TTL-based file cache
  error.rs          # CliError types and display
  dry_run.rs        # Dry-run preview rendering
  prompt.rs         # Interactive prompt helpers
  splash.rs         # Splash screen
```

## License

MIT
