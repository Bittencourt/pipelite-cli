# Server API Diff — New Features to Add to pipelite CLI

> Researched 2026-09-02 against `/home/pedro/programming/pipelite` (Next.js CRM, post-Phase-50).
> Server auth for all `/api/v1` routes: `Authorization: Bearer pk_live_...` (API key).
> Envelopes: list `{data: [...], meta: {total, offset, limit}}` + `X-Total-Count`; single `{data: {...}}`; delete `204`.
> Errors: RFC 7807 (`{type, title, status, detail, errors[]}`). Rate limit 500 req/60s per key (429 + Retry-After).
> Pagination: `offset` (default 0), `limit` (default 50, max 100).

## A. New entity surfaces (API-key usable → CLI-ready)

### 1. Notes
- `GET/POST /api/v1/{deals|organizations|people|activities}/{id}/notes`
- `PATCH/DELETE /api/v1/notes/{noteId}` (no GET on single note)
- Body: `{content}` only (trim, 1–200,000 chars). Author forced to API-key owner (authorId ignored).
- Note shape: `{id, entity_type, entity_id, content, author_id, source(user|migration), created_at, updated_at}`
- PATCH/DELETE gated author-or-admin. Only deals/orgs/people/activities support notes — NOT pipelines/stages/workflows.

### 2. Trash
- `GET /api/v1/trash?type=deals|people|organizations|activities&offset&limit` (offset capped ≤10,000; owner-or-admin scoping)
- Row: `{id, entity_type(singular), type(plural tab), name, secondary, deleted_at, linked_parents[], deleted_by{kind,...}}`
- `DELETE /api/v1/trash/{type}/{id}` — permanent purge, **admin-only** (member → 403). 204.
- `POST /api/v1/trash/{type}/{id}/restore` — owner-or-admin. 204.
- Path `type` uses PLURAL tabs (deals/people/organizations/activities); invalid → 422.
- Record must currently be in trash; live/unknown → 404.

### 3. Audit log (admin-only)
- `GET /api/v1/audit?entity_type&entity_id&actor_kind&workflow_run_id&offset&limit`
- `entity_type`: organization|person|deal|activity|import_session|export
- `actor_kind`: user|workflow_run|api_key|import|system
- Entry: `{id, entity_type, entity_id, action(created|updated|deleted|merged), changes{field:{from,to}}, actor_kind, actor_user_id, workflow_run_id, import_session_id, created_at}`

### 4. Webhooks
- `GET/POST /api/v1/webhooks`; `GET/PUT/DELETE /api/v1/webhooks/{id}` (PUT, not PATCH)
- Create body: `{url (must be https://), events[] (min 1)}`; server generates `secret` (returned ONLY on create); active forced true.
- Update body (all optional): `{url, events[], active}`; secret never updatable.
- Item: `{id, url, events[], active, created_at, updated_at}` (secret excluded after create).
- Foreign webhook → 403 (not 404 like workflows). Hard delete.
- Valid event names (validate client-side; server accepts anything): deal.created, deal.updated, deal.deleted, deal.stage_changed, person.created, person.updated, person.deleted, organization.created, organization.updated, organization.deleted, activity.created, activity.updated, activity.deleted.

### 5. Workflow runs (extends existing workflows command)
- `GET /api/v1/workflows/{id}/runs?offset&limit&status&dry_run=true`
  - status: pending|running|completed|failed|waiting (not validated server-side)
  - dry_run runs HIDDEN by default; `dry_run=true` opts in
- Run: `{id, workflow_id, status, trigger_data, error, depth, dry_run, current_node_id, started_at, completed_at, created_at}`
- `GET /api/v1/workflows/{id}/runs/{runId}` — adds `steps[]`: `{id, run_id, node_id, status(pending|running|completed|failed|skipped|waiting), input, output, error, resume_at, started_at, completed_at, created_at}`
- Foreign/missing workflow → 404 (never 403).

### 6. Workflow templates (global, no ownership)
- `GET/POST /api/v1/workflow-templates`; `GET/DELETE /api/v1/workflow-templates/{id}` (no update endpoint)
- Create body: `{name (1–200), trigger (object, required), description (≤2000), category (≤100), nodes[] (default [])}`
- Item: `{id, name, description, category, trigger, nodes[], created_at}`

