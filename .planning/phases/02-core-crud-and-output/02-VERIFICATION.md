---
phase: 02-core-crud-and-output
verified: 2026-03-24T00:00:00Z
status: human_needed
score: 15/15 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 13/15
  gaps_closed:
    - "User can get a single deal by ID with pipelite deals get <id> in key-value layout -- inverted condition fixed"
    - "Field selection filters columns across all output formats using dot notation -- root-cause bug resolved"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Verify pipelite deals get <id> --fields id,notes renders the notes field"
    expected: "Key-value display showing id and notes values (notes is not in the 6-column default)"
    why_human: "Requires a live Pipelite server with a valid API key to exercise the actual API call path"
  - test: "Verify TTY auto-detection: run pipelite deals list piped to cat"
    expected: "Output format is JSON (not table) when stdout is not a TTY"
    why_human: "TTY detection is verified in code but live behavior requires running the binary"
  - test: "Verify NO_COLOR env var suppresses color in table output"
    expected: "Table headers have no ANSI escape sequences when NO_COLOR is set"
    why_human: "Requires a TTY and live API to exercise the full color path"
---

# Phase 2: Core CRUD and Output Verification Report

**Phase Goal:** Users can perform full CRUD on deals with all output formats, filtering, and field selection -- proving the complete vertical stack
**Verified:** 2026-03-24T00:00:00Z
**Status:** human_needed
**Re-verification:** Yes — after gap closure (plan 02-03, commit fb68f25)

## Re-Verification Summary

Previous verification (2026-03-25T01:39:43Z) found 2 gaps, both tracing to a single root-cause bug: an inverted `is_some()`/`is_none()` conditional in `src/commands/deals/get.rs` that prevented `--fields` from selecting non-default fields on `deals get`. Gap closure plan 02-03 removed the conditional entirely, always passing `all_columns` as the base. The stale `#[allow(dead_code)]` attributes on `render_list` and `render_single` in `src/output/mod.rs` were also removed.

Both gaps are now closed. Full test suite passes with 96 tests, zero failures. No regressions found.

---

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | JSON output renders valid pretty-printed JSON for single items and arrays | VERIFIED | `src/output/json.rs`: `format_list`/`format_single` call `serde_json::to_string_pretty`. Unit tests verify valid JSON array and object output. |
| 2  | Table output renders aligned columns with headers, truncation, and pagination footer | VERIFIED | `src/output/table.rs`: comfy-table with `ContentArrangement::Dynamic`, `ColumnConstraint::UpperBoundary(Width::Fixed(40))` on title, pagination footer "Showing X-Y of Z". Tests verify row count, values, and footer. |
| 3  | CSV output includes header row with proper escaping | VERIFIED | `src/output/csv.rs`: `csv::WriterBuilder` writes header then data rows. Tests confirm header present, special chars quoted, quotes doubled. |
| 4  | Plain output produces tab-separated values with no headers | VERIFIED | `src/output/plain.rs`: joins columns with `\t`, no header row. Tests confirm 1 line per item and no header. |
| 5  | Field selection filters columns across all output formats using dot notation | VERIFIED | `src/output/fields.rs`: `extract_field`/`filter_fields` work with dot-notation. Root-cause bug in `get.rs` fixed: `let columns = all_columns;` — no conditional, all 13 fields always available as base. `render_single` narrows via `--fields`. |
| 6  | Color is applied to table headers/borders and respects NO_COLOR/--no-color | VERIFIED | `src/context.rs`: `color` flag set to false if `--no-color` or `NO_COLOR` env var. `src/output/table.rs`: headers use `col.dimmed()` when color=true. |
| 7  | TTY auto-detection chooses table for terminal and JSON for pipe | VERIFIED | `src/output/mod.rs`: `detect_format` calls `stdout().is_terminal()` to choose Table vs Json when no explicit format given. |
| 8  | User can list deals with pipelite deals list and see a formatted table | VERIFIED | `src/commands/deals/list.rs`: calls `client.list_deals()`, converts to values, calls `output::render_list`. Integration test confirms `--help` shows filter flags. |
| 9  | User can get a single deal by ID with pipelite deals get <id> in key-value layout | VERIFIED | `src/commands/deals/get.rs` line 40: `let columns = all_columns;` — unconditionally uses all 13 fields. Comment at lines 37-39 documents intent. `output::render_single` narrows to user `--fields` when provided. No inverted conditional remains (grep for `is_some`/`is_none`/`deals_table_config` returns empty). |
| 10 | User can create a deal with pipelite deals create --title X --stage Y | VERIFIED | `src/commands/deals/create.rs`: validates title/stage, builds `DealCreate`, calls `client.create_deal()`, renders single result. |
| 11 | User can update a deal with pipelite deals update <id> --title X | VERIFIED | `src/commands/deals/update.rs`: builds `DealUpdate` from optional flags, calls `client.update_deal()`, renders result. |
| 12 | User can delete a deal with pipelite deals delete <id> | VERIFIED | `src/commands/deals/delete.rs`: calls `client.delete_deal()`, prints confirmation respecting quiet flag. |
| 13 | User can filter deals by --stage, --org, --owner flags | VERIFIED | `src/cli/deals.rs`: all three filter flags defined. `src/commands/deals/list.rs`: mapped to `DealsListParams`. `src/api/mod.rs`: `to_query_pairs()` converts them to query params. |
| 14 | User can paginate with --limit and --offset, and --all auto-paginates | VERIFIED | `src/commands/deals/list.rs`: `fetch_page` uses limit/offset; `fetch_all` loops in batches of 100 up to 1000, prints stderr warning when more exist. |
| 15 | Output format auto-detects TTY and can be overridden with --format | VERIFIED | `src/cli/mod.rs`: `--format` global flag. `src/context.rs`: `detect_format(cli.format.clone())`. |

