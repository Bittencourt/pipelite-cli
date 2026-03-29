# Phase 5: Power Features - Research

**Researched:** 2026-03-25
**Domain:** Local caching, dashboard aggregation, ASCII splash, dynamic shell completions
**Confidence:** HIGH

## Summary

Phase 5 adds four independent features to the pipelite CLI: a local file cache for metadata (pipelines, stages, users, entity lists) with TTL-based invalidation, a pipeline dashboard command showing deal counts and values per stage, an ASCII art splash screen on TTY with no subcommand, and cache-powered dynamic shell completions. The codebase already has all the building blocks: `PipeliteClient` with list methods for all entities, `OutputFormat` with generic table/csv/json/plain rendering, `colored` 3.1 for terminal colors, `dialoguer::FuzzySelect` for interactive prompts, and `clap_complete` 4.6 for shell completions.

The cache layer is best implemented as simple JSON files in `~/.pipelite/cache/` with embedded timestamps -- no external crate needed. The project already uses `serde` + `serde_json` + `chrono`, which is everything required for a TTL cache. The dashboard aggregates pipeline/stage/deal data into summary tables using the existing output infrastructure. The splash screen is a hardcoded ASCII art string printed with the `colored` crate. Dynamic completions require enabling `clap_complete`'s `unstable-dynamic` feature and wiring `ArgValueCandidates` to cache reads.

**Primary recommendation:** Build a standalone `src/cache.rs` module with typed cache entries, JSON file persistence, and TTL checking. Wire it into `AppContext` so commands can access cache transparently. Keep the four features (cache, dashboard, splash, dynamic completions) as independent work streams that can be built and tested separately.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Cache lives at `~/.pipelite/cache/` -- alongside config.toml
- All entity types cached: pipelines, stages, users/owners, entity lists (deals, orgs, people, activities)
- Tiered TTLs: long for slow-changing (pipelines, stages, users -- hours), short for fast-changing (entity lists -- minutes)
- Mutations auto-invalidate the relevant entity cache immediately
- `pipelite cache clear` clears all cached data
- `pipelite cache refresh` pre-populates cache by fetching all cacheable data at once
- Dashboard shows ALL pipelines, no selection -- full overview in one command
- Each pipeline: summary header (total deals, total value) + per-stage vertical table
- Per-stage columns: Stage | Deals | Value
- Dashboard always fetches fresh data (no cache) -- accuracy matters
- Dashboard respects `--format` flag (table default, json/csv/plain)
- Splash triggered on TTY with no subcommand; non-TTY shows standard clap help
- Bold block letter ASCII art logo ("PIPELITE" in figlet-style chunky font)
- Single branded accent color for logo, default color for hints
- 3-4 quick-start command hints below logo
- If not configured: show logo + "Get started: pipelite init"
- Respects `--no-color` and `NO_COLOR`
- Dynamic completions suggest entity IDs with name hints: `abc123 -- Big Deal Corp`
- Completions read cache first, API fallback if empty/expired
- FuzzySelect prompts also use cache first, API fallback on miss

### Claude's Discretion
- Cache file format (JSON recommended -- already in deps)
- Exact TTL values per entity type
- Cache key structure and file naming
- ASCII art font choice and exact layout
- Branded color choice (cyan, green, etc.)
- How dynamic completions integrate with clap_complete
- Dashboard value formatting (currency symbols, abbreviations)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CACH-01 | CLI caches pipeline/stage/user metadata locally with TTL-based invalidation | Cache module with JSON files, chrono timestamps, configurable TTLs |
| CACH-02 | User can clear cache with `pipelite cache clear` | Cache subcommand with clear/refresh actions |
| CACH-03 | Cache used for interactive prompt dropdowns and shell completions | Cache reads in prompt.rs helpers and ArgValueCandidates completers |
| DASH-01 | User can view pipeline overview with `pipelite dashboard` | Dashboard command fetching pipelines + stages + deals, rendering with existing output module |
| DASH-02 | Dashboard shows deal counts and total values per pipeline stage | Aggregate deals by stage_id, sum values, render as table per pipeline |
| UX-01 | ASCII art splash screen when running `pipelite` with no subcommand on TTY | Hardcoded ASCII art string, colored crate, TTY detection via IsTerminal |
</phase_requirements>

## Standard Stack

