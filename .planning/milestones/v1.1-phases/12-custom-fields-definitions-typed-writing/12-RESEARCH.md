# Phase 12: Custom Fields — Definitions & Typed Writing - Research

**Researched:** 2026-09-04
**Domain:** Custom field definition CRUD (REST) + type-aware value writing (CLI-side resolution against cached definitions)
**Confidence:** HIGH — every server claim below was verified by reading the server source at `/home/pedro/programming/pipelite` with file:line citations; every CLI claim verified against this repo.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Top-level `custom-fields` group** (list/get/create/update/delete); `--entity-type` filter on list.
- **Create input**: `--entity-type` + `--key` + `--type` (text/number/boolean/date/select/multi_select) + type-specific flags (`--options a,b,c` for select/multi_select, `--description`, `--position`); `--stdin` JSON for the full config shape.
- **Soft-delete marker**: researcher/planner pin the server behavior from source first; if soft-deleted definitions linger, list shows them with a `deleted` column rather than hiding.
- **Delete**: confirm_destructive contract (dry-run → TTY → `--force`; non-TTY refusal exit 1 Validation).
- **Definition resolution**: per-entity-type cache (`KEY_CUSTOM_FIELDS_<type>`), fetched on miss; `--dry-run` NEVER fetches — cache-only fallback with a visible stderr note (criterion 4; quiet-suppressible).
- **Inference rules**: number → JSON number (non-numeric input → exit 2 + hint naming the field's type); boolean → strict `true`/`false`; date → ISO string passthrough; select → string (server validates options); multi_select → comma-split to JSON array (`tags=a,b` → `["a","b"]`); text → string.
- **Unknown keys** (not in cached definitions): send as raw string + one stderr warning "not a defined field — sent as string" (quiet-suppressible).
- **`--custom-field-json`**: raw passthrough, verbatim object (criterion 3); BOTH it and `--custom-field` k=v entries → exit 2 pre-HTTP (no silent merge).
- **2 plans**: (1) definitions CRUD group; (2) typed-writing resolver rewiring the 8 existing create/update handlers (isolated for review/verification).
- **Shared resolver**: one module consumed by all 8 handlers (per-entity cache keys parameterized) — no duplication (WR-05 lesson).
- **Wire format unchanged**: `custom_fields` object in create/update bodies; only VALUE types become correct.
- **Docs in-phase**: api-reference Custom Fields section + CHANGELOG behavior note ("custom-field values are now type-correct").

### the agent's Discretion
- Internal resolver module shape/signature (research recommends below)
- Exact flag naming for definitions CRUD beyond what CONTEXT pins
- Table column choices for `custom-fields list`

### Deferred Ideas (OUT OF SCOPE)
- CFLD-04 (`--custom-field-string` force-string flag) — future milestone
- Custom field definition reordering UI / bulk operations — not in requirements
</user_constraints>

## Summary

This phase has two halves. **Definitions CRUD** is a routine top-level command group against `/api/v1/custom-field-definitions` — the server contract is small and fully pinned below. **Typed writing** replaces the string-only `batch::parse_custom_fields` (src/batch.rs:406-421) with a definitions-aware resolver used by all 8 create/update handlers, fixing the wire-level bug where `--custom-field price=4` stores string `"4"` instead of number `4`.

The server-source verification produced **four findings that change CONTEXT assumptions** and the planner must surface them:

1. **The v1 API validates NOTHING on custom field values.** `validateFieldValues` (server `src/lib/custom-fields.ts:33-113`) is called only from `saveFieldValues` — the session-authenticated UI path. Zero v1 routes reference it (grep across `src/app/api/v1/**` found no callers). The zod schema is `custom_fields: z.record(z.string(), z.unknown())` on all 8 entity routes. CONTEXT's inference rule "select → string (server validates options)" rests on a **false premise**: the server will silently accept invalid select options via the API-key path. The CLI must do all type/option checking client-side.
2. **Soft-deleted definitions are listed but indistinguishable.** The list route deliberately includes soft-deleted rows ("include deleted fields for API completeness", `route.ts:37-38`) but the serializer (`serialize.ts:143-156`) **omits `deleted_at`**. CONTEXT's "list shows them with a `deleted` column" is **not implementable** from current server data.
3. **Blob keys are definition NAMES, not IDs.** `validateFieldValues` indexes `values[def.name]` (custom-fields.ts:41) and formula resolution uses a `byName` map (formula-recalc.ts:518). `.planning/research/SERVER-API-DIFF.md` §C ("keys = field IDs") is **wrong**. The CLI's existing `key=value` behavior is already name-keyed — correct by accident, now confirmed.
4. **CONTEXT's `--description` flag has no server field.** The create schema is `{name, entity_type, type, config?, required?, show_in_list?}` — no description exists on the model or serializer. Similarly, `--position` on **create** is useless: the POST schema has no position field (zod strips it) and the server auto-assigns max+10000. Position is only writable via PUT.

**Primary recommendation:** Build Plan 1 as a standard top-level group modeled on webhooks (delete contract verbatim from `webhooks/delete.rs`), minus `--description`, with `--position` on update only and `--type` values matching the server enum (`single_select`, with `select` accepted as an alias). Build Plan 2's resolver in a new `src/custom_fields.rs` with per-entity cache keys, cache-only dry-run path, strict client-side typing for number/boolean, comma-split arrays for multi_select, and client-side option validation for select (since the server won't do it). Assert everything at the wire level using the existing `tests/common` head+body capturing stub server.