**Score:** 15/15 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/api/models.rs` | Deal, DealCreate, DealUpdate, ApiListResponse\<T\>, PaginationMeta structs | VERIFIED | All 5 structs present. Deal has 13 fields. DealCreate/DealUpdate have `skip_serializing_if` on all optional fields. |
| `src/output/table.rs` | Table rendering with comfy-table — render_list, render_single | VERIFIED | Both functions present and substantive. `ContentArrangement::Dynamic`, pagination footer, key-value single view. Tests pass. |
| `src/output/json.rs` | JSON output rendering — render_list, render_single | VERIFIED | Both present. Field filtering via `filter_fields`. Internal `format_*` helpers for testability. Tests pass. |
| `src/output/csv.rs` | CSV output rendering — render_list | VERIFIED | Present. Uses `csv::WriterBuilder`, header row, proper escaping. Tests pass. |
| `src/output/plain.rs` | Plain tab-separated output — render_list | VERIFIED | Present. Tab-separated, no headers. Tests pass. |
| `src/output/fields.rs` | Field selection and dot-notation extraction — extract_field, filter_fields | VERIFIED | Both present and substantive with correct dot-notation traversal. 9 unit tests. |
| `src/output/format.rs` | Value formatting — format_value | VERIFIED | Present. Relative dates via `HumanTime::from`, currency with comma separators, truncation. 13 unit tests. |
| `src/output/mod.rs` | Clean render_list/render_single without stale dead_code allows | VERIFIED | No `#[allow(dead_code)]` on either function (grep returns empty). Both functions called by command handlers. |
| `src/cli/deals.rs` | DealsCommands enum with List/Get/Create/Update/Delete subcommands | VERIFIED | All 5 variants present with complete args structs. |
| `src/commands/deals/mod.rs` | Deal command dispatch — pub async fn run | VERIFIED | Dispatch function present, routes all 5 subcommands. |
| `src/commands/deals/list.rs` | Deal list handler with filtering, pagination, auto-paginate — pub async fn run | VERIFIED | Substantive implementation. fetch_page and fetch_all paths. |
| `src/commands/deals/get.rs` | Corrected field selection logic — uses all_columns unconditionally | VERIFIED | Line 40: `let columns = all_columns;`. No conditional block, no `deals_table_config` import. 13 fields always available as base. |
| `src/api/mod.rs` | PipeliteClient CRUD methods for deals — list_deals | VERIFIED | All 5 CRUD methods plus batch_create_deals and handle_response generic. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/output/table.rs` | comfy-table | `ContentArrangement::Dynamic` | WIRED | `use comfy_table::{..., ContentArrangement, ...}`. `table.set_content_arrangement(ContentArrangement::Dynamic)`. |
| `src/output/csv.rs` | csv crate | `csv::WriterBuilder` | WIRED | `csv::WriterBuilder::new().from_writer(Vec::new())`. |
| `src/output/format.rs` | chrono-humanize | `HumanTime::from` | WIRED | `use chrono_humanize::HumanTime`. `HumanTime::from(dt)`. |
| `src/output/mod.rs` | all output submodules | `match.*OutputFormat` | WIRED | Lines 54-61 and 74-81: `match format` dispatches to json/table/csv/plain submodules. |
| `src/main.rs` | `src/commands/deals/mod.rs` | `Commands::Deals` match arm | WIRED | `Commands::Deals(ref cmd) => { let ctx = AppContext::build(&cli)?; commands::deals::run(&ctx, cmd).await }`. |
| `src/commands/deals/list.rs` | `src/api/mod.rs` | `client.list_deals()` | WIRED | `ctx.client.list_deals(&params).await?`. |
| `src/commands/deals/list.rs` | `src/output/mod.rs` | `output::render_list()` | WIRED | `output::render_list(...)` in both fetch_page and fetch_all paths. |
| `src/commands/deals/get.rs` | `src/output/mod.rs` | `output::render_single` with all_columns as base | WIRED | Line 42: `output::render_single(&item, &ctx.output_format, &columns, &args.fields, ctx.color)`. `columns` is now always the full 13-field set. |
| `src/cli/deals.rs` | `src/cli/mod.rs` | `Commands::Deals(DealsCommands)` variant | WIRED | `src/cli/mod.rs`: `Deals(DealsCommands)`. |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| OUTP-01 | 02-01 | JSON output with `--format json` (default when piped) | SATISFIED | `detect_format` returns Json when not TTY; `json::render_list/render_single` produce pretty JSON |
| OUTP-02 | 02-01 | Table output (default when interactive TTY) | SATISFIED | `detect_format` returns Table when TTY; `table::render_list` uses comfy-table |
| OUTP-03 | 02-01 | CSV output with `--format csv` | SATISFIED | `csv::render_list` with header row and escaping |
| OUTP-04 | 02-01 | Plain output with `--format plain` | SATISFIED | `plain::render_list` tab-separated, no headers |
| OUTP-05 | 02-01, 02-03 | Field selection with `--fields=id,title,value` | SATISFIED | Bug fixed in 02-03. `get.rs` always passes all 13 fields as base. All fields (including notes, person_id, position) selectable via `--fields` across all commands. |
| OUTP-06 | 02-01 | TTY auto-detection | SATISFIED | `detect_format` uses `stdout().is_terminal()` |
| OUTP-07 | 02-01 | Color with NO_COLOR/--no-color | SATISFIED | `context.rs` checks both `--no-color` flag and `NO_COLOR` env var |
| DEAL-01 | 02-02 | `pipelite deals list` | SATISFIED | Full list command with filtering and pagination |
| DEAL-02 | 02-02, 02-03 | `pipelite deals get <id>` | SATISFIED | Works for basic get; `--fields` can now select any of the 13 fields including notes, position, person_id |
| DEAL-03 | 02-02 | `pipelite deals create` | SATISFIED | Single create and batch stdin create both implemented |
| DEAL-04 | 02-02 | `pipelite deals update <id>` | SATISFIED | All optional fields supported including custom-field |
| DEAL-05 | 02-02 | `pipelite deals delete <id>` | SATISFIED | Deletes and prints confirmation respecting quiet flag |
| FILT-01 | 02-02 | Filter by entity-specific fields (`--stage`, `--owner`) | SATISFIED | --stage, --org, --owner mapped to DealsListParams and query pairs |
| FILT-02 | 02-02 | Paginate with `--limit` and `--offset` | SATISFIED | Both flags present, default 50/0, --all auto-paginates up to 1000 |

**All 14 requirement IDs from plan frontmatter accounted for.**

Orphaned requirements check: REQUIREMENTS.md traceability maps DEAL-01 through DEAL-05, OUTP-01 through OUTP-07, FILT-01, FILT-02 to Phase 2 — all are claimed by plans 02-01, 02-02, and 02-03. No orphaned requirements.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/output/table.rs` | — | `truncate_with_ellipsis` function unused (cargo warning) | Info | Dead helper function; does not affect correctness. Pre-existing, not introduced by this phase. |
| `src/context.rs` | — | Fields `config` and `verbose` never read (cargo warning) | Info | Pre-existing dead fields in AppContext. Does not affect functionality. |

