# Operations Reference

Exact flags, wire vocabulary, pagination limits, and format behavior for every
command group. Verified against v1.1. Global flags (`--format`, `--no-color`,
`-q/--quiet`, `-v/--verbose`, `--no-input`, `--dry-run`) exist on every
subcommand and are not repeated per row below.

## Entities (CRUD)

`pipelite <entity> list / get / create / update / delete` for
`deals`, `orgs`, `people`, `activities`, `pipelines`, `stages`, `workflows`.

### List flags (all 7 entities)

| Flag | Purpose |
|------|---------|
| `--limit <n>` | Page size (server default 50) |
| `--offset <n>` | Page offset |
| `--all` | Auto-paginate up to the server ceiling (1000); emits a stderr warning at the ceiling |
| `--fields a,b,c` | Project specific fields (all formats) |
| `--expand a,b` | Inline related records (payload appears under `expanded` in JSON) |

Entity-specific list filters: `deals list --stage/--org/--owner <id>`;
`stages list [--pipeline <id>]` (omit for all stages across pipelines);
`workflows list --active true|false` (⚠ filters client-side after fetching —
stderr warning says so).

### Create — required flags per entity

| Entity | Required | Notable optional |
|--------|----------|------------------|
| deals | `--title`, `--stage <stage_id>` | `--value`, `--org`, `--person`, `--expected-close-date`, `--notes`, `--custom-field k=v` |
| orgs | `--name` | `--website`, `--industry`, `--notes`, `--custom-field k=v` |
| people | `--first-name`, `--last-name` | `--email`, `--org`, `--custom-field k=v` |
| activities | `--title`, `--type <type_id>` | `--deal`, `--due-at <ISO>`, `--notes`, `--custom-field k=v`; update: `--mark-done` / `--mark-undone` / `--completed-at <ISO>` |
| pipelines | `--name` | `--default` |
| stages | `--name`, `--pipeline <id>` | `--type won\|lost`, `--color`, `--position` |
| workflows | `--name` | `--description`, `--triggers '<json>'`, `--nodes '<json>'`, `--stdin` |

⚠ `workflows create` leaves the workflow **inactive** — `workflows trigger`
returns 409 with an activation hint until
`workflows update <id> --active true`.

### Wire vocabulary ≠ flag vocabulary

JSON field names differ from CLI flags in places. Field names that matter when
parsing JSON output or building `--stdin`/`--custom-field-json` bodies:

| Concept | CLI flag | JSON field |
|---------|----------|------------|
| deal stage | `--stage` | `stage_id` |
| note text | `--body` | `content` |
| note/record kind | `deals/orgs/people/activities` | `entity_type`: `deal/organization/person/activity` (singular) |
| custom fields | `--custom-field k=v` | `custom_fields` object, values typed per definition |
| expand payloads | `--expand` | `expanded` (flatten map on all 7 Base models) |
| org link on person/deal | `--org` | `organization_id` |

All 7 Base models also expose: `id`, `created_at`, `updated_at` (deals add
`position` — may be fractional), and unknown server keys land in `expanded`
instead of being dropped.

## Batch