### Core (already in Cargo.toml)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde + serde_json | 1.0 | Cache serialization/deserialization | Already used everywhere, zero new deps |
| chrono | 0.4 | TTL timestamp checking | Already in deps, `Utc::now()` for cache times |
| colored | 3.1 | ASCII art coloring | Already in deps, `.cyan().bold()` for logo |
| clap_complete | 4.6 | Dynamic shell completions | Already in deps, add `unstable-dynamic` feature |
| dialoguer | 0.12 | FuzzySelect with cache-backed options | Already in deps |
| comfy-table | 7.2 | Dashboard table rendering | Already used by output::table |

### Supporting (no new dependencies needed)
This phase requires zero new crate dependencies. All functionality is built from existing deps plus standard library (`std::fs`, `std::path`, `std::time`).

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| JSON files | SQLite (rusqlite) | Overkill for simple key-value cache, adds binary dep |
| JSON files | bincode | Faster but not human-debuggable, adds dep |
| Hand-rolled TTL | `cached` crate | In-memory only, doesn't persist to disk across invocations |
| Hardcoded ASCII | `figlet-rs` crate | Extra dep for one string -- not worth it |

## Architecture Patterns

### Recommended Project Structure
```
src/
  cache.rs              # Cache module: CacheStore, CacheEntry<T>, TTL logic, file I/O
  commands/
    cache/
      mod.rs            # CacheCommands enum (Clear, Refresh)
      clear.rs          # pipelite cache clear
      refresh.rs        # pipelite cache refresh
    dashboard.rs        # pipelite dashboard
  cli/
    cache.rs            # CacheCommands clap args
    dashboard.rs        # DashboardArgs clap args
```

### Pattern 1: Typed Cache Store
**What:** A `CacheStore` struct that manages reading/writing JSON cache files with embedded timestamps and TTL checking.
**When to use:** Every cache read/write operation.
**Example:**
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct CacheEntry<T> {
    data: T,
    cached_at: DateTime<Utc>,
    ttl_seconds: i64,
}

impl<T> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        let age = Utc::now() - self.cached_at;
        age.num_seconds() > self.ttl_seconds
    }
}

pub struct CacheStore {
    cache_dir: PathBuf,  // ~/.pipelite/cache/
}

impl CacheStore {
    pub fn new() -> anyhow::Result<Self> {
        let home = std::env::var("HOME")?;
        let cache_dir = PathBuf::from(home).join(".pipelite").join("cache");
        std::fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let path = self.cache_dir.join(format!("{}.json", key));
        let content = std::fs::read_to_string(&path).ok()?;
        let entry: CacheEntry<T> = serde_json::from_str(&content).ok()?;
        if entry.is_expired() {
            return None;
        }
        Some(entry.data)
    }

