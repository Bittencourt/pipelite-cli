# API Reference

Complete reference for all Pipelite CLI commands, flags, and behavior.

## Global Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--format` | | `json\|table\|csv\|plain` | `table` (TTY) / `json` (pipe) | Output format |
| `--no-color` | | flag | | Disable colored output |
| `--quiet` | `-q` | flag | | Suppress non-essential output |
| `--verbose` | `-v` | flag | | Show debug information |
| `--no-input` | | flag | | Disable interactive prompts |
| `--dry-run` | | flag | | Preview mutations without executing |

---

## `pipelite init`

Initialize connection to a Pipelite server.

**Interactive mode** (default): Prompts for server URL and API key.
**Headless mode**: Pass `--url` and `--key` flags.

| Option | Type | Description |
|--------|------|-------------|
| `--url` | string | Server URL (e.g., `https://crm.example.com`) |
| `--key` | string | API key for authentication |

**Behavior**:
- Tests connectivity before saving
- Creates `~/.pipelite/config.toml` with 0600 permissions
- Prompts for confirmation before overwriting existing config

**Examples**:
```bash
pipelite init
pipelite init --url https://crm.example.com --key pk_live_abc123
```

---

## `pipelite ping`

Test server connectivity.

Attempts `/api/v1/ping` endpoint first, falls back to `/api/v1/deals?limit=1`.

**Examples**:
```bash
pipelite ping
pipelite ping --format json
```

---

## `pipelite config`

### `pipelite config show`

Display current configuration (redacts API key in table format).

### `pipelite config set <key> <value>`

Set a configuration value using dotted path notation.

| Key | Values | Description |
|-----|--------|-------------|
| `server.url` | URL string | Server URL |
| `server.api_key` | string | API key |
| `output.format` | `json\|table\|csv\|plain` | Default output format |
| `display.no_color` | `true\|false` | Disable colors |

**Examples**:
```bash
pipelite config show
pipelite config set output.format json
pipelite config set display.no_color true
```

---

## `pipelite deals` (alias: `d`)

### `deals list`

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--stage` | string | | Filter by stage ID |
| `--org` | string | | Filter by organization ID |
| `--owner` | string | | Filter by owner ID |
| `--limit` | u64 | `50` | Maximum results per page |
| `--offset` | u64 | `0` | Pagination offset |
| `--all` | flag | | Auto-paginate (up to 1000 results) |
| `--fields` | string[] | | Select specific columns (comma-separated) |
| `--expand` | string[] | | Expand relations (comma-separated) |

### `deals get <id>`

| Argument | Type | Description |
|----------|------|-------------|
| `id` | string | Deal ID (required) |

| Option | Type | Description |
|--------|------|-------------|
| `--fields` | string[] | Select specific fields |
| `--expand` | string[] | Expand relations |

### `deals create`

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--title` | string | Yes | Deal title |
| `--stage` | string | Yes | Stage ID |
| `--value` | f64 | No | Deal value |
| `--org` | string | No | Organization ID |
| `--person` | string | No | Person ID |
| `--expected-close-date` | string | No | ISO date format |
| `--notes` | string | No | Notes |
| `--custom-field` | string | No | `key=value` (repeatable) |
| `--stdin` | flag | No | Read JSON array from stdin for batch create |

### `deals update <id>`

Same options as `create` (all optional), plus `id` as positional argument.

### `deals delete <id>`

| Argument | Type | Description |
|----------|------|-------------|
| `id` | string | Deal ID to delete |

---

## `pipelite orgs` (alias: `o`)

### `orgs list`

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--limit` | u64 | `50` | Maximum results |
| `--offset` | u64 | `0` | Pagination offset |
| `--all` | flag | | Auto-paginate |
| `--fields` | string[] | | Select columns |
| `--expand` | string[] | | Expand relations |

### `orgs get <id>`

| Option | Type | Description |
|--------|------|-------------|
| `--fields` | string[] | Select fields |
| `--expand` | string[] | Expand relations |

### `orgs create`

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--name` | string | Yes | Organization name |
| `--website` | string | No | Website URL |
| `--industry` | string | No | Industry |
| `--notes` | string | No | Notes |
| `--owner` | string | No | Owner ID |
| `--custom-field` | string | No | `key=value` (repeatable) |

### `orgs update <id>`

Same options as `create` (all optional).

### `orgs delete <id>`

---

## `pipelite people` (alias: `p`)

### `people list`

Standard list options (`--limit`, `--offset`, `--all`, `--fields`, `--expand`).