## Phase Requirements

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CFLD-01 | User can list, get, create, update, and delete custom field definitions (`custom-fields` group, `--entity-type` filter on list) | Full contract pinned in "Definition CRUD Contract" section: routes, schemas, serializer fields, status codes, soft-delete behavior, delete-confirmation analog (`webhooks/delete.rs:21-69`) |
| CFLD-02 | `--custom-field key=value` writes type-correct JSON resolved from cached definitions | Server validation matrix (server validates nothing on v1 → all checks client-side); inference rules table; resolver design; handler rewiring inventory (8 sites, exact lines) |
| CFLD-03 | User can bypass type inference with `--custom-field-json '{"key": ...}'` | Raw-passthrough precedent (`post_workflow_template_raw`, api/mod.rs:982-992; `templates/create.rs:192` parse_json_flag); exclusivity → exit 2 via `CliError::InvalidInput` (error.rs:253-258) |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Definition CRUD (list/get/create/update/delete) | API / Backend (server endpoints) | CLI (flags, rendering, confirm gate) | Server owns schema + storage; CLI owns presentation and the destructive-delete gate |
| Type inference for k=v values | CLI (client-side) | — | **Server does zero validation on v1 path** — the CLI is the only line of defense for type correctness |
| Select-option validation | CLI (client-side) | Server UI-path only | `validateFieldValues` runs only on the session UI path, never on API-key writes |
| Definition caching | CLI (file cache) | — | TTL JSON cache at `~/.pipelite/cache/` mirrors pipelines/stages precedent |
| Formula-key exclusion | API / Backend (server strips) | CLI (warn) | Server strips formula keys on v1 writes (T-34-04); CLI should warn the value will be ignored |
| Value storage (jsonb blob, name-keyed) | Database / Storage (server) | — | `custom_fields` jsonb on each entity table; keys = definition names |

## Server Contract Findings (all verified against `/home/pedro/programming/pipelite`)

### Definition CRUD Contract (file:line)

| Operation | Route | Schema / Behavior | Citation |
|-----------|-------|-------------------|----------|
| LIST | `GET /api/v1/custom-field-definitions?entity_type={organization\|person\|deal\|activity}&offset&limit` | `entity_type` optional; invalid value → 422 RFC 7807. **Includes soft-deleted rows** (no `isNull(deletedAt)` filter). Order: `position ASC, createdAt DESC`. Envelope `{data:[...], meta:{total,offset,limit}}` + `X-Total-Count`. Default limit 50, max 100. | `src/app/api/v1/custom-field-definitions/route.ts:24-59` (include-deleted comment at 37-38; order at 44; 422 at 33-35) |
| CREATE | `POST /api/v1/custom-field-definitions` | Body `{name (min 1), entity_type (enum4), type (enum10), config? (object, nullable), required?, show_in_list?}`. **`config` is `z.record(z.unknown())` — NOT validated against type** (any shape accepted). `required`/`show_in_list` default false (`required: required \|\| false`). **`position` auto-assigned max+10000 (first = 10000); a position in the POST body is stripped by zod.** Returns 201 `{data:{...}}`. | `route.ts:14-21` (schema), `route.ts:85-90` (position), `route.ts:93-105` (defaults, insert) |
| GET | `GET /api/v1/custom-field-definitions/{id}` | Returns soft-deleted rows too (no `deletedAt` filter on lookup). 404 when absent. | `[id]/route.ts:25-39` |
| UPDATE | `PUT /api/v1/custom-field-definitions/{id}` | Body `{name?, config?, required?, show_in_list?, position? (z.number)}`. **`entity_type` and `type` are immutable** (absent from schema → silently ignored, not an error). `position` stored as string. **PUT has no `deletedAt` guard — updating a soft-deleted definition silently "succeeds".** | `[id]/route.ts:12-18` (schema), `47-49` (no deleted filter), `81-85` (mapping), `87-92` |
| DELETE | `DELETE /api/v1/custom-field-definitions/{id}` | **Soft delete** — sets `deletedAt = new Date()`. **404 if already deleted** (lookup includes `isNull(deletedAt)`). 204 on success. | `[id]/route.ts:97-117` (guard at 102-108, soft delete at 111-113) |

**Serializer** (`src/lib/api/serialize.ts:143-156`): `{id, entity_type, name, type, config, required, position, show_in_list, created_at, updated_at}` where `position = cfd.position ? parseFloat(cfd.position) : null` — **a JSON float, or null**. No `deleted_at`, no `description`.

**`type` enum (10 values, wire-exact):** `text, number, date, boolean, single_select, multi_select, file, url, lookup, formula` (`route.ts:17`, `db/schema/custom-fields.ts:5-9`).

### Soft-Delete Answer (pinned — resolves the CONTEXT research flag)

- DELETE is a **soft delete**: sets `deletedAt` (`[id]/route.ts:111-113`). Values already stored in entity blobs are **not touched** — data using the field remains.
- **LIST includes soft-deleted definitions** (deliberate, commented "for API completeness", `route.ts:37-38`).
- **BUT the serializer omits `deleted_at`** (`serialize.ts:143-156`). There is **no field in any response** that distinguishes a deleted definition from a live one.
- Consequence: CONTEXT's decision "list shows them with a `deleted` column" **cannot be implemented**. Options for the planner:
  - **(Recommended)** List shows all rows with no deleted column; help text / docs note that deleted definitions remain listed and are indistinguishable via the API. Re-fetch-on-delete behavior (invalidate cache) keeps the resolver from going stale.
  - Alternative: probe each row with DELETE to detect tombstones — **rejected**, destructive and absurd.
- Additional wrinkle: DELETE on an already-deleted definition → 404; PUT on a soft-deleted definition succeeds. The CLI delete contract should treat 404 as the standard NotFound error path (no special casing).

### multi_select Config Answer (pinned — resolves the CONTEXT research flag)

