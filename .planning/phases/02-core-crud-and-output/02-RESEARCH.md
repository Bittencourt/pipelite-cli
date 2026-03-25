# Phase 2: Core CRUD and Output - Research

**Researched:** 2026-03-24
**Domain:** Rust CLI CRUD operations, table/CSV/JSON output formatting, API client patterns
**Confidence:** HIGH

## Summary

Phase 2 builds the complete vertical stack for deals: typed API models, CRUD HTTP methods on `PipeliteClient`, a `deals` subcommand tree (list/get/create/update/delete), and a multi-format output system (table, JSON, CSV, plain). The existing codebase already has `comfy-table 7.2` in dependencies, `OutputFormat` enum with all four variants, TTY detection via `detect_format()`, and an `AppContext` that carries format/color/quiet settings. The `csv` crate (BurntSushi) is the standard Rust CSV library and needs to be added. `chrono` is needed for date parsing and `chrono-humanize` for relative time display in tables.

The primary pattern is: each output format is a trait implementation (or match arm) that takes a generic entity list/single entity and writes to stdout. The output module becomes the reusable backbone -- deals prove it, then Phase 3 entities reuse it unchanged. Column width calculation, truncation with ellipsis, right-alignment for currency, and relative date formatting are the main table rendering concerns. comfy-table handles all of these natively.

**Primary recommendation:** Build a generic output rendering layer in `src/output/` that accepts `serde_json::Value` (or a trait-based approach) so all four formats share one code path per entity, then wire deals as the first consumer. Use `comfy-table` for tables (already a dependency), `csv` crate for CSV, and `serde_json` (already a dependency) for JSON.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Deal data model: typed `Deal` struct with id, title, value (nullable), stage_id, organization_id (nullable), person_id (nullable), owner_id, position (nullable), expected_close_date (nullable), notes (nullable), custom_fields (nullable object), created_at, updated_at
- `DealCreate` requires: title, stage_id. Optional: value, organization_id, person_id, expected_close_date, notes, custom_fields
- `DealUpdate`: all fields optional
- Custom fields are first-class -- dynamic column support in table output, selectable via `--fields`, filterable
- `--expand <relations>` flag on `deals list` and `deals get` to inline related entities (maps to API `?expand=` param). Expanded fields accessible via dot notation in `--fields`
- Default table columns for `deals list`: id, title, value, stage_id, owner_id, updated_at
- Long values truncated with ellipsis to fit terminal width
- `deals get <id>` displays as key-value pairs (vertical layout like `gh issue view`), not a single-row table
- `--format plain` outputs tab-separated values, no headers, no borders
- `--format csv` includes header row, standard CSV escaping
- `--format json` outputs full JSON (array for list, object for get)
- `--fields=id,title,value` selects specific fields across all output formats
- Custom fields accessible in --fields via dot notation: `--fields=id,title,custom_fields.industry`
- Named flags for deal-specific filters: `--stage <id>`, `--org <id>`, `--owner <id>`
- `--limit` and `--offset` pass through to API (default 50, max 100 per request)
- Table footer shows pagination meta: "Showing 1-50 of 234"
- `--all` flag auto-paginates in batches of 100, capped at 1000 records with warning
- Named flags per field for create/update: `--title "Big Deal" --stage abc123 --value 50000`
- Custom fields via repeatable flag: `--custom-field industry=Tech --custom-field priority=high`
- Missing required fields on create = error with hint listing required fields
- Successful create/update echoes the full deal in current output format
- Successful delete prints confirmation message
- Basic batch support via `--stdin`: reads JSON array from stdin, calls `POST /deals/batch`
- Table borders and headers in dim/white, values in default color
- Value formatting: currency values right-aligned, dates formatted as relative ("2h ago") in table, ISO in JSON
- API uses string IDs (not numeric), pagination via offset/limit with meta object: `{ "data": [...], "meta": { "total": N, "offset": N, "limit": N } }`
- API supports relation expansion via `?expand=owner,organization` query param

