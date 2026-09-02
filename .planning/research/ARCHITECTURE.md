# Architecture Research — v1.1 Feature Integration into Existing Rust CLI

**Domain:** Extending an established Rust clap/reqwest CLI (pipelite) with 8 new API surfaces + batch operations
**Researched:** 2026-09-02
**Confidence:** HIGH — grounded in direct reads of the current codebase (`src/api/mod.rs`, `src/cli/`, `src/commands/`, `src/cache.rs`, `src/error.rs`, `src/main.rs`), the drafted phase-01 plans, and SERVER-API-DIFF.md (verified against the server repo 2026-09-02). No external ecosystem research needed; every claim below traces to a repo file.

---

## System Overview (post-v1.1 target state)

```
┌────────────────────────────────────────────────────────────────────┐
│                       CLI Layer (clap derive)                      │
│  Existing: deals orgs people activities pipelines stages workflows │
│            cache config completions dashboard init ping            │
│  NEW:      notes  webhooks  trash  custom-fields  templates        │
│            audit (cmd)  docs (cmd)                                 │
│  EXTENDED: workflows (+ runs subcmd)  deals/orgs/people/activities │
│            (+ batch --stdin update, multi-ID delete)               │
├────────────────────────────────────────────────────────────────────┤
│                     Handlers (src/commands/<area>/)                │
│  one directory per command group; run(ctx, cmd) dispatch           │
├────────────────────────────────────────────────────────────────────┤
│              Shared Services (cross-cutting, reused)               │
│  ┌───────────┐ ┌──────────────┐ ┌──────────────────────────────┐   │
│  │ batch.rs  │ │custom_fields │ │ output/ (json/table/csv/plain│   │
│  │BatchOutcome│ │.rs (NEW: defs│ │ + fields) — no changes needed│   │
│  │stdin read │ │cache + typed │ │ (render_list/_single take    │   │
│  │confirm    │ │value parsing │ │  serde_json::Value today)    │   │
│  └───────────┘ └──────────────┘ └──────────────────────────────┘   │
├────────────────────────────────────────────────────────────────────┤
│                    AppContext (config + client + flags)            │
├────────────────────────────────────────────────────────────────────┤
│                       API Layer (src/api/)                         │
│  PipeliteClient — ALL HTTP lives here; handlers never touch reqwest│
│  NEW methods: notes, runs, webhooks, trash, cfd, templates, audit, │
│  docs  ·  MODIFIED: 403 → Forbidden mapping                        │
├────────────────────────────────────────────────────────────────────┤
│                    Models (src/api/models.rs)                      │
│  NEW: Note, Webhook, TrashItem, AuditEntry, WorkflowRun(+Detail),  │
│  WorkflowTemplate, CustomFieldDefinition  · FIX: Deal.position f64 │
├────────────────────────────────────────────────────────────────────┤
│         Persistence/Local: config.rs (0600 TOML) · cache.rs        │
│         (TTL JSON) · NEW cache keys: custom_fields_*               │
└────────────────────────────────────────────────────────────────────┘
```

The architecture needs **no structural change** — all 8 features fit the existing layering. The work is: 6 new command groups, 1 subcommand extension, ~25 new client methods, ~8 new models, 2 new shared modules (`batch.rs` planned, `custom_fields.rs` new), and one error-enum extension.

---

## Decision 1: Command Group Placement (new group vs subcommand)

The governing principle, derived from how the server itself factors the API (SERVER-API-DIFF §A): **mirror the URL shape**. Resources with parent-scoped URLs get subcommands of the parent group; resources with top-level URLs get their own group. One deliberate exception (notes) where duplication cost outweighs path symmetry.