- Config shape for **both** select types: `SelectConfig = { options: string[] }` (`db/schema/custom-fields.ts:12`). Options live at **`config.options`**, a flat JSON array of strings. `single_select` and `multi_select` share the shape (validation handles both identically, `custom-fields.ts:70-92`).
- The server does **not** validate that `config.options` exists or is well-formed at definition create time (`z.record(z.unknown())`, `route.ts:18`). A CLI create with `--options a,b,c` must build `{"options":["a","b","c"]}` itself.
- Stored **value** shape for multi_select: a JSON **array of strings** (`validateFieldValues` iterates `Array.isArray(value) ? value : [value]`, custom-fields.ts:82; UI `MultiSelectField` is `string[]` typed and explicitly "guards against strings stored by CSV import" — multi-select-field.tsx:19-27 — i.e. the exact bug class this phase fixes).
- Stored value shape for single_select: a **string** (legacy numeric IDs tolerated by the UI validator, custom-fields.ts:76-79).

### Server-Side Value Validation Matrix

The decisive fact: **`custom_fields` passes through the v1 API essentially unvalidated.** `validateFieldValues` has exactly one caller — `saveFieldValues` (session UI path). Zero v1 entity/definition routes call it.

| Definition type | v1 create/update (API-key — what the CLI hits) | UI session path (`saveFieldValues`) | What the server STORES |
|---|---|---|---|
| (schema gate) | `z.record(z.string(), z.unknown())` — any keys, any JSON values | same | whatever JSON is posted, into `custom_fields` **jsonb** |
| number | **no check** | number or numeric string (`isNaN(Number(v))` → error, custom-fields.ts:54-58) | as posted |
| single_select | **no check** | string ∈ `config.options` (custom-fields.ts:75-80) | as posted |
| multi_select | **no check** | each string element ∈ `config.options` (custom-fields.ts:81-89) | as posted |
| boolean | **no check** (no switch case) | **no check** (no switch case) | as posted |
| date | **no check** | **no check** (no case) | as posted (string) |
| text | **no check** | **no check** | as posted |
| url | **no check** | `new URL(value)` parse check (custom-fields.ts:60-68) | as posted |
| lookup | **no check** | FK existence check vs target table (custom-fields.ts:94-108) | as posted |
| required fields | **not enforced** | enforced (custom-fields.ts:44-47) | — |
| formula keys | **stripped server-side** before persist (T-34-04): `stripCallerFormulaKeys` → `stripFormulaKeys` on deals/deals-people/activities writes | stripped + recomputed by server | formula keys never persist from callers |

Formula-strip citations: `src/app/api/v1/deals/route.ts:56-69,297-299`; `deals/[id]/route.ts:297-305`; `people/route.ts:209-210`; `people/[id]/route.ts:224-225`; `activities/[id]/route.ts:314-323`. (Server inconsistency: `organizations/route.ts:94-102` passes custom_fields through **without** stripping on create — immaterial to the CLI but confirms server-side enforcement is best-effort.)

Update semantics: entity updates **merge** the blob — `{...existing, ...stripped}` (`deals/[id]/route.ts:301-304`, `activities/[id]/route.ts:314-323`); keys cannot be deleted via API, only overwritten. Create replaces (fresh blob).

**Key convention (binding for the resolver):** blob keys are **definition NAMES** — `validateFieldValues` reads `values[def.name]` (custom-fields.ts:41); formula references resolve via a byName map (formula-recalc.ts:518). SERVER-API-DIFF.md §C's "keys = field IDs" is wrong.

**Implication for CONTEXT decisions:** the locked inference "select → string (server validates options)" has a false premise on the API-key path. Recommendation (flagged NEEDS DECISION below): keep "send as string" but add **client-side option validation** against the cached definition's `config.options` → exit 2 pre-HTTP naming the valid options, mirroring the number rule. Same logic argues for client-side boolean strictness (already in CONTEXT) — nothing else will catch it.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| (none — zero new dependencies) | — | — | Everything needed is already in tree: `serde_json` (Value/Number), `chrono`, `anyhow`, `reqwest`, `clap`, `dialoguer`, `tempfile` (dev) |

**No external packages are installed this phase.** The Package Legitimacy Audit below is trivially satisfied.

### Supporting (existing, reused)
| Facility | Location | Purpose |
|----------|----------|---------|
| `CacheStore.get/set/invalidate/invalidate_prefix` | src/cache.rs:89-166 | Definition caching; `invalidate_prefix("custom_fields_")` on any definition mutation |
| `spawn_head_capturing_stub_server` | tests/common/mod.rs:49-163 | Wire-level body assertions (records full request bytes) |
| `post_workflow_template_raw` precedent | src/api/mod.rs:982-992 | `--stdin` verbatim JSON passthrough |
| `parse_json_flag` precedent | src/commands/templates/create.rs:192 | Parsing a JSON-object flag value |
| Webhooks delete contract | src/commands/webhooks/delete.rs:21-69 | The confirm_destructive analog (dry-run → TTY Confirm default false → non-TTY Validation refusal) |
| Cache-through fetch + auto-paginate | src/prompt.rs:162-187 (`get_pipelines_cached`) | Cache-first resolution pattern the resolver mirrors |
| HOME-redirected hermetic tests | tests/webhooks_stub_test.rs:30-36, tests/cache_test.rs:40 | Isolating `~/.pipelite/cache` in integration tests |

## Package Legitimacy Audit

**No new external packages** are installed in this phase — the audit gate is trivially satisfied. All functionality builds on existing in-tree dependencies (serde_json, reqwest, clap, dialoguer, anyhow, chrono) and the hand-rolled TcpListener stub-server test pattern (no new crates policy, tests/common/mod.rs:1-8). Nothing to run slopcheck against; nothing to gate behind `checkpoint:human-verify`.

## Architecture Patterns

### System Architecture Diagram

