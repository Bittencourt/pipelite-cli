# pipelite CLI

**pipelite** is a fast, scriptable command-line interface for [Pipelite CRM](https://github.com/Bittencourt/pipelite). Built in Rust, it provides full CRUD access to all CRM entities — deals, organizations, people, activities, pipelines, and stages — directly from your terminal.

Designed to feel like `gh`, `kubectl`, or the Stripe CLI: composable, predictable, and comfortable in both interactive sessions and headless automation.

---

## Features

- 🔐 **API key authentication** — Securely store credentials in `~/.pipelite/config.toml` or pass via environment variables
- 📋 **Full CRUD on all entities** — Deals, organizations, people, activities, pipelines, and stages
- 🖥️ **Smart output formatting** — Auto-detects TTY: table for humans, JSON for pipes
- 📊 **Multiple output formats** — `table`, `json`, `csv`, `plain` with `--format`
- 🔍 **Filtering and pagination** — Filter by stage, owner, organization, type, and more
- 🎯 **Field selection** — Return only the fields you need with `--fields id,title,value`
- 📥 **Stdin piping for batch operations** — `cat deals.json | pipelite deals create --stdin`
- 🎨 **Colored output** — Respects `NO_COLOR` and `--no-color`
- 🤫 **Quiet mode** (`-q`) — Suppress non-essential output for scripting
- ⚙️ **Flexible configuration** — CLI flags > environment variables > config file
- 🏓 **Health check** — Verify connectivity with `pipelite ping`
- 🧩 **Command aliases** — Shorter aliases for frequently used entity commands

---

## Prerequisites

- **A running Pipelite CRM server** — See the [Pipelite project](https://github.com/Bittencourt/pipelite) for setup instructions
- An API key generated from your Pipelite server

### Setting up a Pipelite Server

Refer to the [Pipelite repository](https://github.com/Bittencourt/pipelite) for full server setup documentation. The short version:

1. Clone and configure the Pipelite server
2. Start the server (default: `http://localhost:3000`)
3. Generate an API key from the server's admin interface
4. Use `pipelite init` to connect the CLI (see [Quick Start](#quick-start))

---

## Installation

### From Source (requires [Rust](https://rustup.rs/))

```bash
git clone https://github.com/Bittencourt/pipelite-cli.git
cd pipelite-cli
cargo build --release
# Binary is at ./target/release/pipelite
# Move it somewhere on your PATH:
sudo mv target/release/pipelite /usr/local/bin/
```

### Verify the Installation

```bash
pipelite --version
```

---

## Quick Start

### 1. Initialize your configuration

Run the interactive setup wizard:

```bash
pipelite init
```

You will be prompted for:
- **Server URL** — the base URL of your Pipelite server (e.g., `https://crm.example.com`)
- **API key** — the key generated from your server

For non-interactive or scripted setup, pass flags directly:

```bash
pipelite init --url https://crm.example.com --key pk_live_abc123
```

This saves your credentials to `~/.pipelite/config.toml` with restricted permissions (`0600`).

### 2. Test connectivity

```bash
pipelite ping
```

### 3. Start managing your CRM

```bash
# List your deals
pipelite deals list

# Create an organization
pipelite orgs create --name "Acme Corp"

# Add a contact
pipelite people create --first-name Jane --last-name Doe --email jane@acme.com

# Create a deal
pipelite deals create --title "Acme Corp Enterprise" --value 50000
```

---

## Configuration

The CLI reads configuration from `~/.pipelite/config.toml` (created by `pipelite init`).

```toml
[server]
url = "https://crm.example.com"
api_key = "pk_live_abc123"

[output]
format = "table"   # optional default format

[display]
no_color = false   # optional
```

**Precedence (highest to lowest):**

1. CLI flags (e.g., `--format json`)
2. Environment variables
3. Config file values
4. Built-in defaults

### Environment Variables

| Variable | Description |
|----------|-------------|
| `PIPELITE_API_KEY` | Override the API key from config |
| `PIPELITE_SERVER_URL` | Override the server URL from config |
| `PIPELITE_CONFIG` | Use a custom path for the config file |
| `NO_COLOR` | Disable all colored output (standard) |

### Manage Configuration

```bash
# Show current config
pipelite config show

# Update a value
pipelite config set server.url https://new-server.example.com
pipelite config set server.api_key pk_live_newkey
```

---

## Global Flags

These flags are available on every command:

| Flag | Description |
|------|-------------|
| `--format <fmt>` | Output format: `table`, `json`, `csv`, `plain` |
| `--no-color` | Disable colored output |
| `-q`, `--quiet` | Suppress non-essential output (data only) |
| `-v`, `--verbose` | Show debug information |
| `--version` | Print version and exit |
| `--help` | Show help for any command |

---

## Commands

### `pipelite init`

Initialize the CLI and store your server credentials.

```bash
pipelite init
pipelite init --url https://crm.example.com --key pk_live_abc123
```

### `pipelite ping`

Test connectivity to your Pipelite server.

```bash
pipelite ping
pipelite ping --format json
```

### `pipelite config`

View and update your local configuration.

```bash
pipelite config show
pipelite config set server.url https://crm.example.com
```

---

### Deals — `pipelite deals`

Manage sales deals.

**List deals:**
```bash
pipelite deals list
pipelite deals list --stage stg_abc123
pipelite deals list --owner usr_001 --limit 20
pipelite deals list --all                          # auto-paginate up to 1000 records
pipelite deals list --fields id,title,value        # select specific fields
pipelite deals list --format csv                   # CSV output for spreadsheets
```

**Get a single deal:**
```bash
pipelite deals get deal_abc123
pipelite deals get deal_abc123 --format json
pipelite deals get deal_abc123 --fields id,title,value,stage_id
```

**Create a deal:**
```bash
pipelite deals create --title "Big Deal" --stage stg_abc123
pipelite deals create --title "Enterprise Deal" --stage stg_001 --value 75000 --org org_123
```

**Batch create from stdin:**
```bash
echo '[{"title":"Deal A","stage_id":"stg_001"},{"title":"Deal B","stage_id":"stg_001"}]' \
  | pipelite deals create --stdin
```

**Update a deal:**
```bash
pipelite deals update deal_abc123 --title "Updated Title"
pipelite deals update deal_abc123 --value 90000 --stage stg_closed
```

**Delete a deal:**
```bash
pipelite deals delete deal_abc123
```

**Available fields:** `id`, `title`, `stage_id`, `value`, `organization_id`, `person_id`, `owner_id`, `expected_close_date`, `notes`, `custom_fields`, `created_at`, `updated_at`

**List filters:** `--stage <id>`, `--org <id>`, `--owner <id>`, `--limit N`, `--offset N`, `--all`

---

### Organizations — `pipelite orgs` (alias: `o`)

Manage companies and organizations.

```bash
pipelite orgs list
pipelite orgs list --owner usr_001
pipelite orgs get org_abc123
pipelite orgs create --name "Acme Corp" --website https://acme.com
pipelite orgs update org_abc123 --name "Acme Corporation"
pipelite orgs delete org_abc123
```

**Available fields:** `id`, `name`, `website`, `industry`, `owner_id`, `custom_fields`, `created_at`, `updated_at`

---

### People — `pipelite people` (alias: `p`)

Manage contacts.

```bash
pipelite people list
pipelite people list --org org_123
pipelite people list --owner usr_001
pipelite people get per_abc123
pipelite people create --first-name Jane --last-name Doe --email jane@acme.com
pipelite people update per_abc123 --phone "+1-555-0100"
pipelite people delete per_abc123
```

**Available fields:** `id`, `first_name`, `last_name`, `email`, `phone`, `organization_id`, `owner_id`, `custom_fields`, `created_at`, `updated_at`

---

### Activities — `pipelite activities` (alias: `a`)

Log and manage activities such as calls, meetings, and emails.

```bash
pipelite activities list
pipelite activities list --type call
pipelite activities list --owner usr_001
pipelite activities get act_abc123
pipelite activities create --title "Follow-up call" --type call
pipelite activities update act_abc123 --completed true
pipelite activities delete act_abc123
```

**Available fields:** `id`, `title`, `type`, `description`, `due_date`, `completed`, `owner_id`, `custom_fields`, `created_at`, `updated_at`

---

### Pipelines — `pipelite pipelines` (alias: `pl`)

Manage sales pipelines.

```bash
pipelite pipelines list
pipelite pipelines get pl_abc123
pipelite pipelines create --name "Enterprise Sales"
pipelite pipelines update pl_abc123 --name "SMB Sales"
pipelite pipelines delete pl_abc123
```

**Available fields:** `id`, `name`, `description`, `owner_id`, `stages`, `custom_fields`, `created_at`, `updated_at`

---

### Stages — `pipelite stages` (alias: `s`)

Manage the stages within a pipeline.

```bash
pipelite stages list --pipeline pl_abc123
pipelite stages get stg_abc123
pipelite stages create --name "Qualified" --pipeline pl_abc123
pipelite stages update stg_abc123 --name "Proposal Sent"
pipelite stages delete stg_abc123
```

**Available fields:** `id`, `name`, `pipeline_id`, `position`, `owner_id`, `custom_fields`, `created_at`, `updated_at`

---

## Output Formats

| Format | Flag | Best for |
|--------|------|----------|
| `table` | `--format table` | Interactive terminal use (default when TTY is detected) |
| `json` | `--format json` | Piping to `jq`, APIs, and scripting (default when piped) |
| `csv` | `--format csv` | Spreadsheets (Excel, Google Sheets) |
| `plain` | `--format plain` | Simple value extraction, `xargs`, shell scripting |

The format is auto-detected:
- **TTY (interactive terminal):** `table`
- **Piped / non-TTY:** `json`

Override at any time with `--format`.

### Field Selection

Use `--fields` to control which columns or properties are returned — works with all formats:

```bash
pipelite deals list --fields id,title,value
pipelite deals list --fields id,title --format csv
pipelite deals get deal_abc123 --fields id,title,value --format json
```

---

## Scripting & Automation

pipelite is designed to be composable with standard Unix tools.

### Extract IDs for further processing

```bash
# Delete all deals in a specific stage
pipelite deals list --stage stg_old --format plain --fields id \
  | xargs -I{} pipelite deals delete {}
```

### Filter with `jq`

```bash
# Find high-value deals and pretty-print them
pipelite deals list --format json | jq '.[] | select(.value > 10000)'
```

### Batch create from a JSON file

```bash
cat new_deals.json | pipelite deals create --stdin
```

### Export to CSV for a spreadsheet

```bash
pipelite deals list --all --format csv > deals_export.csv
```

### Use in CI/CD pipelines

Set credentials via environment variables to avoid writing config files:

```bash
export PIPELITE_SERVER_URL=https://crm.example.com
export PIPELITE_API_KEY=pk_live_abc123

pipelite deals list --format json | jq '.[] | .title'
```

### Quiet mode for clean output

Use `-q` to suppress status messages and spinner output — useful when the output is consumed by another program:

```bash
pipelite deals list -q --format json | jq '.'
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Runtime error (API error, network failure, not found, auth failure) |
| `2` | Usage error (invalid flags, missing required arguments) |

---

## Project Links

- 🌐 **Pipelite CRM Server** — [github.com/Bittencourt/pipelite](https://github.com/Bittencourt/pipelite)
- 🛠️ **pipelite CLI** — [github.com/Bittencourt/pipelite-cli](https://github.com/Bittencourt/pipelite-cli)

---

## Contributing

Contributions are welcome. Please open an issue or pull request on [GitHub](https://github.com/Bittencourt/pipelite-cli/issues).

When working on the codebase:

```bash
# Run all tests
cargo test

# Build in release mode
cargo build --release

# Check formatting and lints
cargo fmt --check
cargo clippy
```

---

## License

See [LICENSE](./LICENSE) for details.