### Claude's Discretion
- Table rendering library choice (comfy-table, tabled, custom) -- RECOMMENDATION: Use comfy-table 7.2 (already in Cargo.toml)
- Column width calculation algorithm -- RECOMMENDATION: Use comfy-table's Dynamic content arrangement with UpperBoundary constraints
- CSV library choice -- RECOMMENDATION: Use `csv` crate (BurntSushi, the standard)
- Exact error messages for validation failures
- Batch endpoint error handling (partial success reporting)
- How expanded relations render in table vs key-value view

### Deferred Ideas (OUT OF SCOPE)
- Interactive prompts with dropdowns for stage/pipeline selection -- Phase 4 (INTR-01, INTR-02)
- Bulk operations beyond basic batch create (BULK-01, BULK-02) -- v2
- JSONL streaming output (OUTP-08) -- v2
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DEAL-01 | List deals with `pipelite deals list` | API client GET /deals with query params, table output with pagination footer |
| DEAL-02 | Get deal by ID with `pipelite deals get <id>` | API client GET /deals/:id, key-value vertical display |
| DEAL-03 | Create deal with `pipelite deals create` | API client POST /deals, named flags for fields, validation of required fields |
| DEAL-04 | Update deal with `pipelite deals update <id>` | API client PUT /deals/:id, all optional flags, echo updated deal |
| DEAL-05 | Delete deal with `pipelite deals delete <id>` | API client DELETE /deals/:id, confirmation message |
| OUTP-01 | JSON output with `--format json` (default when piped) | serde_json::to_string_pretty, already have detect_format() |
| OUTP-02 | Table output (default when interactive TTY) | comfy-table 7.2 with Dynamic arrangement, truncation, column constraints |
| OUTP-03 | CSV output with `--format csv` | csv crate Writer with serde Serialize |
| OUTP-04 | Plain output with `--format plain` (values only) | Tab-separated, no headers -- simple write loop |
| OUTP-05 | Field selection with `--fields=id,title,value` | Parse comma-separated field names, filter serde_json::Value keys |
| OUTP-06 | Auto-detect TTY vs pipe | Already implemented in detect_format() |
| OUTP-07 | Colored output with NO_COLOR support | Already in AppContext.color; use colored crate (already dep) for table styling |
| FILT-01 | Filter lists by entity-specific fields | Named clap flags mapping to API query params |
| FILT-02 | Paginate with `--limit` and `--offset` | Pass-through to API, display meta in table footer |
</phase_requirements>

## Standard Stack

### Core (already in Cargo.toml)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| comfy-table | 7.2 | Table rendering with auto-width, truncation, alignment | Already a dependency; handles terminal width detection, UTF-8 truncation, column constraints natively |
| serde + serde_json | 1.0 | JSON serialization/deserialization | Already a dependency; used for API models and JSON output |
| reqwest | 0.13 | HTTP client for API calls | Already a dependency; PipeliteClient wraps it |
| clap | 4.6 | CLI argument parsing with derive | Already a dependency; subcommand structure established |
| colored | 3.1 | Terminal color output | Already a dependency; respects NO_COLOR via AppContext |

### New Dependencies Needed
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| csv | 1.3 | CSV output writing with proper escaping | BurntSushi's csv is THE standard Rust CSV library. 250M+ downloads, handles escaping/quoting correctly |
| chrono | 0.4 | Parse ISO 8601 dates from API responses | Standard Rust datetime library, needed for date parsing and formatting |
| chrono-humanize | 0.2 | Relative time display ("2h ago", "3 days ago") | Small crate that converts chrono DateTime to human-readable relative strings |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| comfy-table | tabled | tabled is more feature-rich but comfy-table is already in deps and sufficient |
| chrono-humanize | Manual formatting | chrono-humanize handles edge cases (pluralization, tense) correctly |
| csv crate | Manual CSV writing | CSV escaping is deceptively complex (embedded commas, quotes, newlines) |

