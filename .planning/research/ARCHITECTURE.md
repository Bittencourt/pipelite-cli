# Architecture Research

**Domain:** Rust CLI tool wrapping a REST API (CRM domain)
**Researched:** 2026-03-23
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      CLI Layer (clap derive)                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │  deals   │  │  orgs    │  │  people  │  │ activities│  ...  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
│       └──────────────┴──────────────┴──────────────┘            │
├─────────────────────────────────────────────────────────────────┤
│                      Context / Config Layer                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  AppContext { config, api_client, output_format, cache } │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                      Service Layer                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │ DealSvc  │  │  OrgSvc  │  │PersonSvc │  │ ActSvc   │  ...  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
│       └──────────────┴──────────────┴──────────────┘            │
├─────────────────────────────────────────────────────────────────┤
│                      API Client Layer                           │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  PipeliteClient { reqwest::Client, base_url, api_key }  │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                      Cross-Cutting Concerns                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │  Output  │  │  Cache   │  │  Errors  │  │ Prompts  │       │
│  │Formatter │  │  Layer   │  │          │  │          │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| CLI Layer | Parse arguments, route to commands, define subcommands | `clap` derive macros with `Commands` enum |
| Context Layer | Merge config file + env vars + CLI flags into single source of truth | Custom `AppContext` struct built at startup |
| Service Layer | Business logic per entity; orchestrates API calls, caching, validation | One module per CRM entity (deals, orgs, etc.) |
| API Client | HTTP communication with Pipelite server; auth, request building, response parsing | `reqwest::Client` wrapper with typed methods |
| Output Formatter | Render data as table/csv/json/plain based on user flag | Trait-based formatting dispatched by `--format` flag |
| Cache Layer | Read-only local cache for pipelines, stages, and other slow-changing data | File-based JSON in `~/.pipelite/cache/` with TTL |
| Error Layer | Unified error types with user-friendly messages | `thiserror` for typed errors, `anyhow` for propagation |
| Prompt Layer | Interactive input for create/update; disabled in headless mode | `dialoguer` crate with headless-mode bypass |

## Recommended Project Structure

```
src/
├── main.rs              # Entry point: parse CLI, build context, dispatch
├── cli/                 # CLI argument definitions
│   ├── mod.rs           # Top-level Cli struct and Commands enum
│   ├── deals.rs         # DealCommands subcommands
│   ├── orgs.rs          # OrgCommands subcommands
│   ├── people.rs        # PersonCommands subcommands
│   ├── activities.rs    # ActivityCommands subcommands
│   ├── pipelines.rs     # PipelineCommands subcommands
│   └── stages.rs        # StageCommands subcommands
├── commands/            # Command execution logic (handlers)
│   ├── mod.rs           # Shared command traits/utilities
│   ├── deals.rs         # Deal command handlers
│   ├── orgs.rs          # Org command handlers
│   ├── people.rs        # Person command handlers
│   ├── activities.rs    # Activity command handlers
│   ├── pipelines.rs     # Pipeline command handlers
│   ├── stages.rs        # Stage command handlers
│   ├── status.rs        # Dashboard / status command
│   └── health.rs        # Connection test / health check
├── api/                 # API client layer
│   ├── mod.rs           # PipeliteClient struct, shared request logic
│   ├── models.rs        # API request/response types (serde structs)
│   ├── deals.rs         # Deal API endpoints
│   ├── orgs.rs          # Org API endpoints
│   ├── people.rs        # Person API endpoints
│   ├── activities.rs    # Activity API endpoints
│   ├── pipelines.rs     # Pipeline API endpoints
│   └── stages.rs        # Stage API endpoints
├── config.rs            # Config loading: file + env + CLI merge
├── context.rs           # AppContext struct definition
├── output/              # Output formatting
│   ├── mod.rs           # OutputFormat enum, Displayable trait
│   ├── table.rs         # Table formatter (comfy-table)
│   ├── json.rs          # JSON formatter (serde_json)
│   ├── csv.rs           # CSV formatter (csv crate)
│   └── plain.rs         # Plain text formatter
├── cache.rs             # File-based read-only cache with TTL
├── prompts.rs           # Interactive prompts (dialoguer), headless bypass
└── error.rs             # Error types (thiserror) and helpers
```

