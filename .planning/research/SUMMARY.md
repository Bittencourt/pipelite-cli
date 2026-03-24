# Project Research Summary

**Project:** Pipelite CLI
**Domain:** Rust CLI tool — REST API client for a CRM (deals, orgs, people, activities, pipelines, stages)
**Researched:** 2026-03-23
**Confidence:** HIGH

## Executive Summary

Pipelite CLI is a command-line REST API client for a CRM system, modeled after the `gh`/`kubectl`/`stripe` school of CLI design: resource-oriented subcommands (`pipelite deals list`), multi-format output (JSON, table, CSV), and full Unix composability via pipes. The research is unambiguous on approach: use clap's derive macros for argument parsing, reqwest + tokio for async HTTP, serde for serialization, and a layered architecture that separates CLI definitions from command handlers from the API client. All four research areas point to well-established Rust CLI patterns with strong ecosystem support, and the Rust CLI ecosystem has fully stabilized — async-std was discontinued in March 2025, making tokio the uncontested async runtime choice.

The recommended approach is to build in four incremental phases. The foundation phase (Phase 1) is the most critical and must get authentication, config file handling, TTY detection, the HTTP client, and the async strategy right before anything else is added — these decisions are expensive to change later. Phases 2–4 layer CRUD, developer experience, and power-user features on top of a proven foundation. The critical path is: Auth → Config → HTTP Client → Entity CRUD → Output Formats, with everything else layered incrementally. After the full stack is proved with a single entity (deals), the remaining five entities follow a mechanical, low-risk pattern.

The primary risks are all Phase 1 concerns: API keys stored in world-readable config files (default permissions are 0644 — must be 0600), blocking calls inside the async runtime (dialoguer prompts must be kept out of async functions), and emitting ANSI codes or prompts when stdout is piped (requires `IsTerminal` check baked into the output abstraction from day one). These are architectural, not bugs, and must be addressed by design in the first phase. If the output abstraction, TTY detection, and async strategy are built correctly upfront, the remaining phases carry minimal architectural risk.

## Key Findings

### Recommended Stack

The Rust CLI ecosystem is mature and the correct choices are clear. Clap 4.6 with derive macros is the unambiguous standard for argument parsing — it handles subcommand routing, shell completion generation, help text, and validation from a single set of annotated structs. Reqwest 0.13 with rustls-tls (no system OpenSSL dependency) is the standard async HTTP client. Tokio 1.50 with `current_thread` flavor is the only viable async runtime, and the current-thread flavor is the right choice for a sequential CLI tool (faster startup, simpler to reason about than multi-thread).

For output, comfy-table handles table rendering, serde\_json and the csv crate cover the other formats, and a `Render` trait centralizes dispatch. Dialoguer handles interactive prompts with clean headless-mode detection. The error handling stack is anyhow (propagation with context) plus thiserror (typed error enums) — simpler and more appropriate than eyre/color-eyre, which target developer-facing tools rather than end-user CLIs.

**Core technologies:**
- `clap` 4.6 (derive, env features): argument parsing, subcommands, shell completions — ecosystem standard
- `tokio` 1.50 (current-thread): async runtime — only viable option post-async-std discontinuation; current-thread for faster CLI startup
- `reqwest` 0.13 (json, rustls-tls): HTTP client — ergonomic, async, no OpenSSL dependency, single static binary
- `serde` + `serde_json`: serialization framework — non-negotiable for structured API data
- `toml` 1.1: config file parsing — matches TOML requirement, simpler than figment for a single config file
- `comfy-table` 7.2: table output — stable, Unicode-aware, programmatic column building (marked "finished" by author)
- `dialoguer` 0.12: interactive prompts — stable, headless-safe, simpler than inquire (which has fork churn)
- `anyhow` 1.0 + `thiserror` 2.0: error handling — anyhow for propagation, thiserror for typed enums
- `directories` 6.0: XDG-compliant config/cache paths — avoids hardcoded `~/.pipelite`
- `indicatif` 0.18: progress spinners for API calls
- `colored` 3.1: terminal colors with `NO_COLOR` support
- `chrono` 0.4 (serde): date/time handling for CRM timestamps

See `.planning/research/STACK.md` for full alternatives analysis and Cargo.toml feature flags.

### Expected Features

The feature research identifies a clear four-phase MVP ladder based on user expectations from comparable CLIs (gh, kubectl, stripe, aws). The critical path is auth → config → HTTP → CRUD → output formats. Shell completions, interactive prompts, and local caching are enhancements that layer cleanly on top.

