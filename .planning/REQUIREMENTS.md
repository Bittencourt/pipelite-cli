# Requirements: Pipelite CLI

**Defined:** 2026-03-23
**Core Value:** Users can manage their entire Pipelite CRM from the terminal — fast, scriptable, and composable with other tools.

## v1 Requirements

### Authentication

- [x] **AUTH-01**: User can initialize CLI with API key via `pipelite init` interactive flow
- [x] **AUTH-02**: User can authenticate via `PIPELITE_API_KEY` environment variable for CI/scripts
- [x] **AUTH-03**: API key is stored in config file with 0600 permissions (never as CLI flag)
- [x] **AUTH-04**: User can test connection with `pipelite ping` showing server status and latency

### Configuration

- [x] **CONF-01**: User can store persistent config in `~/.pipelite/config.toml` (XDG-compliant paths)
- [x] **CONF-02**: User can view config with `pipelite config show`
- [x] **CONF-03**: User can set config values with `pipelite config set <key> <value>`
- [x] **CONF-04**: Config precedence follows flags > env vars > config file

### Entity CRUD — Deals

- [x] **DEAL-01**: User can list deals with `pipelite deals list`
- [x] **DEAL-02**: User can get a deal by ID with `pipelite deals get <id>`
- [x] **DEAL-03**: User can create a deal with `pipelite deals create`
- [x] **DEAL-04**: User can update a deal with `pipelite deals update <id>`
- [x] **DEAL-05**: User can delete a deal with `pipelite deals delete <id>`

### Entity CRUD — Organizations

- [x] **ORG-01**: User can list organizations with `pipelite orgs list`
- [x] **ORG-02**: User can get an org by ID with `pipelite orgs get <id>`
- [x] **ORG-03**: User can create an org with `pipelite orgs create`
- [x] **ORG-04**: User can update an org with `pipelite orgs update <id>`
- [x] **ORG-05**: User can delete an org with `pipelite orgs delete <id>`

### Entity CRUD — People

- [x] **PEOP-01**: User can list people with `pipelite people list`
- [x] **PEOP-02**: User can get a person by ID with `pipelite people get <id>`
- [x] **PEOP-03**: User can create a person with `pipelite people create`
- [x] **PEOP-04**: User can update a person with `pipelite people update <id>`
- [x] **PEOP-05**: User can delete a person with `pipelite people delete <id>`

### Entity CRUD — Activities

- [x] **ACTV-01**: User can list activities with `pipelite activities list`
- [x] **ACTV-02**: User can get an activity by ID with `pipelite activities get <id>`
- [x] **ACTV-03**: User can create an activity with `pipelite activities create`
- [x] **ACTV-04**: User can update an activity with `pipelite activities update <id>`
- [x] **ACTV-05**: User can delete an activity with `pipelite activities delete <id>`

### Entity CRUD — Pipelines

- [x] **PIPE-01**: User can list pipelines with `pipelite pipelines list`
- [x] **PIPE-02**: User can get a pipeline by ID with `pipelite pipelines get <id>`
- [x] **PIPE-03**: User can create a pipeline with `pipelite pipelines create`
- [x] **PIPE-04**: User can update a pipeline with `pipelite pipelines update <id>`
- [x] **PIPE-05**: User can delete a pipeline with `pipelite pipelines delete <id>`

### Entity CRUD — Stages

- [x] **STAG-01**: User can list stages with `pipelite stages list`
- [x] **STAG-02**: User can get a stage by ID with `pipelite stages get <id>`
- [x] **STAG-03**: User can create a stage with `pipelite stages create`
- [x] **STAG-04**: User can update a stage with `pipelite stages update <id>`
- [x] **STAG-05**: User can delete a stage with `pipelite stages delete <id>`

### Output Formatting