### 7. Custom field definitions
- `GET /api/v1/custom-field-definitions?entity_type=organization|person|deal|activity&offset&limit` (includes soft-deleted!)
- `POST` body: `{name, entity_type, type, config?, required?, show_in_list?}`; position auto-assigned (max+10000)
- `GET/PUT/DELETE /api/v1/custom-field-definitions/{id}` — PUT: name/config/required/show_in_list/position only (entity_type+type immutable). DELETE = soft delete; 404 on already-deleted.
- `type` enum (10): text, number, date, boolean, single_select, multi_select, file, url, lookup, formula
- Config shapes: single/multi_select `{options[]}`; lookup `{targetEntity}`; formula `{expression, resultType}`; file `{maxFiles, maxSize}`
- Item: `{id, entity_type, name, type, config, required, position(float!), show_in_list, created_at, updated_at}`

### 8. API docs (public)
- `GET /api/v1/docs` — returns OpenAPI 3.1 spec as JSON (no auth). Candidate for `pipelite docs` or completion tooling.

## B. Session-only routes (NOT usable with API keys — skip or need server work)

- `GET /api/search?q=` — global search (orgs/people/deals, 5 each). Session cookie auth only → **cannot CLI it today**.
- `POST /api/upload`, `GET/DELETE /api/files/{entityId}/{fieldName}/{filename}` — file upload/download (10MB default). Session-only.
- `POST /api/custom-fields/save` — full-replacement blob write. Session-only. BUT custom-field values CAN be written via normal entity update (blob merge `{...existing, ...updates}`; formula keys server-computed/stripped).

## C. Custom fields end-to-end (existing entities)

- Values live in `custom_fields` jsonb blob on deals/orgs/people/activities; keys = field IDs. Returned on every serializer.
- Update MERGES blob (can overwrite keys, cannot delete via API).
- CLI gap: `--custom-field key=value` always stores STRING values ("4" not 4) — no type awareness (number/boolean/date/array).
- Need: `custom-fields` command group (definitions CRUD) + type-aware parsing or `--custom-field-json` escape hatch.

## D. Bugs / dead flags in existing CLI (server ignores these)

1. `people list --org/--owner` — server route reads NEITHER param → silently returns unfiltered data.
2. `orgs list --owner` — server ignores (scopes to key owner).
3. `workflows list --active` — server has no active filter → returns everything.
4. `workflows create --active` — create schema has no active; server always creates inactive.
5. Dead `--custom-field` flags on `pipelines create`/`stages create` (accepted then never parsed/sent; those entities have no custom fields).
6. `--expand` payloads (owner/organization/person/stage/type/deal/stages/pipeline) silently discarded by typed models — re-serialization drops unknown keys.
7. `Deal.position` typed `Option<i64>` but server emits float (parseFloat) — fractional positions would fail deserialization.
8. `stages list` requires `--pipeline`, but server supports omitting pipeline_id to list ALL stages across pipelines.
9. `activities list --done` is client-side only (server has no completed filter) — short pages possible.

## E. No server-side support (verified absent)

- No sort/search/date-range/custom-field filters on any v1 list endpoint (sorts hard-coded per route).
- No activity-types v1 route (types only via `?expand=type`).

## Suggested CLI milestone scope (priority order)

1. **Notes** command group (`pipelite notes list/add/edit/delete` on deals/orgs/people/activities)
2. **Workflow runs** (`workflows runs <id> [--status] [--dry-run]`, `workflows runs <id> <runId>` with steps)
3. **Webhooks** CRUD with event-name validation + secret show-once warning
4. **Trash** (`trash list [--type]`, `trash restore`, `trash purge` with admin/confirm warnings)
5. **Custom fields** definitions CRUD + type-aware `--custom-field` writing
6. **Workflow templates** (list/get/create/delete)
7. **Audit** (`pipelite audit` with filters; surface 403 for non-admin keys clearly)
8. **Fix dead flags / model mismatches** (§D): people/org/workflow filters, expand passthrough, position float, stages all-mode
9. **Docs command** (`pipelite docs` → fetch /api/v1/docs)
10. Server asks: search-by-API-key, file upload by API key (needs server-side work first)
