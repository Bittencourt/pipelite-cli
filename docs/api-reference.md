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

## `pipelite notes`

Notes annotate deals, organizations, people, and activities — the four note-capable entity types. The server orders notes newest first and soft-deletes them; only a note's author or an admin may edit or delete it (403 otherwise, rendered with the registered notes hint).

**No single-note GET:** the server offers no way to fetch one note by ID. Use `notes list <type> <id> --format json` to read a note before editing. A hidden `notes get` subcommand exists only to reject with a hint (exit 2, zero requests).

### `notes list <type> <parent-id>`

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--limit` | u64 | `50` | Maximum notes (server caps pages at 100 — iterate `--offset`; there is no `--all`) |
| `--offset` | u64 | `0` | Pagination offset |
| `--fields` | string[] | | Select columns |

Valid `<type>` values: `deals`, `orgs`, `people`, `activities` — anything else exits 2 with "notes are only available on deals, orgs, people, activities" before any request. Table output truncates note content to ~80 characters on one line (newlines flattened); `--format json` (or plain) prints the full raw text. An empty page prints one stderr hint, suppressed by `--quiet`.

```bash
pipelite notes list deals d1
pipelite notes list deals d1 --limit 100 --format json
pipelite notes list deals d1 --format json   # view before editing
```

### `notes add <type> <parent-id>`

| Option | Type | Description |
|--------|------|-------------|
| `--body` | string | Note text, `@filepath` to read from a file, or `@-` to read stdin |
| `--stdin` | flag | Read the note text from stdin |

The body comes from EXACTLY ONE source: `--body` (literal, `@file`, or `@-`), `--stdin`, or an interactive prompt on a TTY. Two sources (including `--body @-` together with `--stdin`) exit 2 before any request; an unreadable `@file` exits 2 with a check-the-path hint. Whitespace-only content is rejected by the server with a 422 (no client-side trim).

```bash
pipelite notes add deals d1 --body "Followed up"
pipelite notes add deals d1 --body @note.md
echo "Note text" | pipelite notes add deals d1 --stdin
```

### `notes edit <type> <parent-id> <note-id>`

Same body options as `add`. The request carries only the note ID — `<type>` and `<parent-id>` are required by the grammar but otherwise unused. No confirmation (editing is non-destructive); `--dry-run` previews the PATCH. There is no single-note GET — view the current content first with `notes list <type> <id> --format json`.

```bash
pipelite notes edit deals d1 n1 --body "Updated text"
pipelite notes edit deals d1 n1 --body @note.md
```

### `notes delete <type> <parent-id> <note-id>`

| Option | Type | Description |
|--------|------|-------------|
| `--force` | flag | Skip confirmation prompt (required in non-interactive mode) |

Deletion is a soft delete and requires confirmation unless `--force` is given; without a TTY, `--force` is required (exit 1 refusal otherwise, before any request). `--dry-run` previews the request. Re-deleting a deleted note fails with 404 ("Note not found").

```bash
pipelite notes delete deals d1 n1
pipelite notes delete deals d1 n1 --force
pipelite notes delete deals d1 n1 --dry-run
```

---

## `pipelite webhooks`

Webhooks push CRM events to external automations — `pipelite webhooks` manages them. The signing secret is shown exactly once at creation; a webhook belonging to another user always 403s (even with an admin key — this is deliberate, unlike the trash/audit admin powers).

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/webhooks?limit&offset` | List webhooks (owner-scoped) |
| POST | `/api/v1/webhooks` | Create a webhook (201 — response carries the signing secret) |
| GET | `/api/v1/webhooks/:id` | Get a webhook (never carries the secret) |
| PUT | `/api/v1/webhooks/:id` | Update a webhook (merged body) |
| DELETE | `/api/v1/webhooks/:id` | Delete a webhook (204 hard delete) |

**No description field:** the server has no webhook description anywhere (DB schema + validation schemas accept only `{url, events, active}` and silently strip unknown keys) — the CLI offers no description option for webhooks; don't look for one. `--stdin` remains the full-control escape hatch.

### `webhooks create`