### `people get <id>`

Standard get options (`--fields`, `--expand`).

### `people create`

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--first-name` | string | Yes | First name |
| `--last-name` | string | Yes | Last name |
| `--email` | string | No | Email address |
| `--phone` | string | No | Phone number |
| `--notes` | string | No | Notes |
| `--org` | string | No | Organization ID |
| `--owner` | string | No | Owner ID |
| `--custom-field` | string | No | `key=value` (repeatable) |

### `people update <id>`

Same options as `create` (all optional).

### `people delete <id>`

---

## `pipelite activities` (alias: `a`)

### `activities list`

Standard list options.

### `activities get <id>`

Standard get options.

### `activities create`

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--title` | string | Yes | Activity title |
| `--type` | string | Yes | Activity type ID |
| `--deal` | string | No | Associated deal ID |
| `--owner` | string | No | Owner ID |
| `--due-at` | string | No | Due date (ISO format) |
| `--completed-at` | string | No | Completion date (ISO format) |
| `--notes` | string | No | Notes |
| `--custom-field` | string | No | `key=value` (repeatable) |

### `activities update <id>`

Same options as `create` (all optional).

### `activities delete <id>`

---

## `pipelite pipelines` (alias: `pl`)

### `pipelines list`

Standard list options.

### `pipelines get <id>`

Standard get options.

### `pipelines create`

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--name` | string | Yes | Pipeline name |
| `--owner` | string | No | Owner ID |

### `pipelines update <id>`

Same options as `create` (all optional).

### `pipelines delete <id>`

---

## `pipelite stages` (alias: `s`)

### `stages list`

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--pipeline` | string | Yes | Pipeline ID to list stages for |

Plus standard list options.

### `stages get <id>`

Standard get options.

### `stages create`

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--name` | string | Yes | Stage name |
| `--pipeline` | string | Yes | Pipeline ID |
| `--description` | string | No | Description |
| `--color` | string | No | Hex color (e.g., `#4CAF50`) |
| `--type` | string | No | Stage type |
| `--position` | u64 | No | Position in pipeline |

### `stages update <id>`

Same options as `create` (all optional except `id`).

### `stages delete <id>`

---

## `pipelite workflows` (alias: `w`)

### `workflows list`

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--active` | bool | | Filter by active status |
| `--limit` | u64 | `50` | Maximum results |
| `--offset` | u64 | `0` | Pagination offset |
| `--all` | flag | | Auto-paginate |
| `--fields` | string[] | | Select columns |
| `--expand` | string[] | | Expand relations |

### `workflows get <id>`

Standard get options.

### `workflows create`

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--name` | string | Yes | Workflow name |
| `--description` | string | No | Description |
| `--active` | bool | No | Active status |
| `--triggers` | string | No | JSON array of triggers |
| `--nodes` | string | No | JSON array of nodes |
| `--stdin` | flag | No | Read full JSON from stdin |

### `workflows update <id>`

Same options as `create` (all optional).

### `workflows delete <id>`

| Option | Type | Description |
|--------|------|-------------|
| `--force` | flag | Skip confirmation prompt |

### `workflows trigger <id>`

| Option | Type | Description |
|--------|------|-------------|
| `--data` | string | JSON data (inline or `@filepath`) |

---

## `pipelite dashboard`

Pipeline overview showing deal counts and total values per stage.

Auto-paginates to fetch all deals. Also displays active workflow count.

---

## `pipelite cache`

### `cache clear`

Remove all cached data from `~/.pipelite/cache/`.

### `cache refresh`

Re-fetch and cache pipeline and stage data.

---

## `pipelite completions <shell>`

Generate shell completion scripts.

| Argument | Values |
|----------|--------|
| `shell` | `bash`, `zsh`, `fish` |

---

## Error Codes

| Code | Category | Description |
|------|----------|-------------|
| 0 | Success | Command completed successfully |
| 1 | Runtime | Connection, API, validation, or auth errors |
| 2 | Usage | Missing arguments, invalid flags, missing input |

## HTTP Status Mapping

| HTTP Status | CLI Error Type |
|-------------|---------------|
| 401, 403 | `Auth` -- "run `pipelite init` to reconfigure" |
| 404 | `NotFound` -- "run `pipelite <entity> list` to see available items" |
| 422 | `Validation` -- shows server validation message |
| Other 4xx/5xx | `Api` -- shows status code and server message |
| Connection error | `Connection` -- "check your network connection" |
| Timeout | `Connection` -- "server may be slow, try again" |
