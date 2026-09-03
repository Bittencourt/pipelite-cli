# Phase 10: Notes - Research

**Researched:** 2026-09-03
**Domain:** Parent-scoped sub-resource CRUD (notes on deals/orgs/people/activities) — Rust CLI + Next.js CRM server contract
**Confidence:** HIGH (all server facts verified directly against server source; all CLI facts verified against repo source)

## Summary

Notes is the first parent-scoped sub-resource in the CLI. The server exposes exactly four collection routes (`GET`/`POST /api/v1/{deals|organizations|people|activities}/{id}/notes`) and one item route (`PATCH`/`DELETE /api/v1/notes/{noteId}`) with **no single-note GET** — every handler lives in one shared module (`src/lib/api/notes-collection.ts`), so the four parents are byte-identical apart from a compile-time `entityType` literal. Writes accept `{content}` only (trim → 1–200,000 chars); `PATCH`/`DELETE` are gated **author-or-admin** (403) while `GET`/`POST` never 403. Delete is a **soft delete** returning 204. The `notes` key is **already present** in the CLI's `forbidden_hint` table (Phase 8 pre-registered it at `src/api/mod.rs:96`) — Phase 10 only needs to pass `"notes"` as the surface string.

The CLI work is a faithful application of the Phase 9 `templates` top-level-group pattern: `cli/notes.rs` (subcommand enum + per-subcommand args), `commands/notes/` (list/add/edit/delete handlers), 4 client methods in `api/mod.rs`, a `Note`/`NoteCreate`/`NoteUpdate` model triplet, and a `notes_stub_test.rs` integration file. New UX surface: a body-input resolver (`--body` flag with `@file`/`@-` forms > `--stdin` > prompt) with pre-HTTP rejection of multiple explicit sources. **No new crates are needed** — the entire phase uses existing dependencies.

Two CONTEXT conditions resolved by this research: (1) the server page cap is **100, not 1000** (`MAX_PAGE_SIZE = 100`, `src/lib/api/pagination.ts:4`) → **no `--all` flag**; (2) notes do **not** support `--expand` (no expand logic anywhere in the notes routes) → **no `expanded` flatten map** on the model.