Works on all 7 entities (create's server batch route covers deals/orgs/people/activities; other entities loop per-item):

| Operation | Form | Notes |
|-----------|------|-------|
| Update many | `echo '[{"id":"…","title":"New"}, …]' \| <entity> update --stdin` | Each object needs `"id"` + patched fields; JSON **array only** (NDJSON is rejected, exit 2) |
| Delete many | `<entity> delete id1 id2 …` or `--stdin` with `["id1","id2"]` | Non-TTY requires `--force` |
| Create many | `<entity> create --stdin` with array of create objects | Single-object stdin is rejected — must be an array |

Behavior: items continue past per-item failures (there is no
`--continue-on-error` flag — continuation is built in). stderr gets
`[i/n] Failed <id>: <reason>` per failure plus a final `N ok, M failed`
summary that survives `--quiet`. Exit 0 only when every item succeeded; any
failure → exit 1. Missing `id`s / malformed JSON → exit 2, zero HTTP.
Piped (non-TTY) deletes refuse without `--force`.

## Notes

`pipelite notes <list|add|edit|delete> <deals|orgs|people|activities> <parent-id> [note-id]`

- Body sources, exactly one: `--body "text"` (or `--body @file.md`, `@-` = stdin) > `--stdin` > interactive prompt. Two explicit sources → exit 2.
- No single-note GET: read with `notes list <type> <id> --format json` (the JSON field is `content`).
- Edit/delete address the note by ID; the parent arguments are context only.
- Delete is soft; non-TTY requires `--force`.

## Webhooks

`pipelite webhooks <list|get|create|update|delete>`

- `create --url https://… --events deal.created,deal.updated` (https enforced, 13 valid events validated pre-HTTP — an unknown event lists all 13).
- The signing secret appears **exactly once**, full, in create output. Never in list/get. Never cached.
- `update` merges: flags you pass are applied to the fetched webhook, omitted keys unchanged. `--stdin` sends a verbatim PUT body instead.
- Another user's webhook 403s even with an admin key.
- Valid events: `deal.created`, `deal.updated`, `deal.deleted`, `deal.stage_changed`, `person.created`, `person.updated`, `person.deleted`, `organization.created`, `organization.updated`, `organization.deleted`, `activity.created`, `activity.updated`, `activity.deleted`.

## Workflow runs

- `workflows runs list --workflow <id> [--status <s>] [--include-dry-run] [--limit/--offset]` — statuses: pending/running/completed/failed/waiting. Test runs hidden unless `--include-dry-run`.
- `workflows runs get <run-id> --workflow <id>` — full run + flattened `steps[]` (node, status, input, output, error, timing).
- `--watch` polls every 2s until terminal (completed/failed); tolerates 3 consecutive poll failures; Ctrl-C exits 130. `--exit-status` → exit 1 if the run failed (works even if already terminal when you start).

## Templates

`pipelite templates <list|get|create|delete>` — **no update exists** (a hidden `templates update` explains this, exit 2).

- `create --name X --workflow <id>` snapshots the workflow's first trigger + nodes (warns if multiple triggers); `--trigger '<json>'` for inline; `--stdin` for a verbatim body.
- Wire shape: `{name, description?, category?, trigger, nodes}`.

## Trash

`pipelite trash <list|restore|purge>`

- Types accept 9 aliases: `deal/deals`, `organization/orgs/organizations`, `person/people`, `activity/activities` → normalized to 4 server tabs.
- `list [--type <t>]` — ⚠ without `--type` you only see the **deals** tab.
- `restore <type> <id>` — no confirmation; restorable rows carry `linked_parents` (also-in-trash parents) and `deleted_by`.
- `purge [--type <t>]` — irreversible, admin-only, strongest confirmation; non-TTY without `--force` → exit 2, zero HTTP. Fan-out per record; summary `N permanently destroyed`.

## Audit

`pipelite audit list [--entity-type] [--entity-id] [--actor-kind] [--workflow-run-id] [--limit --offset]`

- Admin-gated: non-admin keys get `Forbidden` + "audit log requires an admin key".
- Filters pass through verbatim (server validates); empty flags are omitted from the request.
- `--limit` clamped to 1..=100 client-side; **no `--all`**.
- `changes` payload visible only via `--format json`.

## Custom fields

`pipelite custom-fields <list|get|create|update|delete>`

- `create --entity-type <t> --key <name> --type <type> [--options a,b,c] [--required] [--show-in-list] [--stdin]`
  - Types: `text`, `number`, `boolean`, `date`, `single_select` (alias `select`), `multi_select`, `file`, `url`, `lookup`, `formula`.
  - `--options` → `config.options` (required for select kinds).
  - No `--description` and no create-time `--position` (server assigns position = max+10000; reorder via `update --position`).
- `update <id> --name/--config '<json>'/--required/--no-required/--show-in-list/--no-show-in-list/--position <f64>`
- `delete <id>` — soft delete; confirm contract; deleted definitions remain in list output (the server does not mark them).
- List filters `--entity-type`; definitions mutations invalidate the `custom_fields_` cache prefix.

Typed writing on deals/orgs/people/activities (`--custom-field k=v`):

| Definition type | Stored JSON |
|-----------------|-------------|
| number | number (`4`, never `"4"`; non-numeric input → exit 2) |
| boolean | strict `true`/`false` |
| date | ISO string |
| single_select/select | string, validated against `config.options` |
| multi_select | comma-split array (`a,b` → `["a","b"]`), options validated |
| text, file, url, lookup, unknown name | raw string |
| formula | refused, exit 2 (server strips formula writes) |

`--custom-field-json '{"k": …}'` writes verbatim. `--custom-field`,
`--custom-field-json`, `--stdin` are mutually exclusive (exit 2).
`--dry-run` types from cache only; cold cache → raw strings + a visible note.

## Docs, config, cache, misc

- `pipelite docs [--save <file>] [--force]` — OpenAPI 3.1 spec, no API key sent; `--format` accepted but ignored; `--save` creates parent dirs, refuses overwrite without `--force`.
- `pipelite config <show|get|set>` — `set server.url <u>` / `set server.api_key <k>` / `set output.format <f>`; edits preserve comments.
- `pipelite cache <clear|refresh>` — clear wipes `~/.pipelite/cache/`; refresh pre-populates from the server.
- `pipelite dashboard` — pipeline overview (deal counts/values per stage).
- `pipelite completions <shell>` — shell completions.
- `pipelite init [--url <u>]` — guided first-run setup (interactive; pass `--no-input`-friendly env vars instead in headless environments).

## Pagination limits (server-enforced)

| Surface | Page default | Cap | `--all` |
|---------|-------------|-----|---------|
| 7 entities list | 50 | 1000 (ceiling warning) | yes |
| workflow runs | 50 | — | no (offset loop) |
| templates | 50 | — | no |
| notes | 50 | 100 | no |
| trash | 50 | offset 10,000 | yes (fan-out) |
| audit | 50 | 100 | no |
| custom-field definitions | 50 | — | no |