No blockers. No warnings on the two files modified by plan 02-03 (`get.rs`, `output/mod.rs`). The 3 remaining cargo warnings are all pre-existing and outside the scope of this phase.

---

### Human Verification Required

#### 1. Field Selection on Get with Non-Default Fields

**Test:** Run `pipelite deals get <real_id> --fields id,notes,position` against a live server
**Expected:** Key-value output showing id, notes, and position values (previously broken fields)
**Why human:** Requires a live Pipelite server with a valid API key to exercise the actual API call path

#### 2. TTY Auto-Detection in Practice

**Test:** Run `pipelite deals list | cat` in a terminal with a valid config
**Expected:** Output is JSON (not table) because stdout is piped
**Why human:** TTY detection behavior depends on runtime environment; code confirms it should work but only a live run proves it

#### 3. NO_COLOR Env Var

**Test:** Run `NO_COLOR=1 pipelite deals list` against a live server
**Expected:** Table headers are plain text with no ANSI color codes
**Why human:** Requires a live server and TTY to verify color/no-color rendering

---

### Test Suite Results

Full test suite after gap closure:

| Suite | Tests | Result |
|-------|-------|--------|
| Unit tests (src/) | 69 | ok |
| output tests | 7 | ok |
| api/models tests | 10 | ok |
| cli_skeleton_tests | 2 | ok |
| command_help_tests | 5 | ok |
| quiet_mode_test | 2 | ok |
| version_test | 1 | ok |
| **Total** | **96** | **0 failures** |

---

### Gaps Summary

All gaps from the previous verification are closed. There are no remaining automated-check failures.

The single root-cause bug — an inverted conditional in `src/commands/deals/get.rs` — was resolved by plan 02-03 (commit fb68f25). The fix removed the conditional entirely (`let columns = all_columns;`), ensuring all 13 deal fields are always available as the base column set for single-item view. The `render_single` function was already correct; it narrows the base set to user-specified `--fields` when provided.

The stale `#[allow(dead_code)]` attributes on `render_list` and `render_single` in `src/output/mod.rs` were also removed. Neither attribute remains (verified by grep).

Phase 2 goal is achieved at the code level. Three items require live-server confirmation and remain as human verification items.

---

_Verified: 2026-03-24T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