**Primary recommendation:** One plan is feasible (surface end-to-end, ~8 files + 1 test file): CLI group + models + 4 client methods + body resolver + list/add/edit/delete handlers + hidden `get` + non-capable-parent rejection + docs/after_help, then `notes_stub_test.rs`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Grammar**: `pipelite notes <list|add|edit|delete> <entity-type> <parent-id> ...` — top-level command group, entity-type ∈ {deals, orgs, people, activities} (plural CLI names).
- **Body input precedence** (add/edit): `--body` flag > `@file` path > `--stdin` > interactive prompt (TTY only; `--no-input` → MissingInput exit 2). Multiple explicit sources → pre-HTTP InvalidInput exit 2.
- **`@file` semantics**: `--body @path/to.md` reads the file; missing/unreadable → InvalidInput exit 2 + hint; `@-` reads stdin (equivalent to `--stdin`).
- **No client-side body length cap** — server validates; 422 errors[] renders via Phase 8 layer.
- **List default columns**: id, created_at, body truncated (~80 chars); full text via `--json`/plain; `--json` documented as the view-before-edit path.
- **List ordering/pagination**: server order + `--limit/--offset` passthrough; no `--all` unless research shows a 1000-cap like other lists. *(Research: cap is 100/page — no `--all`.)*
- **Edit flow**: `notes edit <type> <parent-id> <note-id> --body ...`; prompt text explains "the server offers no single-note GET — run `notes list <type> <id> --json` to view existing content first".
- **Edit confirmation**: none; `--dry-run` previews the PUT (PATCH).
- **Delete**: confirm_destructive flow (dry-run preview → TTY prompt → `--force` bypass; non-TTY without `--force` → exit 1 Validation); single delete (by note ID, one at a time).
- **Non-capable parents** (`notes list pipelines <id>`): pre-HTTP InvalidInput exit 2 — "notes are only available on deals, orgs, people, activities" + valid types listed; zero HTTP.
- **`notes get` attempt**: hidden subcommand → exit 2 with hint "the server has no single-note GET — use `notes list <type> <id> --json`".
- **403 on foreign notes**: Forbidden via Phase 8 per-surface hint — "deleting a note requires its author or an admin" (register a notes surface key in forbidden_hint). *(Research: key already registered.)*
- **Plan structure**: 2 plans max — planner decides with real file counts (single plan acceptable if small).
- **No cache for notes** — direct reads, no invalidation keys, no completion candidates.
- **Note model**: follow the server serializer exactly; `#[serde(flatten)] expanded` only if notes support `--expand`. *(Research: they don't.)*
- **Docs**: update `docs/api-reference.md` + after_help examples in the same phase.

### the agent's Discretion
- Exact wording of hints/prompts beyond the locked strings above.
- Table rendering mechanics for the truncated body column.
- Test file organization beyond the stub-server convention.

### Deferred Ideas (OUT OF SCOPE)
- Note search/filtering by author or date.
- Bulk note operations.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| NOTE-01 | List notes on a deal, organization, person, or activity | `GET /api/v1/{route}/{id}/notes` contract (§API-1); newest-first order; `--limit/--offset` passthrough (cap 100); serializer fields (§Serializer) |
| NOTE-02 | Add a note via flag, `@file`, stdin, or interactive prompt | `POST .../notes` `{content}` contract (§API-2); body-resolver pattern (§Pattern: body input resolution); `@`-precedent in `workflows/trigger.rs:61-82` |
| NOTE-03 | Edit a note by ID (no single-note GET — document list --json path) | `PATCH /api/v1/notes/{noteId}` contract (§API-3); hidden-`get` rejection pattern (§Pattern: hidden variant) |
| NOTE-04 | Delete a note by ID with confirmation and `--force` bypass | `DELETE /api/v1/notes/{noteId}` → 204 soft delete (§API-4); delete contract in `commands/templates/delete.rs:47-94` |
| SC-5 | Non-capable parents and `notes get` rejected with actionable hints | Pre-HTTP entity-type validation (§Pattern: non-capable parents); hidden `get` variant (§Pattern: hidden variant) |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Body input resolution (flag/@file/stdin/prompt) | CLI (client) | — | Pure input plumbing; zero HTTP; exit-2 rejections must fire pre-connection |
| Entity-type validation (4 capable types) | CLI (client) | — | Locked: pre-HTTP InvalidInput; server has no `pipelines/.../notes` route at all (404/HTML, not a clean error) |
| Parent existence check | Server | — | CLI cannot pre-check without an extra GET; server returns 404 "Deal not found" etc. |
| Content validation (trim, 1–200,000) | Server | — | Locked: no client-side cap; 422 renders via Phase 8 parse_rfc7807 |
| Author-or-admin authorization | Server | — | 403 → `CliError::Forbidden` with `forbidden_hint("notes")` (already registered) |
| Pagination (`--limit/--offset`) | CLI → Server | — | Pass-through query params; server clamps (limit [1,100], offset [0,1e6]) |
| Rendering (table/json/csv/plain) | CLI (client) | — | Existing `output::render_list` / `render_single` |
| Soft-delete semantics | Server | — | CLI just handles 204 and 404-on-redelete |

## Server API Contract (verified against `/home/pedro/programming/pipelite`)

Shared handler: `src/lib/api/notes-collection.ts` (`noteCollectionHandlers`). Item handler: `src/app/api/v1/notes/[noteId]/route.ts`. The four route files own only the `entityType` literal:

| CLI type | Route segment | Server entityType (response `entity_type`) | 404 label |
|----------|---------------|---------------------------------------------|-----------|
| `deals` | `/api/v1/deals/{id}/notes` | `deal` | "Deal not found" (`deals/[id]/notes/route.ts:13-14`) |
| `orgs` | `/api/v1/organizations/{id}/notes` | `organization` | "Organization not found" (`organizations/[id]/notes/route.ts:13-14`) |
| `people` | `/api/v1/people/{id}/notes` | `person` | "Person not found" (`people/[id]/notes/route.ts:13-14`) |
| `activities` | `/api/v1/activities/{id}/notes` | `activity` | "Activity not found" (`activities/[id]/notes/route.ts:13-14`) |

**⚠ `entity_type` is SINGULAR in responses** — CLI positional is plural, response payload is singular. A `Note.entity_type` typed as a serde enum needs `deal|organization|person|activity` values (or plain `String`).

### API-1: List — `GET /api/v1/{segment}/{parent_id}/notes`

- Query params: `offset` (default 0), `limit` (default 50). Server clamps: limit → [1, 100], offset → [0, 1,000,000]; unparseable falls back to default (`src/lib/api/pagination.ts:3-4, 40, 55-71`). **No 1000-cap → no `--all` flag.**
- Check order: parse pagination → parent existence (`id` + `deletedAt IS NULL`; soft-deleted parent == missing parent) → query (`notes-collection.ts:88-92, 64-72`).
- Filter: `entityType = literal AND entityId = id AND deletedAt IS NULL` (`notes-collection.ts:95-99`).
- **Order: `createdAt DESC, id DESC` — newest first**, id tiebreaker for same-millisecond rows (`notes-collection.ts:108`). Not configurable; no sort params.
- Success 200: `{data: Note[], meta: {total, offset, limit}}` + `X-Total-Count` header (`response.ts:20-28`).
- Parent missing/soft-deleted → 404 `{type: "https://api.pipelite.app/errors/ENTITY_NOT_FOUND", title: "Not Found", status: 404, detail: "<Label> not found"}` (`errors.ts:57-58`).
- **No ownership scoping on list** — `parentExists` checks only id + not-deleted; any valid API key reads notes on any live parent. CLI consequence: nothing (server concern).
- **No `--expand`**: no expand parsing anywhere in `notes-collection.ts` (grep-verified absent).
- No auth-free path: bad key → 401 via `withApiAuth`.

### API-2: Add — `POST /api/v1/{segment}/{parent_id}/notes`

- Request body: `{"content": "..."}` — nothing else is read (`notes-collection.ts:13` reuses `updateNoteSchema = z.object({content})`, `src/lib/mutations/notes.ts:47-49`).
- Validation (zod, `notes.ts:20, 28-32`): `.trim()` then `.min(1)` then `.max(200000)`. Consequences:
  - Whitespace-only body → 422, `errors: [{field: "content", code: "too_small", message: "Note content is required"}]`.
  - Internal line breaks **survive** (trim strips edges only).
  - >200,000 chars → 422 "Note must be 200000 characters or less".
  - Unknown extra JSON keys are silently stripped (non-strict zod object) — CLI sends `{content}` only anyway.
- `authorId` is **always the API key's user** (`context.userId`, `notes-collection.ts:161`); a client-supplied authorId would be ignored/stripped (anti-forgery, T-35-28). `source` forced `"user"` (`notes.ts:142`).
- Check order: body parse → schema → parent existence (`notes-collection.ts:128-162`). Invalid JSON → 422 `errors: [{field: "body", code: "invalid_json", message: "Invalid JSON body"}]` (`notes-collection.ts:133-140`).
- Parent missing/soft-deleted → 404 `<Label> not found` (also post-race, `notes-collection.ts:153-155, 167-169`).
- Success **201**: `{data: Note}` (`createdResponse`, `response.ts:46`).
- No CRM event is emitted for note writes (`notes.ts:109-118`) — notes never trigger workflows.

### API-3: Edit — `PATCH /api/v1/notes/{noteId}`

- **No GET on single note** — the route file exports only PATCH and DELETE (`notes/[noteId]/route.ts`).
- **Authorization runs BEFORE the body is read** (`route.ts:64-78`): `findNoteById` (filters soft-deleted; missing and soft-deleted are the identical 404 — no existence oracle, T-35-10) → `resolveActorRole` → `isAuthorOrAdmin` (`route.ts:43-62`).
- Body: `{"content": "..."}`, same zod rule as POST. Invalid JSON or schema violation → 422 (shape as API-2).
- Success 200: `{data: Note}` (`singleResponse`, `route.ts:114`).
- Sets `content` + `updatedAt` only; `createdAt` never written (`notes.ts:183-190, 162-166`).
- Concurrent soft-delete race → 404 "Note not found" (`route.ts:25, 107-109`).
- Note IDs are server-generated UUIDs (`db/schema/notes.ts:9`) — CLI never constructs them.

### API-4: Delete — `DELETE /api/v1/notes/{noteId}`

- **Soft delete only** — sets `deletedAt` (+`updatedAt`); the route never issues a SQL DELETE (`route.ts:122-128`; `notes.ts:213-241`).
- Same author-or-admin gate BEFORE (`route.ts:134-137`).
- Success **204 No Content** (`route.ts:150`) — handled by existing `handle_delete_response`.
- Missing/soft-deleted → 404 "Note not found".
- Idempotency nuance: *concurrent* double-delete → both 204 (lost race counts as success, `notes.ts:234-237`); *sequential* re-delete after completion → **404** (row already filtered). CLI should treat 404-on-redelete as the normal failure path — no special casing.

## Note Serializer Field Inventory

`src/lib/api/serializers/note.ts:13-22` — the single serializer for ALL five route files (`SerializedNote`):

| Field | Type (TS) | Rust type recommendation | Notes |
|-------|-----------|--------------------------|-------|
| `id` | string | `String` | UUID |
| `entity_type` | `"deal" \| "organization" \| "person" \| "activity"` | `String` (or serde enum, singular variants) | SINGULAR |
| `entity_id` | string | `String` | parent id |
| `content` | string | `String` | full body — the CLI input flag is `--body`, the wire field is `content` |
| `author_id` | string \| null | `Option<String>` | null for migrated notes / deleted author |
| `source` | `"user" \| "migration"` | `String` or small serde enum | never settable via API |
| `created_at` | string \| null | `Option<String>` (serde default) | typed nullable in serializer; practically always present |
| `updated_at` | string \| null | `Option<String>` (serde default) | same |
| `deleted_at` | **absent** | — | deliberately omitted — soft-delete oracle prevention (T-35-06) |

No `expanded` map — notes support no expand (CONTEXT conditional resolved). Dates are ISO-8601 UTC strings.

## Author-or-Admin 403 Semantics

**Applies to: `PATCH` and `DELETE` ONLY.** `GET` and `POST` never return 403 (401 for bad keys only).

Predicate (`src/lib/notes/authorize.ts:44-60`):
- Actor role re-read from DB per request (API-key auth context carries no role, T-35-24); unresolvable/soft-deleted actor → 403 fail-closed (`route.ts:52-55`).
- `admin` role → allowed for any live note.
- Otherwise allowed iff `note.author_id !== null && note.author_id === actor user id`.
- **`author_id = null` notes (migration-source or author deleted) are admin-only** — a member key gets 403 even though "nobody owns it" (`authorize.ts:40-43, 57-59`).

Error body (identical for both operations, `errors.ts:49-55`):
```json
{"type": "https://api.pipelite.app/errors/FORBIDDEN", "title": "Forbidden", "status": 403, "detail": "You don't have access to this resource"}
```

CLI handling: `handle_response` / `handle_delete_response` with `surface = "notes"` → `CliError::Forbidden` with hint from the table at `src/api/mod.rs:92-100` — **the `"notes"` entry already exists (line 96): "You can only modify your own notes (or use an admin key)."** No registration work; just pass the key. (404 detail "Note not found" flows through `parse_rfc7807` as-is.)

## Pagination & Ordering Facts

- **Order**: `createdAt DESC, id DESC` — newest note first; hard-coded, no sort param (`notes-collection.ts:106-108`).
- **Page cap: 100** (`MAX_PAGE_SIZE`, `pagination.ts:4`) — default 50. Server clamps silently (limit >100 → 100; limit 0/negative → 1; offset >1e6 → clamped, not reset). → **No `--all` flag**; a user paging through thousands of notes iterates `--offset`.
- Envelope: `{data, meta: {total, offset, limit}}` — `meta.total` is the *live* (non-deleted) count for that parent.
- Empty result: `{data: [], meta: {total: 0, ...}}` — render existing empty-table behavior; unlike `workflows runs`, there are no hidden rows (soft-deleted always filtered server-side), so **no probe request** is needed; at most a simple stderr "no notes" hint suppressed by `--quiet` (pattern: `workflows/runs/list.rs:44-69`).

## Recommended Approach per Requirement

### NOTE-01 — `notes list <type> <parent-id>`
- CLI args: `entity_type` (String, validated in handler), `parent_id`, `--limit` (default 50), `--offset` (default 0), `--fields`, plus global `--format`.
- Client: `list_notes(route_segment, parent_id, limit, offset) -> ApiListResponse<Note>` — GET with `.query(&[("limit", …), ("offset", …)])`, `handle_response(resp, "notes")` (pattern: `list_workflow_templates`, `api/mod.rs:929-941`).
- Table: `notes_table_config()` → `default_columns: ["id", "created_at", "content"]`. When converting `Note` → `Value` for table/csv, flatten newlines (`\n` → space) and truncate `content` to ~80 chars via `truncate_with_ellipsis` (`output/format.rs:96-110`, currently `#[allow(dead_code)]` — remove the attribute when used). `--format json`/`plain` must receive the FULL content (truncate only in the table-value builder, or gate on format).
- 404 from server (missing/soft-deleted parent) → existing NotFound error; hint is generic — acceptable, detail carries "Deal not found".

### NOTE-02 — `notes add <type> <parent-id> [--body <val>|--stdin]`
- Body resolver (new small helper, e.g. `commands/notes/body.rs` or inline in `add.rs`/`edit.rs` shared):
  1. Explicit-source detection: `--body` present XOR `--stdin` present. Both → `CliError::InvalidInput` (exit 2) pre-HTTP, hint listing the two sources.
  2. `--body` starting with `@`: `@-` → read stdin to end; otherwise `fs::read_to_string(path)` — failure → `CliError::InvalidInput` exit 2 + "Check that the file path is correct…" hint (precedent: `workflows/trigger.rs:61-82`, which uses `Validation`; CONTEXT locks **InvalidInput** for notes — follow CONTEXT).
  3. No explicit source: TTY && !`--no-input` → `dialoguer::Input` prompt (label explains existing content is not shown for edit: locked wording "the server offers no single-note GET — run `notes list <type> <id> --json` to view existing content first"); else → `CliError::MissingInput` (exit 2) via the `check_missing` pattern (`prompt.rs:145-160`).
- Client: `create_note(route_segment, parent_id, &NoteCreate {content}) -> Note` — POST `.json(body)`, unwrap `ApiSingleResponse` (201 `{data}`), `handle_response(resp, "notes")` (pattern: `create_workflow_template`, `api/mod.rs:961-971`).
- `--dry-run` → `render_dry_run("POST", url, &json!({"content": …}), …)` BEFORE any prompt/network (precedent: `templates/create.rs` dry-run-first ordering).
- Success output: match templates create style — id confirmation line under `--quiet` suppression; `--format json` prints the created note.

### NOTE-03 — `notes edit <type> <parent-id> <note-id> [--body <val>|--stdin]`
- Same body resolver; same pre-HTTP rules. **No confirmation.**
- Client: `update_note(note_id, &NoteUpdate {content}) -> Note` — `self.client.patch(&url)` to `/api/v1/notes/{note_id}`, unwrap `ApiSingleResponse`, `handle_response(resp, "notes")`.
- `--dry-run` → `render_dry_run("PATCH", url, body, …)`.
- Grammar note: `<type> <parent-id>` are accepted per the locked grammar but the PATCH request uses only the note ID. Do not use them in the URL. (Optionally a post-response sanity warning if `entity_id` mismatches — planner's call; simplest is ignore.)
- Hidden `notes get` subcommand: parse-then-error (`commands/templates/mod.rs:23-28` is the exact pattern): `CliError::InvalidInput` exit 2, hint "the server has no single-note GET — use `notes list <type> <id> --json`".

### NOTE-04 — `notes delete <type> <parent-id> <note-id> [--force]`
- Copy `single_delete` from `commands/templates/delete.rs:47-94` verbatim, adjusted:
  - Dry-run FIRST: `render_dry_run_delete("note", note_id, url, …)` — never prompts.
  - TTY && !force → `dialoguer::Confirm` "Delete note {id}?"; non-TTY && !force → `CliError::Validation` (exit 1) "Refusing to delete without confirmation in non-interactive mode." + `--force` hint.
  - Client: `delete_note(note_id)` — DELETE `/api/v1/notes/{note_id}`, `handle_delete_response(resp, "notes")` (204 → Ok).
  - No cache invalidation (no notes cache).
- No batch: note deletes are one at a time (locked). `type`/`parent-id` accepted per grammar, unused by the request.

### SC-5 — Rejection surfaces
- Non-capable parents: validate `entity_type` string in the handler against {deals, orgs, people, activities} → unknown → `CliError::InvalidInput` exit 2, detail/hint "notes are only available on deals, orgs, people, activities" + list valid types. **Must fire before any HTTP** (assert stub-server request counter == 0 in tests). Do NOT use `clap::ValueEnum` for the positional — clap's own rejection would bypass the locked message.
- Hidden `notes get`: as NOTE-03 above.

## Architecture Patterns

### System Architecture Diagram

```
                         ┌──────────────────────────────────────────────┐
                         │                pipelite CLI                  │
                         │                                              │
 user input              │  cli/notes.rs        commands/notes/         │
 ───────────────►        │  (grammar +          list.rs ──► GET  /api/v1/{segment}/{pid}/notes
 (flags, TTY,            │   after_help)        add.rs  ──► POST /api/v1/{segment}/{pid}/notes   ──►  PipeliteServer
  @file, stdin)          │                      edit.rs ──► PATCH /api/v1/notes/{nid}          (Next.js)
                         │  body resolver ─┐    delete.rs──► DELETE /api/v1/notes/{nid}          │
                         │  (flag>@>@-/>   │            │                                       │
                         │   stdin>prompt) ┘            │                                       ▼
                         │        │                     │                          ┌────────────────────┐
 pre-HTTP rejections     │        ▼                     │                          │ notes-collection.ts│
 (exit 2, zero HTTP):    │  multiple sources            │                          │  GET: parent exists│
  • >1 explicit source   │  unreadable @file            │                          │   → 404 "<L> not   │
  • bad entity type      │  unknown entity type         │                          │   → list, new-first│
  • missing body,        │  (all validated in           │                          │  POST: {content}   │
    non-TTY/--no-input   │   commands/notes/)           │                          │   → 201 {data}     │
                         │                              │                          ├────────────────────┤
                         │  src/api/mod.rs client       │                          │ notes/[noteId]/    │
                         │  handle_response/_delete:    │                          │  author-or-admin   │
                         │   403 → Forbidden +          │◄────────────────────────►│   403 / 404 /      │
                         │   forbidden_hint("notes")    │   RFC 7807 problem JSON  │   422 / 200/201/204│
                         └──────────────────────────────────────────────┘          └────────────────────┘
```

Primary use case trace: `notes add deals 123 --body @note.md` → entity type validated → body read from file → POST `/api/v1/deals/123/notes` with `{"content": …}` → server validates + inserts (author = key owner) → 201 `{data: Note}` → CLI prints created id / JSON.

### Recommended Project Structure

```
src/
├── cli/notes.rs            # NotesCommands enum: List/Add/Edit/Delete/Get(hidden); arg structs
├── commands/notes/
│   ├── mod.rs              # dispatch + hidden Get rejection + entity-type validation helper
│   ├── list.rs             # NOTE-01
│   ├── add.rs              # NOTE-02
│   ├── edit.rs             # NOTE-03
│   └── delete.rs           # NOTE-04
├── api/mod.rs              # + list_notes / create_note / update_note / delete_note
└── api/models.rs           # + Note, NoteCreate, NoteUpdate, notes_table_config()
tests/
└── notes_stub_test.rs      # stub-server integration tests
docs/
└── api-reference.md        # + "## pipelite notes" section
```

Wiring: `cli/mod.rs` (`pub mod notes;` + `Commands::Notes(NotesCommands)` after Templates, `cli/mod.rs:152-157` pattern), `main.rs:98-101` dispatch pattern.

### Pattern 1: Top-level command group (templates analog)
**What:** `#[derive(Subcommand)]` enum with per-variant `after_help` examples + hidden `Get` variant; group-level after_help on the `Commands` variant (`cli/mod.rs:155` shows the multi-line format).
**When to use:** exactly here — `notes` is a top-level group per locked decision.
**Example:** `src/cli/templates.rs:37-70` (variants + `#[command(hide = true)]`), `src/commands/templates/mod.rs:17-29` (dispatch + parse-then-error rejection).

### Pattern 2: Body input resolution (new, but precedented)
**What:** one resolver returning `Result<String>` used by add + edit; enforces single-explicit-source, `@file`/`@-` reads, prompt fallback, MissingInput.
**Example (@file read precedent):**
```rust
// Adapted from src/commands/workflows/trigger.rs:61-68
let value = if let Some(path) = s.strip_prefix('@') {
    if path == "-" { std::io::read_to_string(std::io::stdin())? }
    else { fs::read_to_string(path).map_err(|e| CliError::InvalidInput {
        detail: format!("Failed to read file '{path}': {e}"),
        hint: "Check that the file path is correct and the file exists.".into(),
    })? }
} else { s.clone() };
```

### Pattern 3: Client method + surface key
```rust
// Pattern: src/api/mod.rs:998-1003 (delete_workflow_template)
pub async fn delete_note(&self, note_id: &str) -> Result<()> {
    let url = format!("{}/api/v1/notes/{}", self.base_url, note_id);
    let request = self.client.delete(&url);
    let response = self.send_with_retry(request).await?;
    self.handle_delete_response(response, "notes").await
}
```

### Anti-Patterns to Avoid
- **Nesting notes under entities** (`deals notes …`) — locked decision says top-level group; also ×4 CLI surface bloat.
- **`clap::ValueEnum` on entity-type positional** — kills the locked custom rejection message; validate in the handler.
- **Client-side content trimming/length caps** — locked: server owns validation; pre-trimming would silently change content (server trims anyway, and 422 messages are better).
- **Caching note IDs for completions** — locked: no cache; append-heavy data goes stale instantly.
- **`--all` pagination flag** — cap is 100, but the decision was conditional on a 1000-cap; do not add speculative flags.
- **Prompting during `--dry-run`** — dry-run must precede prompt AND network everywhere (delete.rs:48-59 ordering).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| RFC 7807 error rendering | custom 422 parser | `parse_rfc7807` + `handle_response` (`api/mod.rs:299-368`) | handles errors[] join + detail/title fallbacks, already tested |
| 403 hint text | per-call strings | `forbidden_hint("notes")` (`api/mod.rs:96`) | surface table already has the notes entry |
| 204 DELETE handling | manual status check | `handle_delete_response` | maps 401/403/404/409/429 identically to all other deletes |
| 429 retry | retry loop | `send_with_retry` | one retry honoring Retry-After, already wired into every method |
| Table cell truncation | ad-hoc slicing | `truncate_with_ellipsis` (`output/format.rs:96`) | char-safe (UTF-8), tested |
| Confirm/dry-run delete flow | new flow | copy `templates/delete.rs::single_delete` | exact locked contract (exit codes, ordering, wording) |
| List rendering + meta footer | custom table code | `output::render_list(items, format, columns, fields, color, Some(&meta))` | 4 formats + fields selection for free |

**Key insight:** the entire phase is composition of Phase 7/8/9 primitives — the only genuinely new code is the body resolver and the entity-type validator, both ~30 lines.

## Common Pitfalls

### Pitfall 1: Plural/singular entity-type mismatch
**What goes wrong:** typing `Note.entity_type` against the CLI's plural names (`deals`) breaks deserialization of every list response.
**Why:** the server serializer emits the DB discriminator — singular (`serializers/note.ts:15`, `EntityType`).
**How to avoid:** model uses singular strings; CLI positional maps plural → route segment only.
**Warning signs:** deserialization error "unknown variant `deals`" on first stub test.

### Pitfall 2: `content` vs `body` field-name confusion
**What goes wrong:** POSTing `{"body": …}` → server 422 "Note content is required" (zod strips unknown key, content missing).
**Why:** CONTEXT/UX language says "body"; the wire field is `content`.
**How to avoid:** `NoteCreate { content }`; `--body` is only the flag name. Assert the captured stub body equals `{"content":"…"}`.
**Warning signs:** 422 too_small on add tests.

### Pitfall 3: Multi-line content breaking table rows
**What goes wrong:** a note with newlines renders as multiple garbage table rows.
**Why:** table renderer writes cells verbatim; content is free text with line breaks (server preserves them by design).
**How to avoid:** in the table-value builder only: replace `\n`/`\r` with a space (or `\\n`), then truncate to ~80. JSON/plain formats keep raw text.
**Warning signs:** manual `notes list` on a multi-line note; add a stub test with `"line1\nline2"`.

### Pitfall 4: Forgetting authorization precedes body on PATCH/DELETE
**What goes wrong:** tests expecting 422 for a bad body + foreign note get 403 instead, and a plan might assert the wrong order.
**Why:** server checks author-or-admin BEFORE parsing the body (`route.ts:64-78`).
**How to avoid:** when scripting stub sequences, remember a foreign-note edit returns 403 regardless of body validity.

### Pitfall 5: Sequential re-delete expects 204 (idempotency misreading)
**What goes wrong:** test does delete → delete and expects both 204.
**Why:** server idempotency covers only the concurrent race; a completed soft delete makes the next DELETE a 404 ("Note not found") — soft-deleted == missing, deliberately (`notes.ts:94-104`).
**How to avoid:** stub scripts: delete → 204, second delete → 404 test asserts NotFound error.

### Pitfall 6: Using clap to reject `pipelines` as parent
**What goes wrong:** clap's default "invalid value" error bypasses the locked hint text and its exit code/message drift.
**Why:** locked decision requires the specific "notes are only available on…" message listing valid types.
**How to avoid:** `entity_type: String` positional; validate in handler before touching the client; test asserts zero HTTP (stub counter == 0).

### Pitfall 7: Missing-input path silently reading stdin twice
**What goes wrong:** `--body @-` and `--stdin` both read stdin — if the multiple-sources guard misses a combination, one read consumes the other's data.
**Why:** both sources are stdin-backed.
**How to avoid:** the explicit-source XOR check runs before ANY read; treat `--body @-` + `--stdin` as two explicit sources → exit 2.

### Pitfall 8: Help pages advertising what doesn't exist
**What goes wrong:** `notes --help` implying a `get` subcommand or `--all` flag breaks the Phase 9 truthfulness test convention (`help_examples_test.rs:106-120` asserts templates help does NOT advertise update).
**Why:** hidden variants must stay `#[command(hide = true)]`; group after_help should state the no-single-GET fact.
**Warning signs:** CI help test failures; add an analogous `notes_help` assertion.

## Code Examples

### Envelope shapes (stub-server literals)
```jsonc
// GET list 200 — Source: pipelite src/lib/api/response.ts:20-28
{"data": [{"id":"n1","entity_type":"deal","entity_id":"d1","content":"hello","author_id":"u1","source":"user","created_at":"2026-09-01T10:00:00.000Z","updated_at":"2026-09-01T10:00:00.000Z"}],
 "meta": {"total": 1, "offset": 0, "limit": 50}}
// + header: X-Total-Count: 1

// POST/PATCH 201/200 — Source: response.ts:37-46
{"data": { ...same shape... }}

// DELETE 204 — empty body
```

### Error bodies (RFC 7807)
```jsonc
// 404 parent — Source: errors.ts:57-58
{"type":"https://api.pipelite.app/errors/ENTITY_NOT_FOUND","title":"Not Found","status":404,"detail":"Deal not found"}
// 404 note — detail: "Note not found"
// 403 — Source: errors.ts:49-55
{"type":"https://api.pipelite.app/errors/FORBIDDEN","title":"Forbidden","status":403,"detail":"You don't have access to this resource"}
// 422 — Source: errors.ts:60-68 + notes-collection.ts:143-151
{"type":"https://api.pipelite.app/errors/VALIDATION_ERROR","title":"Validation Error","status":422,"detail":"Request validation failed",
 "errors":[{"field":"content","code":"too_small","message":"Note content is required"}]}
```

## State of the Art

Not applicable (no library churn — everything is in-repo Rust against a stable internal API). Server notes surface is post-Phase-50 and stable (soft-delete + author-or-admin invariants are documented invariants, not moving targets).

## Package Legitimacy Audit

**No new external packages are installed in this phase.** All work uses existing dependencies (`clap`, `reqwest`, `serde`, `serde_json`, `dialoguer`, `colored`, `anyhow`, `assert_cmd` — already in `Cargo.toml` and in production use). Nothing to audit; no slopcheck run required.

## Runtime State Inventory

Not a rename/refactor/migration phase — omitted per protocol. (Greenfield addition of a new command group; no stored data, live config, OS registrations, secret keys, or build artifacts carry a "notes" name today except the pre-registered `forbidden_hint("notes")` string, which is already correct.)

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo | build/test | ✓ | 1.94.0 | — |
| Existing dev-deps (assert_cmd, tokio, etc.) | stub tests | ✓ | in Cargo.toml lockfile | — |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** none.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | "Multiple explicit sources" means any two of {`--body` (incl. `@`-forms), `--stdin`} together → exit 2 (precedence chain then only describes the single-source fallback) | Recommended Approach NOTE-02 | Low — CONTEXT states the rejection explicitly; only the boundary case `--body @-` + `--stdin` is interpretation |
| A2 | `notes edit/delete` accept `<type> <parent-id>` per grammar but the request uses only the note ID (no transmission, no validation against response) | NOTE-03/04 | Low — server route provably takes only noteId; grammar is locked regardless |
| A3 | Interactive prompt may be single-line (`dialoguer::Input`) — multi-line bodies are the `@file`/`--stdin` paths' job | NOTE-02 | Low — UX nicety; locked precedence doesn't mandate multi-line prompt |
| A4 | `created_at`/`updated_at` modeled `Option<String>` despite practically always present | Serializer inventory | Very low — only affects hypothetical null rendering |

## Open Questions (RESOLVED)

1. **Should `notes list` print an empty-state stderr hint?**
   - What we know: runs list prints stderr hints suppressed by `--quiet`; templates list prints nothing special.
   - What's unclear: desired wording/whether notes needs one at all.
   - Recommendation: simple `"No notes on this <type> yet."` stderr hint, `--quiet`-suppressed, exit 0 — matches runs-list convention without the probe (no hidden rows exist for notes).

2. **Post-edit/delete parent-id mismatch warning?**
   - What we know: PATCH response carries `entity_id`; grammar collects parent-id that the request ignores.
   - What's unclear: whether to warn when the note's `entity_id` ≠ given parent-id (typo'd parent, wrong note).
   - Recommendation: skip (defer); note IDs are UUIDs looked up via `notes list --json`, so mismatch implies user error already surfaced by 404.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test + assert_cmd (integration, `tests/`) |
| Config file | none needed (convention-based) |
| Quick run command | `cargo test --test notes_stub_test` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| NOTE-01 | list renders id/created_at/content; multi-line+truncation in table; full text in json | integration | `cargo test --test notes_stub_test` | ❌ Wave 0 (`tests/notes_stub_test.rs`) |
| NOTE-01 | `--limit/--offset` reach the wire as query params | integration (head capture) | same | ❌ Wave 0 |
| NOTE-01 | parent 404 → NotFound exit 1, detail "<Label> not found" | integration | same | ❌ Wave 0 |
| NOTE-01 | empty list → exit 0, quiet-suppressed hint | integration | same | ❌ Wave 0 |
| NOTE-02 | `--body` flag → POST body exactly `{"content":"…"}` | integration (body capture) | same | ❌ Wave 0 |
| NOTE-02 | `--body @file` reads file; missing file → InvalidInput exit 2 pre-HTTP (counter==0) | integration | same | ❌ Wave 0 |
| NOTE-02 | `--body @-` and `--stdin` read stdin | integration (`write_stdin`) | same | ❌ Wave 0 |
| NOTE-02 | two explicit sources → InvalidInput exit 2, zero HTTP | integration | same | ❌ Wave 0 |
| NOTE-02 | `--no-input` without source → MissingInput exit 2, zero HTTP | integration | same | ❌ Wave 0 |
| NOTE-02 | server 422 errors[] renders joined message | integration | same | ❌ Wave 0 |
| NOTE-02 | `--dry-run` previews POST, counter==0, never prompts | integration | same | ❌ Wave 0 |
| NOTE-03 | edit PATCHes `/api/v1/notes/{id}` with `{"content": …}`; renders result | integration | same | ❌ Wave 0 |
| NOTE-03 | 403 → Forbidden exit 1 with notes hint text | integration | same | ❌ Wave 0 |
| NOTE-03 | hidden `notes get` → InvalidInput exit 2 + locked hint, zero HTTP; not in help | integration + help test | same | ❌ Wave 0 |
| NOTE-04 | non-TTY without `--force` → Validation exit 1 refusal; `--force` sends DELETE → 204 handled | integration | same | ❌ Wave 0 |
| NOTE-04 | `--dry-run` previews delete, never prompts | integration | same | ❌ Wave 0 |
| NOTE-04 | re-delete after 204 → 404 NotFound | integration | same | ❌ Wave 0 |
| SC-5 | `notes list pipelines <id>` → InvalidInput exit 2, locked message, zero HTTP | integration | same | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --test notes_stub_test`
- **Per wave merge:** `cargo test`
- **Phase gate:** full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/notes_stub_test.rs` — all mappings above (helpers exist: `tests/common/mod.rs::cmd_with_server`, `cmd`, `spawn_head_capturing_stub_server` — supports GET-before-POST scripts and body capture out of the box)
- [ ] Model unit tests can live in `src/api/models.rs` `#[cfg(test)]` (note envelope deserialization, plural/singular) — no new framework config needed

## Security Domain

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes (transport) | Bearer API key via existing client default headers |
| V3 Session Management | no | stateless API-key CLI |
| V4 Access Control | yes | server author-or-admin; CLI renders 403 with surface hint — never guesses permissions client-side |
| V5 Input Validation | yes | server zod validation; CLI adds only path-safety on `@file` reads (error on unreadable, no shell interpolation) |
| V6 Cryptography | no | no crypto in this phase |

### Known Threat Patterns for this stack
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Note content with control chars/newlines breaking table output | Tampering (display) | flatten newlines in table builder only |
| `@file` pointing at sensitive path / huge file | Information disclosure / DoS | error clearly on unreadable; no size cap by design (locked — server caps at 200k) |
| Secret leakage in dry-run JSON | — | body is user-supplied content only; no secrets involved in notes |

## Sources

### Primary (HIGH confidence — direct server-source verification)
- `/home/pedro/programming/pipelite/src/lib/api/notes-collection.ts` — list/add handlers, order, parent-exists, author forcing
- `/home/pedro/programming/pipelite/src/app/api/v1/notes/[noteId]/route.ts` — PATCH/DELETE, author-or-admin order, 204, NOT_FOUND constant
- `/home/pedro/programming/pipelite/src/lib/mutations/notes.ts` — content schema (trim/1–200k), soft delete, idempotency semantics
- `/home/pedro/programming/pipelite/src/lib/api/serializers/note.ts` — SerializedNote field inventory
- `/home/pedro/programming/pipelite/src/lib/notes/authorize.ts` — isAuthorOrAdmin predicate, null-author admin-only
- `/home/pedro/programming/pipelite/src/lib/api/pagination.ts`, `errors.ts`, `response.ts` — clamps, problem shapes, envelopes
- Route files ×4 (`deals|organizations|people|activities/[id]/notes/route.ts`) — entityType literals + labels

### Secondary (HIGH confidence — CLI repo verification)
- `src/api/mod.rs` (forbidden_hint table incl. pre-registered "notes"; handle_response/_delete; template client methods)
- `src/cli/templates.rs`, `src/commands/templates/*`, `src/cli/mod.rs`, `src/main.rs` — group pattern
- `src/commands/workflows/trigger.rs` (@file precedent), `src/commands/workflows/runs/list.rs` (empty-hint pattern)
- `src/output/format.rs` (truncate_with_ellipsis), `src/dry_run.rs`, `src/prompt.rs`, `src/error.rs` (exit codes)
- `tests/common/mod.rs`, `tests/templates_stub_test.rs`, `tests/help_examples_test.rs` — test conventions
- `.planning/research/SERVER-API-DIFF.md` — cross-checked; §A.1 consistent with source (one correction: it lists authorId "ignored" — precise mechanism is zod stripping + forced `context.userId`)

## Metadata

**Confidence breakdown:**
- API contract: HIGH — every endpoint read line-by-line from server source; no training-data claims
- CLI integration: HIGH — all patterns verified in-repo with file:line references
- Pitfalls: HIGH — derived from actual server invariants (trim, soft-delete 404s, auth-before-body) and CLI conventions

**Research date:** 2026-09-03
**Valid until:** 2026-10-03 (server notes surface is stable; re-verify only if server milestone touches `/api/v1` notes routes)