| Feature | Placement | Command Surface | Rationale |
|---------|-----------|-----------------|-----------|
| **Notes** | **New top-level group** `pipelite notes` | `notes list <deal\|org\|person\|activity> <parent-id>` · `notes create <type> <parent-id> --content "..."` (or `--stdin`) · `notes update <note-id> --content` · `notes delete <note-id> [--force]` | API is *split*: GET/POST are nested under 4 parents, but PATCH/DELETE are parent-less (`/notes/{noteId}`). Nesting under each entity would duplicate args structs, handlers, and wiring ×4 for 2 of 4 verbs that don't even nest. Entity type as a `ValueEnum` positional mirrors the path segment and stays scriptable. **No `get` subcommand** — server has no single-note GET; list-then-filter is the workflow. |
| **Workflow runs** | **Subcommands of existing `workflows`** | `workflows runs <wf-id>` (list, `--status --dry-run --limit --offset --all`) · `workflows runs <wf-id> <run-id>` (detail with `steps[]`) | Runs have *no top-level endpoint* — only `/workflows/{id}/runs`. Precedent exists: `workflows trigger` already extends the group the same way. A top-level `runs` group would force a redundant `--workflow` flag. |
| **Webhooks** | **New top-level group** `pipelite webhooks` | `webhooks list/get/create/update/delete` — full standard CRUD | Top-level `/webhooks` resource with own lifecycle. Standard CRUD pattern applies almost verbatim (docs/SKILL.md pattern), with two deviations: update uses **PUT** (not PATCH), and create must warn **secret show-once**. No `--active` on create (server forces true). |
| **Trash** | **New top-level group** `pipelite trash` | `trash list [--type deals\|people\|organizations\|activities]` · `trash restore <type> <id>` · `trash purge <type> <id> [--force]` | Cross-entity concern (spans 4 entities) — doesn't belong under any one. Type positional **must be the plural tab name** (server path contract; invalid → 422). Use `ValueEnum` to validate client-side. `purge` mirrors batch-delete confirmation semantics; `restore` and `purge` return 204 (use `handle_delete_response`-style empty-body handling). |
| **Custom fields** | **New top-level group** `pipelite custom-fields` + flags on existing entities | `custom-fields list [--entity-type]` · `get/create/update/delete <id>` — definitions CRUD. Separately: typed `--custom-field` writing through existing `deals/orgs/people/activities create/update`. | Definitions are a first-class resource (`/custom-field-definitions`). The *writing* side is not a new command — it upgrades the existing `--custom-field` flag semantics inside 8 existing handlers (create+update × 4 entities). Also: remove dead `--custom-field` from `pipelines create` / `stages create` (SERVER-API-DIFF §D-5). |
| **Workflow templates** | **New top-level group** `pipelite templates` | `templates list/get/create/delete` — **no update** (server has none) | Global, ownership-free resource at `/workflow-templates`. Top-level beats nesting under `workflows` because templates are *inputs to* workflow creation, not workflow state. Optional differentiator (defer-able): `workflows create --from-template <id>` pre-fills trigger/nodes from a template. |
| **Audit** | **New top-level single command** `pipelite audit` | `audit [--entity-type] [--entity-id] [--actor-kind] [--workflow-run-id] --limit --offset --all` | Read-only viewer, one endpoint, admin-only. Arg struct only, no subcommands — same shape as `dashboard`. |
| **Docs** | **New top-level single command** `pipelite docs` | `docs [--output <file>]` (default: pipe raw JSON to stdout) | Public no-auth endpoint; utility command like `ping`. Output must stay raw/pipeable (`--format json` is a no-op here; plain table makes no sense for an OpenAPI spec). |

**Rejected alternatives, explicitly:**
- `deals notes ...` ×4 (nested per entity): rejected — 4× clap structs + 4× handler dirs + 4× wiring for a resource whose mutation verbs are already parent-less. If discoverability complains, add `after_help` cross-references on `deals/orgs/people/activities --help` ("Manage notes: pipelite notes list deal <id>").
- Top-level `pipelite runs --workflow <id>`: rejected — breaks the "mirror the URL" rule and orphans runs from their only parent context.
- `workflows templates ...`: viable but rejected — adds a third nesting level (`workflows templates create`) for no routing benefit, and templates are conceptually upstream of workflows.