```
                        Plan 1: definitions CRUD
  ┌──────────┐  flags/--stdin  ┌──────────────┐  HTTPS  ┌─────────────────────────────────┐
  │   user   │ ──────────────▶ │ custom-fields │ ──────▶ │ /api/v1/custom-field-definitions │
  └──────────┘                 │  group (new)  │         │  (CRUD; DELETE=soft; list incl.  │
     │  delete                 └──────┬───────┘         │   tombstones; serializer has NO  │
     │ confirm gate                   │ invalidate       │   deleted_at)                    │
     └───────────────────────────────┤ prefix           └───────────────┬─────────────────┘
                                     ▼                                  │
                          ~/.pipelite/cache/custom_fields_<entity>.json ◀─ fetched on miss
                                     │
                        Plan 2: typed writing
  ┌──────────┐  --custom-field  ┌───────────────────┐   cache hit?  ┌───────────────┐
  │   user   │ ───────────────▶ │ resolver           │◀── yes ──────│ cached defs    │
  └──────────┘  k=v (+ --json) │ src/custom_fields.rs│              └───────────────┘
     │                         │  infer types,      │◀── no + --dry-run ⇒ cache-ONLY:
     │                          │  validate, warn    │     strings + stderr note, 0 HTTP
     │                          └─────────┬─────────┘◀── no + live  ⇒ GET list (paginate),
     │                                    │                              then cache
     │                                    ▼ typed Value object
  │                    ┌────────────────────────────┐        ┌──────────────────────────┐
  └───────────────────▶│ 8 create/update handlers    │ ─PUT/─POST────▶│ /api/v1/{deals|organizations│
                       │ (existing, rewired)         │                │ |people|activities}         │
                       └────────────────────────────┘        └────────────┬─────────────┘
                                                    server merges blob (update) / stores jsonb
                                                    strips formula keys, validates NOTHING
```

Primary use case trace: `deals update d1 --custom-field price=4` → handler calls resolver → resolver reads cached deal definitions (fetching first if cold and not dry-run) → `price` matches a `number` definition → parses `4` as JSON number → handler embeds typed object in `DealUpdate.custom_fields` → PUT body carries `"custom_fields": {"price": 4}`.

### Recommended Project Structure

```
src/
├── custom_fields.rs        # NEW (Plan 2): EntityType enum, FieldDefinition model,
│                           #   cache key constants co-located OR in cache.rs, resolver fn,
│                           #   infer_typed_value(), unit tests
├── cache.rs                # + KEY_CUSTOM_FIELDS_DEAL/ORG/PEOPLE/ACTIVITY, TTL_CUSTOM_FIELDS
├── api/
│   ├── mod.rs              # + 5 client methods for definitions CRUD (Plan 1)
│   └── models.rs           # + CustomFieldDefinition struct (position: f64 FROM BIRTH)
├── cli/
│   └── custom_fields.rs    # NEW (Plan 1): CustomFieldsCommands enum + args structs
└── commands/
    └── custom_fields/      # NEW (Plan 1): mod.rs, list.rs, get.rs, create.rs, update.rs, delete.rs
tests/
├── custom_fields_stub_test.rs      # Plan 1: definitions CRUD wire-level
└── typed_writing_stub_test.rs      # Plan 2: wire-level body assertions per type
```

(Naming/placement is discretion — the hard constraint is **one shared resolver module** consumed by all 8 handlers, not per-entity copies.)

### Pattern 1: Typed-Writing Resolver (the heart of Plan 2)

**What:** One async function that turns `--custom-field k=v` pairs into a typed `serde_json::Value` object, using cached definitions as the type source.

**Recommended shape:**

```rust
// src/custom_fields.rs (sketch — planner finalizes)
pub enum CfEntityType { Deal, Organization, Person, Activity }
// maps: (cli subcommand, api entity_type, cache key)
//   Deal         -> "deals"   -> "deal"          -> KEY_CUSTOM_FIELDS_DEAL
//   Organization -> "orgs"    -> "organization"  -> KEY_CUSTOM_FIELDS_ORG
//   Person       -> "people"  -> "person"        -> KEY_CUSTOM_FIELDS_PEOPLE
//   Activity     -> "activities" -> "activity"   -> KEY_CUSTOM_FIELDS_ACTIVITY

pub struct FieldDefinition {
    pub id: String,
    pub entity_type: String,
    pub name: String,
    pub type_: String,               // server enum (10 values)
    pub config: Option<serde_json::Value>,
    pub required: bool,
    pub position: Option<f64>,       // numeric(20,10) + parseFloat — f64 FROM BIRTH
    pub show_in_list: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Resolve --custom-field k=v pairs against definitions into a typed JSON object.
pub async fn resolve_custom_fields(
    ctx: &AppContext,
    entity: CfEntityType,
    pairs: &[String],
) -> Result<Option<serde_json::Value>>
```

Resolution algorithm:
1. Definitions lookup:
   - If `ctx.dry_run`: **cache only** — `cache.get(KEY_CUSTOM_FIELDS_*)`. On miss: emit stderr note "custom-field definitions not cached — values sent as raw strings (run once without --dry-run to warm the cache)" (suppressed when `ctx.quiet`), treat every key as unknown, return string-valued object. **Zero HTTP under dry-run, ever.**
   - Else: cache-first; on miss GET the list (auto-paginate, limit 100 — mirrors `get_pipelines_cached`, prompt.rs:162-187), cache with `TTL_CUSTOM_FIELDS` (3600 — definitions change rarely, mirrors pipelines/stages).
2. For each `k=v` pair: split on first `=` (reuse the existing error for missing `=`, batch.rs:413-416); look up `k` among definitions **by name**.
3. Known key → infer by `definition.type` (table below); inference failure → `CliError::InvalidInput` (exit 2) with detail naming the field and its type, hint naming expected input shape. formula type → hard error or warning (see NEEDS DECISION).
4. Unknown key → `Value::String(v)` + one stderr warning "not a defined field — sent as string" (aggregate to ONE warning per invocation, suppressed by `--quiet`).
5. Return `Some(object)` if any pairs, else `None`.

**Inference rules (recommendation per type):**