**Must have (table stakes):**
- API key authentication (`pipelite init`; env var `PIPELITE_API_KEY`; never as a CLI flag)
- Full CRUD on all 6 entities: deals, orgs, people, activities, pipelines, stages
- JSON output (`--format json`; auto-default when stdout is not a TTY)
- Table output (default for interactive TTY; auto-truncate long fields)
- `--help` on every command with usage examples
- Non-zero exit codes on failure (0=success, 1=runtime error, 2=misuse)
- Actionable error messages with suggested next commands
- Config file (`~/.config/pipelite/config.toml` via XDG; `pipelite config set/get/show`)
- Connection test (`pipelite ping`)
- List filtering (`--stage`, `--owner`, `--limit`, `--offset`)
- CSV output (`--format csv`)
- Quiet mode (`-q`)

**Should have (differentiators):**
- Interactive prompts for create/update (dialoguer wizard; TTY-gated; skip in headless)
- Headless mode (`--no-input`; safe for CI/CD and AI agents; explicit over implicit)
- Stdin piping for bulk operations (`cat deals.json | pipelite deals create --stdin`)
- Shell completions for bash, zsh, fish (clap\_complete; dynamic completions via cache)
- Local caching for pipelines/stages/users (TTL-based; `pipelite cache clear`)
- Pipeline dashboard (`pipelite dashboard`; ASCII table output, not a TUI)
- Field selection (`--fields=id,title,value`)
- `--dry-run` flag for mutations
- Config profiles (`--profile=staging`)
- Activity logging shortcut (`pipelite log`)

**Defer (v2+):**
- Plain and JSONL output formats (low complexity but low urgency)
- ASCII splash screen (cosmetic; defer until core works)
- TUI (explicitly out of scope per project brief — kills pipeability, massive complexity)
- Webhooks, admin/user management, offline sync, plugin system, custom DSL, auto-update

See `.planning/research/FEATURES.md` for full feature dependency graph and anti-features rationale.

### Architecture Approach

The architecture follows the standard layered pattern for Rust CLIs wrapping REST APIs, as documented by clap's author and the Rust CLI Book. The key principle is strict separation of concerns across four layers: CLI (clap structs; argument definitions only), Commands (handlers; business logic), API Client (reqwest wrapper; all HTTP concerns), and Cross-Cutting (output, cache, prompts, errors). A single `AppContext` struct is built at startup by merging config file, env vars, and CLI flags — then passed by reference to all handlers. This eliminates config-passing spaghetti and makes dependencies explicit.

**Major components:**
1. `cli/` — clap derive structs; pure argument definitions; one module per entity; never contains logic
2. `commands/` — handler module per entity; orchestrates API + cache + prompts + output
3. `api/` — `PipeliteClient` wrapper with typed async methods per entity; all HTTP in one place
4. `output/` — `Render` trait + format dispatch; commands return typed data, never write to stdout directly
5. `context.rs` — `AppContext` struct; merged config + shared client + cache handle + headless flag
6. `cache.rs` — file-based read-only TTL cache in `~/.cache/pipelite/`
7. `prompts.rs` — dialoguer wrappers; every prompt gated by `ctx.headless`
8. `error.rs` — thiserror typed errors + anyhow propagation helpers

The entity-first build order matters: build error types and models first (leaf dependencies), then the API client and output layer (platform), then prove the full stack end-to-end with one entity (deals), then replicate across the remaining five entities. Cache and prompts are enhancement layers that come last.

See `.planning/research/ARCHITECTURE.md` for full component diagram, data flow diagrams, code patterns, and anti-patterns.

### Critical Pitfalls

Six critical pitfalls are identified, all with Phase 1 prevention requirements. Three are architectural decisions that become expensive rewrites if not addressed upfront.