### Structure Rationale

- **cli/ vs commands/:** Separates argument definition (what the CLI accepts) from execution logic (what it does). CLI structs are pure data; command handlers contain logic. This is the pattern Kevin K (clap author) recommends -- CLI parsing should not leak into business logic.
- **api/:** Isolates all HTTP concerns. Command handlers never touch `reqwest` directly; they call typed methods on `PipeliteClient`. This makes testing straightforward (mock the client, not HTTP).
- **output/:** Centralized formatting means every command returns data in a uniform struct, and the output layer decides how to render it. Adding a new format means adding one file, not touching every command.
- **One module per entity in each layer:** Keeps files focused. A deal-related change touches `cli/deals.rs`, `commands/deals.rs`, and `api/deals.rs` -- easy to find, easy to review.

## Architectural Patterns

### Pattern 1: Context Struct as Single Source of Truth

**What:** A single `AppContext` struct holds the merged configuration from all sources (config file, environment variables, CLI flags) plus shared resources (API client, cache handle). Built once at startup, passed by reference to all command handlers.

**When to use:** Always. This is the foundational pattern for Rust CLIs with configuration.

**Trade-offs:** Slight boilerplate at startup, but eliminates config-passing spaghetti and makes every command handler's dependencies explicit.

**Example:**
```rust
pub struct AppContext {
    pub config: AppConfig,
    pub client: PipeliteClient,
    pub cache: Cache,
    pub output_format: OutputFormat,
    pub headless: bool,
}

impl AppContext {
    pub fn build(cli: &Cli) -> Result<Self> {
        let config = AppConfig::load()?;           // ~/.pipelite/config.toml
        let config = config.merge_env()?;           // PIPELITE_* env vars
        let config = config.merge_cli(cli)?;        // CLI flag overrides
        let client = PipeliteClient::new(&config)?;
        let cache = Cache::open(&config)?;
        Ok(Self { config, client, cache, output_format: cli.format, headless: cli.headless })
    }
}
```

### Pattern 2: Typed API Client with Per-Entity Methods

**What:** A single `PipeliteClient` wraps `reqwest::Client` and exposes typed async methods per entity (`list_deals()`, `create_deal()`, etc.). Handles auth headers, base URL, error mapping internally.

**When to use:** Any CLI wrapping a REST API. Never let command handlers build raw HTTP requests.

**Trade-offs:** More upfront code in the API layer, but command handlers become trivial and testable.

**Example:**
```rust
pub struct PipeliteClient {
    http: reqwest::Client,
    base_url: String,
}

impl PipeliteClient {
    pub async fn list_deals(&self, filters: &DealFilters) -> Result<Vec<Deal>> {
        let resp = self.http
            .get(format!("{}/api/deals", self.base_url))
            .query(filters)
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }
}
```

### Pattern 3: Output Trait for Format-Agnostic Commands

**What:** Command handlers return typed data structs. A `Render` trait (or similar) converts any data struct to the requested output format. The dispatch happens once in main, not in every command.

**When to use:** When supporting multiple output formats (table, JSON, CSV, plain).

**Trade-offs:** Requires all displayable data to implement the trait, but the uniformity pays off quickly. Adding a new format requires zero changes to command handlers.

**Example:**
```rust
pub trait Render {
    fn render_table(&self) -> String;
    fn render_json(&self) -> Result<String>;
    fn render_csv(&self) -> Result<String>;
    fn render_plain(&self) -> String;
}

pub fn output<T: Render>(data: &T, format: OutputFormat) -> Result<()> {
    let text = match format {
        OutputFormat::Table => data.render_table(),
        OutputFormat::Json => data.render_json()?,
        OutputFormat::Csv => data.render_csv()?,
        OutputFormat::Plain => data.render_plain(),
    };
    println!("{text}");
    Ok(())
}
```