| definition.type | CLI input | Wire value | On bad input |
|-----------------|-----------|------------|--------------|
| `text` | any | `Value::String` | never fails |
| `number` | `4`, `4.5`, `-3` | `Number` — parse **i64 first** (`Number::from`), fall back to f64 (`Number::from_f64`) so `price=4` emits exactly `4` on the wire, never `4.0` | `exit 2` InvalidInput; hint names field + "expects a number" |
| `boolean` | `true` / `false` (strict, lowercase) | `Value::Bool` | `exit 2`; hint "expects true or false" |
| `date` | ISO string | `Value::String` passthrough | never fails (server never validates dates — custom-fields.ts has no date case) |
| `single_select` | string | `Value::String`; **validate against `config.options` client-side** → exit 2 listing valid options (NEEDS DECISION: strict vs warn) | see left |
| `multi_select` | `a,b` | `Value::Array` of `Value::String` (comma-split; empty segments skipped) | never fails on shape; commas-in-values must use `--custom-field-json` (document) |
| `url` | string | `Value::String` (treat as text; optional client URL check is discretion) | never fails |
| `lookup` | entity ID string | `Value::String` | never fails |
| `file` | string descriptor | `Value::String` | never fails |
| `formula` | any | **server strips it** (T-34-04) — recommend stderr warning "formula field — server-computed; value will be ignored" and send anyway (or refuse — NEEDS DECISION) | warning path recommended |
| (unknown key) | any | `Value::String` + one stderr warning | never fails |