**Installation:**
```bash
cargo add csv@1.3 chrono@0.4 --features chrono/serde chrono-humanize@0.2
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── api/
│   ├── mod.rs           # PipeliteClient with new CRUD methods
│   └── models.rs        # Deal, DealCreate, DealUpdate, ApiListResponse<T>, PaginationMeta
├── cli/
│   ├── mod.rs           # Cli struct, Commands enum (add Deals variant)
│   ├── deals.rs         # DealsCommands enum + DealsListArgs, DealsCreateArgs, etc.
│   ├── config.rs        # (existing)
│   └── init.rs          # (existing)
├── commands/
│   ├── mod.rs           # (add deals module)
│   ├── deals/
│   │   ├── mod.rs       # dispatch to list/get/create/update/delete
│   │   ├── list.rs      # deals list handler
│   │   ├── get.rs       # deals get handler
│   │   ├── create.rs    # deals create handler
│   │   ├── update.rs    # deals update handler
│   │   └── delete.rs    # deals delete handler
│   ├── config/          # (existing)
│   ├── init.rs          # (existing)
│   └── ping.rs          # (existing)
├── output/
│   ├── mod.rs           # OutputFormat enum, detect_format() (existing)
│   ├── table.rs         # Generic entity table rendering with comfy-table
│   ├── json.rs          # JSON output (pretty-print for TTY, compact for pipe)
│   ├── csv.rs           # CSV output with header row
│   ├── plain.rs         # Tab-separated plain output
│   ├── fields.rs        # --fields parsing and value extraction (dot notation)
│   └── format.rs        # Value formatting: currency, dates, truncation
├── context.rs           # AppContext (existing)
├── config.rs            # (existing)
├── error.rs             # CliError (add Validation variant)
└── main.rs              # (add deals dispatch)
```

### Pattern 1: Generic API Response Wrapper
**What:** All list endpoints return `{ "data": [...], "meta": { "total": N, "offset": N, "limit": N } }`. Model this once.
**When to use:** Every list command.
**Example:**
```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiListResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Deserialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}
```

### Pattern 2: Output Rendering via serde_json::Value
**What:** Convert typed structs to `serde_json::Value` for uniform field selection and formatting across all output modes.
**When to use:** Every command that produces output.
**Example:**
```rust
use serde_json::Value;

/// Render a list of entities in the requested format.
pub fn render_list(
    items: &[Value],
    format: &OutputFormat,
    fields: &[String],
    color: bool,
    meta: Option<&PaginationMeta>,
) -> Result<()> {
    match format {
        OutputFormat::Json => json::render_list(items, fields),
        OutputFormat::Table => table::render_list(items, fields, color, meta),
        OutputFormat::Csv => csv::render_list(items, fields),
        OutputFormat::Plain => plain::render_list(items, fields),
    }
}
```

### Pattern 3: Entity-Specific Table Config
**What:** Each entity defines its default columns, column constraints, and value formatters.
**When to use:** When rendering tables for different entity types.
**Example:**
```rust
pub struct TableConfig {
    pub default_columns: Vec<&'static str>,
    pub column_formatters: HashMap<String, Box<dyn Fn(&Value) -> String>>,
    pub column_alignments: HashMap<String, CellAlignment>,
}

pub fn deals_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "title", "value", "stage_id", "owner_id", "updated_at"],
        // value -> right-aligned currency, updated_at -> relative time
        ..
    }
}
```

