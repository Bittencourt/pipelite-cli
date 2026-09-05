# Changelog

## v1.1 — Server v2 Parity (released 2026-09-04)

This release removes dead flags for real — every Breaking Changes entry
below names what to use instead.

### Breaking Changes

- **`people list --org` / `people list --owner` removed** — these flags were
  dead: the server ignored them and returned unfiltered data, silently lying
  about filtering. They now reject with exit code 2 before any HTTP call.
  Instead: fetch people (`pipelite people list`) and filter client-side, e.g.
  with `jq`.
- **`pipelines create --custom-field` / `stages create --custom-field`
  removed** — pipelines and stages do not support custom fields; values were
  silently dropped. Both flags now reject with exit code 2 before any HTTP
  call. `--custom-field` on deals/orgs/people/activities create+update is
  unaffected.
- **`workflows create --active` removed** — the server always creates
  workflows inactive, so the flag (and the interactive "Set workflow
  active?" prompt) misrepresented what would happen. `workflows create` now
  rejects `--active` with exit code 2, never prompts, and never sends
  `active` on create. New workflows start inactive; activate with
  `pipelite workflows update <id> --active true`.
- **`workflows list --active true|false` now filters client-side** — the
  server ignores the `active` query parameter, so the old flag silently
  returned unfiltered results. It now fetches all records and filters them
  locally, printing exactly one stderr warning per invocation:
  `warning: --active filters client-side after fetching all records`.
  Results differ from the old (silently unfiltered) behavior; the reported
  total reflects the filtered rows.
- **`--all` ceiling is now loud** — auto-pagination still stops at 1000
  records, but the truncation is now visible on stderr:
  `warning: --all stopped at 1000 records (server ceiling); results may be
  incomplete`. Exit code stays 0.
- **Position fields are now f64** — `Deal.position` and `Stage.position`
  deserialize fractional values (the server stores positions as numeric and
  may emit e.g. `10010.5`; the old `i64` model errored on them). JSON output
  now renders whole numbers as `10000.0` where the server emitted `10000` —
  correct but cosmetically different (serde f64 rendering).

Known limitations: `orgs list --owner` is also ignored by the server, but is
out of scope for this release and remains unchanged.

### Changed / Fixed

- **`--custom-field` values are now type-correct** — numbers are stored as
  JSON numbers (`price=4` → `4`, not `"4"`), booleans as `true`/`false`,
  multi-select as JSON arrays — resolved from cached field definitions
  (`pipelite custom-fields`) and matched by name. The server does not
  validate custom-field values via the API, so the CLI now validates
  client-side: select values must match the definition's options (exit 2
  otherwise, listing the valid options) and formula fields refuse writes
  (the server strips them). Unknown field names are sent as strings with a
  warning. Adds `--custom-field-json` for verbatim raw objects; it is
  mutually exclusive with `--custom-field` and `--stdin` (exit 2,
  consistent across all 8 create/update commands — `--stdin` exclusivity
  now always exits 2, previously exit 1 on the four create commands).
  `--dry-run` never fetches definitions — it falls back to the local cache
  and says so; with a warm cache it still writes typed values.

### Added

- **Batch update/delete across all 7 entities** — `<entity> update --stdin`
  (JSON array of `{"id":...}` objects, per-item PUT) and
  `<entity> delete id1 id2 ...` / `--stdin` (JSON array of IDs) with
  confirmation-bypassing `--force` for scripts. Per-item failures never abort
  the batch; the final `N ok, M failed` summary survives `--quiet`. Exit
  codes are trustable: `0` only when every item succeeded, `1` on any item
  failure, `2` for structurally broken input rejected before the first HTTP
  call. Batch `deals/orgs/people create --stdin` posts to the server batch
  endpoints; activities/pipelines/stages loop individual creates with the
  same summary.
- **Workflow runs** — `workflows runs list --workflow <id>` (`--status`,
  `--include-dry-run`) and `workflows runs get <run-id> --workflow <id>`
  with `--watch` (2-second poll, tolerates 3 consecutive failed polls) and
  `--exit-status` (exit 1 when the watched run ends `failed`).
- **Workflow templates** — `templates list/get/create/delete`. Create
  snapshots `--workflow <id>` (first trigger + nodes, multi-trigger warning)
  or takes inline `--trigger`/`--nodes` JSON or a verbatim `--stdin` body.
  There is no update — the server exposes none.
- **`pipelite docs`** — fetches the server's OpenAPI 3.1 spec from the
  public route (a keyless client: the API key is never sent);
  `--save <file>` creates parent directories and refuses overwrites unless
  `--force`.
- **Notes** — `notes list/add/edit/delete` on deals, organizations, people,
  and activities, with body-source precedence (`--body`/`@file`/`@-`,
  `--stdin`, or a TTY prompt — exactly one). No single-note GET (the server
  has none); table output truncates, JSON keeps full text.
- **Webhooks** — CRUD with the signing secret shown exactly once at create
  (never on list/get), 13-event vocabulary validated before any request,
  https-only URLs, and verbatim `--stdin` bodies. Another user's webhook
  403s even with an admin key.
- **Trash** — `trash list` (dual type vocabulary: 9 singular/plural aliases
  normalized to the 4 plural URL tabs), `trash restore` (no confirmation),
  and admin-only `trash purge` (per-record DELETEs with an
  "N permanently destroyed, M failed" summary; refuses non-TTY without
  `--force` at exit 2 before any request).
- **Audit log viewer** — `audit list` (admin key required) with four
  verbatim filters (`--entity-type`, `--entity-id`, `--actor-kind`,
  `--workflow-run-id`), client-side `--limit` clamping to the server's
  1..=100 range, and per-field `{from, to}` change payloads via
  `--format json`.
- **Custom-field definitions + typed writing** — `custom-fields
  list/get/create/update/delete` (the type source for `--custom-field`),
  with typed values resolved from cached definitions (cross-referenced
  above in Changed/Fixed — that data-correctness fix is the headline of
  this release).
- RFC 7807 error parsing: API errors now surface the server's real reason
  (joined `errors[]` detail) instead of generic messages.
- `403 Forbidden` is distinguished from `401 Unauthorized`, with
  per-surface permission hints.
- `409 Conflict` on an inactive workflow trigger names the activation path.

### Fixed

- `stages list` works without `--pipeline` (single unfiltered call); rows
  are distinguishable via the `pipeline_id` column.
- `--expand` payloads (e.g. `owner`, `organization`) survive to output:
  always in `--format json`, and in table/csv/plain via `--fields <key>`.
