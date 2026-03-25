# Roadmap: Pipelite CLI

## Overview

Pipelite CLI delivers a terminal-first CRM client in five phases. Phase 1 builds the foundation that every subsequent phase depends on: authentication, configuration, HTTP client, error handling, and the output abstraction with TTY detection. Phase 2 proves the full vertical stack end-to-end with deals (CRUD + all output formats + filtering). Phase 3 replicates that proven pattern across the remaining five entities. Phase 4 adds developer experience features (interactive prompts, headless mode, shell completions, dry-run). Phase 5 layers power-user features (caching, dashboard, splash screen). Each phase delivers a coherent, verifiable capability.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation** - Auth, config, HTTP client, error handling, and output abstraction (completed 2026-03-25)
- [ ] **Phase 2: Core CRUD and Output** - Full deal CRUD with all output formats, filtering, and field selection
- [x] **Phase 3: Full Entity Coverage** - Replicate CRUD pattern across orgs, people, activities, pipelines, and stages (completed 2026-03-25)
- [ ] **Phase 4: Developer Experience** - Interactive prompts, headless mode, shell completions, and dry-run
- [ ] **Phase 5: Power Features** - Local caching, pipeline dashboard, and splash screen

## Phase Details

### Phase 1: Foundation
**Goal**: Users can authenticate with a Pipelite CRM server, manage configuration, and verify connectivity
**Depends on**: Nothing (first phase)
**Requirements**: AUTH-01, AUTH-02, AUTH-03, AUTH-04, CONF-01, CONF-02, CONF-03, CONF-04, ERRH-01, ERRH-02, ERRH-03, UX-02, UX-03
**Success Criteria** (what must be TRUE):
  1. User can run `pipelite init` and authenticate with an API key that is stored securely (0600 permissions)
  2. User can authenticate via `PIPELITE_API_KEY` env var without any config file
  3. User can run `pipelite ping` and see server status and latency
  4. User can view and modify config with `pipelite config show` and `pipelite config set`
  5. Failed commands return non-zero exit codes with actionable error messages and `--help` shows usage examples on every command
**Plans:** 2/2 plans complete

Plans:
- [ ] 01-01-PLAN.md -- Project skeleton, CLI definitions, error types, and config module
- [ ] 01-02-PLAN.md -- HTTP client, AppContext, all commands (init/ping/config), and integration tests

### Phase 2: Core CRUD and Output
**Goal**: Users can perform full CRUD on deals with all output formats, filtering, and field selection -- proving the complete vertical stack
**Depends on**: Phase 1
**Requirements**: DEAL-01, DEAL-02, DEAL-03, DEAL-04, DEAL-05, OUTP-01, OUTP-02, OUTP-03, OUTP-04, OUTP-05, OUTP-06, OUTP-07, FILT-01, FILT-02
**Success Criteria** (what must be TRUE):
  1. User can list, get, create, update, and delete deals via `pipelite deals` subcommands
  2. Output auto-detects TTY vs pipe: table format in terminal, JSON when piped
  3. User can explicitly choose output format with `--format json|table|csv|plain` and select fields with `--fields`
  4. User can filter deal lists with entity-specific flags and paginate with `--limit` and `--offset`
  5. Colored output works in terminal and respects `NO_COLOR` env var and `--no-color` flag
**Plans:** 3 plans (2 executed, 1 gap closure)

Plans:
- [ ] 02-01-PLAN.md -- Deal data models, API response wrappers, and generic output rendering layer (table/JSON/CSV/plain)
- [ ] 02-02-PLAN.md -- CLI deal subcommands, API client CRUD methods, command handlers, filtering, and integration tests
- [ ] 02-03-PLAN.md -- Gap closure: fix inverted field-selection logic in deals get, remove stale dead_code allows

### Phase 3: Full Entity Coverage
**Goal**: Users can perform full CRUD on all remaining entities (orgs, people, activities, pipelines, stages) using the same patterns proven with deals
**Depends on**: Phase 2
**Requirements**: ORG-01, ORG-02, ORG-03, ORG-04, ORG-05, PEOP-01, PEOP-02, PEOP-03, PEOP-04, PEOP-05, ACTV-01, ACTV-02, ACTV-03, ACTV-04, ACTV-05, PIPE-01, PIPE-02, PIPE-03, PIPE-04, PIPE-05, STAG-01, STAG-02, STAG-03, STAG-04, STAG-05
**Success Criteria** (what must be TRUE):
  1. User can list, get, create, update, and delete organizations via `pipelite orgs` subcommands
  2. User can list, get, create, update, and delete people via `pipelite people` subcommands
  3. User can list, get, create, update, and delete activities via `pipelite activities` subcommands
  4. User can list, get, create, update, and delete pipelines and stages via `pipelite pipelines` and `pipelite stages` subcommands
  5. All entity commands support the same output formats, filtering, and field selection as deals
**Plans:** 3/3 plans complete

Plans:
- [ ] 03-01-PLAN.md -- Organizations and People: CRUD, batch create via /batch endpoint, integration tests
- [ ] 03-02-PLAN.md -- Activities: CRUD with individual-create loop, --done filter, --mark-done/--mark-undone
- [ ] 03-03-PLAN.md -- Pipelines and Stages: CRUD with --pipeline enforcement on stages list/create

### Phase 4: Developer Experience
**Goal**: Users get interactive prompts for human use and explicit headless mode for automation, plus shell completions and dry-run safety
**Depends on**: Phase 3
**Requirements**: INTR-01, INTR-02, INTR-03, HEAD-01, HEAD-02, HEAD-03, SHLL-01, SHLL-02, SHLL-03, UX-04
**Success Criteria** (what must be TRUE):
  1. Running create/update with no flags opens interactive prompts with dropdowns for known values (stages, pipelines), and prompts only activate when stdin is a TTY
  2. User can pass `--no-input` to guarantee no interactive prompts, and headless mode fails with clear error if required input is missing
  3. All mutations can be performed entirely via flags without any prompts
  4. User can generate shell completions for bash, zsh, and fish
  5. User can preview mutations with `--dry-run` showing what would be sent without executing
**Plans**: TBD

Plans:
- [ ] 04-01: TBD
- [ ] 04-02: TBD

### Phase 5: Power Features
**Goal**: Power users get local caching for speed, a pipeline dashboard for overview, and polish touches
**Depends on**: Phase 4
**Requirements**: CACH-01, CACH-02, CACH-03, DASH-01, DASH-02, UX-01
**Success Criteria** (what must be TRUE):
  1. CLI caches pipeline/stage/user metadata locally with TTL-based invalidation, and user can clear cache with `pipelite cache clear`
  2. Cache is used for interactive prompt dropdowns and shell completions, making them faster
  3. User can view pipeline overview with `pipelite dashboard` showing deal counts and total values per stage
  4. Running `pipelite` with no subcommand on a TTY shows an ASCII art splash screen
**Plans**: TBD

Plans:
- [ ] 05-01: TBD
- [ ] 05-02: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 2/2 | Complete    | 2026-03-25 |
| 2. Core CRUD and Output | 2/3 | In Progress|  |
| 3. Full Entity Coverage | 3/3 | Complete   | 2026-03-25 |
| 4. Developer Experience | 0/0 | Not started | - |
| 5. Power Features | 0/0 | Not started | - |
