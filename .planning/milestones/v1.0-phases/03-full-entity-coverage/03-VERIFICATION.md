---
phase: 03-full-entity-coverage
verified: 2026-03-25T12:00:00Z
status: passed
score: 21/21 must-haves verified
---

# Phase 3: Full Entity Coverage Verification Report

**Phase Goal:** Replicate CRUD pattern across orgs, people, activities, pipelines, and stages
**Verified:** 2026-03-25
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can run `pipelite orgs list` and see organizations in table format | VERIFIED | OrgsCommands::List wired through main.rs -> commands/orgs/list.rs with orgs_table_config(); 8 integration tests pass |
| 2 | User can run `pipelite orgs get <id>` and see full org details | VERIFIED | commands/orgs/get.rs (29 lines) calls client.get_org() and render_single |
| 3 | User can create, update, and delete organizations via CLI flags | VERIFIED | commands/orgs/create.rs (113 lines), update.rs (53 lines), delete.rs (17 lines) all wired |
| 4 | User can run `pipelite people list` and see people in table format | VERIFIED | PeopleCommands::List wired through main.rs -> commands/people/list.rs with people_table_config(); 8 integration tests pass |
| 5 | User can run `pipelite people get <id>` and see full person details | VERIFIED | commands/people/get.rs (32 lines) calls client.get_person() and render_single |
| 6 | User can create, update, and delete people via CLI flags | VERIFIED | commands/people/create.rs (128 lines) validates --first-name and --last-name, update.rs (55 lines), delete.rs (17 lines) |
| 7 | Both orgs and people support --stdin batch create using /batch API endpoint | VERIFIED | orgs/create.rs calls client.batch_create_orgs(); people/create.rs calls client.batch_create_people() |
| 8 | User can run `pipelite activities list` and see activities in table format | VERIFIED | ActivitiesCommands::List wired; activities_table_config() returns id, title, type_id, deal_id, due_at, completed_at |
| 9 | User can filter activities with --deal, --type, and --done flags | VERIFIED | ActivitiesListArgs has --type, --deal, --done; list.rs maps to ActivitiesListParams |
| 10 | User can run `pipelite activities get <id>` and see full activity details | VERIFIED | commands/activities/get.rs (33 lines) calls client.get_activity() |
| 11 | User can create an activity with --title and --type (required) | VERIFIED | activities/create.rs (158 lines) validates both --title and --type at runtime |
| 12 | User can update an activity and mark it done with --mark-done | VERIFIED | activities/update.rs (120 lines) sets completed_at via chrono::Utc::now() for --mark-done; sends null via update_activity_raw() for --mark-undone |
| 13 | User can delete an activity | VERIFIED | commands/activities/delete.rs (17 lines) calls client.delete_activity() |
| 14 | Batch create via --stdin uses individual-create loop with progress | VERIFIED | activities/create.rs contains eprint!("\rCreating {}/{}...", i + 1, total) with per-item create_activity() calls |
| 15 | User can run `pipelite pipelines list` and see pipelines in table format | VERIFIED | PipelinesCommands::List wired; pipelines_table_config() returns id, name, updated_at; 7 integration tests pass |
| 16 | User can create, get, update, and delete pipelines | VERIFIED | All 5 command handlers present and substantive (317 lines total) |
| 17 | User can run `pipelite stages list --pipeline <id>` and see stages for that pipeline | VERIFIED | StagesListParams has required pipeline_id (String, not Option); list.rs validates and passes to client.list_stages() |
| 18 | Running `pipelite stages list` without --pipeline fails with a helpful error | VERIFIED | stages/list.rs line 16: "Missing required flag: --pipeline" via CliError::Validation; stages_list_without_config_fails_with_exit_code_1 test passes |
| 19 | User can create a stage with --name and --pipeline (both required) | VERIFIED | stages/create.rs (121 lines) validates both required flags; StageCreate has pipeline_id: String required |
| 20 | User can get, update, and delete stages by ID (no --pipeline needed) | VERIFIED | stages/get.rs, update.rs, delete.rs take only stage ID as positional arg |
| 21 | Both pipelines and stages support --stdin with individual-create loop | VERIFIED | pipelines/create.rs (115 lines) and stages/create.rs (121 lines) both implement individual-create loop pattern |