### Pattern 4: Clap Subcommand Tree for Entity CRUD
**What:** Noun-verb structure: `pipelite deals list|get|create|update|delete`
**When to use:** Every entity command.
**Example:**
```rust
#[derive(Subcommand)]
pub enum DealsCommands {
    /// List deals
    #[command(after_help = "Examples:\n  pipelite deals list\n  pipelite deals list --stage abc123 --limit 10")]
    List(DealsListArgs),
    /// Get a deal by ID
    Get(DealsGetArgs),
    /// Create a new deal
    Create(DealsCreateArgs),
    /// Update a deal
    Update(DealsUpdateArgs),
    /// Delete a deal
    Delete(DealsDeleteArgs),
}

#[derive(Args)]
pub struct DealsListArgs {
    /// Filter by stage ID
    #[arg(long)]
    pub stage: Option<String>,
    /// Filter by organization ID
    #[arg(long)]
    pub org: Option<String>,
    /// Filter by owner ID
    #[arg(long)]
    pub owner: Option<String>,
    /// Maximum number of results
    #[arg(long, default_value = "50")]
    pub limit: u64,
    /// Offset for pagination
    #[arg(long, default_value = "0")]
    pub offset: u64,
    /// Fetch all results (auto-paginate, max 1000)
    #[arg(long)]
    pub all: bool,
    /// Select specific fields
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
    /// Expand related entities
    #[arg(long, value_delimiter = ',')]
    pub expand: Option<Vec<String>>,
}
```

### Pattern 5: PipeliteClient CRUD Methods
**What:** Typed async methods on the client for each API operation.
**When to use:** All API interactions.
**Example:**
```rust
impl PipeliteClient {
    pub async fn list_deals(&self, params: &DealsListParams) -> Result<ApiListResponse<Deal>> {
        let url = format!("{}/api/v1/deals", self.base_url);
        let response = self.client.get(&url)
            .query(&params.to_query_pairs())
            .send()
            .await
            .map_err(|e| self.map_request_error(e))?;
        self.handle_response(response).await
    }

    pub async fn get_deal(&self, id: &str) -> Result<Deal> { ... }
    pub async fn create_deal(&self, data: &DealCreate) -> Result<Deal> { ... }
    pub async fn update_deal(&self, id: &str, data: &DealUpdate) -> Result<Deal> { ... }
    pub async fn delete_deal(&self, id: &str) -> Result<()> { ... }
    pub async fn batch_create_deals(&self, deals: &[DealCreate]) -> Result<Vec<Deal>> { ... }
}
```

### Pattern 6: Key-Value Vertical Display for Single Entity
**What:** `deals get <id>` renders as key-value pairs, not a table row.
**When to use:** Single-entity detail views.
**Example output:**
```
Title:              Big Enterprise Deal
Value:              $50,000
Stage:              abc-123
Owner:              def-456
Organization:       ghi-789
Expected Close:     2026-04-15
Notes:              Follow up next week
Created:            3 days ago
Updated:            2 hours ago
```

### Anti-Patterns to Avoid
- **Hand-rolling CSV escaping:** CSV has edge cases with embedded commas, quotes, and newlines. Use the `csv` crate.
- **Formatting in command handlers:** Keep formatting in `src/output/`. Command handlers should return data, not format strings.
- **One-off output code per entity:** Build the output layer generically now so Phase 3 entities reuse it without duplication.
- **Blocking stdin read without checking:** When implementing `--stdin`, always check if stdin is a pipe/redirect, not a TTY. Use `std::io::stdin().is_terminal()`.
- **Hardcoding default columns in the output layer:** Default columns are entity-specific; pass them as configuration, not constants in the output module.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CSV output | Manual comma-joining with quote escaping | `csv` crate Writer | Embedded commas, quotes, newlines, Unicode -- CSV escaping has many edge cases |
| Table layout | Manual column width calculation and padding | `comfy-table` Dynamic arrangement | Terminal width detection, UTF-8 grapheme handling, word wrapping, truncation with indicator |
| Relative dates | Manual "X minutes ago" math | `chrono-humanize` | Pluralization, tense, edge cases (just now, yesterday, etc.) |
| HTTP error mapping | Per-method error handling | Shared `handle_response()` method on PipeliteClient | 401/403/404/422/500 mapping should be consistent across all endpoints |
| Query string building | Manual `format!("?limit={}&offset={}")` | reqwest `.query()` with serialize | Handles URL encoding, optional params, special characters |