## Decision 2: Batch Utility Structure (so all entities reuse it)

Phase 01's drafted `src/batch.rs` (01-01-PLAN) is the right nucleus. To maximize reuse across the 7 entities **without** introducing a trait-abstraction layer the codebase doesn't have (it is deliberately concrete — one handler file per action, no generics over entities), the module should own exactly the three things that are identical everywhere, and leave per-item HTTP to entity handlers:

```rust
// src/batch.rs — final shape (extends the 01-01 draft)
pub struct BatchOutcome { /* as drafted: total/succeeded/failed */ }
impl BatchOutcome {
    pub fn new(total: usize) -> Self;
    pub fn record_success(&mut self);
    pub fn record_failure(&mut self, index: usize, id: &str, err: &dyn Display);
    pub fn finalize(self, entity_name: &str, operation: &str) -> Result<()>;
}

/// Parse a JSON array from stdin (objects for update, strings for ID lists).
pub fn read_stdin_json<T: DeserializeOwned>(entity_hint: &str) -> Result<Vec<T>>;

/// Stdin-array update driver: reads Vec<Value>, extracts "id", hands each
/// item to a caller closure with continue-on-error, collects outcome.
pub async fn run_stdin_update<F, Fut>(
    ctx: &AppContext, entity_hint: &str, on_item: F,
) -> Result<()>
where F: Fn(String, serde_json::Value) -> Fut, Fut: Future<Output = Result<serde_json::Value>>;

/// Shared destructive-action confirmation. THE reason to extract this:
/// the dry-run-before-prompt ordering (01-CONTEXT D-05) is an invariant
/// that belongs in one function, not copy-pasted 7 times.
pub fn confirm_destructive(ctx: &AppContext, prompt: String) -> Result<bool>;
//   → false if ctx.dry_run was already handled by caller;
//     checks no_input + io::stdin().is_terminal(); dialoguer::Confirm default(false)
```

**Why not a full `trait BatchEntity` runner:** the per-entity differences (typed `*Update` deserialization, cache key, table columns, client method name) would force the trait to leak handler concerns, and clap args stay per-entity regardless. The 01-01/01-02 draft approach — shared outcome accounting + stdin parsing, entity-specific loops — is the correct granularity. The one amendment worth making to the drafted plan: **extract `confirm_destructive` into `batch.rs` (or `prompt.rs`) in 01-01** rather than inlining `dialoguer::Confirm` in `deals/delete.rs`, because trash `purge` (later phase) needs the identical dry-run→confirm ordering, and 7 copy-pasted prompt blocks is where the ordering invariant will eventually get broken.

**Reuser map for `batch.rs`:**

| Consumer | Uses |
|----------|------|
| `commands/{deals,orgs,people,activities}/{update,delete}.rs` (phase 01) | `BatchOutcome`, `read_stdin_json`, `confirm_destructive`, dry-run loop |
| `commands/pipelines|stages|workflows/create.rs` (retrofit, optional — 01-CONTEXT leaves open) | `BatchOutcome` around existing individual-create loops |
| `commands/trash/purge.rs` (later phase) | `confirm_destructive` (single-ID destructive confirm, same invariant) |
| `commands/webhooks/delete.rs` (later phase) | `confirm_destructive` if delete prompts; else `--force` like workflows delete |

Note: batch is **not** in scope for the new v1.1 surfaces (notes/webhooks/etc. have no batch endpoints server-side — SERVER-API-DIFF §E confirms no batch routes). Phase 01 stays closed over the 7 existing entities.

## Decision 3: Type-Aware Custom-Field Parsing Flow

**New module: `src/custom_fields.rs`** — the only genuinely new shared component this milestone. It owns definitions cache access and value typing, so the 8 touched handlers stay thin and the logic is unit-testable without HTTP.