    pub fn set<T: Serialize>(&self, key: &str, data: &T, ttl_seconds: i64) -> anyhow::Result<()> {
        let entry = CacheEntry {
            data,
            cached_at: Utc::now(),
            ttl_seconds,
        };
        let path = self.cache_dir.join(format!("{}.json", key));
        let json = serde_json::to_string_pretty(&entry)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    pub fn invalidate(&self, key: &str) {
        let path = self.cache_dir.join(format!("{}.json", key));
        let _ = std::fs::remove_file(&path);
    }
}
```

### Pattern 2: Cache-Through for Prompts and Completions
**What:** Helper functions that try cache first, then fall back to API.
**When to use:** FuzzySelect prompts and dynamic completions.
**Example:**
```rust
/// Get pipelines from cache or API, caching the result.
pub async fn get_pipelines_cached(
    cache: &CacheStore,
    client: &PipeliteClient,
) -> anyhow::Result<Vec<(String, String)>> {
    // Try cache first
    if let Some(cached) = cache.get::<Vec<(String, String)>>("pipelines") {
        return Ok(cached);
    }
    // Fetch from API
    let response = client.list_pipelines(&Default::default()).await?;
    let items: Vec<(String, String)> = response.data.iter()
        .map(|p| (p.id.clone(), p.name.clone()))
        .collect();
    // Store in cache
    cache.set("pipelines", &items, 3600)?; // 1 hour TTL
    Ok(items)
}
```

### Pattern 3: No-Subcommand Splash Screen
**What:** Check before clap parsing whether args are empty + TTY, show splash instead of clap help.
**When to use:** `main.rs` entry point.
**Example:**
```rust
// In main(), before Cli::parse():
let args: Vec<String> = std::env::args().collect();
if args.len() == 1 && std::io::stdout().is_terminal() {
    print_splash(&load_config(None).ok());
    return;
}
// Otherwise proceed with normal clap parsing
```

### Pattern 4: Dashboard Fresh-Fetch Aggregation
**What:** Dashboard fetches all pipelines, their stages, and deals, then aggregates counts/values per stage.
**When to use:** `pipelite dashboard` command.
**Example:**
```rust
// Pseudocode for dashboard aggregation
for pipeline in pipelines {
    let stages = client.list_stages(&StagesListParams { pipeline: Some(pipeline.id) }).await?;
    let deals = client.list_deals(&DealsListParams { /* all for pipeline */ }).await?;

    // Group deals by stage_id
    let mut stage_summary: BTreeMap<String, (usize, f64)> = BTreeMap::new();
    for deal in &deals {
        let entry = stage_summary.entry(deal.stage_id.clone()).or_default();
        entry.0 += 1;  // count
        entry.1 += deal.value.unwrap_or(0.0);  // value
    }

    // Render pipeline header + stage table
}
```

### Anti-Patterns to Avoid
- **Cache in PipeliteClient:** Don't put caching logic inside the HTTP client. Keep it as a separate layer so tests can mock cache independently of API.
- **Global mutable cache state:** Don't use `lazy_static` or `OnceCell` for cache. Pass `CacheStore` through `AppContext` like other services.
- **Parsing args manually for splash:** Don't reimplement arg parsing. Use `Cli::try_parse()` and catch the "no subcommand" case, or check `std::env::args().len() == 1` before parsing.
- **Caching dashboard data:** The user explicitly decided dashboard fetches fresh. Don't cache dashboard results.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON serialization | Custom format parser | serde_json (already in deps) | Handles all edge cases, zero bugs |
| Terminal color detection | Manual ANSI checks | colored crate control + IsTerminal (already in deps) | Handles NO_COLOR, pipe detection |
| Table rendering | Manual column alignment | comfy-table via output::table (already in deps) | Handles unicode width, wrapping |
| Shell completion generation | Shell-specific scripts | clap_complete unstable-dynamic | Handles bash/zsh/fish differences |

**Key insight:** This phase has zero new dependencies because the existing stack covers everything. The work is wiring, not infrastructure.

## Common Pitfalls

### Pitfall 1: Cache Directory Race Conditions
**What goes wrong:** Two concurrent pipelite processes write the same cache file simultaneously, corrupting it.
**Why it happens:** CLI tools can be invoked in parallel (scripts, tab completion while command runs).
**How to avoid:** Write to a temp file then rename (atomic on most filesystems). On read failure, treat as cache miss (delete corrupted file).
**Warning signs:** Intermittent JSON parse errors from cache files.

### Pitfall 2: Splash Screen Blocking Clap Help
**What goes wrong:** If `Commands` is `#[command(subcommand)]` with `command` as required, running `pipelite` with no args gives clap's error, not the splash.
**Why it happens:** Clap parses before you can intercept.
**How to avoid:** Make the subcommand optional (`Option<Commands>`) or check `std::env::args().len() == 1` before `Cli::parse()`. The pre-parse check is simpler and doesn't change the type system.
**Warning signs:** `pipelite` shows "error: requires subcommand" instead of splash.

### Pitfall 3: Dynamic Completions Feature Flag
**What goes wrong:** `ArgValueCandidates` and `CompletionCandidate` don't exist at compile time.
**Why it happens:** They require `features = ["unstable-dynamic"]` on `clap_complete`.
**How to avoid:** Add the feature flag in Cargo.toml: `clap_complete = { version = "4.6", features = ["unstable-dynamic"] }`.
**Warning signs:** "unresolved import" errors for `clap_complete::engine`.

### Pitfall 4: Cache Invalidation After Mutations
**What goes wrong:** User creates a deal, then tab-completes -- the new deal doesn't appear.
**Why it happens:** Create/update/delete didn't invalidate the relevant cache.
**How to avoid:** After any mutation command succeeds, call `cache.invalidate("entity_type")`. Wire this into the command handlers, not the API client.
**Warning signs:** Stale data in completions after create/update/delete.

### Pitfall 5: Dashboard Auto-Pagination
**What goes wrong:** Dashboard only shows first 100 deals per pipeline, totals are wrong.
**Why it happens:** API has default pagination limits.
**How to avoid:** Use `--all` / auto-paginate logic (already exists in list commands) when fetching deals for dashboard. Fetch ALL deals to get accurate counts and totals.
**Warning signs:** Deal counts don't match web UI totals.

### Pitfall 6: Completions Blocking on Network
**What goes wrong:** Tab completion hangs for 5-30 seconds waiting for API response.
**Why it happens:** Cache miss triggers synchronous API call during shell completion.
**How to avoid:** For dynamic completions specifically, only return cached results. If cache is empty, return no completions (the user should run `pipelite cache refresh`). The FuzzySelect prompts can still fall back to API since the user expects a brief wait there.
**Warning signs:** Shell feels unresponsive when tabbing through arguments.

## Code Examples

### Cache Module File Layout
```rust
// src/cache.rs
use std::path::PathBuf;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Default TTLs in seconds
pub const TTL_PIPELINES: i64 = 3600;    // 1 hour
pub const TTL_STAGES: i64 = 3600;       // 1 hour
pub const TTL_USERS: i64 = 7200;        // 2 hours
pub const TTL_ENTITY_LIST: i64 = 300;   // 5 minutes

/// Cache keys for each entity type
pub const KEY_PIPELINES: &str = "pipelines";
pub const KEY_STAGES: &str = "stages";
pub const KEY_USERS: &str = "users";
pub const KEY_DEALS: &str = "deals";
pub const KEY_ORGS: &str = "orgs";
pub const KEY_PEOPLE: &str = "people";
pub const KEY_ACTIVITIES: &str = "activities";
```

### Splash Screen
```rust
// Hardcoded ASCII art -- no figlet crate needed
const LOGO: &str = r#"
 ____  _            _ _ _
|  _ \(_)_ __   ___| (_) |_ ___
| |_) | | '_ \ / _ \ | | __/ _ \
|  __/| | |_) |  __/ | | ||  __/
|_|   |_| .__/ \___|_|_|\__\___|
        |_|
"#;

fn print_splash(config: Option<&AppConfig>, color: bool) {
    if color {
        println!("{}", LOGO.cyan().bold());
    } else {
        println!("{}", LOGO);
    }

    let configured = config.map_or(false, |c| !c.server.api_key.is_empty());
    if configured {
        println!("  Try:  pipelite deals list");
        println!("        pipelite dashboard");
        println!("        pipelite --help");
    } else {
        println!("  Get started: pipelite init");
    }
}
```

### Dynamic Completions with ArgValueCandidates
```rust
// In cli/deals.rs args, for dynamic completion of deal IDs:
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

fn deal_id_candidates() -> Vec<CompletionCandidate> {
    let cache = match CacheStore::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let deals: Vec<(String, String)> = cache
        .get("deals")
        .unwrap_or_default();

    deals.into_iter()
        .map(|(id, name)| {
            CompletionCandidate::new(id)
                .help(Some(name.into()))
        })
        .collect()
}

// Usage in arg definition:
#[arg(add = ArgValueCandidates::new(deal_id_candidates))]
pub id: String,
```

### Dashboard Output Using Existing Render
```rust
// Dashboard builds serde_json::Value rows and uses existing output::render_list
let rows: Vec<serde_json::Value> = stage_summaries.iter().map(|s| {
    serde_json::json!({
        "stage": s.name,
        "deals": s.deal_count,
        "value": format!("${:.2}", s.total_value),
    })
}).collect();

let columns = vec!["stage".into(), "deals".into(), "value".into()];
output::render_list(&rows, &ctx.output_format, &columns, &None, ctx.color, None)?;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| clap_complete AOT only | clap_complete unstable-dynamic with CompleteEnv | clap_complete 4.4+ | Dynamic completions from runtime data |
| ansi_term crate | colored 3.x | 2024 | colored is actively maintained, ansi_term archived |
| Static completion scripts | ArgValueCandidates with custom closures | clap_complete 4.5+ | Cache-backed dynamic completions possible |

**Deprecated/outdated:**
- `clap_complete::generator` module: deprecated in favor of `clap_complete::aot`
- `ansi_term` crate: archived, use `colored` instead

## Open Questions

1. **API endpoint for "deal counts by stage"**
   - What we know: Dashboard needs deal counts per stage per pipeline. Current API lists deals with pagination.
   - What's unclear: Does the Pipelite API have an aggregate/summary endpoint, or must we fetch all deals and count client-side?
   - Recommendation: Implement client-side aggregation (fetch all deals per pipeline, group by stage_id). If API has summary endpoint, optimize later.

2. **clap_complete unstable-dynamic stability**
   - What we know: The feature flag is `unstable-dynamic` -- explicitly marked unstable.
   - What's unclear: How stable the API actually is in practice with clap_complete 4.6.
   - Recommendation: Use it -- it's the only path for dynamic completions with clap. The "unstable" label means API may change between versions, not that it's buggy. Pin version in Cargo.toml.

3. **Completions: CompleteEnv vs AOT + custom shell scripts**
   - What we know: The project currently generates AOT completions. Dynamic completions need `CompleteEnv`.
   - What's unclear: Whether to replace AOT entirely or offer both.
   - Recommendation: Keep AOT completions as-is (they work for subcommands/flags). Add `CompleteEnv` as a separate integration for dynamic value completions. Users opt in by sourcing the dynamic completion setup.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | assert_cmd 2.0 + predicates 3.0 (integration), built-in #[test] (unit) |
| Config file | tests/ directory with integration tests |
| Quick run command | `cargo test --test cache_test --test dashboard_test --test splash_test` |
| Full suite command | `cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CACH-01 | Cache stores and retrieves with TTL | unit | `cargo test cache::tests -q` | No -- Wave 0 |
| CACH-02 | `pipelite cache clear` removes files | integration | `cargo test --test cache_test -q` | No -- Wave 0 |
| CACH-03 | Completions read from cache | unit | `cargo test cache::tests::completions -q` | No -- Wave 0 |
| DASH-01 | `pipelite dashboard` produces output | integration | `cargo test --test dashboard_test -q` | No -- Wave 0 |
| DASH-02 | Dashboard shows deal counts and values | unit | `cargo test commands::dashboard::tests -q` | No -- Wave 0 |
| UX-01 | No-subcommand on TTY shows splash | integration | `cargo test --test splash_test -q` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib -q` (unit tests)
- **Per wave merge:** `cargo test` (full suite)
- **Phase gate:** Full suite green before verify

### Wave 0 Gaps
- [ ] `src/cache.rs` unit tests for get/set/clear/invalidate/TTL expiry
- [ ] `tests/cache_test.rs` integration test for `pipelite cache clear` and `pipelite cache refresh` (dry-run style with unreachable server)
- [ ] `tests/dashboard_test.rs` integration test for `pipelite dashboard` (dry-run or mock)
- [ ] `tests/splash_test.rs` integration test for no-subcommand behavior

## Sources

### Primary (HIGH confidence)
- Project source code: `Cargo.toml`, `src/config.rs`, `src/context.rs`, `src/prompt.rs`, `src/cli/mod.rs`, `src/main.rs`, `src/commands/completions.rs`
- [clap_complete docs - env module](https://docs.rs/clap_complete/latest/clap_complete/env/index.html) -- CompleteEnv API
- [clap_complete docs - ArgValueCandidates](https://docs.rs/clap_complete/latest/clap_complete/engine/struct.ArgValueCandidates.html) -- custom completers
- [CompletionCandidate docs](https://docs.rs/clap_complete/latest/clap_complete/struct.CompletionCandidate.html) -- .help() method for display hints
- [clap_complete crate page](https://docs.rs/clap_complete/latest/clap_complete/index.html) -- unstable-dynamic feature flag

### Secondary (MEDIUM confidence)
- [colored crate GitHub](https://github.com/colored-rs/colored) -- API: `.cyan().bold()`, `NO_COLOR` support
- [clap_complete crates.io](https://crates.io/crates/clap_complete) -- version compatibility

### Tertiary (LOW confidence)
- [Rust caching strategies blog](https://oneuptime.com/blog/post/2026-02-01-rust-caching-strategies/view) -- general patterns, validated against project needs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in Cargo.toml, zero new deps
- Architecture: HIGH -- follows established project patterns (command modules, AppContext, output rendering)
- Cache design: HIGH -- simple JSON + chrono + serde, well-understood pattern
- Dynamic completions: MEDIUM -- `unstable-dynamic` feature works but API may shift in future clap versions
- Pitfalls: HIGH -- based on direct code analysis and established Rust patterns

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable domain, no fast-moving deps)