**Key insight:** The output rendering layer is the most reusable asset in this phase. Invest in making it generic -- it serves 6 entity types across 5 output operations each.

## Common Pitfalls

### Pitfall 1: Serde Flatten for Custom Fields
**What goes wrong:** Using `#[serde(flatten)]` with `HashMap<String, Value>` for custom_fields can cause deserialization issues when the API wraps custom fields in a `custom_fields` object.
**Why it happens:** The API returns `"custom_fields": { "industry": "Tech" }` -- it's a nested object, not flattened keys.
**How to avoid:** Model `custom_fields` as `Option<serde_json::Value>` or `Option<HashMap<String, serde_json::Value>>`. Access dot-notation fields by parsing the path and traversing the Value tree.
**Warning signs:** Deserialization errors when API returns deals with custom fields.

### Pitfall 2: Terminal Width in Non-TTY Contexts
**What goes wrong:** comfy-table's auto-width detection fails when stdout is piped (no terminal).
**Why it happens:** Table format should only be used in TTY; but if user forces `--format table` while piping, terminal width is unknown.
**How to avoid:** When `--format table` is forced and stdout is not a TTY, fall back to a default width (e.g., 120 columns). Use `comfy-table`'s `set_width()` explicitly.
**Warning signs:** Tables rendering with broken layout when piped.

### Pitfall 3: Auto-Pagination Memory
**What goes wrong:** `--all` flag fetches up to 1000 records into memory before rendering.
**Why it happens:** Need all data to render a table with consistent column widths.
**How to avoid:** Cap at 1000 with clear warning. Fetch in batches of 100, accumulate into Vec. For JSON/plain/csv output, could stream -- but table needs all data upfront for column width calculation.
**Warning signs:** High memory usage with large result sets.

### Pitfall 4: Colored Output in Tests
**What goes wrong:** Tests that assert on output strings fail because colored output includes ANSI escape codes.
**Why it happens:** `colored` crate checks environment for TTY.
**How to avoid:** Use `colored::control::set_override(false)` in test setup, or assert on content without color codes. The existing codebase already shows this pattern.
**Warning signs:** Test assertions on exact string matches failing in CI.

### Pitfall 5: Option Fields in Update Payloads
**What goes wrong:** Sending `null` for unset optional fields on update overwrites existing values.
**Why it happens:** `serde` serializes `None` as `null` by default with `#[serde(serialize_with)]`.
**How to avoid:** Use `#[serde(skip_serializing_if = "Option::is_none")]` on all DealUpdate fields so only provided fields are sent in the PATCH/PUT body.
**Warning signs:** Updating one field clears other optional fields.

### Pitfall 6: Stdin JSON Parsing for Batch
**What goes wrong:** `--stdin` reads from stdin but blocks indefinitely when stdin is a TTY.
**Why it happens:** No EOF signal from terminal input.
**How to avoid:** Check `std::io::stdin().is_terminal()` before reading. If TTY and `--stdin` is set, return an error with hint: "Pipe JSON data via stdin or use named flags."
**Warning signs:** CLI hangs when user types `pipelite deals create --stdin` without piping data.

## Code Examples

### comfy-table with Dynamic Arrangement, Truncation, and Alignment
```rust
// Source: https://docs.rs/comfy-table/latest/comfy_table/
use comfy_table::{Table, ContentArrangement, CellAlignment, ColumnConstraint, Width};

fn build_deals_table(rows: &[Vec<String>], headers: &[&str], color: bool) -> Table {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_truncation_indicator("...");
    table.set_header(headers);

    for row in rows {
        table.add_row(row);
    }

    // Right-align the "value" column (index 2 in default columns)
    if let Some(col) = table.column_mut(2) {
        col.set_cell_alignment(CellAlignment::Right);
    }

    // Set upper boundary on title column to prevent it dominating width
    if let Some(col) = table.column_mut(1) {
        col.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(40)));
    }

    table
}
```