```
pipelite deals update deal_1 --custom-field budget=4000 --custom-field tier=enterprise
  │
  ▼
commands/deals/update.rs: collect raw pairs (existing flag, unchanged syntax)
  │
  ▼
custom_fields::resolve(ctx, EntityKind::Deal, raw_pairs) -> Result<(Map<String,Value>, Vec<Warning>)>
  │
  ├─1─ defs = cache.get<Vec<CustomFieldDefinition>>("custom_fields_deal")
  │      ├─ HIT  → proceed
  │      ├─ MISS + !ctx.dry_run → client.list_custom_field_definitions("deal")
  │      │                        → cache.set(KEY, TTL 3600)   ← definitions change rarely
  │      └─ MISS + ctx.dry_run  → NO HTTP (dry-run invariant!) → string fallback + stderr note
  │
  ├─2─ for each (key, raw): match def.type
  │      number       → JSON number (reject non-numeric with Validation error naming the field)
  │      boolean      → JSON bool from "true"/"false"
  │      date         → string (light ISO check; server validates authoritatively)
  │      text / url   → string
  │      single_select→ string; warn if not in def.config.options (don't block — server is source of truth)
  │      multi_select → JSON array from "a,b,c" (comma-split, trimmed)
  │      lookup       → string (target entity ID; warn if def lookup target can't be verified — no client check)
  │      formula      → hard Validation error: server-computed, not writable
  │      file         → hard Validation error: upload is session-only (SERVER-API-DIFF §B)
  │      unknown key  → keep string value + warning (offline tolerance; server blob-merge accepts any key)
  │
  ▼
merge over --custom-field-json escape hatch (raw JSON object, wins on conflict) → DealUpdate.custom_fields
```

**Key integration constraints (these are the ones that cause rewrites if missed):**

