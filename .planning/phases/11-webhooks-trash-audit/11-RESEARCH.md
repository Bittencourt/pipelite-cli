# Phase 11: Webhooks, Trash & Audit - Research

**Researched:** 2026-09-03
**Domain:** Three new CLI surfaces (webhooks CRUD, trash restore/purge, audit viewer) against verified server contracts at `/home/pedro/programming/pipelite`
**Confidence:** HIGH (every server contract below was read directly from server source this session; all CLI patterns read from this repo)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Webhooks CRUD & Secret Handling**
- Unknown event names rejected client-side (exit 2 pre-HTTP) with an actionable error listing all 13 valid events.
- Secret shown exactly once on create: "Signing secret (save it now — shown only once):" + full 64-char secret on its own line. List/get output shows `(shown once at creation)`. The secret NEVER enters the cache or any other output. The 64-char truncation test is written FIRST (ROADMAP note).
- Create input: `--url`, `--events <e1,e2,...>` (comma-separated), `--description`; `--stdin` JSON for full control. *(⚠ Research conflict on `--description` — see Surprise S1.)*
- Update = get→merge→PUT: update flags merge into the fetched current webhook; omitted keys remain unchanged (no-op). PUT sends the merged full object.

**Trash & Recovery**
- Type normalization: accept singular and plural forms (deal/deals, organization/orgs/organizations, person/people, activity/activities) → normalize to the server's type token; enables `trash list | jq | trash restore` round-trips.
- `linked_parents` rendering: truncated table cell; full array via `--json`.
- Restore grammar: `trash restore <type> <id>` (positional); no confirmation (restore IS the recovery act).
- Purge grammar: `trash purge [--type <t>]` (all trashed, or per type). Strongest confirmation in the codebase: prompt names the scope + says "permanently destroys"; non-TTY without `--force` → **exit 2** with zero HTTP (deliberately stricter than the standard delete's exit 1, per criterion 4).

**Audit Log**
- Four passthrough filters: `--entity-type`, `--entity-id`, `--actor-kind`, `--workflow-run-id` (server validates values).
- Pagination: `--limit/--offset` passthrough; `--all` only if research shows a hard cap (pinned below: trash YES, audit NO).
- Default columns: timestamp, actor (kind/id), action, entity (type/id); changes payload visible via `--json` only.
- Admin gating: Forbidden + existing per-surface hint "audit log requires an admin key" (registered in Phase 8) — verified in this phase.

**Structure, Cache & Confirmation Hierarchy**
- 3 plans: webhooks / trash / audit — sequential waves (shared wiring files).
- Cache: webhooks get `KEY_WEBHOOKS` with invalidation on mutations; trash and audit are never cached.
- Confirmation hierarchy: purge (strongest wording + exit-2 no-input refusal) > webhook delete (standard confirm + `--force`, non-TTY refusal exit 1) > restore (none).
- Test emphasis: secret-truncation test FIRST; per-plan stub-server suites; purge zero-HTTP refusal test pinned; Forbidden hints probed per surface.

### the agent's Discretion
- Exact default table columns for trash/audit/webhook lists (CONTEXT names audit's four; trash/webhooks open)
- linked_parents cell truncation mechanics; where the `(shown once at creation)` placeholder renders
- Restore/purge 403 surface-key choice (see Pitfall P6)
- `--all` flag placement (pinned recommendation below)

### Deferred Ideas (OUT OF SCOPE)
- AUDT-03 (changes rendered as field: from → to diffs) — future milestone
- WHOK-04 (event-name shell completions) — future milestone
- Webhook delivery log viewer / test-ping endpoint — not in requirements
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WHOK-01 | List, get, create, update (PUT), delete webhooks | Full CRUD contracts verified (§Webhooks); client method + command patterns from templates/notes precedents |
| WHOK-02 | Event names validated client-side against 13 events, actionable error | Exact 13 strings verified from server's compiler-enforced `CrmEventMap` (§The 13 Event Names); exit-2 pre-HTTP pattern from notes get / templates update |
| WHOK-03 | Secret displayed exactly once on create, full, own line, never cached/listed | Server returns secret ONLY in POST response (webhooks/route.ts:111,125); list/get/PUT serializers exclude it; 64-char = 32 random bytes hex |
| TRSH-01 | List trashed records with `--type` filter | GET /api/v1/trash contract incl. type enum, default `deals`, offset cap 10,000, row shape (§Trash) |
| TRSH-02 | Restore by type + ID | POST /api/v1/trash/{type}/{id}/restore — plural-tab token, owner-or-admin, 404 if not in trash, 204 |
| TRSH-03 | Permanent purge — admin-only, strongest confirm, exit-2 no-input refusal zero-HTTP, singular/plural normalization | DELETE /api/v1/trash/{type}/{id} admin-only; NO bulk endpoint → CLI fan-out design (§Recommended Approach, Plan 2); exit-2 mechanism = `CliError::InvalidInput` (error.rs:87-93) |
| AUDT-01 | Audit list with 4 filters | GET /api/v1/audit contract — filter enums, min-length rules, pagination, sort order, entry shape (§Audit) |
| AUDT-02 | Non-admin keys get first-class Forbidden hint | Hints already registered (api/mod.rs:92-100); audit gates BEFORE validation (audit/route.ts:148-153); admin matrix §Admin-Gating Matrix |
</phase_requirements>

## Summary

All three server surfaces exist and are stable. Every contract in this document was read directly from server source this session (file:line cited) — nothing relies on training data. The server is at `/home/pedro/programming/pipelite` (Next.js + Drizzle/Postgres, post-Phase-50). All three surfaces use the standard `/api/v1` conventions: `Authorization: Bearer pk_live_...`, `{data:...}` / `{data,meta}` envelopes, RFC 7807 errors, pagination default 50 / max 100.

Three findings materially affect planning. **(1)** The server has **no webhook `description` field** — the DB table and both zod schemas contain only `{url, events, active}` and unknown keys are silently stripped — so CONTEXT's `--description` flag has no server backing and must be dropped or reduced. **(2)** There is **no bulk purge endpoint** — `DELETE /api/v1/trash/{type}/{id}` is per-record only — so the locked `trash purge [--type <t>]` grammar is implemented as a CLI-side fan-out: list (paged) then per-record DELETE. **(3)** The server's webhook PUT is already partial-merge semantics (all fields optional, omitted keys untouched), so the locked get→merge→PUT is safe belt-and-braces, not a workaround for a real full-replace hazard.

The CLI-side foundation is complete: forbidden hints for `audit`, `trash`, and `webhooks` are pre-registered (Phase 8), the exit-2 mechanism exists (`InvalidInput`/`MissingInput` → 2 vs `Validation` → 1), and the Phase 9/10 stub-server test helpers (`tests/common/mod.rs`) plus three stub-suite precedents cover every test shape this phase needs. No new crates are required.

**Primary recommendation:** Three sequential plans (webhooks → trash → audit) reusing the notes/templates group pattern, Phase 9 stub helpers, and the existing hint table. Purge is a list-then-fan-out loop with `InvalidInput` (exit 2) refusal; webhook secret rendering bypasses table cells entirely (own-line print after render).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Event-name validation (13) | CLI (pre-HTTP) | — | Server accepts ANY strings (zod is `z.string().min(1)`); only the CLI can reject early. Exit 2, zero HTTP |
| Secret show-once + never-cache | CLI (render/cache layer) | Server (create-only return) | Server emits secret only in POST response; CLI must not persist it into cache/completions |
| Webhook CRUD | API server | CLI renders | Ownership-scoped CRUD; CLI is a thin faithful client |
| Trash type normalization | CLI (pre-HTTP) | — | Server accepts exactly 4 plural tabs (422 otherwise); CLI widens to singular/plural aliases |
| Trash scope enforcement | API server | — | Owner-or-admin scoping built server-side from storage-resolved role |
| Purge admin gate | API server | CLI hint rendering | Server 403s non-admins before record lookup; CLI renders the `trash` hint |
| Purge fan-out (scope→N deletes) | CLI | API server (per-record DELETE) | No bulk endpoint exists; only the CLI can iterate |
| Audit filter validation | API server | CLI passthrough | Server validates enums + min-length (422); CLI passes values verbatim |
| Audit admin gate | API server | CLI hint rendering | Server 403s before even parsing query string |

## Standard Stack

### Core (all already in tree — zero new dependencies)

| Library | Version (in tree) | Purpose | Why Standard |
|---------|-------------------|---------|--------------|
| clap (derive) | existing | New `Webhooks`/`Trash` command groups + `Audit` command | Every existing group |
| reqwest | existing | 5 new client methods (webhooks ×5, trash ×3, audit ×1) | Existing `PipeliteClient` |
| serde / serde_json | existing | New models: `Webhook`, `WebhookCreate`, `TrashRow`, `AuditEntry` | Existing models.rs pattern |
| dialoguer | existing | Purge confirmation (strongest wording), webhook delete confirm | Existing confirm flows |
| comfy-table | existing | trash/audit/webhook list tables | Existing output/table.rs |
| assert_cmd | existing (dev) | Stub-server integration suites | Phase 9/10 precedent |

**Installation:** none — no new packages. Package Legitimacy Audit below is trivially clean.

## Package Legitimacy Audit

**No external packages are installed this phase.** All functionality uses dependencies already vendored in `Cargo.toml` (clap, reqwest, serde, serde_json, dialoguer, comfy-table, anyhow; dev-deps assert_cmd, predicates). No slopcheck run required; no registry lookups required; no `checkpoint:human-verify` gates needed.

## Architecture Patterns

### System Architecture Diagram

```
                        pipelite CLI (this phase)
 ┌────────────────────────────────────────────────────────────────────┐
 │                                                                    │
 │  webhooks create --url --events ──► validate 13 events (exit 2)    │
 │        │                              │ ok                         │
 │        ▼                              ▼                            │
 │  --stdin JSON ──────────────► POST /api/v1/webhooks                 │
 │        │                              │ 201 {data:{...secret}}     │
 │        ▼                              ▼                            │
 │  update: GET webhook ──► merge flags ──► PUT merged object          │
 │        │                              │ 200 (no secret ever)       │
 │        ▼                              ▼                            │
 │  create output: render + "Signing secret (save it now —            │
 │  shown only once):" + 64-char secret ON ITS OWN LINE               │
 │  (never written to ~/.pipelite/cache/)                             │
 │                                                                    │
 │  trash list/restore/purge ──► normalize type token                 │
 │        │                      (deal→deals, org→organizations…)     │
 │        ▼                                                           │
 │  purge: GET /trash?type= (paged) ──► confirm scope ──►             │
 │         N × DELETE /trash/{type}/{id}   [admin only, 204 each]     │
 │                                                                    │
 │  audit list --entity-type --entity-id --actor-kind                 │
 │             --workflow-run-id ──► GET /api/v1/audit?...            │
 │        │                              │ 403 non-admin → hint       │
 │        ▼                              ▼                            │
 │  table: ts/actor/action/entity; --json adds changes                │
 └────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
src/
├── cli/
│   ├── webhooks.rs        # WebhooksCommands: List/Get/Create/Update/Delete
│   ├── trash.rs           # TrashCommands: List/Restore/Purge
│   └── audit.rs           # AuditCommands (List or bare-audit args)
├── commands/
│   ├── webhooks/          # mod.rs, list.rs, get.rs, create.rs, update.rs, delete.rs
│   ├── trash/             # mod.rs (normalize_trash_type), list.rs, restore.rs, purge.rs
│   └── audit/             # mod.rs, list.rs
├── api/
│   ├── mod.rs             # +8 client methods (surface keys: "webhooks"/"trash"/"audit")
│   └── models.rs          # + Webhook, WebhookCreate, WebhookUpdate, TrashRow, DeletedBy, AuditEntry
└── cache.rs               # + KEY_WEBHOOKS: &str = "webhooks"
tests/
├── webhooks_stub_test.rs  # WHOK-01..03 (secret truncation test FIRST)
├── trash_stub_test.rs     # TRSH-01..03 (purge zero-HTTP refusal pinned)
└── audit_stub_test.rs     # AUDT-01..02
```

### Pattern 1: Type-Normalizing Positional (notes precedent, widened)

**What:** Map user type alias → server tab token before any HTTP; unknown → `CliError::InvalidInput` (exit 2).
**When to use:** `trash list/restore/purge` type arguments.
**Example:**

```rust
// Source: src/commands/notes/mod.rs:45-60 (existing), widened per CONTEXT D-decision
pub fn normalize_trash_type(t: &str) -> Result<&'static str> {
    match t {
        "deal" | "deals" => Ok("deals"),
        "organization" | "orgs" | "organizations" => Ok("organizations"),
        "person" | "people" => Ok("people"),
        "activity" | "activities" => Ok("activities"),
        other => Err(CliError::InvalidInput {
            detail: format!("Unknown trash type '{other}'"),
            hint: "Valid types: deal(s), organization(s)/orgs, person/people, activity/activities"
                .to_string(),
        }.into()),
    }
}
```

Deliberately NOT a clap `ValueEnum` — same reasoning as notes (commands/notes/mod.rs:36-44): the locked exit-2 rejection with hint must fire, not clap's generic invalid-value error. Round-trip note: the server's `type` field on trash rows is ALREADY a plural tab, so `trash list --format json | jq .type` pipes directly into restore/purge; singular forms are a user convenience.

### Pattern 2: Purge as Scope→Fan-Out (no bulk endpoint)

**What:** `trash purge [--type <t>]` resolves the scope via paged `GET /api/v1/trash`, confirms, then issues one `DELETE /api/v1/trash/{type}/{id}` per record.
**When to use:** Plan 2 (trash).
**Flow:** normalize type → (1) list page(s) with `limit=100` until `data.len() < limit` or accumulated ≥ `meta.total` → collect `(type, id)` pairs → (2) dry-run intercept first (`--dry-run` lists what WOULD be purged; list GETs only, zero DELETEs) → (3) confirm (TTY, strongest wording naming scope + "permanently destroys") → (4) non-TTY & !force → `CliError::InvalidInput` exit 2 **before any HTTP** (the zero-HTTP contract applies to the refusal path; it is testable with `cmd()` unreachable server) → (5) delete each; on first 403 abort with the `trash` hint (admin key required for every delete — continuing would just 403 N times) → (6) summary `N permanently destroyed`.

### Pattern 3: Show-Once Secret Rendering (bypass table cells)

**What:** After rendering the create response in the chosen format, print the secret warning + raw secret to stdout on its own line; never route it through the table renderer or cache.
**When to use:** `webhooks create` only.
**Why:** comfy-table applies a fixed 120-col width for non-TTY stdout (src/output/table.rs:49-51) and wraps/constrains cell content — a 64-char secret in a cell is exactly the truncation hazard the ROADMAP test targets. Own-line `println!` is width-independent.

### Pattern 4: Hidden-Variant / Parse-Then-Error (existing)

Not needed for a "no route" case this phase (all three surfaces have their routes), but the same pre-HTTP `InvalidInput` rejection shape (notes get, commands/notes/mod.rs:24-34; templates update, src/cli/templates.rs:148-167) is the template for: unknown event names, unknown trash types, and the purge no-input refusal.

### Anti-Patterns to Avoid

- **Sending the secret anywhere but stdout:** no cache write, no completion-candidate list, no log line, no dry-run body echo (secret only exists in POST create responses anyway — but don't "helpfully" persist it).
- **Empty-string filter passthrough on audit:** server 422s `entity_id=` (min(1), audit/route.ts:75,77). Only include a filter param in the query when the flag was actually provided with a non-empty value.
- **Singular types in trash URLs:** the write routes validate the `{type}` segment against plural `TRASH_TABS` only (restore/route.ts:63-73) — normalization must resolve to the plural tab BEFORE building the URL. The row's `entity_type` (singular) is display/correlation vocabulary, never URL vocabulary.
- **Assuming trash `meta.total` covers all tabs:** it is the count of the SELECTED tab only (trash/route.ts:280-285, `counts[tab]`).
- **Paging trash past offset 10,000 expecting data:** past-cap pages return empty `data` with truthful `meta.total` (trash/route.ts:94-99). Any `--all`/fan-out loop must break on an empty page, not loop until accumulated == total.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Confirmation + refusal gate | New confirm helper | `confirm` shape from src/batch.rs:174-193 (webhook delete: exit-1 `Validation` refusal) + a purge-specific variant with `InvalidInput` (exit 2) | Existing ordering (dry-run → confirm) is battle-tested; only the error variant and wording differ |
| Forbidden hint mapping | Per-command 403 strings | `forbidden_hint(surface)` — keys `audit`/`trash`/`webhooks` already registered (src/api/mod.rs:92-100) | Zero registration work; consistent wording guaranteed |
| Stub HTTP servers | mockall / wiremock | `spawn_head_capturing_stub_server` (tests/common/mod.rs:49-163) | Records heads+bodies for wire assertions; proven in 3 Phase 9/10 suites; no new dev-deps |
| RFC 7807 error text | Manual parse | `parse_rfc7807` + `handle_response`/`handle_delete_response` (src/api/mod.rs:299-390) | Phase 8 error layer already extracts `detail`/`errors[]` |
| 429 handling | Custom retry | `send_with_retry` | Already retry-once per Retry-After |
| Pagination envelope | Custom struct | `ApiListResponse<T>` + `PaginationMeta` (src/api/models.rs:29-39) | Exact envelope match |

**Key insight:** This phase is almost entirely composition of existing Phase 7/8/9/10 machinery; the genuinely new logic is (a) the 13-event allow-list, (b) trash type normalization, (c) the purge fan-out loop, (d) secret show-once rendering.

## The 13 Event Names (verified, compiler-enforced server-side)

From `CrmEventMap` (server `src/lib/events/types.ts:40-54`) — and the webhook subscriber's `ALL_EVENT_FLAGS: Record<CrmEventName, true>` (server `src/lib/events/subscribers/webhook.ts:31-47`) makes a missing/extra entry a **compile error**, so this list cannot drift:

```
deal.created          person.created        organization.created   activity.created
deal.updated          person.updated        organization.updated   activity.updated
deal.deleted          person.deleted        organization.deleted   activity.deleted
deal.stage_changed
```

Ordering for the error message: group by entity (deal, person, organization, activity), `created|updated|deleted` then `stage_changed` — matches the map order above. Note `deal.stage_changed` is the only non-{created,updated,deleted} action and exists only for deals.

**Server-side validation is intentionally absent** (`events: z.array(z.string().min(1))` — webhooks/route.ts:17): unknown strings like `deal.archived` are ACCEPTED and silently never fire. Client-side validation is therefore the only defense — the CONTEXT exit-2 decision is load-bearing.

## Standard Stack — API Contracts (file:line verified)

### Webhooks

| Op | Route | Request | Response | Citations |
|----|-------|---------|----------|-----------|
| List | `GET /api/v1/webhooks?offset&limit` | — | 200 `{data:[Webhook], meta}`; owner-scoped; NO secret | webhooks/route.ts:60-85 |
| Create | `POST /api/v1/webhooks` | `{url, events[]}` — url MUST start `https://`; events min 1, any strings | **201 `{data: Webhook & {secret}}`** — the ONLY response containing secret; `active` forced true; secret = `crypto.randomBytes(32).toString("hex")` = **64 hex chars** | webhooks/route.ts:12-18, 111, 119, 125 |
| Get | `GET /api/v1/webhooks/{id}` | — | 200 `{data: Webhook}` no secret; foreign → **403** (NO admin bypass); missing → 404 | webhooks/[id]/route.ts:48-68 |
| Update | `PUT /api/v1/webhooks/{id}` | all optional `{url, events[], active}`; **server merges only provided keys**; secret never updatable (no regeneration endpoint exists — comment at [id]/route.ts:111) | 200 `{data: Webhook}` no secret; foreign → 403 | webhooks/[id]/route.ts:10-17, 107-127 |
| Delete | `DELETE /api/v1/webhooks/{id}` | — | 204 hard delete; foreign → 403 | webhooks/[id]/route.ts:131-153 |

**Webhook item shape** (webhooks/route.ts:21-28): `{id, url, events: string[], active: bool, created_at, updated_at}` — **no `description` field anywhere** (DB schema `src/db/schema/webhooks.ts`: only id, userId, url, secret, events, active, createdAt, updatedAt; both zod schemas strip unknown keys silently).

**get→merge→PUT verdict:** server PUT is already partial (`if (updates.url !== undefined) updateData.url = updates.url`, [id]/route.ts:116-118). The locked decision is harmless belt-and-braces — merged-full-object PUT produces identical server state. Cost: one extra GET per update; foreign webhook 403s at the GET already. Proceed as locked.

### Trash

| Op | Route | Behavior | Citations |
|----|-------|----------|-----------|
| List | `GET /api/v1/trash?type=&offset=&limit=` | `?type` optional; **absent → `deals`** (parseTrashTab); **invalid → 422** listing the four tabs (NOT silent default); offset clamped to ≤ **10,000** (50 × 200 pages), past-cap → empty `data` + truthful `meta.total`; `meta.total` = selected tab's count only; owner-or-admin scoping (member sees own rows, admin sees all); unresolvable actor → 403 | trash/route.ts:60-62, 81-99, 216-291; entity-types.ts:25; queries.ts:34 |
| Restore | `POST /api/v1/trash/{type}/{id}/restore` | `{type}` = **plural tab only** (`deals|people|organizations|activities`), invalid → 422 listing tabs; owner-or-admin (member non-owner → 403); record must be in trash — live/missing → 404 (no existence oracle); success → **204** no body | restore/route.ts:63-73, 87-137 |
| Purge | `DELETE /api/v1/trash/{type}/{id}` | **ADMIN-ONLY** — gate runs BEFORE record lookup (anti-enumeration: non-admin always 403 regardless of id); invalid {type} → 422; not in trash → 404; success → **204** no body. **No bulk route exists.** | [id]/route.ts:55-75, 77-125 |

**Trash row shape** (`SerializedTrashRow`, trash/route.ts:178-189):

```json
{
  "id": "…",
  "entity_type": "deal",                    // SINGULAR — correlation vocabulary
  "type": "deals",                          // PLURAL tab — the URL round-trip token
  "name": "…",
  "secondary": "…",                         // string | null
  "deleted_at": "2026-09-01T12:00:00.000Z",
  "linked_parents": ["Acme Corp"],          // string[] of parent names ALSO in trash; ALWAYS [] for organizations
  "deleted_by": { "kind": "user", "name": "…", "email": "…" }
}
```

**`deleted_by` closed union** (trash/route.ts:115-155): `{kind:"not_recorded"}` | `{kind:"user", name: str|null, email: str|null}` | `{kind:"unknown_user"}` | `{kind:"workflow_run", workflow_name: str|null}` | `{kind:"api_key"}` (no name field — deliberate) | `{kind:"import"}` | `{kind:"system"}`. Table column: render `kind` (+ name/email or workflow_name when present); `--json` shows full object.

### Audit

| Aspect | Contract | Citations |
|--------|----------|-----------|
| Route | `GET /api/v1/audit?entity_type&entity_id&actor_kind&workflow_run_id&offset&limit` | audit/route.ts:145-214 |
| Admin gate | 403 for non-admin; **gate runs BEFORE query-string validation** (a non-admin learns nothing from bad filters); unresolvable actor → 403 | audit/route.ts:124-136, 148-153 |
| `entity_type` enum | `organization \| person \| deal \| activity \| import_session \| export` — invalid → 422 | audit/route.ts:45-57, 73-78 |
| `actor_kind` enum | `user \| workflow_run \| api_key \| import \| system` — invalid → 422 | audit/route.ts:59-65 |
| `entity_id`, `workflow_run_id` | strings, `min(1)` — **empty string → 422** | audit/route.ts:75, 77 |
| Pagination | `parsePagination`: limit default 50 clamped [1,100]; offset default 0 clamped [0, 1,000,000] — **no trash-style low cap** | pagination.ts:16, 44-61 |
| Sort | `createdAt DESC, id DESC` (newest first, stable tiebreaker) — not configurable | audit/route.ts:200-204 |
| Entry shape | `{id, entity_type, entity_id, action, changes, actor_kind, actor_user_id, workflow_run_id, import_session_id, created_at}` | audit/route.ts:81-92 |
| `action` values | `created \| updated \| deleted \| merged` (4-value closed union, audit-log.ts:40) | db/schema/audit-log.ts:40 |
| `changes` | `Record<field, {from, to}>`; `{}` legitimate for create/delete; returned verbatim | db/schema/audit-log.ts:52; audit/route.ts:100-103 |

## Admin-Gating Matrix (verified per-route)

| Operation | Gate | Non-admin / foreign result | CLI hint (already registered) |
|-----------|------|---------------------------|-------------------------------|
| `webhooks list/create` | any API key (owner-scoped list) | 200 / 201 | — |
| `webhooks get/update/delete` — own | ownership | 200 / 200 / 204 | — |
| `webhooks get/update/delete` — foreign | ownership **only — even an ADMIN key gets 403** (no role bypass in route) | 403 | `"webhooks"` → "This webhook belongs to another user." (api/mod.rs:97) |
| `trash list` | not gated — scoped (member: own rows; admin: all rows); unresolvable actor → 403 | 200 (scoped) | `"trash"` hint only on the unresolvable-403 edge |
| `trash restore` — own record | owner-or-admin | 204 | — |
| `trash restore` — foreign record (member) | owner-or-admin | 403 | see Pitfall P6 (wording) |
| `trash purge` (every DELETE) | **ADMIN-ONLY**, checked before lookup | 403 always | `"trash"` → "Permanent purge requires an admin API key." (api/mod.rs:95) |
| `audit list` | **ADMIN-ONLY**, checked before query validation | 403 always | `"audit"` → "The audit log requires an admin API key." (api/mod.rs:94) |

Contrast worth preserving in docs: webhooks are ownership-exclusive (admin gets 403 too), while trash/audit admin powers are real. AUDT-02's "first-class Forbidden hint" is satisfied by the existing table — verification tests only.

## Runtime State Inventory

Not a rename/refactor/migration phase — all surfaces are additive. Checked anyway for cache state:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — no existing webhook/trash/audit records in `~/.pipelite/`; cache dir holds existing entity keys only | None |
| Live service config | None (server-side state only) | None |
| OS-registered state | None | None |
| Secrets/env vars | None new; existing `PIPELITE_API_KEY` untouched. Webhook secrets are **server-generated** — CLI never stores/transmits them after echo | None |
| Build artifacts | None | None |

## Common Pitfalls

### P1: `--description` silently vanishes (CONTEXT conflict)
**What goes wrong:** A `--description` flag would parse, be merged into the JSON body, and the server's non-strict zod object would silently strip it — user believes it saved; list shows nothing.
**Why it happens:** Server webhooks table + create/update schemas have no description field (see §Webhooks).
**How to avoid:** Drop `--description` from Plan 1. Keep `--stdin` as the "full control" escape hatch (documenting that the server accepts only `{url, events, active}`). Flag for user confirmation since it amends a CONTEXT decision.
**Warning signs:** Stub test asserting the POST body contains `description` would pass while the real server ignores it — test the server contract, not the wish.

### P2: Purge refusal exits 1 instead of 2
**What goes wrong:** Reusing the batch-delete refusal (`CliError::Validation`, src/batch.rs:184-191) gives exit 1 — violates criterion 4.
**Why it happens:** Copy-paste from `run_batch_delete`.
**How to avoid:** Purge refusal must construct `CliError::InvalidInput` (exit 2 per src/error.rs:87-93). The stub test asserts both exit code AND `counter == 0`.
**Warning signs:** `exit_code_test.rs` conventions; any purge test asserting exit 1.

### P3: Trash `--all`/fan-out infinite-ish loop past the offset cap
**What goes wrong:** Paging loop `while accumulated < meta.total` never terminates past offset 10,000 (server returns empty pages with truthful total).
**Why it happens:** Trusting `meta.total` as reachable.
**How to avoid:** Break on empty page OR `data.len() < limit` OR accumulated ≥ total OR offset > 10,000.
**Warning signs:** Trash list tests scripting 3 pages then hanging.

### P4: Empty-string audit filters 422
**What goes wrong:** `--entity-id ""` (or a shell var that expands empty) sends `entity_id=` → 422 from `min(1)`.
**How to avoid:** Treat empty-string flag values as absent; omit the param. (Or pass through and let the 422 flow — but pre-validating matches the phase's fail-early posture.)
**Warning signs:** audit stub test with `--entity-id ""`.

### P5: Singular type in a trash write URL
**What goes wrong:** `trash restore deal d1` → `POST /api/v1/trash/deal/d1/restore` → 422 (`Invalid entity type "deal": expected one of deals, people, organizations, activities`).
**Why it happens:** Normalization skipped on one of the three subcommands.
**How to avoid:** Single shared `normalize_trash_type` called by list/restore/purge alike; row's `entity_type` (singular) never used in URLs.
**Warning signs:** Any hand-built URL containing `entity_type`.

### P6: Restore 403 shows the purge hint
**What goes wrong:** Member restoring another user's record gets 403; passing surface `"trash"` renders "Permanent purge requires an admin API key" — misleading (restore is owner-or-admin, not admin-only).
**How to avoid:** Planner choice: pass `"general"` for restore (→ "Your API key doesn't have permission for this action.") or add one new hint key (small, acceptable deviation from "zero registration work"). Purge and audit keep their dedicated keys.
**Warning signs:** restore 403 stub test asserting hint text.

### P7: Secret leaking into cache/completions
**What goes wrong:** `KEY_WEBHOOKS` cache stores `(id, url)` pairs for completions (templates pattern, cli/templates.rs:6-17); accidentally storing the create response would persist the secret to disk at `~/.pipelite/cache/`.
**How to avoid:** Cache writes only `{id, url}` tuples built from list/get responses (which never contain the secret); the truncation test suite asserts the cache directory contains no 64-hex-char string after create.
**Warning signs:** any `cache.set(KEY_WEBHOOKS, ...)` fed from the create response.

### P8: Assuming webhook 404 vs 403 semantics like workflows
**What goes wrong:** Workflows hide foreign resources (404); webhooks deliberately 403 (webhooks/[id]/route.ts:61-63 — unlike workflows' 404, per SERVER-API-DIFF §A.4).
**How to avoid:** Webhook error docs/tests assert 403 for foreign; hint text already correct.

## Code Examples

### Client method shape (follow list_notes precedent, src/api/mod.rs:1016-1043)

```rust
// Source: project pattern src/api/mod.rs (notes section)
/// List webhooks owned by the API key. GET /api/v1/webhooks?limit&offset.
/// The server NEVER returns the signing secret on this route.
pub async fn list_webhooks(&self, limit: u64, offset: u64) -> Result<ApiListResponse<Webhook>> {
    let url = format!("{}/api/v1/webhooks", self.base_url);
    let request = self.client.get(&url).query(&[
        ("limit", limit.to_string()),
        ("offset", offset.to_string()),
    ]);
    let response = self.send_with_retry(request).await?;
    self.handle_response(response, "webhooks").await
}

/// Purge (permanently destroy) one trashed record. DELETE /api/v1/trash/{type}/{id}.
/// ADMIN-ONLY — non-admin keys get 403 + the "trash" hint. `type` is the PLURAL
/// tab token (deals|people|organizations|activities), caller-normalized.
pub async fn purge_trash(&self, trash_type: &str, id: &str) -> Result<()> {
    let url = format!("{}/api/v1/trash/{}/{}/", self.base_url, trash_type, id); // note: build without trailing slash
    let request = self.client.delete(&url);
    let response = self.send_with_retry(request).await?;
    self.handle_delete_response(response, "trash").await
}
```

### Purge refusal (exit 2, zero HTTP) — the ONE new error shape

```rust
// Deliberately INVALID-INPUT (exit 2), NOT Validation (exit 1) — criterion 4.
// Compare src/batch.rs:184-191 which uses Validation for standard deletes.
if !force && ctx.no_input {
    return Err(CliError::InvalidInput {
        detail: "Refusing to permanently destroy trashed records without confirmation \
                 in non-interactive mode."
            .to_string(),
        hint: "Re-run with --force to skip confirmation (scripts), or run interactively."
            .to_string(),
    }.into());
}
```

### Event validation (pre-HTTP, exit 2)

```rust
pub const WEBHOOK_EVENTS: [&str; 13] = [
    "deal.created", "deal.updated", "deal.deleted", "deal.stage_changed",
    "person.created", "person.updated", "person.deleted",
    "organization.created", "organization.updated", "organization.deleted",
    "activity.created", "activity.updated", "activity.deleted",
];
// On unknown: InvalidInput, detail names the bad event,
// hint lists all 13 (comma-separated) + "e.g. --events deal.created,deal.updated".
// Runs before ANY HTTP (test with cmd() unreachable server + request counter == 0).
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Trash REST routes returned only `entity_type` (singular) | Rows carry BOTH `entity_type` (singular) and `type` (plural round-trip token) | Server T-37-03 fix | CLI round-trips use `type`; never "helpfully" accept singular in URLs |
| Trash route converted offset→cumulative page (cost bomb) | True LIMIT/OFFSET window + 10,000 offset cap | Server WR-03/IN-04 | Constant-cost paging; past-cap = empty data |
| Audit unbounded offset → Postgres 500s | Global `MAX_OFFSET = 1,000,000` clamp in parsePagination | Server WR-09 | All v1 list routes share the clamp |
| `AuditAction` 3 values | + `merged` (Phase 39) | 39-09 | CLI action column must tolerate all 4 |
| 6th audit entity_type `export` | exists since Phase 40 | WR-04 | `--entity-type` help text should list all 6 |

**Deprecated/outdated:** none affecting this phase.

## Recommended Approach per Plan

### Plan 1 — Webhooks (WHOK-01..03)
1. Models: `Webhook {id, url, events: Vec<String>, active, created_at: Option<String>, updated_at: Option<String>}`; `WebhookCreate {url, events}` (+ `active` only in update); NO description field.
2. Client: 5 methods, surface `"webhooks"`; create returns `Webhook` with `secret: Option<String>` (Some only from create).
3. CLI group `webhooks` (List/Get/Create/Update/Delete) — after_help examples; `--events` comma-delimited; `--stdin` raw JSON (templates-create precedent, src/cli/templates.rs:98-131).
4. Validation: exact-match against 13; unknown → exit 2 InvalidInput, hint lists all 13; runs before HTTP.
5. Update: GET → merge flags (`--url`, `--events`, `--active`/`--inactive`) → PUT full merged object; `--stdin` bypasses merge (verbatim body, full control).
6. Secret rendering: create → render response normally, then `println!("Signing secret (save it now — shown only once):")` + secret alone on next line (all formats; in `--format json` the response body itself also carries `secret` — that IS the show-once moment). List/get: no secret exists in responses; render `(shown once at creation)` placeholder per CONTEXT (e.g. as a `secret` column value or footer note — planner pins). Cache: `KEY_WEBHOOKS` storing `(id, url)` for completions; invalidate on create/update/delete.
7. Delete: single → templates-delete flow (confirm, `--force`, refusal = `Validation` exit 1, dry-run first); batch via existing `run_batch_delete`.
8. Tests FIRST: `webhooks_secret_shown_once_test` (64-char full, own line, cache-absence) then CRUD stub suite + event-validation zero-HTTP tests + foreign-403 hint probe.

### Plan 2 — Trash (TRSH-01..03)
1. Models: `TrashRow` exactly as §Trash shape; `DeletedBy` as untagged-ish enum via `kind` tag (serde `tag = "kind"`); `linked_parents: Vec<String>`.
2. Client: 3 methods (`list_trash(type: Option<&str>, limit, offset)`, `restore_trash(type, id)` POST, `purge_trash(type, id)` DELETE) — surfaces: list/restore pass `"general"` (or new key, P6), purge passes `"trash"`.
3. `normalize_trash_type` (Pattern 1) shared by all three subcommands.
4. `trash list [--type] [--limit --offset --all]` — table columns: name, type, deleted_at, deleted_by (kind+name), linked_parents (truncated cell); `--json` full. `--all` IS justified: server caps offset at 10,000 → bounded loop (CONTEXT pinned). after_help documents round-trip: `trash list --format json | jq -r '.data[] | select(.type=="deals") | .id'`.
5. `trash restore <type> <id>` — positional, NO confirm, 204 → success line; 404 → "not in trash (already restored or purged, or never existed)" hint.
6. `trash purge [--type <t>]` — Pattern 2 fan-out; strongest-wording confirm naming scope + "permanently destroys"; `--dry-run` lists victims (list GETs only); no-input refusal = exit 2 zero-HTTP; first 403 aborts with `trash` hint; summary counts.
7. Tests: normalization round-trips (incl. `trash list` JSON `.type` → restore), purge zero-HTTP refusal (pinned), purge 403 hint, fan-out multi-delete script, restore no-confirm.

### Plan 3 — Audit (AUDT-01..02)
1. Model: `AuditEntry` exactly as §Audit shape; `changes: serde_json::Value` (deferred AUDT-03 rendering).
2. Client: 1 method `list_audit(filters, limit, offset)`, surface `"audit"`; build query only from `Some(non-empty)` values (P4).
3. Command: bare `pipelite audit [filters]` (docs-style single command) — CONTEXT calls it a "viewer"; a one-command group is also fine (planner pins). Default columns per CONTEXT: created_at, actor (`kind/id`), action, entity (`type/id`); `--json` exposes `changes` + all actor ids.
4. Pagination: `--limit/--offset` passthrough; **no `--all`** (no server cap; matches notes precedent — after_help says "pages capped at 100 — iterate --offset"). after_help lists the 6 entity_type + 5 actor_kind values.
5. Non-admin: 403 renders existing `"audit"` hint — verify with stub probe (gate fires even with invalid filters — test that ordering).
6. Tests: 4 filters appear verbatim on the wire (heads capture), empty-flag omission, 403 hint probe, sort-order note, json changes passthrough.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `--format csv/plain` should work for trash/audit lists like other list commands | Plans 2/3 | Low — cosmetic; planner may scope to json/table only |
| A2 | The create response in `--format json` legitimately includes `secret` (it is the server's body); CONTEXT's "never in cache or other output" refers to CLI-added persistence | Plan 1 | Low — if user intends to strip it even from json, a filter step is needed; confirm in discuss/planning |
| A3 | `--all` for trash terminates at the 10,000-offset cap and is acceptable per CONTEXT's "hard cap" clause | Plan 2 | Low — worst case drop `--all` |
| A4 | Restore/purge fan-out with a member (non-admin) key aborts on first 403 rather than pre-checking (no cheap "am I admin?" endpoint exists) | Plan 2 | Medium — alternative is probing with a 1-record purge first; abort-on-first-403 is simpler and safe |

Everything else above is verified against server/CLI source with citations — no other assumptions.

## Open Questions

1. **`--description` removal (S1)** — amends a CONTEXT decision.
   - What we know: server schema + both zod schemas lack description; unknown keys silently stripped.
   - Recommendation: drop the flag; note in CHANGELOG/help that webhooks carry no description. Confirm with user at plan review (one-line amendment).
2. **Purge granularity** — `trash purge` is scope-based (all / per type); no per-ID purge exists in the locked grammar.
   - What we know: server supports per-record DELETE; REQUIREMENTS TRSH-03 says "purge a trashed record"; CONTEXT grammar says `trash purge [--type <t>]`.
   - Recommendation: implement CONTEXT grammar as locked; single-record purge = user's deliberate scope choice. Surface at plan review for explicit acceptance.
3. **Restore 403 hint key** (P6) — planner pins `"general"` vs one new key.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo/rustc | build + tests | ✓ | 1.94.0 | — |
| Server repo (source reading only) | contract verification | ✓ | /home/pedro/programming/pipelite | — |
| New crates | — | n/a | — | none needed |

**Missing dependencies with no fallback:** none.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` + assert_cmd against hand-rolled stub servers (no new dev-deps) |
| Config file | none needed — `tests/common/mod.rs` shared helpers |
| Quick run command | `cargo test --test webhooks_stub_test --test trash_stub_test --test audit_stub_test` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WHOK-01 | Webhook list/get/create/update/delete against stub | integration (stub) | `cargo test --test webhooks_stub_test` | ❌ Wave 0 |
| WHOK-02 | Unknown event → exit 2, zero HTTP, message lists all 13 | integration (unreachable server, counter==0) | `cargo test --test webhooks_stub_test event` | ❌ Wave 0 |
| WHOK-03 | Secret: full 64 chars, own line, once; absent from list/get output AND cache dir | integration — **write FIRST per ROADMAP** | `cargo test --test webhooks_stub_test secret` | ❌ Wave 0 |
| TRSH-01 | `trash list --type` normalizes deal/deal→deals etc.; query string on wire; offset-cap empty-page break | integration (stub) | `cargo test --test trash_stub_test list` | ❌ Wave 0 |
| TRSH-02 | restore positional, no confirm, 204 success, 404 hint | integration (stub) | `cargo test --test trash_stub_test restore` | ❌ Wave 0 |
| TRSH-03 | purge: exit-2 no-input refusal with counter==0 (PINNED); strongest confirm wording; --force bypass; fan-out deletes; 403 abort hint | integration (stub) | `cargo test --test trash_stub_test purge` | ❌ Wave 0 |
| AUDT-01 | 4 filters verbatim on wire; empty flags omitted; pagination passthrough | integration (stub) | `cargo test --test audit_stub_test` | ❌ Wave 0 |
| AUDT-02 | audit 403 → "audit log requires an admin API key" hint; trash purge 403 → purge hint; webhook foreign 403 → ownership hint (gate fires even with bad filters) | integration (stub) | `cargo test --test audit_stub_test forbidden` + trash/webhooks suites | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --test <surface>_stub_test` (per-plan suite)
- **Per wave merge:** `cargo test` (full suite)
- **Phase gate:** full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/webhooks_stub_test.rs` — WHOK-01..03 (secret-truncation test as first commit of Plan 1)
- [ ] `tests/trash_stub_test.rs` — TRSH-01..03 (purge zero-HTTP refusal pinned)
- [ ] `tests/audit_stub_test.rs` — AUDT-01..02
- [ ] No framework install needed — assert_cmd/common helpers exist

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no (existing API-key auth untouched) | — |
| V3 Session Management | no (stateless CLI) | — |
| V4 Access Control | yes (display side) | Server enforces; CLI renders surface-specific 403 hints (api/mod.rs:92-100) — never masks 403 as 404 |
| V5 Input Validation | yes | Allow-list: 13 event names, trash type tokens, audit filter enums passthrough; reject-early exit 2; no string interpolation into URLs (format! with normalized tokens only) |
| V6 Cryptography | no | CLI performs no crypto; HMAC signing is server-side (deliver.ts) |
| V7 Error/Logging | yes | Secrets never logged/cached (WHOK-03); errors carry hints, never echo secrets |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Secret leakage to disk (cache/completions/log) | Information Disclosure | Cache stores `(id, url)` only; test asserts cache dir free of 64-hex strings after create |
| Secret truncation in table renderer → silent mis-copy | Tampering (user pastes broken secret → signature mismatch confusion) | Own-line println; Wave-0 test asserts full 64 chars |
| Purge as id-enumeration oracle | Information Disclosure | Server gates before lookup (non-admin always 403); CLI must not pre-probe ids before the gate either (fan-out lists first — list is scoped legitimately) |
| Empty/oversized filter injection into query | Tampering | Passthrough + server 422s; omit empty; `limit` clamped client-side to ≤100 to match server |
| Purge accidental mass deletion | Elevation/Destruction | Strongest confirm wording naming scope + count; exit-2 no-input refusal; --force explicit |

## Sources

### Primary (HIGH confidence — read directly this session)
- Server: `src/app/api/v1/webhooks/route.ts`, `webhooks/[id]/route.ts` — full CRUD contracts
- Server: `src/app/api/v1/trash/route.ts`, `trash/[type]/[id]/route.ts`, `trash/[type]/[id]/restore/route.ts` — full trash contracts
- Server: `src/app/api/v1/audit/route.ts`, `src/db/schema/audit-log.ts` — audit contract + enums
- Server: `src/lib/events/types.ts:40-54`, `src/lib/events/subscribers/webhook.ts:31-47` — 13 events (compiler-enforced)
- Server: `src/db/schema/webhooks.ts` — no description column; secret column shape
- Server: `src/lib/api/pagination.ts` — DEFAULT 50 / MAX 100 / MAX_OFFSET 1e6
- Server: `src/lib/trash/entity-types.ts:25`, `src/lib/trash/queries.ts:34` — TRASH_TABS, page size 50
- CLI: `src/api/mod.rs:92-100` (hint table), `src/error.rs:87-93` (exit codes), `src/batch.rs:142-193` (confirm), `src/commands/notes/mod.rs:45-60` (normalization), `src/cli/templates.rs`, `src/commands/templates/delete.rs`, `src/cache.rs:20-33`, `src/context.rs:41`, `src/output/table.rs:49-51`, `tests/common/mod.rs`
- Planning: `.planning/phases/11-webhooks-trash-audit/11-CONTEXT.md`, `.planning/research/SERVER-API-DIFF.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md` (Phase 11 section)

### Secondary (MEDIUM confidence)
- None needed — no external/ecosystem claims.

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- API contracts: HIGH — every endpoint read from server source with line citations this session
- 13-event list: HIGH — compiler-enforced twice server-side (CrmEventMap + ALL_EVENT_FLAGS Record)
- CLI patterns: HIGH — all precedents read from this repo
- Pitfalls: HIGH — each traced to a specific line or schema fact

**Research date:** 2026-09-03
**Valid until:** 2026-10-03 (server is local & stable; re-verify only if server repo advances the trash/webhook/audit routes)

### Surprises (requested return items)
- **S1 — No webhook `description` field server-side** (DB + zod). CONTEXT's `--description` is unimplementable; recommend dropping (Open Question 1).
- **S2 — No bulk purge endpoint**: `trash purge [--type]` is a CLI fan-out of per-record DELETEs; abort-on-first-403 design.
- **S3 — Server webhook PUT is already partial-merge**: locked get→merge→PUT is harmless redundancy, not a hazard fix.
- **S4 — Webhooks have NO admin bypass**: even admin keys get 403 on foreign webhooks (unlike workflows' 404-hiding and unlike trash/audit admin powers).
- **S5 — Audit gate precedes filter validation** (non-admin learns nothing from bad filters — test asserts this ordering).
- **S6 — `deleted_by` has NO api_key name field by design** (trash/route.ts:107-113); `meta.total` on trash is per-selected-tab; past-cap offset returns empty data with truthful total.
- **S7 — Purge exit-2 mechanism**: refusal must use `InvalidInput` (exit 2), NOT the batch-delete `Validation` (exit 1) — the two contracts live 10 lines apart in the same error enum.