### CSV Output with csv Crate
```rust
// Source: https://docs.rs/csv/latest/csv/
use csv::WriterBuilder;
use std::io;

fn render_csv(items: &[serde_json::Value], fields: &[String]) -> Result<()> {
    let mut wtr = WriterBuilder::new().from_writer(io::stdout());

    // Write header
    wtr.write_record(fields)?;

    // Write rows
    for item in items {
        let row: Vec<String> = fields.iter()
            .map(|f| extract_field(item, f))
            .collect();
        wtr.write_record(&row)?;
    }

    wtr.flush()?;
    Ok(())
}
```

### Relative Date Formatting
```rust
// Source: https://docs.rs/chrono-humanize/latest/chrono_humanize/
use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;

fn format_date_relative(iso_str: &str) -> String {
    match iso_str.parse::<DateTime<Utc>>() {
        Ok(dt) => {
            let ht = HumanTime::from(dt);
            format!("{}", ht)  // e.g., "2 hours ago", "3 days ago"
        }
        Err(_) => iso_str.to_string(),  // fallback to raw string
    }
}
```

### Field Extraction with Dot Notation
```rust
fn extract_field(value: &serde_json::Value, path: &str) -> String {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in &parts {
        match current.get(*part) {
            Some(v) => current = v,
            None => return String::new(),
        }
    }

    match current {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}
```

### Auto-Pagination Loop
```rust
async fn fetch_all_deals(client: &PipeliteClient, params: &mut DealsListParams) -> Result<(Vec<Deal>, PaginationMeta)> {
    let mut all_deals = Vec::new();
    let max_records = 1000u64;
    params.limit = 100;
    params.offset = 0;

    loop {
        let response = client.list_deals(params).await?;
        let total = response.meta.total;
        all_deals.extend(response.data);

        if all_deals.len() as u64 >= total || all_deals.len() as u64 >= max_records {
            if total > max_records {
                eprintln!(
                    "Showing {} of {}. Use --limit/--offset for more.",
                    max_records, total
                );
            }
            let meta = PaginationMeta {
                total,
                offset: 0,
                limit: all_deals.len() as u64,
            };
            return Ok((all_deals, meta));
        }

        params.offset += 100;
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| prettytable-rs | comfy-table 7.x | 2023+ | prettytable-rs is unmaintained; comfy-table is actively maintained with UTF-8 support |
| Manual color codes | colored 3.x with NO_COLOR standard | 2024+ | colored 3.x supports the NO_COLOR spec natively |
| csv 1.1 | csv 1.3 | 2024 | Minor improvements, same API |
| chrono without serde feature | chrono 0.4 with `serde` feature | stable | Always enable serde feature for API date parsing |

**Deprecated/outdated:**
- `prettytable-rs`: Unmaintained since 2020. Use comfy-table.
- `term_size` crate: Deprecated. comfy-table handles terminal width internally via crossterm.

## Open Questions

1. **API error response format for validation failures (422)**
   - What we know: The API likely returns structured error details for validation failures
   - What's unclear: Exact shape of error response body (field-level errors? single message?)
   - Recommendation: Implement a generic `ApiError` struct that deserializes the error body, with a fallback to status code + raw body if parsing fails. Refine once real API responses are observed.

2. **Batch endpoint partial success behavior**
   - What we know: `POST /deals/batch` exists
   - What's unclear: Does it return all-or-nothing, or partial results with per-item errors?
   - Recommendation: Assume it can return partial success. Model response as `Vec<Result<Deal, ApiError>>` or similar. Print successes and failures separately.

3. **Expand parameter response structure**
   - What we know: API supports `?expand=owner,organization` query param
   - What's unclear: Does expansion inline the full related object or just key fields?
   - Recommendation: Model expanded fields as `Option<serde_json::Value>` initially. Render expanded fields in key-value view using dot notation. Refine types once real API responses are observed.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + assert_cmd 2.0 for integration |
| Config file | Cargo.toml `[dev-dependencies]` |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DEAL-01 | `deals list` subcommand parses and dispatches | integration | `cargo test --test deals_cli_test` | No -- Wave 0 |
| DEAL-02 | `deals get <id>` subcommand parses | integration | `cargo test --test deals_cli_test` | No -- Wave 0 |
| DEAL-03 | `deals create` validates required flags | integration | `cargo test --test deals_cli_test` | No -- Wave 0 |
| DEAL-04 | `deals update <id>` parses optional flags | integration | `cargo test --test deals_cli_test` | No -- Wave 0 |
| DEAL-05 | `deals delete <id>` parses | integration | `cargo test --test deals_cli_test` | No -- Wave 0 |
| OUTP-01 | JSON output renders valid JSON | unit | `cargo test --lib output::json` | No -- Wave 0 |
| OUTP-02 | Table output renders with correct columns | unit | `cargo test --lib output::table` | No -- Wave 0 |
| OUTP-03 | CSV output has header + escaped values | unit | `cargo test --lib output::csv` | No -- Wave 0 |
| OUTP-04 | Plain output is tab-separated, no headers | unit | `cargo test --lib output::plain` | No -- Wave 0 |
| OUTP-05 | Field selection filters output columns | unit | `cargo test --lib output::fields` | No -- Wave 0 |
| OUTP-06 | TTY auto-detection | unit | Already tested via detect_format in output/mod.rs | Partial |
| OUTP-07 | Color respects NO_COLOR and --no-color | unit | `cargo test --lib output::table` | No -- Wave 0 |
| FILT-01 | Entity-specific filter flags parse correctly | unit | `cargo test --lib cli::deals` | No -- Wave 0 |
| FILT-02 | Limit/offset pass through to API params | unit | `cargo test --lib api` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/deals_cli_test.rs` -- integration tests for deals subcommand parsing and help text
- [ ] `src/output/table.rs` tests -- table rendering unit tests with mock data
- [ ] `src/output/json.rs` tests -- JSON output formatting tests
- [ ] `src/output/csv.rs` tests -- CSV escaping and header tests
- [ ] `src/output/plain.rs` tests -- tab-separated output tests
- [ ] `src/output/fields.rs` tests -- dot notation field extraction tests
- [ ] `src/api/mod.rs` tests -- CRUD method URL construction and query param tests (can mock or test URL building)