1. **API key in world-readable config files** — Set file permissions to 0600 explicitly using `std::os::unix::fs::PermissionsExt` on creation; support `PIPELITE_API_KEY` env var as primary auth method; never accept API key as a CLI flag (leaks to shell history and `ps` output)
2. **Failing to distinguish TTY vs piped output** — Use `std::io::IsTerminal` (stable since Rust 1.70) on stdout AND stdin independently; auto-switch to JSON and suppress ANSI codes when not a TTY; route all decorations to stderr; must be in the output abstraction from day one (retrofitting is near-full-rewrite cost)
3. **Blocking calls inside the async runtime** — Use `tokio::runtime::Builder::new_current_thread()`; keep dialoguer prompts in synchronous code or behind `spawn_blocking`; async strategy must be decided in Phase 1 (changing later touches every function signature)
4. **No timeout or retry on HTTP requests** — reqwest has no default timeout; set `connect_timeout(5s)` + `timeout(30s)` on client construction; add exponential backoff retry for 429/5xx; centralized client makes this trivial to add correctly
5. **Brittle serde deserialization on unexpected API responses** — Use `Option<T>` and `#[serde(default)]` on all API response structs; never use `deny_unknown_fields`; design defensive pattern in the first entity (deals) and replicate
6. **Hardcoded `~/.pipelite` path ignoring XDG** — Use `directories` crate (`ProjectDirs`) for platform-appropriate paths; support `PIPELITE_CONFIG` env var override; wrong early means migration logic for existing users

See `.planning/research/PITFALLS.md` for full prevention strategies, warning signs, recovery costs, and a "Looks Done But Isn't" checklist.

## Implications for Roadmap

Based on combined research, a four-phase structure is strongly indicated by the feature dependency graph (FEATURES.md), the architecture build order (ARCHITECTURE.md), and the pitfall-to-phase mapping (PITFALLS.md). All research files independently converge on the same ordering.

### Phase 1: Foundation

**Rationale:** Auth, config, HTTP client, and the output/TTY system are leaf dependencies that everything else builds on. Three of the six critical pitfalls live here and are expensive to retrofit. Getting these right makes every subsequent phase straightforward; getting them wrong creates rewrites.

**Delivers:** Working `pipelite init`, `pipelite ping`, and `pipelite config show/set/get`. No entity CRUD yet, but auth, config, HTTP client with timeouts/retries, TTY-aware output, and the AppContext are correct and tested.

**Addresses (from FEATURES.md):** API key authentication, configuration file with XDG paths, env var support, connection health check, quiet mode scaffolding, `--version`

**Avoids (from PITFALLS.md):**
- API key in world-readable file (0600 permissions at config creation)
- Async runtime misuse (current-thread tokio; prompts kept in sync code)
- TTY vs pipe output broken (IsTerminal check in output abstraction from day one)
- HTTP hangs (connect\_timeout + timeout configured on reqwest Client at construction)
- Hardcoded config path (directories crate from day one)

**Architecture components built:** `error.rs`, `config.rs`, `api/mod.rs` (PipeliteClient skeleton with timeout/retry), `api/models.rs` (skeleton), `output/` (Render trait + all formats + TTY dispatch), `context.rs` (AppContext builder), `main.rs` (~30 lines), `commands/health.rs`

**Research flag:** Standard patterns — all decisions are documented with high-confidence sources. No additional research needed.

### Phase 2: Entity CRUD and Output

**Rationale:** This is the product becoming useful. The feature dependency graph is explicit — CRUD is the trunk from which filtering, field selection, error handling, and formatted output all branch. All 6 entities follow the same pattern once proved end-to-end with deals.

**Delivers:** Full list/get/create/update/delete on all 6 entities. JSON, table, and CSV output with TTY auto-detection. List filtering and field selection. Non-zero exit codes. Actionable error messages. Colored output with NO\_COLOR support.

**Addresses (from FEATURES.md):** Full CRUD on all entities, JSON/table/CSV output formats, list filtering (`--stage`, `--owner`, `--limit`), field selection (`--fields`), non-zero exit codes, actionable errors, colored output

**Avoids (from PITFALLS.md):**
- Brittle serde deserialization (design `#[serde(default)]` + `Option<T>` pattern in deals; replicate)
- Pagination incomplete (implement `--all`; check pagination metadata on every list endpoint)
- JSON envelope assumption (expect `{ "data": [...], "meta": {...} }` wrapper)
- Server URL trailing slash (normalize in config load)
- Opaque errors (HTTP status + URL called + human hint on every failure)

**Architecture components built:** `cli/` (all 6 entity subcommand modules), `commands/` (all 6 entity handler modules), `api/` (all 6 entity endpoint modules)

**Research flag:** Needs API schema validation. The actual Pipelite CRM API response shapes, auth header convention, pagination format, and error envelope structure were not available during research. Must be confirmed against the real API before implementing entity models. This is the largest unknown.

### Phase 3: Developer Experience