### Pattern 4: Headless Mode Guard

**What:** A boolean `headless` flag on AppContext. Interactive prompts check this flag and either prompt the user or fail with "missing required field" errors. This lets the same commands work for both humans and scripts/agents.

**When to use:** Any CLI that serves both interactive and automated workflows.

**Trade-offs:** Every prompt site needs the guard, but it is a simple if/else. The alternative (separate command paths) is far worse.

**Example:**
```rust
pub fn prompt_or_require<T: FromStr>(
    ctx: &AppContext,
    field_name: &str,
    cli_value: Option<T>,
    prompt_msg: &str,
) -> Result<T> {
    match cli_value {
        Some(v) => Ok(v),
        None if ctx.headless => Err(anyhow!("--{field_name} is required in headless mode")),
        None => Ok(dialoguer::Input::new().with_prompt(prompt_msg).interact()?),
    }
}
```

## Data Flow

### Command Execution Flow

```
User types: pipelite deals list --format json --stage "negotiation"
    |
    v
[main.rs] Parse CLI args (clap)
    |
    v
[context.rs] Build AppContext (load config, init client, open cache)
    |
    v
[commands/deals.rs] handle_list(ctx, filters)
    |
    +--> [cache.rs] Check cache for stages (resolve "negotiation" -> stage_id)
    |
    +--> [api/deals.rs] ctx.client.list_deals(filters)
    |        |
    |        v
    |    [HTTP] GET /api/deals?stage_id=123
    |        |
    |        v
    |    [api/models.rs] Deserialize JSON -> Vec<Deal>
    |
    v
[output/] Render Vec<Deal> as JSON
    |
    v
stdout (pipeable)
```

### Create Flow (Interactive)

```
User types: pipelite deals create
    |
    v
[main.rs] Parse CLI -> no fields provided
    |
    v
[commands/deals.rs] handle_create(ctx, partial_fields)
    |
    +--> [prompts.rs] headless? NO -> prompt for title, value, org, stage
    |                  headless? YES -> error "missing required fields"
    |
    +--> [api/deals.rs] ctx.client.create_deal(new_deal)
    |
    v
[output/] Render created Deal
    |
    v
stdout
```

### Configuration Priority (highest wins)

```
CLI flags  >  Environment vars (PIPELITE_*)  >  Config file (~/.pipelite/config.toml)
```

### Key Data Flows

1. **Read operations:** CLI -> Context -> Cache check -> API call (on miss) -> Output formatter -> stdout
2. **Write operations:** CLI -> Context -> Prompt layer (if interactive) -> API call -> Output formatter -> stdout
3. **Config resolution:** File load -> Env overlay -> CLI flag overlay -> Frozen AppContext

## Build Order (Dependency Graph)

Components should be built in this order based on dependencies:

```
Phase 1: Foundation (no inter-dependencies)
  ├── error.rs          (needed by everything)
  ├── config.rs         (needed by context)
  └── api/models.rs     (needed by API client and output)

Phase 2: Core Infrastructure
  ├── api/mod.rs        (depends on: error, models)
  ├── output/           (depends on: error, models)
  └── context.rs        (depends on: config, error)

Phase 3: CLI Skeleton
  ├── cli/mod.rs        (depends on: nothing runtime, just clap)
  └── main.rs           (depends on: cli, context)

Phase 4: First Entity End-to-End
  ├── api/deals.rs      (depends on: api/mod, models)
  ├── cli/deals.rs      (depends on: clap)
  └── commands/deals.rs (depends on: context, api, output)

Phase 5: Remaining Entities (parallel)
  ├── orgs, people, activities, pipelines, stages
  └── (same pattern as deals, low risk)

Phase 6: Enhancement Layers
  ├── cache.rs          (depends on: config, models)
  ├── prompts.rs        (depends on: context)
  ├── status dashboard  (depends on: api, output)
  └── shell completions (depends on: cli)
```

**Why this order:** Error types and models are leaf dependencies -- everything uses them, they use nothing. The API client and output formatter are the two "platform" layers that enable all commands. Building one entity end-to-end (deals) proves the architecture before replicating across all six entities. Cache and prompts are enhancements layered on top of working commands.