## Sources

### Primary (HIGH confidence)
- [comfy-table docs](https://docs.rs/comfy-table/latest/comfy_table/) - Table, Column, ColumnConstraint, CellAlignment APIs verified
- [comfy-table crates.io](https://crates.io/crates/comfy-table) - Version 7.2, active maintenance confirmed
- [csv crate docs](https://docs.rs/csv) - Writer API with serde support verified
- [chrono-humanize crates.io](https://crates.io/crates/chrono-humanize) - HumanTime API for relative dates

### Secondary (MEDIUM confidence)
- [comfy-table GitHub releases](https://github.com/nukesor/comfy-table/releases) - UTF-8 truncation improvements in recent releases
- [csv GitHub](https://github.com/BurntSushi/rust-csv) - Standard CSV library for Rust ecosystem

### Tertiary (LOW confidence)
- Batch endpoint behavior -- assumed from CONTEXT.md notes, needs validation against real API
- API error response format for 422 -- needs real API testing

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all core libraries already in Cargo.toml or are the canonical Rust choices
- Architecture: HIGH - patterns follow established codebase conventions (clap derive, module structure, AppContext passing)
- Pitfalls: HIGH - based on direct experience with serde, comfy-table, and CLI output patterns
- API integration: MEDIUM - API schema confirmed from OpenAPI spec but error responses and batch behavior unverified

**Research date:** 2026-03-24
**Valid until:** 2026-04-24 (stable domain, all libraries mature)
