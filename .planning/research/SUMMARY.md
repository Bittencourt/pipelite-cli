# Project Research Summary

**Project:** Pipelite CLI — Milestone v1.1 "Server v2 Parity"
**Domain:** Rust CLI extension — 8 new API surfaces + batch operations added to an established 7-entity CRUD tool
**Researched:** 2026-09-02
**Confidence:** HIGH

## Executive Summary

Pipelite v1.1 is a parity catch-up milestone: the server (v2) already exposes notes, workflow runs, webhooks, trash, custom-field definitions, templates, an audit log, and self-documentation — the CLI must grow these surfaces without breaking the conventions that make v1.0 good (4 output formats, `--dry-run` = zero HTTP, hint-bearing errors, `--no-input`/`--quiet`/`--no-color`). All four research tracks converge on one headline: **this is a low-risk milestone with zero new dependencies and zero architectural change.** The existing stack (clap 4.6, reqwest 0.13, serde, dialoguer, comfy-table, indicatif, chrono) covers every feature; the only Cargo.toml edit is adding the `"time"` feature to tokio for 429 `Retry-After` backoff. Mature-CLI research (gh, Stripe, Helm, kubectl — verified against live docs) confirms the UX patterns are all established precedents, not invention.

**The recommended approach:** replicate the existing CRUD pattern per `docs/SKILL.md` for every new group, add exactly two new shared modules (`batch.rs` — already drafted in phase-01 plans — and `custom_fields.rs` for definitions-cached typed parsing), and extend the client with ~25 typed methods. Command placement follows a "mirror the URL shape" rule with one deliberate exception (notes as a top-level group because the API's mutation verbs are parent-less). The genuinely novel work is narrow: typed custom-field resolution against cached definitions, the batch continue-on-error contract, and the webhook show-once secret.

**The key risk is not the new features — it is the shared infrastructure they all flow through.** The v1.0 error layer conflates 403 into "check your API key / run `pipelite init`" (actively misleading on every admin-gated surface), parses RFC 7807 error bodies with the wrong keys (server detail messages are silently discarded), and has a silent 1000-record pagination cap. Models reject fractional `position` floats the server emits. Pitfalls research is emphatic: these must be fixed in a **foundations phase at the front of the milestone**, not as cleanup at the end — otherwise every new surface ships with broken error UX baked in. Secondary risks: the webhook secret being truncated by the output layer on its single appearance (unrecoverable without delete-and-recreate), and trash purge (permanent, admin-only) inheriting v1.0's no-confirmation delete pattern.

## Key Findings

### Recommended Stack

**Zero new crates.** Every milestone feature resolves to the existing dependency set; researchers explicitly rejected `futures`, `url`, `hmac`/`sha2`, `regex`, `uuid`, OpenAPI parsers, and any TUI crate as unnecessary (see STACK.md "What NOT to Use"). The single change: `tokio = { version = "1", features = ["rt", "macros", "time"] }` for async `sleep` on 429 backoff.

**Core technologies (all existing):**
- **clap 4.6** — subcommand nesting (`workflows runs`), `ValueEnum` for client-side validation (webhook events, trash types, run status), multi-ID args
- **reqwest 0.13** — grows typed methods; needs `put` alongside `patch` (webhooks/definitions use PUT)
- **serde_json** — `Value` passthrough for arbitrary server shapes (run steps, audit `changes`, template `trigger/nodes`); streaming stdin via `Deserializer::from_reader` (array) + `BufRead::lines` (NDJSON)
- **comfy-table 7.2** — nested data flattened to rows (one row per run step; field/from/to per audit change); stay on 7.x, defer the 8.0 major bump
- **dialoguer + indicatif + chrono + colored** — destructive confirms, batch progress, date parsing, secret warning styling

### Expected Features

**Must have (table stakes — all verified against server API diff):**
- **Notes** group: parent-scoped list/create (deals/orgs/people/activities only), global edit/delete by note ID (no GET-single — asymmetry is server-imposed)
- **Workflow runs**: list with `--status`/`--dry-run` filters + detail with `steps[]`; dry-run runs hidden by default (hint on empty results)
- **Webhooks CRUD**: 13-event client-side validation, https prefix check, **PUT** (not PATCH) update, secret shown once with warning
- **Trash** list/restore/purge: plural tab names in paths, purge = permanent + admin-only + strongest confirmation in the codebase
- **Custom field definitions CRUD** + **type-aware `--custom-field` writing** (fixes "stores `"4"` not `4`") + **`--custom-field-json`** escape hatch
- **Batch update/delete** with continue-on-error: per-item results, summary, aggregate exit codes (plans already drafted)
- **Audit list** with server-mirrored filter flags + prominent admin-403 hint
- **Templates** list/get/create/delete (**no update** — server has none)
- **`pipelite docs`** — fetch/save the OpenAPI 3.1 spec
- **§D fixes**: dead flags (people `--org/--owner`, workflows `--active`, create `--active`, dead CF flags), `--expand` passthrough, `position` → f64, stages all-mode
- Cross-cutting: every new command honors all v1.0 global flags

**Should have (differentiators, v1.1.x or late-milestone):**
- `workflows runs --watch --exit-status` (gh run watch pattern — converts fire-and-forget trigger into observe-and-react)
- Pipeable failed-ID batch summary (one-liner retry)
- Audit changes diff rendering (`field: from → to`)
- Webhook event-name shell completions; `--custom-field-string` force-string flag

**Defer (v2+, server-blocked):**
- Global search, file upload via API key (session-only routes)
- Sort/date-range/custom-field list filters (no server support — request upstream, never emulate client-side)
- Fake "live-streaming" run logs (no log endpoint exists — polling watch is the honest pattern)

### Architecture Approach

**No structural change.** All 8 surfaces slot into the existing layering: clap args (`src/cli/`) → handler files (`src/commands/<area>/`) → typed client methods (`src/api/mod.rs`) → generic renderers (which already operate on `serde_json::Value` rows — zero renderer changes). Two new shared modules: `batch.rs` (BatchOutcome accounting + stdin parsing + `confirm_destructive` — extract the confirm helper early, not copy-pasted per entity) and `custom_fields.rs` (`resolve()` taking `&AppContext`; cache-only in dry-run — the dry-run-must-not-fetch invariant is the constraint that causes rewrites if missed).

**Major components:**
1. **`src/batch.rs`** — shared batch utility: exit-code contract (0/1/2), 100% pre-validation before first HTTP, `confirm_destructive` with dry-run-before-prompt ordering
2. **`src/custom_fields.rs`** — definitions cache (`custom_fields_<entity>`, TTL 3600, invalidated by definitions CUD) + per-type value parsing; filter soft-deleted definitions before caching
3. **Error layer** — new `CliError::Forbidden { detail, hint }`; 401→Auth, 403→Forbidden with per-surface hints, 409 route-aware hints; one shared RFC 7807 extraction helper (`detail` → `errors[]` → fallback)
4. **Shared paginator** — one loop for all new list surfaces: offset stepping to `meta.total`, loud stderr warning at any ceiling, trash stops cleanly at offset 10,000

### Critical Pitfalls

1. **403/409 conflated into "Auth" + RFC 7807 bodies parsed with wrong keys** — every admin-gated surface (audit, trash purge, notes edit, foreign webhooks) tells users to re-run `pipelite init`, and every 422 discards the server's actual error detail. Fix in foundations, before any 403-heavy surface ships. (Pitfalls 1+2)
2. **Webhook secret destroyed by table truncation on its one appearance** — server returns the secret only on create; the generic renderer's `truncate_with_ellipsis` renders it as `whsec_abc...` and it's gone forever. Webhook create must bypass generic rendering for the secret: full, untruncated, own line, save-it-now warning. Never cached, never a CLI flag, never fabricated in dry-run. (Pitfall 5)
3. **Batch contract defined implicitly** — exit codes (0=all ok, 1=any failed even with `--continue-on-error`, 2=structural failure pre-HTTP), stderr summary that survives `--quiet`, JSON per-item results array, fail-fast default, and 100% input validation before the first mutation. Scripts are written against this the day it ships. (Pitfalls 3+4)
4. **Trash purge is irreversible** — singular/plural type mismatch scripts into 422s; purge must normalize types client-side, confirm interactively, and **refuse under `--no-input` without `--force`** (exit 2, zero HTTP). (Pitfall 6)
5. **Silent pagination caps + float `position` failures** — inherited `--all` stops at 1000 records without warning (fatal for audit exports); `Deal.position: Option<i64>` breaks deserialization entirely on fractional positions (kills `deals list` and batch stdin for affected records). Both fixed in foundations. (Pitfalls 8+9)

## Implications for Roadmap

Based on research, suggested phase structure (7 phases — architecture and pitfalls research independently converged on the same order; note this **differs from the milestone doc's original ordering, which listed fixes last** — all research recommends promoting shared-infra fixes to the front):

### Phase 1: Batch operations for existing entities
**Rationale:** Already drafted and validated (01-01…01-04 plans exist) — momentum + it establishes `batch.rs` (BatchOutcome, stdin parsing, `confirm_destructive`) that later phases reuse.
**Delivers:** Batch update/delete across the 7 existing entities with the full exit-code/summary contract.
**Addresses:** Batch continue-on-error (FEATURES P1); `confirm_destructive` extracted here, not inlined.
**Avoids:** Pitfalls 3+4 (implicit exit-code semantics, stdin edge cases); anti-pattern "trait-abstracting the batch layer" — keep per-entity loops over shared helpers.
**Amendment to drafted plans:** extract `confirm_destructive` into the utility in 01-01 (trash purge needs the identical dry-run→confirm ordering later).

### Phase 2: Foundations — error layer, models, pagination, §D fixes
**Rationale:** Every subsequent phase consumes the error layer, models, and paginator. Pitfalls research is explicit: this is a *prerequisite*, not cleanup. One-time shared work before the 403-heavy surfaces land.
**Delivers:** `Forbidden` variant + 409 hints + RFC 7807 parsing; `position` → f64 (Deal/Stage/definitions); `--expand` output-path passthrough (design decision: render from raw Value, keep typed models for requests); shared paginator with ceiling warnings; dead-flag removals (§D-1..5, §D-8); `expect_empty_success` 204 helper.
**Addresses:** §D fixes (FEATURES P1 — "trust repair; dead flags actively lie").
**Avoids:** Pitfalls 1, 2, 8, 9, 10.
**Note:** dead-flag removal is a breaking change — changelog callout required.

### Phase 3: Workflow runs + templates + docs
**Rationale:** Small, independent, zero-dependency read surfaces; first consumers of the foundation work. Mutually parallelizable with nothing blocking them.
**Delivers:** `workflows runs` list/detail (steps flattened to rows), `templates` CRUD-minus-update, `pipelite docs` passthrough.
**Addresses:** Runs list/detail, templates, docs (FEATURES table stakes).
**Avoids:** Client-side enum validation for run `status` (server doesn't validate); empty-result hint for hidden dry-run runs; no `update` subcommand for templates; `serde_json::Value` for step input/output and audit changes (typed models here are Pitfall-9-class bugs).

### Phase 4: Notes
**Rationale:** First parent-scoped sub-resource; independent; highest-use surface and fast win (pure CRUD-pattern replication).
**Delivers:** `notes list/create/update/delete` — top-level group with entity-type `ValueEnum` positional (NOT nested ×4 under entities — rejected for 4× duplication of structs/wiring for verbs that don't nest).
**Addresses:** Notes group (FEATURES P1).
**Avoids:** API asymmetry traps — no `--author` flag (forced server-side), no `get` subcommand (no single-note GET), reject pipelines/stages/workflows parents with hint, author-or-admin 403 → Forbidden hint from Phase 2.

### Phase 5: Webhooks + trash + audit
**Rationale:** The 403-heavy, safety-critical group — consumes Forbidden hints, `confirm_destructive`, and client-validation patterns from Phases 2–4. Grouped because all three are permission-tiered surfaces.
**Delivers:** Webhooks CRUD (show-once secret, PUT semantics, event enum), trash list/restore/purge (plural normalization, `--force` gating), audit list (admin-403 hint, server-mirrored filters only).
**Addresses:** Webhooks, trash, audit (FEATURES P1); secret show-once is the Stripe-verified pattern.
**Avoids:** Pitfall 5 (write the 64-char-secret truncation test FIRST); Pitfall 6 (singular→plural mapping so `trash list | jq | restore` round-trips; `--no-input` purge without `--force` exits 2); audit `changes` typed as `Value`; no caching of trash/audit results.

### Phase 6: Custom fields — definitions CRUD + typed writing
**Rationale:** **Last of the feature phases** — it touches the most existing files (8 handlers: create+update × 4 entities) and benefits from all prior patterns being stable. Definitions CRUD must land before the typed parser (it's the type source); `--custom-field-json` can land either side.
**Delivers:** `custom-fields` group; `custom_fields.rs` (cache + `resolve()`); type-aware `--custom-field` semantics upgrade (syntax unchanged — non-breaking); `--custom-field-json` escape hatch; removal of dead CF flags on pipelines/stages.
**Addresses:** Definitions CRUD, typed writing (FEATURES P1); fixes the "stores `\"4\"` not `4`" data-correctness bug.
**Avoids:** Pitfall 7 (split on first `=` only; never guess types from value shape; formula/file hard-error with hint; empty-value error not silent send); dry-run must never trigger the definitions fetch (cache-only fallback + visible note); never fetch-modify-write the blob (server merges — send changed keys only); filter soft-deleted definitions before caching.

### Phase 7: Integration hardening + docs refresh
**Rationale:** Cross-phase verification of the "Looks Done But Isn't" checklist (PITFALLS.md) — batch exit codes under `--quiet`, secret truncation, trash round-trips, 403 hints — plus `docs/SKILL.md` endpoint table and convention updates.
**Delivers:** End-to-end integration tests against the full surface; docs current.
**Avoids:** the 13-item "looks done but isn't" checklist being skipped per-phase under time pressure.

### Phase Ordering Rationale

- **Shared infrastructure first** (Phase 2) — the meta-finding of pitfalls research: error layer/paginator/models are consumed by everything; fixing them last means every surface ships with misleading errors and silently capped lists.
- **Risk-ascending feature order** (Phases 3→6) — read-only surfaces, then a nested sub-resource, then the destructive/admin surfaces (which need Foundation's Forbidden + confirm helpers), then the widest-blast-radius change (typed custom fields in 8 existing handlers).
- **Batch first** (Phase 1) because plans already exist and the utility it creates (`confirm_destructive`, outcome accounting) is consumed by Phases 5 and 6.
- **Parallelizable waves** exist if needed: notes ∥ workflows-runs ∥ docs are mutually independent; webhooks ∥ trash ∥ audit likewise. The linear order above is recommended, not a strict chain beyond the marked dependencies.

### Research Flags

Phases likely needing deeper research during planning (`--research-phase`):
- **Phase 6 (Custom fields):** verify the exact soft-delete marker on definition rows and the `multi_select` config shape against server code before writing the parser (per ARCHITECTURE.md phase flags).
- **Phase 5 (Webhooks/trash/audit):** minor — 30-minute server check on trash `linked_parents` rendering in table mode; also verify webhook PUT semantics (does omitted key = no-op, or is full object required? FEATURES.md flags this as unverified).

Phases with standard patterns (skip research-phase):
- **Phase 1 (Batch):** plans drafted and validated.
- **Phase 2 (Foundations):** §D list verified against server source; design decision for `--expand` belongs in planning, not research.
- **Phase 3 (Runs/templates/docs):** shapes fully specified in SERVER-API-DIFF.
- **Phase 4 (Notes):** single wrinkle (no GET-single) already researched and resolved.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Zero-new-crates verdict grounded in per-feature mapping against the existing Cargo.toml; crates.io versions fetched 2026-09-02 |
| Features | HIGH | Core UX patterns verified against live gh/Stripe/Helm docs 2026-09-02; API surface taken as fact from server-verified diff. kubectl batch behavior MEDIUM (reference page partially retrieved) |
| Architecture | HIGH | Grounded in direct full reads of the current codebase + drafted phase-01 plans; every claim traces to a repo file |
| Pitfalls | HIGH | Every pitfall grounded in two verified sources: actual v1.0 source (read directly, with file/line citations) + server diff |

**Overall confidence:** HIGH — unusually so, because the server surface was verified against server source code (SERVER-API-DIFF.md) and the existing codebase was read directly rather than inferred.

### Gaps to Address

- **Webhook PUT semantics** (full object required vs. omitted-keys-as-no-op): verify against server code during Phase 5 planning; PUT suggests full-object, which changes the update UX.
- **Batch `Retry-After` handling decision:** sequential batches ≳400 items will 429 against the 500 req/60s limit. Decide in Phase 1 planning: automatic bounded retry (needs tokio `"time"`) vs. per-item "rate limited" failure. Research recommends: parse `Retry-After`, retry once, then classify as failure with detail.
- **`--force` vs `--yes` flag naming:** FEATURES.md leans `--yes` (gh precedent); PITFALLS.md mandates `--force` (the existing `workflows delete` convention). **Recommendation: `--force`** — codebase consistency beats external precedent; settle once in requirements.
- **Notes edit confirmation UX** without GET-single: `notes list --json` is the documented "view before edit" path; prompt text must state why content can't be shown.
- **Batch + custom fields interaction:** batch `--stdin` payloads carry `custom_fields` as raw JSON — typed inference is bypassed there; document explicitly in Phase 6.
- **`api/mod.rs` size tripwire:** currently ~1,080 lines, +500–700 incoming. Keep single-file through the milestone; if it crosses ~2,000 lines, split into `api/client.rs` + `api/methods/` — an execution-time watch item, not a planning item.
- **Interactive picker truncation** (>100 records silently vanish from prompts): opportunistic fix; at minimum add a truncation warning if touched.

## Sources

### Primary (HIGH confidence)
- `.planning/research/SERVER-API-DIFF.md` — full API surface, auth gating, RFC 7807 envelope, rate limit (500/60s), pagination caps, §D bugs — verified against server source by its author, 2026-09-02
- Direct reads of v1.0 source (2026-09-02): `src/api/mod.rs`, `src/api/models.rs`, `src/cli/`, `src/commands/deals/`, `src/cache.rs`, `src/error.rs`, `src/dry_run.rs`, `src/prompt.rs`, `src/output/`, `Cargo.toml`
- `.planning/phases/01-batch-operations-for-all-entities/` — drafted 01-01/01-02 plans + 01-CONTEXT decisions D-01…D-07
- gh CLI manual (cli.github.com) — run watch, issue comment, repo delete confirm/`--yes` semantics
- Stripe CLI docs (docs.stripe.com/cli/listen) — secret show-once + `--print-secret` pattern
- Helm docs (helm.sh) — `--set` / `--set-string` / `--set-json` typed-input pattern
- serde.rs "stream-array" + comfy-table docs (via Context7) — incremental deserialization; dynamic content arrangement

### Secondary (MEDIUM confidence)
- kubectl apply reference — continue-on-error batch behavior (partially retrieved; well-established in widespread use)
- GitHub/Slack audit-log filter conventions (partially retrieved; pipelite server params are the ground truth regardless)

### Tertiary (LOW confidence)
- `stripe openapi` command existence (training data; docs returned a stub) — not load-bearing for any decision

---
*Research completed: 2026-09-02*
*Ready for roadmap: yes*