- [x] **OUTP-01**: User can get JSON output with `--format json` (default when piped)
- [x] **OUTP-02**: User can get table output (default when interactive TTY)
- [x] **OUTP-03**: User can get CSV output with `--format csv`
- [x] **OUTP-04**: User can get plain output with `--format plain` (values only, no headers)
- [x] **OUTP-05**: User can select specific fields with `--fields=id,title,value`
- [x] **OUTP-06**: Output auto-detects TTY vs pipe and adjusts format accordingly
- [x] **OUTP-07**: Colored output with `NO_COLOR` env var and `--no-color` flag support

### List Filtering

- [x] **FILT-01**: User can filter lists by entity-specific fields (e.g., `--stage`, `--owner`)
- [x] **FILT-02**: User can paginate results with `--limit` and `--offset`

### Error Handling

- [x] **ERRH-01**: CLI returns non-zero exit codes on failure (1=runtime, 2=misuse)
- [x] **ERRH-02**: Error messages are actionable with suggested next commands
- [x] **ERRH-03**: CLI shows `--help` with usage examples on every command

### Interactive Mode

- [ ] **INTR-01**: Running create/update with no flags opens interactive prompts for required fields
- [ ] **INTR-02**: Interactive prompts show dropdowns for known values (stages, pipelines)
- [ ] **INTR-03**: Interactive prompts only activate when stdin is a TTY

### Headless Mode

- [ ] **HEAD-01**: User can pass `--no-input` to guarantee no interactive prompts
- [ ] **HEAD-02**: Headless mode fails with clear error if required input is missing
- [ ] **HEAD-03**: All mutations can be performed entirely via flags (no prompts needed)

### Shell Completions

- [ ] **SHLL-01**: User can generate shell completions for bash
- [ ] **SHLL-02**: User can generate shell completions for zsh
- [ ] **SHLL-03**: User can generate shell completions for fish

### Local Caching

- [ ] **CACH-01**: CLI caches pipeline/stage/user metadata locally with TTL-based invalidation
- [ ] **CACH-02**: User can clear cache with `pipelite cache clear`
- [ ] **CACH-03**: Cache is used for interactive prompt dropdowns and shell completions

### Dashboard

- [ ] **DASH-01**: User can view pipeline overview with `pipelite dashboard`
- [ ] **DASH-02**: Dashboard shows deal counts and total values per pipeline stage

### UX Polish

- [ ] **UX-01**: ASCII art splash screen when running `pipelite` with no subcommand on TTY
- [x] **UX-02**: Quiet mode (`-q`) suppresses non-essential output
- [x] **UX-03**: `pipelite --version` shows version string
- [ ] **UX-04**: `--dry-run` on mutations shows what would be sent without executing

## v2 Requirements

### Bulk Operations

- **BULK-01**: User can pipe JSON/JSONL via stdin for bulk creates (`cat deals.json | pipelite deals create --stdin`)
- **BULK-02**: User can chain commands via pipes for bulk workflows

### Config Profiles

- **PROF-01**: User can switch between CRM instances with `--profile=staging`
- **PROF-02**: Profiles stored as `[profiles.name]` sections in config.toml

### Activity Shortcut

- **ALOG-01**: User can quickly log activity with `pipelite log --deal=123 --type=call --note="..."`

### Additional Output Formats

- **OUTP-08**: JSONL output format (one JSON object per line for streaming)

## Out of Scope