1. **Dry-run must never trigger the definitions fetch.** `--dry-run` means zero HTTP (CLAUDE.md invariant). Cache-only in dry-run; fall back to today's string behavior with a visible note. This forces `resolve()` to take `&AppContext` (for cache + client + dry_run) — it cannot be a pure function.
2. **Cache per entity_type**, keys `custom_fields_deal|person|org|activity`, `TTL 3600` (definitions change as rarely as pipelines). Invalidation: any definitions CUD calls `cache.invalidate_prefix("custom_fields_")` (the prefix helper already exists in `cache.rs`).
3. **Definitions list includes soft-deleted rows** (SERVER-API-DIFF §A-7) — filter `deleted_at`-equivalent (or server's soft-delete marker) out before caching, else typed parsing resurrects dead fields.
4. **Merge semantics:** server blob-merges `custom_fields` on update, so the CLI sends only the keys being changed — never fetch-modify-write the existing blob (that would race and could overwrite concurrent edits; also there is no API way to *delete* a key — document that in the flag's long help).
5. **`--custom-field-json` escape hatch** (`--custom-field-json '{"budget": 4000}'`) for power users/agents: raw object, merged over parsed flags. Cheap to add, removes every "typed parsing got it wrong" deadlock.

**Handler touchpoints (8 files):** `commands/{deals,orgs,people,activities}/{create,update}.rs` each replace their current string-map `--custom-field` assembly with a `custom_fields::resolve(...)` call. The CLI arg syntax (`--custom-field key=value`, repeatable) stays identical — this is a semantics upgrade, not a breaking change.

## Decision 4: Admin-Only Commands & the 403 Error Pattern

**Problem:** `api/mod.rs::handle_response` maps `401 | 403` → `CliError::Auth` with hint *"Check your API key. Run `pipelite init` to reconfigure."* That was correct when 403 ≈ bad key. v1.1 makes 403 mean **"valid key, insufficient role"** on: `audit` (entire surface), `trash purge` (member → 403), notes update/delete (author-or-admin), webhook ops on foreign webhooks. The current hint actively misleads — users will re-run `pipelite init` for nothing.

**Recommendation: add `CliError::Forbidden { detail, hint }`** (title: "Permission denied"):

```rust
// src/error.rs
#[error("Permission denied")]
Forbidden { detail: String, hint: String },
// + one match arm in display_error; exit_code() already falls through to 1 (correct)

// src/api/mod.rs — split the mapping:
401 => Auth { hint: "Check your API key. Run `pipelite init` to reconfigure." }
403 => Forbidden { hint: forbidden_hint.unwrap_or(DEFAULT_ADMIN_HINT) }
```

- Give `handle_response` / `handle_delete_response` an optional per-call `forbidden_hint: Option<&str>` (or a small `handle_response_with_hints` wrapper used only by the admin surfaces). Call-site hints:
  - **audit:** "`pipelite audit` requires an admin API key. Ask your workspace admin for an admin key, or use a key with the admin role."
  - **trash purge:** "Purging from trash is admin-only. `trash restore` works with owner-or-admin keys." (restore's 403 = not owner: hint "Only the deleter or an admin can restore this item.")
  - **notes update/delete:** "You can only edit/delete your own notes unless you use an admin key."
  - **webhooks:** "This webhook belongs to another key. Foreign webhooks return 403, not 404." (SERVER-API-DIFF §A-4 — worth stating because it differs from workflows' 404 convention.)
- **Blast radius:** any existing test asserting 403→"Authentication failed" needs updating (small; grep `Auth` in tests). The one-time cost lands *before* the phases that add the 403-heavy surfaces, which is why error work is sequenced first in the build order below.
- Do **not** pre-validate admin-ness client-side (there's no "who am I" endpoint) — the server's 403 with a good hint *is* the UX.

## Recommended Project Structure (delta from current tree)

```
src/
├── api/
│   ├── mod.rs              # MODIFY: +~25 methods (notes, runs, webhooks, trash,
│   │                       #   cfd, templates, audit, docs) + 403 split
│   └── models.rs           # MODIFY: +8 models & table configs; FIX Deal.position → f64;
│                           #   drop deny-on-unknown for expand passthrough fix
├── cli/
│   ├── mod.rs              # MODIFY: +6 Commands variants, +mod decls
│   ├── notes.rs            # NEW (parent-type ValueEnum + parent-id + note-id args)
│   ├── webhooks.rs         # NEW (incl. EVENTS const — 13 valid names, client-validated)
│   ├── trash.rs            # NEW (plural-tab ValueEnum)
│   ├── custom_fields.rs    # NEW (definitions CRUD args)
│   ├── templates.rs        # NEW (no update args — server has none)
│   ├── audit.rs            # NEW (filter args)
│   └── workflows.rs        # MODIFY: +Runs variant (list/detail)
├── commands/
│   ├── mod.rs              # MODIFY: +6 mod decls
│   ├── notes/{mod,list,create,update,delete}.rs      # NEW
│   ├── webhooks/{mod,list,get,create,update,delete}.rs # NEW
│   ├── trash/{mod,list,restore,purge}.rs             # NEW
│   ├── custom_fields/{mod,list,get,create,update,delete}.rs # NEW
│   ├── templates/{mod,list,get,create,delete}.rs     # NEW
│   ├── audit.rs            # NEW (single list handler)
│   ├── docs.rs             # NEW
│   ├── workflows/runs.rs   # NEW (list + detail; wired from workflows/mod.rs)
│   ├── {deals,orgs,people,activities}/update.rs      # MODIFY: batch + typed CF
│   └── {deals,orgs,people,activities}/delete.rs      # MODIFY: batch + confirm helper
├── batch.rs                # NEW (phase 01, drafted) — amend: +confirm_destructive
├── custom_fields.rs        # NEW — definitions cache + typed value resolution
├── cache.rs                # MODIFY: KEY/TTL_CUSTOM_FIELDS + per-entity key helper
├── error.rs                # MODIFY: +Forbidden variant
├── dry_run.rs              # unchanged (render_dry_run covers all new methods)
└── main.rs                 # MODIFY: mod decls + match arms
```

**`api/mod.rs` size guard:** the file is ~1,080 lines and v1.1 adds roughly 500–700. Keep the single-file convention (comment-banner grouping, as today) through this milestone, but set a tripwire: **if it crosses ~2,000 lines, split into `api/client.rs` (transport + response handling) + `api/methods/` per area.** Don't split preemptively — the transport/handle_response split point is only obvious after all methods exist.

## Data Flow (new surfaces, canonical path)

```
argv → clap (src/cli/<area>.rs) → Commands variant → main.rs run() match arm
     → commands/<area>/mod.rs::run → action handler (src/commands/<area>/<action>.rs)
     → [dry_run? → render + return]  → [custom_fields::resolve? → cache→client→typed blob]
     → [batch? → BatchOutcome loop]  → PipeliteClient method (src/api/mod.rs)
     → handle_response (403→Forbidden, 404→NotFound, 422→Validation)
     → cache.invalidate (on mutation) → output::render_list / render_single
     → batch? → outcome.finalize → stderr summary + non-zero exit on partial failure
```

Nothing about this flow is new — every new surface slots into the existing pipeline. The two flows with real novelty are the **typed custom-field flow** (Decision 3, adds a cache→parse stage inside 8 handlers) and **batch flow** (Decision 2, adds loop+outcome around existing single-item calls).

## Architectural Patterns to Follow

### Pattern 1: Exact CRUD-pattern replication (docs/SKILL.md)
Every new top-level group (webhooks, custom-fields, templates) copies the `stages`/`workflows` skeleton: `Commands` enum → args structs → `commands/<area>/mod.rs` dispatcher → per-action files → `models.rs` Base/Create(/Update) trio → client methods with comment banner → table config. Webhooks and templates deviate only where the server does (PUT-not-PATCH; no-update-at-all).

### Pattern 2: Parent-scoped subcommand (workflows runs)
`runs` takes the parent ID as first positional (completion candidates from `KEY_WORKFLOWS` cache, same as `workflows get`), optional second positional switches list→detail. Detail adds `steps[]` — render as nested list in table mode, natural JSON in json mode.

### Pattern 3: Client-side validation where the server doesn't validate
Webhook event names (server accepts anything — §A-4) and trash type names (server 422s but a `ValueEnum` gives completions + instant feedback) are validated in `src/cli/` via `ValueEnum`/const lists. This is the established pattern (workflow `--data @file` handling).

## Anti-Patterns to Avoid

1. **Duplicating notes under 4 entity groups** — 4× maintenance, and PATCH/DELETE don't nest anyway (Decision 1).
2. **HTTP in dry-run for definitions** — typed parsing that fetches on cache-miss inside `--dry-run` violates the CLI's core invariant. Cache-only fallback (Decision 3, constraint 1).
3. **"Run `pipelite init`" as the 403 hint** — misleads non-admin users into reconfiguring. Forbidden variant with per-surface hints (Decision 4).
4. **Caching audit/trash results** — they're fresh-state views (and trash offset caps at 10,000). These commands bypass the cache entirely; only custom-field *definitions* get cached.
5. **Storing the webhook secret** — it exists only in the create response. Print once with a "won't be shown again" warning; never persist to config or cache.
6. **Plural/singular confusion in trash** — path type is plural (`deals`), row's `entity_type` field is singular (`deal`). One `ValueEnum` with an explicit `-> path segment` mapping prevents the 422 class entirely.
7. **Trait-abstracting the batch layer** — premature; per-entity loops + shared outcome/stdin/confirm helpers is the right granularity for this codebase (Decision 2).
8. **Fetch-modify-write of `custom_fields` blob** — races with concurrent edits and can clobber; server merges, so send only changed keys.

## Build Order (dependency-driven)

```
Phase 01  Batch operations (already planned 01-01…01-04)
            │  establishes batch.rs (BatchOutcome, stdin, confirm_destructive)
            │  ▼
Phase 02  Foundations: error Forbidden variant + model fixes
            │  (Deal.position f64, --expand passthrough, dead filters §D-1..4,
            │   stages all-mode §D-8, remove pipelines/stages dead CF flags §D-5)
            │  Rationale: one-time shared-error work BEFORE the 403-heavy and
            │  model-adjacent surfaces land; fixes are small and de-risk later phases
            │  ▼
Phase 03  Small read/nested surfaces: workflows runs + templates + docs
            │  (runs depends on nothing new; templates/docs are isolated CRUD/utility;
            │   first consumer of handle_delete_response-style 204 handling for restore)
            │  ▼
Phase 04  Notes (first parent-scoped sub-resource; independent)
            │  ▼
Phase 05  Webhooks + trash + audit
            │  (consumes Forbidden hints, confirm_destructive for purge,
            │   client-side event/type validation patterns from Phase 03)
            │  ▼
Phase 06  Custom fields: definitions CRUD → cache keys → custom_fields.rs
            │  → typed --custom-field in 8 handlers → --custom-field-json
            │  LAST because it touches the most existing files (8 handlers)
            │  and benefits from all prior patterns being stable
            ▼
Phase 07  Integration tests + SKILL.md/docs refresh (per-phase summaries may
            cover most; final pass updates docs/SKILL.md endpoint table + conventions)
```

**Hard dependencies:** batch.rs before per-entity batch (01-01→01-02/03) ✓ already planned · definitions cache before typed parsing (both inside the custom-fields phase, sequenced as plan 1→2 within it) · Forbidden variant before webhooks/trash/audit · model fixes before any phase that reads `Deal.position` or relies on expand.

**Parallelizable (waves):** notes ∥ workflows-runs ∥ docs are mutually independent; webhooks ∥ trash ∥ audit likewise. The linear order above is the *recommended* sequence (risk-ascending, shared-infra-first), not a strict chain beyond the marked dependencies.

## Phase-Specific Research Flags

| Phase | Needs deeper research? | Note |
|-------|------------------------|------|
| 01 batch | No | Plans drafted and validated |
| 02 error/model fixes | No | §D list is verified against server |
| 03 runs/templates/docs | No | Shapes fully specified in SERVER-API-DIFF |
| 04 notes | No | Asymmetry (no single GET) is the only wrinkle |
| 05 webhooks/trash/audit | Maybe — trash restore edge cases (linked_parents rendering in table mode) worth a 30-min server check at plan time |
| 06 custom fields | Yes at plan time — confirm exact soft-delete marker on definitions rows and multi_select config shape before writing the parser |

## Sources

- `src/api/mod.rs`, `src/api/models.rs` — client/model conventions, 403 mapping, param-struct pattern (read in full, 2026-09-02) — HIGH
- `src/cli/mod.rs`, `src/cli/workflows.rs`, `src/commands/deals/mod.rs`, `src/commands/mod.rs`, `src/main.rs` — wiring/dispatch patterns — HIGH
- `src/cache.rs` — TTL/keys/invalidate_prefix (reused for custom-field definitions cache) — HIGH
- `src/error.rs` — CliError variants, display/exit-code paths (Forbidden variant impact) — HIGH
- `.planning/phases/01-batch-operations-for-all-entities/01-01-PLAN.md`, `01-02-PLAN.md`, `01-CONTEXT.md` — batch utility draft, decisions D-01…D-07 — HIGH
- `.planning/research/SERVER-API-DIFF.md` — all endpoint shapes, auth gating, §D bugs (verified against server repo by its author, 2026-09-02) — HIGH
- `docs/SKILL.md`, `.planning/PROJECT.md` — conventions, milestone scope — HIGH

---
*Architecture research for: pipelite CLI v1.1 Server-v2-parity milestone integration*
*Researched: 2026-09-02*