**Rationale:** Once CRUD works, the tool is useful but not ergonomic. Interactive prompts, headless mode, shell completions, and dry-run make the tool pleasant for humans and safe for automation. These are the primary differentiators identified in FEATURES.md.

**Delivers:** Interactive create/update wizard (dialoguer; TTY-gated; skip in headless). Explicit `--no-input` headless mode safe for CI/CD and AI agents. Shell completions for bash/zsh/fish (clap\_complete). `--dry-run` flag on all mutations.

**Addresses (from FEATURES.md):** Interactive prompts, headless mode (`--no-input`), shell completions, `--dry-run`, `--help` with usage examples

**Avoids (from PITFALLS.md):**
- Prompts hanging in piped mode (`stdin.is_terminal()` guard in `prompts.rs`)
- Missing `--help` examples (add `after_help` examples to every subcommand via clap)
- Slow startup for non-network commands (`--version`, `--help`: lazy AppContext initialization)

**Architecture components built:** `prompts.rs` (dialoguer wrappers + headless guard pattern), shell completion generation wired into `main.rs` via clap\_complete

**Research flag:** Standard patterns — dialoguer + clap\_complete are well-documented. Headless mode guard pattern is fully specified in ARCHITECTURE.md with code example. No additional research needed.

### Phase 4: Power User Features

**Rationale:** Local caching, stdin piping, the pipeline dashboard, and config profiles turn the tool from good to excellent for power users and automation workflows. These all depend on working CRUD (Phase 2) and are largely independent of each other — they can be built in parallel or reordered based on user demand.

**Delivers:** Local TTL cache for pipelines/stages/users (fast interactive prompts; dynamic shell completions). Stdin piping for bulk operations (`cat deals.json | pipelite deals create --stdin`). Pipeline dashboard (`pipelite dashboard`). Config profiles (`--profile=staging`). Activity logging shortcut (`pipelite log`). Plain and JSONL output formats.

**Addresses (from FEATURES.md):** Local caching, stdin piping, pipeline dashboard, config profiles, activity logging shortcut, JSONL and plain output

**Avoids (from PITFALLS.md):**
- Unbounded cache growth (TTL-based expiration + max cache size; `pipelite cache clear`)
- Cache never invalidated after mutations (invalidate relevant cache keys on create/update/delete)
- Client-side filtering performance trap (cache only reduces round-trips for slow-changing metadata; use server-side query params for actual filtering)

**Architecture components built:** `cache.rs` (file-based TTL cache with explicit size/eviction), stdin JSON/JSONL parsing in command handlers, `commands/status.rs` (dashboard), profile resolution in `context.rs`

**Research flag:** Cache invalidation strategy needs design thought during planning. TTL values for different entity types (pipeline definitions vs deal lists vs user lists have very different change rates) need product judgment. The general pattern is clear; the specific values are not.

### Phase Ordering Rationale

- **Dependency-driven ordering:** The architecture build-order graph and the FEATURES.md critical path both require the same sequence: foundation before CRUD, CRUD before enhancements. This is not a preference — it is a hard dependency.
- **Pitfall front-loading:** All critical pitfalls that are expensive to retrofit (TTY detection, async strategy, config paths, HTTP client defaults) are Phase 1 concerns. Addressing them first means Phases 2–4 carry minimal architectural risk.
- **Entity CRUD parallelism:** After proving the full stack with deals (Phase 2 start), the remaining 5 entities follow the same mechanical pattern and can be built in parallel. This is the largest time-saving opportunity in the entire roadmap.
- **Enhancement independence:** Phase 4 features (cache, stdin, dashboard, profiles) are independent of each other. They can be reordered or dropped without affecting the rest of the roadmap if scope needs trimming.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (Entity CRUD):** Actual Pipelite CRM API response schemas were not available during research. The exact field names, pagination format, auth header convention, and error envelope structure must be confirmed against the real API before implementing entity models. This is the largest gap in the research.
- **Phase 4 (Caching):** Cache invalidation semantics and appropriate TTL values per entity type were not researched in depth. General pattern is clear; specific values need product judgment.