**When to use:** all 8 flag-path create/update handlers. NOT the `--stdin` batch paths (those deserialize typed `*Create`/`*Update` structs — user JSON is already typed and passes through verbatim; extending inference there is out of scope per CONTEXT's "rewiring the 8 existing create/update handlers").

### Pattern 2: `--custom-field-json` Raw Bypass (CFLD-03)

- New `#[arg(long = "custom-field-json")] pub custom_field_json: Option<String>` on all 8 arg structs.
- Parse with the `parse_json_flag` pattern (templates/create.rs:192): `serde_json::from_str::<serde_json::Value>` → must be `Value::Object` (arrays/scalars → `CliError::InvalidInput`, exit 2).
- Insert verbatim into the body's `custom_fields` — no type touch (verbatim-object precedent: `post_workflow_template_raw`, api/mod.rs:982-992).
- **Exclusivity:** if BOTH `--custom-field` and `--custom-field-json` present → `CliError::InvalidInput` (exit 2) **pre-HTTP**. Note CONTEXT says exit 2 explicitly — use `InvalidInput`, **not** `Validation` (some existing exclusivity checks use Validation/exit 1, e.g. deals/create.rs:33-37 — do NOT copy that; `collect_delete_ids` uses InvalidInput correctly, batch.rs:106-110).
- Also thread the new flag into all 12 existing `has_flags` mutual-exclusivity sites (see inventory) so `--custom-field-json --stdin` is rejected like `--custom-field --stdin`.

### Pattern 3: Definitions CRUD group (Plan 1)

Follows the templates/webhooks group pattern exactly (cli/mod.rs Commands enum + after_help, main.rs match arm, commands/<entity>/{mod,list,get,create,update,delete}.rs):

- **list**: `--entity-type` (optional; client-side enum check `organization|person|deal|activity` for a good hint, else server 422), `--limit/--offset` + standard output renderers. Columns: `id, entity_type, name, type, position, required, show_in_list`.
- **get**: by ID.
- **create**: `--entity-type` (required) `--key <name>` (required) `--type <t>` (required) `--options a,b,c` (builds `config.options`) `--required` `--show-in-list`; `--stdin` = full verbatim body (raw passthrough). **Recommend dropping `--description`** (no server field — CONTEXT mismatch) **and not offering `--position` on create** (server strips it and auto-assigns max+10000).
- **update**: `--name` `--config '<json>'` `--required/--no-required` `--show-in-list/--no-show-in-list` **`--position <f64>`** (PUT-only — this is where position is writable); `--stdin` verbatim.
- **delete**: webhooks delete contract **verbatim** (webhooks/delete.rs:21-69): dry-run intercept FIRST → TTY `dialoguer::Confirm` default false → non-TTY without `--force` → `CliError::Validation` refusal (exit 1) → delete → `cache.invalidate_prefix("custom_fields_")` → quiet-suppressed confirmation line.
- **Cache invalidation:** every successful definition create/update/delete invalidates the whole `custom_fields_` prefix (definitions are global per entity type; there is no per-key cache file to surgically invalidate).

**`--type` naming (NEEDS DECISION):** CONTEXT says `select`; the server enum value is `single_select`. Recommendation: accept `select` as an alias that maps to `single_select` on the wire, list the server's raw enum strings in list/get output; help text shows both.

### Anti-Patterns to Avoid
- **Fetching definitions under `--dry-run`:** violates criterion 4. Cache-only, always.
- **Per-entity resolver copies:** WR-05 lesson — one module, parameterized entity.
- **`serde_json::json!(4.0)` for integers:** emits `4.0` on the wire. Parse i64-first so `4` stays `4`. The wire test must assert `is_number()` AND `== 4` AND `as_i64() == Some(4)`.
- **Silent merge of `--custom-field` and `--custom-field-json`:** CONTEXT forbids it; exit 2 pre-HTTP.
- **Using `CliError::Validation` for the exclusivity check:** that's exit 1; CONTEXT demands exit 2 (`InvalidInput`).
- **Keying resolution by definition ID:** blob keys are names (custom-fields.ts:41).
- **Trust list to exclude tombstones:** it doesn't, and you can't detect them — don't add dead "deleted" column code.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Typed-value JSON construction | string manipulation / manual JSON escaping | `serde_json::Value` + `Number::from/from_f64` | escaping, precision, i64-vs-f64 distinction handled |
| HTTP stubbing for tests | a mocking framework / new dev-dep | `tests/common::spawn_head_capturing_stub_server` | already records heads + full bodies; zero new crates |
| Delete confirmation flow | new confirm helper | copy webhooks/delete.rs contract | it IS the pinned contract (CONTEXT names it) |
| Verbatim JSON passthrough | partial re-serialization through typed structs | raw `serde_json::Value` client method (post_workflow_template_raw pattern) | typed models drop unknown keys — destroys "verbatim" |
| Cache TTL/atomicity | custom cache | `CacheStore` + `invalidate_prefix` | atomic writes, corruption handling already proven |
| Auto-pagination | one-shot list assumptions | get_pipelines_cached loop pattern (prompt.rs:173-187) | server default limit 50/max 100 — a resolver fetching one page silently misses definitions |

**Key insight:** the data-correctness bug this phase fixes exists precisely because the server treats `custom_fields` as an opaque bag. Nothing downstream will ever type-check for the CLI — the resolver is the only validation layer, so it must be strict where CONTEXT says strict and explicit (warnings) where it isn't.

## Common Pitfalls

### Pitfall 1: Assuming the server validates values (CONTEXT's false premise)
**What goes wrong:** skipping client-side select-option validation because "server validates options".
**Why it happens:** CONTEXT inference note + `validateFieldValues` exists in the server codebase — but only on the session UI path.
**How to avoid:** client-side validation per the matrix above; wire tests assert invalid option → exit 2, zero HTTP.
**Warning signs:** a test asserting a 422 from the stub for bad options — no such server behavior exists to mirror.

### Pitfall 2: i64/f64 wire representation
**What goes wrong:** `--custom-field price=4` produces `4.0`.
**Why:** modeling all numbers as f64.
**How to avoid:** i64 parse first; f64 fallback; wire assertion `body["custom_fields"]["price"].as_i64() == Some(4)`.
**Warning signs:** assertion `== serde_json::json!(4)` passing while raw body shows `4.0` (json! comparison can mask the distinction depending on construction — assert on the raw captured body string too for the canonical test).

### Pitfall 3: `--dry-run` triggering a definitions fetch
**What goes wrong:** cold cache + `--dry-run` makes a GET — violates criterion 4 and the global "dry-run never does HTTP" convention.
**How to avoid:** resolver checks `ctx.dry_run` before any cache miss path; integration test with request counter == 0.
**Warning signs:** any `client.` call reachable from the dry-run branch.

### Pitfall 4: Exit-code drift on exclusivity/precondition errors
**What goes wrong:** both-flags error returns exit 1 (Validation) — CONTEXT says exit 2.
**How to avoid:** use `CliError::InvalidInput` for: both-flags, custom-field-json-not-object, bad number, bad boolean, select option mismatch. Reserve `Validation` (exit 1) for the delete non-TTY refusal (per CONTEXT).
**Warning signs:** copying deals/create.rs:33-37 (Validation) as the exclusivity template.

### Pitfall 5: Position typed as integer
**What goes wrong:** `position: Option<i64>` breaks on `position: 10000.5` (the exact Phase 8 `Deal.position` failure class).
**How to avoid:** `Option<f64>` from birth (note already planted at models.rs:54-56).
**Warning signs:** deserialization errors on fractional positions from the stub.

### Pitfall 6: Forgetting the activities `--mark-undone` raw path
**What goes wrong:** resolver output computed but dropped, or applied twice, on the `update_with_null_completed` path (activities/update.rs:75-79 → 151-199) which builds a raw payload and uses `update_activity_raw`.
**How to avoid:** resolve once at activities/update.rs:75 and pass the typed Value through the existing `custom_fields` parameter into both paths; the raw path already inserts it at line 181-182 unchanged.
**Warning signs:** grep showing only 7 rewired call sites instead of 8.

### Pitfall 7: Missing `has_flags` updates for `--custom-field-json`
**What goes wrong:** `deals create --custom-field-json '{...}' --stdin` slips through the mutual-exclusivity gate (12 sites currently check only `!args.custom_field.is_empty()`).
**How to avoid:** add `|| args.custom_field_json.is_some()` at every site listed in the inventory.
**Warning signs:** any has_flags expression still reading only `custom_field`.

### Pitfall 8: Resolver cache invalidation on definition mutation
**What goes wrong:** Plan 1 creates/updates/deletes a definition but Plan 2's resolver keeps serving the stale cached type (e.g. field retyped text→number).
**How to avoid:** all three mutating definition commands call `cache.invalidate_prefix("custom_fields_")`.
**Warning signs:** a definition mutation handler without a cache call.

### Pitfall 9: Building `config.options` wrongly on create
**What goes wrong:** `--options "a,b,c"` sent as a comma-string instead of `["a","b","c"]`, or `config` sent as a bare array instead of `{"options":[...]}`.
**How to avoid:** build `config: {"options": [..split..]}`; wire-test the POST body shape.
**Warning signs:** UI select fields rendering a single option containing commas.

## Code Examples

### Wire-level typed-writing test (canonical, Plan 2)
```rust
// Source: pattern from tests/templates_stub_test.rs + tests/common/mod.rs:49
let defs = r#"{"data":[{"id":"cf1","entity_type":"deal","name":"price","type":"number",
  "config":null,"required":false,"position":10000.0,"show_in_list":false,
  "created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}],
  "meta":{"total":1,"offset":0,"limit":100}}"#;
let created = r#"{"data":{"id":"d1","title":"T","custom_fields":{"price":4}}}"#;
let (base, counter, _heads, bodies) = spawn_head_capturing_stub_server(&[(200, defs), (201, created)]);
let tmp = tempfile::TempDir::new().unwrap(); // cold cache ⇒ exercises fetch+cache path
cmd_with_server(&base)
    .env("HOME", tmp.path())
    .args(["deals", "create", "--title", "T", "--stage", "s1", "--custom-field", "price=4"])
    .assert().success();
let body = last_captured_body(&bodies);          // split at first "\r\n\r\n"
let json: serde_json::Value = /* parse body */;
assert_eq!(json["custom_fields"]["price"], serde_json::json!(4));
assert!(json["custom_fields"]["price"].is_number());
assert!(json["custom_fields"]["price"].as_i64().is_some()); // 4, not 4.0
assert_eq!(counter.load(Ordering::SeqCst), 2);   // GET defs + POST deal
```

### Dry-run never fetches (criterion 4)
```rust
// Empty HOME (cold cache), --dry-run: stub counter must stay 0.
cmd_with_server(&base).env("HOME", tmp.path())
    .args(["deals", "create", "--title", "T", "--stage", "s1",
           "--custom-field", "price=4", "--dry-run"])
    .assert().success();                          // strings + note path
assert_eq!(counter.load(Ordering::SeqCst), 0);
```

### Resolver cache-only dry-run branch (handler shape)
```rust
let custom_fields = if ctx.dry_run {
    custom_fields::resolve_cache_only(ctx, entity, &args.custom_field)?   // never fetches
} else {
    custom_fields::resolve_custom_fields(ctx, entity, &args.custom_field).await?
};
```

### Definition delete contract (Plan 1 — webhooks verbatim)
Source: `src/commands/webhooks/delete.rs:21-69`. Order: `ctx.dry_run` intercept → `!args.force` { TTY? `dialoguer::Confirm` default false, "Aborted" on decline : `CliError::Validation` refusal (exit 1) } → `client.delete_custom_field_definition(id)` → `cache.invalidate_prefix("custom_fields_")` → quiet-suppressed `Deleted custom field definition {id}`.

## Runtime State Inventory

Not a rename/refactor/migration phase — omitted. (Feature phase; no pre-existing stored state keyed on names introduced here. The only runtime artifact added is `~/.pipelite/cache/custom_fields_*.json`, which is new and self-managed.)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `batch::parse_custom_fields` — all values `Value::String` (batch.rs:406-421) | definitions-aware resolver producing typed `Value`s | this phase | fixes the `"4"` vs `4` data-correctness bug (ROADMAP Phase 12) |
| Server-API-DIFF assumption "keys = field IDs" | keys = definition **names** (pinned this research) | correction | resolver lookups match by `name` |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | CLI `--type select` should be accepted as alias for wire value `single_select` (CONTEXT names `select`; server enum has only `single_select`) | Definitions CRUD / NEEDS DECISION | Cosmetic mismatch; list output would show `single_select` either way |
| A2 | select-option mismatch should be a strict exit-2 client-side error (CONTEXT assumed server did it) | Inference rules | If user prefers warn-only, change is one match arm; wire tests adjusted |
| A3 | Writing to a `formula` definition should warn (value is server-stripped) rather than silently send | Inference rules | Cosmetic; worst case a silently-ignored value (today's behavior) |
| A4 | `TTL_CUSTOM_FIELDS = 3600` mirrors pipelines/stages precedent | Resolver design | Stale types up to 1h after out-of-band server changes; invalidation-on-mutation covers the CLI's own writes |
| A5 | Dropping `--description` (no server field) and create-time `--position` (server strips) from flag set | Definitions CRUD | Deviation from CONTEXT wording — flagged for planner/user confirmation; trivially addable as no-op flags if desired |

## Open Questions (RESOLVED — Q1 select strictness: STRICT exit 2 per orchestrator amendment, 12-02; Q2 formula keys: REFUSE exit 2, 12-02; Q3 --description/create-position omission: 12-01 amendments; Q4 select aliasing: normalize to single_select, 12-01)

1. **Select-option validation: strict (exit 2) or warn?** *(NEEDS DECISION)*
   - What we know: server does NOT validate on the API path (custom-fields.ts:33 called only from UI save). CONTEXT's rationale is void.
   - Recommendation: strict exit 2 listing valid options — consistent with the number rule and the "type-correct writes" success criterion.
2. **`formula` definitions: refuse or warn-and-send?** *(NEEDS DECISION)*
   - What we know: server strips formula keys on all v1 writes (T-34-04) — the value never persists.
   - Recommendation: stderr warning + send anyway (non-fatal, matches unknown-key treatment).
3. **`--description` and create `--position` flags:** omit (recommended — no server backing) or accept-and-ignore with a help note? Planner should confirm the CONTEXT deviation with the user.
4. **`--type` vocabulary:** alias `select`→`single_select` (A1) or expose raw server enums only?

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo/rustc | build + tests | ✓ | 1.94.0 | — |
| tempfile (dev-dep) | hermetic HOME in tests | ✓ (in tree) | — | — |
| Live server | none in CI/tests (stub servers used) | n/a | — | hand-rolled TcpListener stubs |

No blocking or missing dependencies. No external services required — all tests are hermetic (stub servers + redirected HOME).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]`/`#[tokio::test]` via `cargo test` + `assert_cmd` integration tests |
| Config file | none (Cargo default; tests/ integration binaries) |
| Quick run command | `cargo test custom_field` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CFLD-01 | list renders defs, sends `entity_type` query when filtered | integration (wire) | `cargo test --test custom_fields_stub_test` | ❌ Wave 0 |
| CFLD-01 | create POST body: name/entity_type/type/config.options array shape | integration (wire body) | same | ❌ Wave 0 |
| CFLD-01 | update PUT: name/config/required/show_in_list/position (f64 on wire); entity_type/type immutable (not sent) | integration (wire body) | same | ❌ Wave 0 |
| CFLD-01 | delete: dry-run zero-HTTP; non-TTY without --force refuses exit 1 zero-HTTP; --force → 204 + cache-prefix invalidated | integration (wire) | same | ❌ Wave 0 |
| CFLD-01 | --stdin create passes body verbatim (post_workflow_template_raw contract) | integration (wire body) | same | ❌ Wave 0 |
| CFLD-02 | number: `price=4` → body `4` (is_number + as_i64), `price=abc` → exit 2 zero-HTTP | integration (wire body) | `cargo test --test typed_writing_stub_test` | ❌ Wave 0 |
| CFLD-02 | boolean: strict true/false → `Value::Bool`; `yes` → exit 2 zero-HTTP | integration (wire body) | same | ❌ Wave 0 |
| CFLD-02 | multi_select: `tags=a,b` → `["a","b"]` array of strings | integration (wire body) | same | ❌ Wave 0 |
| CFLD-02 | date/text: string passthrough | integration (wire body) | same | ❌ Wave 0 |
| CFLD-02 | select option check per Open Question 1 outcome | integration (wire) | same | ❌ Wave 0 |
| CFLD-02 | unknown key → string value + one stderr warning (suppressed by --quiet) | integration (wire + stderr) | same | ❌ Wave 0 |
| CFLD-02 | resolver cache: miss → GET + cache write; second command → 1 request only; definition mutation invalidates prefix | integration (wire) | same | ❌ Wave 0 |
| CFLD-02 | --dry-run with cold cache: 0 HTTP requests, strings sent, stderr note (absent under --quiet) | integration (wire counter) | same | ❌ Wave 0 |
| CFLD-03 | --custom-field-json object passes through verbatim (nested object survives untouched) | integration (wire body) | same | ❌ Wave 0 |
| CFLD-03 | both flags → exit 2 pre-HTTP, 0 requests; non-object --custom-field-json → exit 2 | integration (wire counter) | same | ❌ Wave 0 |
| CFLD-02 | resolver unit tests: inference table, i64/f64 split, comma-split edge cases | unit | `cargo test custom_fields::` | ❌ Wave 0 (in src/custom_fields.rs) |
| — | 8th call site (activities --mark-undone raw path) carries typed values | integration (wire body) | `cargo test --test typed_writing_stub_test mark_undone` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test custom_field` (fast subset)
- **Per wave merge:** `cargo test` (full suite)
- **Phase gate:** full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/custom_fields_stub_test.rs` — definitions CRUD (CFLD-01), reuses tests/common
- [ ] `tests/typed_writing_stub_test.rs` — typed writing wire assertions (CFLD-02/03), reuses tests/common
- [ ] No framework install needed (`cargo test` + existing dev-deps cover everything)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | unchanged — Bearer key via existing client |
| V3 Session Management | no | CLI, no sessions |
| V4 Access Control | no | server-side scoping unchanged; CLI surfaces 403s (Forbidden variant exists) |
| V5 Input Validation | **yes** | the resolver IS the validation layer: strict type checks, option membership, JSON-object gate on `--custom-field-json` |
| V6 Cryptography | no | no crypto introduced |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Formula-key spoofing (server T-34-04: caller writes server-computed values) | Tampering | server strips (existing); CLI warns value will be ignored (A3) |
| Type-confusion in jsonb blob (string `"4"` poisoning numeric workflows/filters) | Tampering | the typed resolver itself — this phase's raison d'être |
| Config-file/cache tampering feeding wrong types | Tampering | cache corruption already handled (silently deleted on bad JSON, cache.rs:96-103); TTL bounds staleness |

## Sources

### Primary (HIGH confidence — server source read this session)
- `/home/pedro/programming/pipelite/src/app/api/v1/custom-field-definitions/route.ts` — list/create schema, soft-delete inclusion, position auto-assign
- `/home/pedro/programming/pipelite/src/app/api/v1/custom-field-definitions/[id]/route.ts` — get/put/delete, immutability, soft-delete guard
- `/home/pedro/programming/pipelite/src/lib/api/serialize.ts:143-156` — definition serializer (no deleted_at)
- `/home/pedro/programming/pipelite/src/db/schema/custom-fields.ts` — SelectConfig shape, jsonb config, numeric(20,10) position
- `/home/pedro/programming/pipelite/src/lib/custom-fields.ts` — validateFieldValues (UI-only), name keying, required enforcement
- `/home/pedro/programming/pipelite/src/lib/formula-recalc.ts:518` — byName resolution
- v1 entity routes (deals/orgs/people/activities + [id]) — `custom_fields` zod schemas, formula stripping, merge-on-update
- Per-type UI components (`boolean-field.tsx`, `multi-select-field.tsx`, `number-field.tsx`) — canonical value shapes

### Secondary (HIGH confidence — CLI source read this session)
- `src/batch.rs:406-421`, all 8 handlers, `src/cache.rs`, `src/api/mod.rs:925-1004`, `src/api/models.rs:44-56`, `src/commands/webhooks/delete.rs`, `src/commands/templates/create.rs:192`, `src/prompt.rs:162-187`, `tests/common/mod.rs`, `src/error.rs:87,222-271`, `src/cli/{deals,orgs,people,activities}.rs`
- `.planning/research/SERVER-API-DIFF.md` §C/§7 (used with one correction: name-keying)

### Tertiary
- None — no web sources needed; both codebases are local and authoritative.

## Metadata

**Confidence breakdown:**
- Server contracts: HIGH — every claim read from source with file:line; nothing from training data
- CLI inventory: HIGH — every call site grep-located and read
- Inference rules / resolver design: HIGH for mechanics (MEDIUM only for the three flagged NEEDS DECISION items)

**Surprises (for the orchestrator's return summary):**
1. v1 API validates nothing on custom-field values — `validateFieldValues` is UI-path-only; CONTEXT's "server validates options" premise is false.
2. List includes soft-deleted definitions but the serializer omits `deleted_at` — the "deleted column" decision is unimplementable.
3. Blob keys are definition names, not IDs (SERVER-API-DIFF was wrong).
4. CONTEXT's `--description` flag and create-time `--position` have no server backing (description doesn't exist; POST strips position and auto-assigns).
5. PUT succeeds on soft-deleted definitions (no guard); DELETE 404s on them.
6. Server strips formula keys on v1 writes — writing to a formula definition is a silent no-op value-wise.
7. Existing exclusivity checks are exit-code-inconsistent (Validation exit 1 in some places) — CONTEXT demands exit 2 for the both-flags rule; use `InvalidInput`.

**Research date:** 2026-09-04
**Valid until:** ~2026-10-04 (server source is local and stable; re-verify only if the server repo changes)