| Option | Type | Description |
|--------|------|-------------|
| `--url` | string | Webhook endpoint URL — **must use https://** (rejected client-side, exit 2, zero requests otherwise) |
| `--events` | string[] | Comma-separated event names to subscribe |
| `--stdin` | flag | Read a raw JSON body from stdin (verbatim; a `url` key must be https:// and `events` must hold only valid names) |

Valid events (13):

```
deal.created, deal.updated, deal.deleted, deal.stage_changed,
person.created, person.updated, person.deleted,
organization.created, organization.updated, organization.deleted,
activity.created, activity.updated, activity.deleted
```

Unknown event names exit 2 **before any request** with all 13 listed — the server would silently accept them and never fire. THE SIGNING SECRET IS SHOWN EXACTLY ONCE: the warning line `Signing secret (save it now — shown only once):` is followed by the full 64-char secret alone on its own line (in `--format json` the secret is inside the rendered body and the warning goes to stderr, so `| jq` keeps working). Save it immediately — it cannot be retrieved later.

```bash
pipelite webhooks create --url https://example.com/hook --events deal.created,deal.updated
echo '{"url":"https://example.com/hook","events":["deal.created"]}' | pipelite webhooks create --stdin
```

### `webhooks list`

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--limit` | u64 | `50` | Maximum webhooks (server caps pages at 100 — iterate `--offset`; there is no `--all`) |
| `--offset` | u64 | `0` | Pagination offset |
| `--fields` | string[] | | Select columns |

The `secret` column reads `(shown once at creation)` in every format — the real secret exists only in the create response. An empty list prints one stderr hint, suppressed by `--quiet`.

```bash
pipelite webhooks list
pipelite webhooks list --limit 100 --format json
```

### `webhooks get <id>`

Shows one webhook with the same `(shown once at creation)` secret placeholder. A webhook belonging to another user fails with 403 **even with an admin key** ("This webhook belongs to another user.") — webhooks are ownership-exclusive by design, unlike the trash purge / audit log admin powers.

```bash
pipelite webhooks get wh_abc123
pipelite webhooks get wh_abc123 --format json
```

### `webhooks update <id>`

| Option | Type | Description |
|--------|------|-------------|
| `--url` | string | New endpoint URL (must be https://) |
| `--events` | string[] | Comma-separated event names (replaces the current set) |
| `--active` | flag | Activate the webhook |
| `--inactive` | flag | Deactivate the webhook |
| `--stdin` | flag | PUT a raw JSON body verbatim (server accepts `{url, events, active}`) — bypasses the fetch+merge |

Update is a **get→merge→PUT**: the CLI fetches the webhook, merges your flags into it, and PUTs the full merged object — omitted keys are unchanged. Flags are validated before any request (unknown events / non-https URL exit 2 with zero HTTP); `--dry-run` previews the PUT with zero requests. The signing secret is never updatable and never shown on update.

```bash
pipelite webhooks update wh_abc123 --url https://example.com/new-hook
pipelite webhooks update wh_abc123 --inactive
echo '{"url":"https://example.com/hook","active":false}' | pipelite webhooks update wh_abc123 --stdin
```

### `webhooks delete <id>`

| Option | Type | Description |
|--------|------|-------------|
| `--force` | flag | Skip confirmation prompt (required in non-interactive mode) |

Deletion is a hard delete and requires confirmation unless `--force` is given; without a TTY, `--force` is required (exit 1 refusal otherwise, before any request). `--dry-run` previews the request. Deleting another user's webhook 403s even with an admin key.

```bash
pipelite webhooks delete wh_abc123
pipelite webhooks delete wh_abc123 --force
pipelite webhooks delete wh_abc123 --dry-run
```

---

## `pipelite trash`

The trash holds soft-deleted records — `pipelite trash` lists, restores, or permanently purges them. `restore` needs no confirmation (restore IS the recovery act); `purge` is admin-only and permanently destroys records — it cannot be undone.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/trash?type&offset&limit` | List trashed records (owner-or-admin scoped; `?type` optional — the server defaults to the deals tab) |
| POST | `/api/v1/trash/:type/:id/restore` | Restore one record (204; owner-or-admin; 404 if not in the trash) |
| DELETE | `/api/v1/trash/:type/:id` | Permanently destroy one record (204; **ADMIN-ONLY** — gated before record lookup) |

### Type aliases (9 → 4 tabs)

All type arguments accept singular and plural aliases, normalized to the PLURAL tab used in every request URL (the server 422s singular tokens). List rows carry both tokens: `entity_type` (singular, display) and `type` (plural — the round-trip token that pipes directly into `restore`/`purge`).

| You type | Tab used in URLs |
|----------|------------------|
| `deal` / `deals` | `deals` |
| `organization` / `orgs` / `organizations` | `organizations` |
| `person` / `people` | `people` |
| `activity` / `activities` | `activities` |

Unknown types are rejected **before any request** with exit 2 and the valid aliases in the hint.

### `trash list`

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--type` | string | (server default: deals tab) | Filter by one tab — any of the 9 aliases |
| `--limit` | u64 | `50` | Rows per page (server caps pages at 100) |
| `--offset` | u64 | `0` | Pagination offset |
| `--all` | flag | | Fetch every page of the selected tab — stops at the server's 10,000 offset cap |
| `--fields` | string[] | | Select columns |

Table columns: name, type, deleted_at, deleted_by, linked_parents. Table cells truncate `linked_parents` (joined names, ~80 chars) and collapse `deleted_by` to a kind label (`user (Jane <jane@x.com>)`, `workflow_run (Nightly)`, bare `api_key`/`import`/`system`/`not_recorded`/`unknown_user` — api_key has no name by server design); `--format json` exposes the FULL array and the FULL deleted_by objects. An empty trash prints one stderr hint, suppressed by `--quiet`.

```bash
pipelite trash list
pipelite trash list --type deals --format json | jq -r '.data[].id'   # round-trip: ids feed restore
pipelite trash list --all
```

### `trash restore <type> <id>`

Restores one record — **no confirmation**: restore IS the recovery act. `--dry-run` previews the POST. A 404 means the record is not in the trash anymore (already restored or purged, or never existed). Restoring another user's record with a member key fails with 403 and the general permission hint (restore is owner-or-admin, not admin-only).

```bash
pipelite trash restore deals t_abc123
pipelite trash restore people per_abc123 --dry-run
```

### `trash purge [--type <t>]`

Permanently destroys trashed records — **this cannot be undone. ADMIN-ONLY**: every per-record delete requires an admin API key (the server gates before record lookup; non-admin keys always 403 with "Permanent purge requires an admin API key."). There is no bulk endpoint — the CLI pages the scoped trash and issues one DELETE per record, continuing past per-item failures and reporting an "N permanently destroyed, M failed" summary (exit 1 if anything failed).

| Option | Type | Description |
|--------|------|-------------|
| `--type` | string | Limit the purge to one tab's trash (any alias). Omit to purge ALL tabs |
| `--force` | flag | Skip the confirmation prompt (required in non-interactive mode) |

Interactive use confirms with the strongest prompt in the CLI, naming the scope, the record count, and the words "permanently destroys". Non-interactive use without `--force` refuses with **exit 2 and zero requests** — deliberately stricter than the standard delete's exit-1 refusal. `--dry-run` previews the victim list with list requests only (zero deletes).

```bash
pipelite trash purge --type deals
pipelite trash purge --type deals --force
pipelite trash purge --dry-run
```

---

## `pipelite audit`

The audit log answers "who changed what" — a read-only record of every create/update/delete/merge across entities. **Admin-gated**: non-admin keys receive 403 for every audit command, and the server checks the key BEFORE validating the query — so the "The audit log requires an admin API key." hint appears whatever the filters (absent, valid, or invalid).

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/audit?entity_type&entity_id&actor_kind&workflow_run_id&offset&limit` | List audit entries (newest first, server-fixed sort) |

### `audit list`

All four filters are passed through **verbatim** — the server validates the values and 422s invalid ones (the message renders untouched). Empty flag values are ignored (never sent). Results are newest first (server-fixed order).

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--entity-type` | string | | Filter by entity type — `organization`, `person`, `deal`, `activity`, `import_session`, `export` |
| `--entity-id` | string | | Filter by entity ID |
| `--actor-kind` | string | | Filter by actor kind — `user`, `workflow_run`, `api_key`, `import`, `system` |
| `--workflow-run-id` | string | | Filter by workflow run ID |
| `--limit` | u64 | `50` | Entries per page — clamped to the server's 1..=100 range client-side |
| `--offset` | u64 | `0` | Pagination offset |
| `--fields` | string[] | | Select columns |

There is deliberately **no `--all`**: pages are capped at 100 and the server has no trash-style low offset cap, so iterate `--offset` to page deeper into history.

Default columns: timestamp (`created_at`), actor (kind + id), action (`created` \| `updated` \| `deleted` \| `merged`), entity (type/id). The `changes` payload is **visible via `--format json` only** — it carries per-field `{from, to}` pairs (`{}` for create/delete entries) and never renders in the table; field-level diff rendering is a deferred future feature.

```bash
pipelite audit list
pipelite audit list --entity-type deal --format json
pipelite audit list --actor-kind workflow_run --workflow-run-id wr9
pipelite audit list --offset 100   # next page — iterate --offset to page deeper
```

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