Phases with standard patterns (can skip `/gsd:research-phase`):
- **Phase 1 (Foundation):** All decisions are documented with high-confidence sources. Stack is locked. Patterns are established. Architecture is specified with code examples.
- **Phase 3 (Developer Experience):** dialoguer + clap\_complete are well-documented. Headless guard pattern is fully specified with code examples in ARCHITECTURE.md.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crates verified on crates.io 2026-03-23. Alternatives analysis thorough. async-std discontinuation confirmed via corrode.dev. |
| Features | HIGH | Grounded in reference CLIs (gh, kubectl, stripe, aws) with direct documentation links. Feature prioritization follows clig.dev principles. |
| Architecture | HIGH | Patterns sourced from clap author (Kevin K), Rust CLI Book, and reqwest docs. Layered architecture is standard and well-proven for this problem type. |
| Pitfalls | HIGH | Security and async pitfalls verified against Rust stdlib docs and Tokio official guides. TTY detection uses stable Rust 1.70 API. |

**Overall confidence:** HIGH

### Gaps to Address

- **Pipelite CRM API schema (must resolve in Phase 2 planning):** API response shapes, auth header convention, pagination format, and error envelope structure are unknown. Must be resolved by reading API docs, running the server locally, or examining an OpenAPI spec before implementing entity models.
- **API rate limits:** Whether the Pipelite CRM API has rate limits and the format of `Retry-After` headers were not researched. The retry/backoff pattern assumes standard HTTP 429 behavior.
- **Cache TTL values (Phase 4):** Appropriate TTL values per entity type need product judgment rather than research.
- **Windows support scope:** If Windows is in scope, the CI test matrix should include it from Phase 1. If out of scope, document it explicitly so users are not surprised.
- **Binary distribution strategy:** Cross-compilation and release distribution (cargo-dist, GitHub releases, package managers) were not researched. This is an operational concern but should be addressed before v1.0.

## Sources

### Primary (HIGH confidence)
- [crates.io — clap 4.6.0](https://crates.io/crates/clap) — argument parsing, derive macros, shell completions
- [crates.io — reqwest 0.13.2](https://crates.io/crates/reqwest) — HTTP client, rustls-tls, async patterns
- [crates.io — tokio 1.50.0](https://crates.io/crates/tokio) — async runtime, current-thread flavor
- [crates.io — directories 6.0.0](https://crates.io/crates/directories) — XDG-compliant config paths
- [crates.io — dialoguer 0.12.0](https://crates.io/crates/dialoguer) — interactive prompts, headless detection
- [Command Line Interface Guidelines (clig.dev)](https://clig.dev/) — comprehensive CLI design principles, output format, error messages, interactivity
- [Kevin K — CLI Structure in Rust series](https://kbknapp.dev/cli-structure-01/) — context struct pattern, cli/commands layer separation
- [Rust CLI Book](https://rust-cli.github.io/book/index.html) — official community patterns
- [Rust std::io::IsTerminal](https://doc.rust-lang.org/beta/std/io/trait.IsTerminal.html) — TTY detection (stable since Rust 1.70)
- [Tokio bridging with sync code](https://tokio.rs/tokio/topics/bridging) — official guide on mixing sync/async
- [GitHub CLI Manual](https://cli.github.com/manual/) — reference implementation for JSON output, interactive flows, profiles
- [kubectl Command Reference](https://kubernetes.io/docs/reference/kubectl/) — CRUD-on-resources CLI patterns

### Secondary (MEDIUM confidence)
- [Rust CLI Patterns 2026 — dasroot.net](https://dasroot.net/posts/2026/02/rust-cli-patterns-clap-cargo-configuration/) — clap + config patterns
- [Comparison of Rust CLI Prompts — fadeevab.com](https://fadeevab.com/comparison-of-rust-cli-prompts/) — dialoguer vs inquire vs cliclack analysis
- [async-std discontinuation — corrode.dev](https://corrode.dev/blog/async/) — March 2025 confirmation
- [HubSpot CLI](https://developers.hubspot.com/docs/developer-tooling/local-development/hubspot-cli/install-the-cli) — CRM-adjacent CLI auth/config patterns
- [Rust error handling guide — sheshbabu.com](https://www.sheshbabu.com/posts/rust-error-handling/) — thiserror vs anyhow usage split
- [Tokio runtime pitfalls — techbuddies.io](https://www.techbuddies.io/2026/03/21/top-5-tokio-runtime-mistakes-that-quietly-kill-your-async-rust/) — blocking the async runtime

### Tertiary (needs validation)
- Pipelite CRM API schema — not researched; must be validated against the actual API during Phase 2 planning
- Cache TTL values — inferred from common patterns; need product judgment

---
*Research completed: 2026-03-23*
*Ready for roadmap: yes*