## Anti-Patterns

### Anti-Pattern 1: Leaking clap Types into Business Logic

**What people do:** Pass `clap::ArgMatches` or derived CLI structs directly to command handlers and API methods.
**Why it's wrong:** Couples every handler to the CLI argument structure. Makes testing require constructing fake CLI args. Makes headless mode harder (you test business logic, not argument parsing).
**Do this instead:** CLI layer extracts values and passes them as plain types or domain-specific structs to handlers via the context pattern.

### Anti-Pattern 2: HTTP Logic in Command Handlers

**What people do:** Build URLs, set headers, call `reqwest::get()` directly inside command handler functions.
**Why it's wrong:** Duplicates auth logic, base URL construction, error mapping across every command. Impossible to mock for tests. Changing the API version touches every file.
**Do this instead:** All HTTP goes through `PipeliteClient`. Command handlers call typed methods like `client.list_deals()`.

### Anti-Pattern 3: Format-Specific Code in Commands

**What people do:** Write `if format == "json" { ... } else if format == "table" { ... }` inside each command handler.
**Why it's wrong:** Every new command re-implements formatting. Every new format requires touching every command. Bugs in formatting are scattered.
**Do this instead:** Commands return data structs. A single output dispatch function handles all formatting.

### Anti-Pattern 4: Blocking Async Runtime

**What people do:** Use `reqwest::blocking` or `block_on()` scattered throughout the codebase instead of committing to async.
**Why it's wrong:** Mixing sync and async creates deadlock risk and confusing code. You either pay the async tax once (in main) or fight it everywhere.
**Do this instead:** Use `#[tokio::main]` on main, keep everything async internally. The CLI tool does not need high concurrency, but the consistency is worth it.

### Anti-Pattern 5: Monolithic main.rs

**What people do:** Put CLI definition, config loading, API calls, output formatting, and error handling all in main.rs.
**Why it's wrong:** Untestable, unmaintainable, grows to thousands of lines.
**Do this instead:** `main.rs` should be ~30 lines: parse CLI, build context, match command, call handler, handle top-level errors.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Pipelite CRM API | REST/JSON via `reqwest` with API key in `Authorization` header | Only external dependency. All endpoints go through `PipeliteClient`. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| CLI -> Commands | Function calls with extracted args | CLI structs never cross into commands |
| Commands -> API | Method calls on `PipeliteClient` | Commands never build HTTP requests |
| Commands -> Output | Return data structs implementing `Render` | Commands never write to stdout directly |
| Commands -> Cache | Method calls on `Cache` struct from context | Cache is read-only, transparent miss-through to API |
| Commands -> Prompts | Call `prompt_or_require()` helper | Gated by `ctx.headless` flag |

## Sources

- [Kevin K (clap author) - CLI Structure in Rust series](https://kbknapp.dev/cli-structure-01/) - Context struct pattern, CLI architecture (HIGH confidence)
- [Rust CLI Book](https://rust-cli.github.io/book/index.html) - Official community patterns (HIGH confidence)
- [reqwest documentation](https://docs.rs/reqwest/latest/reqwest/) - Client reuse, async patterns (HIGH confidence)
- [comfy-table](https://github.com/Nukesor/comfy-table) - Table output library (HIGH confidence)
- [dialoguer](https://docs.rs/dialoguer/latest/dialoguer/) - Interactive prompt library (HIGH confidence)
- [Rust CLI patterns - dasroot.net](https://dasroot.net/posts/2026/02/rust-cli-patterns-clap-cargo-configuration/) - Clap + config patterns (MEDIUM confidence)
- [Error handling comparison: anyhow vs thiserror](https://dev.to/leapcell/rust-error-handling-compared-anyhow-vs-thiserror-vs-snafu-2003) - Error crate selection (HIGH confidence)

---
*Architecture research for: Rust CLI wrapping REST API (Pipelite CRM)*
*Researched: 2026-03-23*