| Feature | Reason |
|---------|--------|
| TUI / full-screen interface | Kills pipeability, massive complexity, different product |
| Webhook management | Server-side concern, not CLI responsibility |
| User/permission management | Admin features, defer to web UI |
| Offline mode with sync | Conflict resolution too complex for v1, caching is read-only |
| Custom scripting language / DSL | Shell (bash/zsh) is the scripting language; CLI should be composable |
| Plugin/extension system | Bounded domain (6 entities) doesn't warrant extensibility overhead |
| Real-time streaming / watching | Different interaction model; users can use system `watch` command |
| Import/export wizards | Complex multi-step flows better handled by web UI |
| Auto-update mechanism | Distribution concern; use cargo-binstall or package managers |
| Email/notification sending | Server-side concern; CLI reads/writes CRM data only |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| AUTH-01 | Phase 1 | Complete |
| AUTH-02 | Phase 1 | Complete |
| AUTH-03 | Phase 1 | Complete |
| AUTH-04 | Phase 1 | Complete |
| CONF-01 | Phase 1 | Complete |
| CONF-02 | Phase 1 | Complete |
| CONF-03 | Phase 1 | Complete |
| CONF-04 | Phase 1 | Complete |
| DEAL-01 | Phase 2 | Complete |
| DEAL-02 | Phase 2 | Complete |
| DEAL-03 | Phase 2 | Complete |
| DEAL-04 | Phase 2 | Complete |
| DEAL-05 | Phase 2 | Complete |
| ORG-01 | Phase 3 | Complete |
| ORG-02 | Phase 3 | Complete |
| ORG-03 | Phase 3 | Complete |
| ORG-04 | Phase 3 | Complete |
| ORG-05 | Phase 3 | Complete |
| PEOP-01 | Phase 3 | Complete |
| PEOP-02 | Phase 3 | Complete |
| PEOP-03 | Phase 3 | Complete |
| PEOP-04 | Phase 3 | Complete |
| PEOP-05 | Phase 3 | Complete |
| ACTV-01 | Phase 3 | Complete |
| ACTV-02 | Phase 3 | Complete |
| ACTV-03 | Phase 3 | Complete |
| ACTV-04 | Phase 3 | Complete |
| ACTV-05 | Phase 3 | Complete |
| PIPE-01 | Phase 3 | Complete |
| PIPE-02 | Phase 3 | Complete |
| PIPE-03 | Phase 3 | Complete |
| PIPE-04 | Phase 3 | Complete |
| PIPE-05 | Phase 3 | Complete |
| STAG-01 | Phase 3 | Complete |
| STAG-02 | Phase 3 | Complete |
| STAG-03 | Phase 3 | Complete |
| STAG-04 | Phase 3 | Complete |
| STAG-05 | Phase 3 | Complete |
| OUTP-01 | Phase 2 | Complete |
| OUTP-02 | Phase 2 | Complete |
| OUTP-03 | Phase 2 | Complete |
| OUTP-04 | Phase 2 | Complete |
| OUTP-05 | Phase 2 | Complete |
| OUTP-06 | Phase 2 | Complete |
| OUTP-07 | Phase 2 | Complete |
| FILT-01 | Phase 2 | Complete |
| FILT-02 | Phase 2 | Complete |
| ERRH-01 | Phase 1 | Complete |
| ERRH-02 | Phase 1 | Complete |
| ERRH-03 | Phase 1 | Complete |
| INTR-01 | Phase 4 | Pending |
| INTR-02 | Phase 4 | Pending |
| INTR-03 | Phase 4 | Pending |
| HEAD-01 | Phase 4 | Pending |
| HEAD-02 | Phase 4 | Pending |
| HEAD-03 | Phase 4 | Pending |
| SHLL-01 | Phase 4 | Pending |
| SHLL-02 | Phase 4 | Pending |
| SHLL-03 | Phase 4 | Pending |
| CACH-01 | Phase 5 | Pending |
| CACH-02 | Phase 5 | Pending |
| CACH-03 | Phase 5 | Pending |
| DASH-01 | Phase 5 | Pending |
| DASH-02 | Phase 5 | Pending |
| UX-01 | Phase 5 | Pending |
| UX-02 | Phase 1 | Complete |
| UX-03 | Phase 1 | Complete |
| UX-04 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 68 total
- Mapped to phases: 68
- Unmapped: 0

---
*Requirements defined: 2026-03-23*
*Last updated: 2026-03-23 after roadmap creation*
