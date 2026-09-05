# Milestones

## v1.1 Server v2 Parity (Shipped: 2026-09-05)

**Phases completed:** 7 phases, 18 plans, 33 tasks

**Key accomplishments:**

- Batch operations for all 7 entities (`--stdin` JSON, multi-ID delete, continue-on-error, exit-code contract 0/1/2, --force gating, 429 Retry-After) — hardened through 2 review iterations + gap closure + live UAT.
- Truthful foundations: RFC 7807 error layer (Forbidden ≠ Auth, per-surface hints, 409 activation hint), honest models (fractional positions, --expand passthrough), honest lists (stages all-mode, --all ceiling warning), 5 dead flags removed with exit-2 hints, v1.1 CHANGELOG.
- Workflow runs (list/get/--watch with script exit contract), workflow templates (no-update-by-design), and `docs` OpenAPI fetch (unauthenticated).
- Top-level `notes` group over 4 parent entities with locked body-source precedence and no-single-GET contract.
- Webhooks (show-once 64-char secret, 13-event client-side validation, get→merge→PUT), trash (9-alias normalization, strongest-confirm purge), audit (4 filters, admin-gated hints).
- Custom fields: definitions CRUD + shared typed-writing resolver across 8 handlers — `--custom-field price=4` stores JSON number 4 (the milestone's data-correctness fix), `--custom-field-json` bypass.
- Integration hardening: 61-test contract matrix (zero violations), live-server E2E 21 PASS / 0 FAIL (2 live-found bugs fixed with regression tests), docs brought current.

Docs brought current with the shipped v1.1 CLI (SKILL.md 9-surface extension, api-reference gap audit with --help-verified examples, README enumeration, CHANGELOG v1.1 released-ready), loose ends closed (api/mod.rs keep-single-file decision, evidence-based deferred-items audit), and the live-server E2E authored + safety-guarded + EXECUTED (21 PASS / 0 FAIL, 2 live-found bugs fixed with regression test).

---

## v1.0 MVP (Shipped: 2026-03-29)

**Phases completed:** 6 phases, 18 plans, 34 tasks

**Key accomplishments:**

- Clap derive CLI skeleton with rich --version, TOML config with 0600 permissions and env var merge, and thiserror structured error display on stderr
- PipeliteClient with auth headers and timeouts, init wizard with headless mode, ping with spinner/latency, config show/set/get with comfy-table, plus 10 integration tests
- Generic output rendering (table/JSON/CSV/plain) with dot-notation field selection, relative dates, currency formatting, and Deal data models with serde skip_serializing_if
- Full deal CRUD command tree with filtering, auto-pagination, batch stdin create, custom fields, and typed API error handling
- Fixed inverted field-selection conditional in deals get and removed stale dead_code allows from output module
- Full CRUD for organizations (4 fields) and people (7 fields) with batch create, auto-pagination, and 16 integration tests
- Full activities CRUD with individual-create loop, client-side done filter, and mark-done/undone via chrono timestamps
- Models (src/api/models.rs):
- Global --no-input/--dry-run flags, shared prompt and dry-run modules, shell completions, and deals as proof-of-pattern for interactive/headless/dry-run support
- Interactive prompts, headless validation, and dry-run support replicated from deals to all 5 remaining entities (orgs, people, activities, pipelines, stages)
- 21 integration tests validating headless validation (exit code 2, batch missing), shell completions (bash/zsh/fish), and dry-run request preview without HTTP calls
- TTL-based JSON file cache at ~/.pipelite/cache/ with get/set/clear/invalidate/invalidate_prefix, CLI commands, and AppContext wiring
- Pipeline dashboard with deal aggregation per stage and ASCII art splash screen with TTY-aware display
- Cache-through helpers for FuzzySelect prompts: pipelines, stages, and orgs load from local cache first with auto-paginating API fallback on miss
- All 18 mutation commands (create/update/delete x 6 entities) auto-invalidate relevant cache entries after successful API calls, with cascading pipeline-to-stages invalidation
- Cache-backed dynamic shell completions for all entity ID args using clap_complete unstable-dynamic ArgValueCandidates
- Commit:
- Commit:

---