**Score:** 21/21 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/api/models.rs` | All 5 entity structs + Create/Update + table configs | VERIFIED | Organization, Person, Activity, Pipeline, Stage structs + payloads + 5 table_config functions |
| `src/cli/orgs.rs` | OrgsCommands enum | VERIFIED | 5 subcommands (List/Get/Create/Update/Delete) |
| `src/cli/people.rs` | PeopleCommands enum | VERIFIED | 5 subcommands with first_name/last_name args |
| `src/cli/activities.rs` | ActivitiesCommands enum | VERIFIED | 5 subcommands with --mark-done/--mark-undone |
| `src/cli/pipelines.rs` | PipelinesCommands enum | VERIFIED | 5 subcommands, alias "pl" |
| `src/cli/stages.rs` | StagesCommands enum | VERIFIED | 5 subcommands with --pipeline flag, alias "s" |
| `src/commands/orgs/mod.rs` | Org command dispatch | VERIFIED | 5-arm match dispatching to handlers |
| `src/commands/people/mod.rs` | People command dispatch | VERIFIED | 5-arm match dispatching to handlers |
| `src/commands/activities/mod.rs` | Activity command dispatch | VERIFIED | 5-arm match dispatching to handlers |
| `src/commands/pipelines/mod.rs` | Pipeline command dispatch | VERIFIED | 5-arm match dispatching to handlers |
| `src/commands/stages/mod.rs` | Stage command dispatch | VERIFIED | 5-arm match dispatching to handlers |
| `src/commands/stages/list.rs` | Stages list with --pipeline validation | VERIFIED | "Missing required flag: --pipeline" error at line 16 |
| `tests/orgs_integration.rs` | Integration tests | VERIFIED | 8 tests, all passing |
| `tests/people_integration.rs` | Integration tests | VERIFIED | 8 tests, all passing |
| `tests/activities_integration.rs` | Integration tests | VERIFIED | 8 tests, all passing |
| `tests/pipelines_integration.rs` | Integration tests | VERIFIED | 7 tests, all passing |
| `tests/stages_integration.rs` | Integration tests | VERIFIED | 8 tests, all passing |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| src/main.rs | src/commands/orgs/mod.rs | Commands::Orgs match arm | WIRED | Line 43 |
| src/main.rs | src/commands/people/mod.rs | Commands::People match arm | WIRED | Line 47 |
| src/main.rs | src/commands/activities/mod.rs | Commands::Activities match arm | WIRED | Line 51 |
| src/main.rs | src/commands/pipelines/mod.rs | Commands::Pipelines match arm | WIRED | Line 55 |
| src/main.rs | src/commands/stages/mod.rs | Commands::Stages match arm | WIRED | Line 59 |
| src/commands/orgs/create.rs | src/api/mod.rs | client.batch_create_orgs() | WIRED | Batch endpoint call confirmed |
| src/commands/people/create.rs | src/api/mod.rs | client.batch_create_people() | WIRED | Batch endpoint call confirmed |
| src/commands/activities/create.rs | src/api/mod.rs | client.create_activity() in loop | WIRED | Individual-create loop confirmed |
| src/commands/stages/list.rs | src/api/mod.rs | client.list_stages() | WIRED | Pipeline ID passed as required param |
| src/commands/stages/create.rs | src/api/mod.rs | client.create_stage() | WIRED | Pipeline ID included in payload |
| src/cli/mod.rs | All 5 CLI modules | use + Commands enum | WIRED | Lines 13-20: all imports; lines 87-122: all enum variants |
| src/commands/mod.rs | All 5 command modules | pub mod declarations | WIRED | All 5 modules registered |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-----------|-------------|--------|----------|
| ORG-01 | 03-01 | List organizations | SATISFIED | commands/orgs/list.rs with auto-pagination |
| ORG-02 | 03-01 | Get org by ID | SATISFIED | commands/orgs/get.rs calls client.get_org() |
| ORG-03 | 03-01 | Create org | SATISFIED | commands/orgs/create.rs with --stdin batch |
| ORG-04 | 03-01 | Update org | SATISFIED | commands/orgs/update.rs builds OrganizationUpdate |
| ORG-05 | 03-01 | Delete org | SATISFIED | commands/orgs/delete.rs calls client.delete_org() |
| PEOP-01 | 03-01 | List people | SATISFIED | commands/people/list.rs with auto-pagination |
| PEOP-02 | 03-01 | Get person by ID | SATISFIED | commands/people/get.rs calls client.get_person() |
| PEOP-03 | 03-01 | Create person | SATISFIED | commands/people/create.rs validates first_name + last_name |
| PEOP-04 | 03-01 | Update person | SATISFIED | commands/people/update.rs builds PersonUpdate |
| PEOP-05 | 03-01 | Delete person | SATISFIED | commands/people/delete.rs calls client.delete_person() |
| ACTV-01 | 03-02 | List activities | SATISFIED | commands/activities/list.rs with --done filter |
| ACTV-02 | 03-02 | Get activity by ID | SATISFIED | commands/activities/get.rs calls client.get_activity() |
| ACTV-03 | 03-02 | Create activity | SATISFIED | commands/activities/create.rs with individual-create loop for --stdin |
| ACTV-04 | 03-02 | Update activity | SATISFIED | commands/activities/update.rs with --mark-done/--mark-undone |
| ACTV-05 | 03-02 | Delete activity | SATISFIED | commands/activities/delete.rs calls client.delete_activity() |
| PIPE-01 | 03-03 | List pipelines | SATISFIED | commands/pipelines/list.rs with auto-pagination |
| PIPE-02 | 03-03 | Get pipeline by ID | SATISFIED | commands/pipelines/get.rs calls client.get_pipeline() |
| PIPE-03 | 03-03 | Create pipeline | SATISFIED | commands/pipelines/create.rs validates --name |
| PIPE-04 | 03-03 | Update pipeline | SATISFIED | commands/pipelines/update.rs builds PipelineUpdate |
| PIPE-05 | 03-03 | Delete pipeline | SATISFIED | commands/pipelines/delete.rs calls client.delete_pipeline() |
| STAG-01 | 03-03 | List stages | SATISFIED | commands/stages/list.rs with required --pipeline |
| STAG-02 | 03-03 | Get stage by ID | SATISFIED | commands/stages/get.rs (ID only, no --pipeline) |
| STAG-03 | 03-03 | Create stage | SATISFIED | commands/stages/create.rs validates --name + --pipeline |
| STAG-04 | 03-03 | Update stage | SATISFIED | commands/stages/update.rs builds StageUpdate |
| STAG-05 | 03-03 | Delete stage | SATISFIED | commands/stages/delete.rs calls client.delete_stage() |

**All 25 requirements SATISFIED. No orphaned requirements.**

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | - | - | - | - |

No TODOs, FIXMEs, PLACEHOLDERs, unimplemented!(), or todo!() found in any phase 3 files.

### Human Verification Required

### 1. Live API CRUD Operations

**Test:** Run `pipelite orgs create --name "Test Org"` against a real Pipelite instance, then list, get, update, and delete.
**Expected:** Full round-trip succeeds for all 5 entities.
**Why human:** Requires live API key and network access; integration tests only verify CLI arg parsing and help output.

### 2. Batch Create from Stdin

**Test:** Run `echo '[{"name":"Org1"},{"name":"Org2"}]' | pipelite orgs create --stdin` against a real instance.
**Expected:** Both records created, table output shows 2 rows.
**Why human:** Stdin piping and batch API calls require live environment.

### 3. Activities Mark-Done/Undone

**Test:** Create an activity, then run `pipelite activities update <id> --mark-done`, verify completed_at is set. Then `--mark-undone`, verify completed_at is null.
**Expected:** completed_at toggles correctly via the API.
**Why human:** Requires verifying API accepts the chrono timestamp format and null-clearing via raw JSON.

### 4. Stages --pipeline Enforcement

**Test:** Run `pipelite stages list` without --pipeline flag.
**Expected:** Exit code 1 with "Missing required flag: --pipeline" error and usage hint.
**Why human:** Integration test verifies exit code 1 but the error message quality needs human review.

## Build and Test Evidence

- `cargo build` -- clean build, no errors, no warnings
- `cargo test api::models::tests` -- 14/14 model tests pass (10 for phase 3 entities)
- `cargo test --test orgs_integration` -- 8/8 pass
- `cargo test --test people_integration` -- 8/8 pass
- `cargo test --test activities_integration` -- 8/8 pass
- `cargo test --test pipelines_integration` -- 7/7 pass
- `cargo test --test stages_integration` -- 8/8 pass
- Total: 39 integration tests + 10 model unit tests = 49 phase 3 tests passing

## Summary

Phase 3 goal is fully achieved. All 5 entities (organizations, people, activities, pipelines, stages) have complete CRUD implementations following the deals pattern from Phase 2. Each entity has:

- Data models with Create/Update payloads and table config
- 5 API client methods (list, get, create, update, delete) + batch where applicable
- CLI arg definitions with entity-specific flags and filters
- 5 command handlers with full logic (1842 lines total across 30 files)
- Integration tests verifying CLI structure and argument parsing

Key behavioral requirements are met:
- Orgs and People use /batch endpoint for --stdin batch create
- Activities use individual-create loop (no batch endpoint) with progress output
- Stages enforce --pipeline on list and create with helpful error messages
- Activities support --mark-done/--mark-undone for completed_at management
- Correct aliases: orgs="o", people="p", activities="a", pipelines="pl", stages="s"

---

_Verified: 2026-03-25_
_Verifier: Claude (gsd-verifier)_
