# Pitfalls Research

**Domain:** Rust CLI wrapping a REST API (CRM domain)
**Researched:** 2026-03-23
**Confidence:** HIGH (most findings verified across multiple sources and Rust ecosystem docs)

## Critical Pitfalls

### Pitfall 1: Leaking API Keys in Config Files with World-Readable Permissions

**What goes wrong:**
The CLI stores the API key in `~/.pipelite/config.toml` with default file permissions (often 0644 on Linux). Any local user or process can read the API key. Worse, users copy config files into dotfile repos and accidentally commit secrets to Git.

**Why it happens:**
`std::fs::write` and `std::fs::File::create` use the process umask for permissions, which is typically 0022 -- resulting in world-readable files. Developers test on single-user machines and never notice.

**How to avoid:**
- Set file permissions to 0600 on config files containing secrets using `std::os::unix::fs::PermissionsExt`
- On creation, use `OpenOptions` with explicit mode bits
- Support `PIPELITE_API_KEY` environment variable as the primary auth method (env vars are process-scoped, not file-scoped)
- Consider the `keyring` crate for OS keychain integration as an optional backend
- Warn in docs: never commit config files containing keys

**Warning signs:**
- Config file creation code that does not explicitly set permissions
- No environment variable fallback for the API key
- No `.gitignore` entry for `~/.pipelite/`

**Phase to address:**
Phase 1 (Authentication/Config) -- must be correct from the first implementation. Retrofitting secure credential handling is painful because early users will already have insecure config files.

---

### Pitfall 2: Blocking the Tokio Runtime with Synchronous Operations

**What goes wrong:**
Using `#[tokio::main]` with a multi-threaded runtime for a CLI that makes sequential HTTP calls adds startup overhead and complexity for no benefit. Alternatively, accidentally calling blocking I/O (file reads, stdin prompts) inside async contexts starves the runtime -- other futures stall, timeouts stop firing, and the CLI hangs unpredictably.

**Why it happens:**
Developers default to `#[tokio::main]` because that is what tutorials show. Then they mix blocking `dialoguer` prompts or `std::fs` calls inside async functions without `spawn_blocking`. For a CLI that makes one API call at a time, the full async machinery is unnecessary overhead.

**How to avoid:**
- Use `tokio::runtime::Builder::new_current_thread()` -- a CLI tool rarely needs multi-threaded async. Current-thread is faster to start and simpler to reason about.
- If using interactive prompts (`dialoguer`, `inquire`), keep them in synchronous code before entering the async runtime, or use `tokio::task::spawn_blocking`
- Consider whether you even need async: `reqwest::blocking` is simpler for sequential CLI flows. Only go async if you need concurrent requests (e.g., batch operations).

**Warning signs:**
- `#[tokio::main(flavor = "multi_thread")]` in a CLI binary
- `dialoguer` or `std::io::stdin` calls inside `async fn`
- CLI startup feels slow (>100ms) with no network calls

**Phase to address:**
Phase 1 (Project scaffolding) -- the async strategy must be decided upfront. Switching from async to sync reqwest (or vice versa) later touches every function signature in the call chain.

---

### Pitfall 3: Failing to Distinguish Interactive vs Piped Output

**What goes wrong:**
The CLI emits ANSI color codes, progress spinners, or interactive prompts when stdout is piped to another program. This breaks `pipelite deals list | jq .` because jq receives ANSI escape sequences instead of clean JSON. Conversely, headless mode suppresses useful information that humans need.

**Why it happens:**
Developers test exclusively in interactive terminals. They add colors and spinners for polish without checking whether stdout is a TTY. The `--headless` flag exists but interactive mode is the implicit default even when output is piped.

**How to avoid:**
- Use `std::io::IsTerminal` (stable since Rust 1.70) to detect TTY on stdout AND stderr independently
- Default behavior: if stdout is not a TTY, automatically switch to machine-readable output (JSON) and suppress decorations
- Colors/spinners go to stderr only (stderr is for humans, stdout is for data)
- Interactive prompts: check stdin.is_terminal() -- if false, require all input via flags or error with a clear message
- Provide `--color=auto|always|never` flag for explicit override

**Warning signs:**
- `println!` with ANSI codes and no TTY check
- Prompts that hang when the CLI is used in a pipeline
- Users report "broken JSON output" when piping

**Phase to address:**
Phase 1 (Output system) -- this must be baked into the output abstraction from day one. Retrofitting TTY-awareness into scattered `println!` calls is a full rewrite of output handling.

---

### Pitfall 4: No Retry Logic or Timeout on HTTP Requests

**What goes wrong:**
A single network blip or slow server response causes the CLI to hang indefinitely or crash with an opaque error. Users in flaky network environments (VPNs, mobile hotspots) experience the CLI as unreliable. Without retry logic, transient 500/502/503 errors that would resolve in seconds become user-visible failures.

**Why it happens:**
reqwest's default timeout is *no timeout* -- requests wait forever. Developers test on localhost or fast networks where timeouts never trigger. Retry logic is seen as "nice to have" and deferred.

**How to avoid:**
- Set explicit timeouts on the reqwest `Client`: `connect_timeout(5s)`, `timeout(30s)`
- Implement retry with exponential backoff for 429 (rate limit) and 5xx errors using `reqwest-middleware` + `reqwest-retry`, or the `backoff` crate
- Parse `Retry-After` headers on 429 responses
- Never retry 4xx errors (except 429) -- those are client bugs, not transient failures
- Show a spinner/message on stderr during retries so the user knows the CLI is working

**Warning signs:**
- `reqwest::Client::new()` with no `.timeout()` configuration
- No error handling for HTTP status codes (only checking `response.json()` deserialization)
- User reports of the CLI "hanging" or "freezing"

**Phase to address:**
Phase 1 (HTTP client setup) -- the reqwest Client should be constructed once with proper defaults. Adding retry middleware later is easy if the client is centralized, nightmare if HTTP calls are scattered.

---

### Pitfall 5: Serde Deserialization Panic on Unexpected API Responses

**What goes wrong:**
The CLI defines strict Rust structs for API responses, then calls `.json::<Deal>()` which panics or returns an opaque serde error when the API returns an unexpected field, a null where a value was expected, or a different response shape (e.g., error envelope vs. data envelope). Since the CLI targets "any Pipelite CRM server," different server versions may return slightly different schemas.

**Why it happens:**
Developers model the API response based on one server version and assume it is stable. Serde's default behavior is to fail on unknown fields (if using `deny_unknown_fields`) or silently ignore them, and required fields cause hard failures when missing.

**How to avoid:**
- Use `#[serde(default)]` on structs and `Option<T>` for fields that might be absent
- Add `#[serde(rename_all = "camelCase")]` or `#[serde(rename_all = "snake_case")]` to match API conventions consistently
- Never use `deny_unknown_fields` -- the server will add new fields over time
- Deserialize into `serde_json::Value` first for debugging, then attempt typed deserialization with a clear error message showing the raw response on failure
- Wrap deserialization in a custom error that includes the HTTP status code, URL, and truncated response body

**Warning signs:**
- All struct fields are non-Optional with no defaults
- Error messages say "missing field `x`" with no context about which API call failed
- No integration tests against real or mocked API responses

**Phase to address:**
Phase 2 (Entity CRUD) -- every entity struct needs defensive deserialization. Design the pattern once in the first entity, then replicate.

---

### Pitfall 6: Hardcoded ~/.pipelite Path Ignoring XDG and Platform Conventions

**What goes wrong:**
The CLI hardcodes `~/.pipelite/config.toml` using `$HOME` expansion, which fails on Windows (`%USERPROFILE%`), breaks when `HOME` is not set (some CI environments), and ignores the XDG Base Directory Specification that Linux users expect (`$XDG_CONFIG_HOME/pipelite/`).

**Why it happens:**
`~/.tool/config.toml` looks clean in docs and the project spec says to use it. Developers on macOS/Linux never encounter the problem. CI/CD users hit it first.

**How to avoid:**
- Use the `dirs` crate: `dirs::config_dir()` returns the platform-appropriate config directory (`~/.config/` on Linux, `~/Library/Application Support/` on macOS, `%APPDATA%` on Windows)
- Fall back to `~/.pipelite/` only if `dirs::config_dir()` returns None
- Support `PIPELITE_CONFIG` environment variable for explicit override (essential for CI/CD and testing)
- Cache directory: use `dirs::cache_dir()` for cached data, not the config directory
- Document the actual paths for each platform

**Warning signs:**
- String literal `"~/.pipelite"` or `format!("{}/.pipelite", env::var("HOME"))`
- No Windows CI in the test matrix
- Users open issues about config not being found in Docker containers

**Phase to address:**
Phase 1 (Configuration) -- path resolution is foundational. If you get it wrong early, migration is needed for existing users.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `unwrap()` on HTTP responses | Faster prototyping | Panic in production on any API error | Never in release code; use in tests only |
| Single monolithic `main.rs` | Quick to start | Untestable logic, painful refactoring | First 100 lines only; extract modules immediately |
| Cloning strings everywhere | Avoids lifetime complexity | Memory waste, slow for large response lists | MVP, but refactor before batch operations |
| `anyhow::Result` in library code | Easy error propagation | Consumers cannot match on error variants programmatically | Application binary only; library modules should use typed errors |
| Inline HTTP URLs | No abstraction needed | URL changes require find-and-replace across codebase | Never; centralize base URL + endpoint construction |

## Integration Gotchas

Common mistakes when connecting to the Pipelite CRM API.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| REST API authentication | Sending API key as query parameter (visible in logs/history) | Use `Authorization: Bearer <key>` header or custom `X-API-Key` header per API spec |
| Pagination | Fetching only the first page and treating it as complete data | Always check for pagination metadata; implement `--all` flag or auto-pagination with a sane limit |
| JSON response envelopes | Assuming data is the top-level object (`response.json::<Vec<Deal>>()`) | Expect an envelope like `{ "data": [...], "meta": {...} }` and deserialize the wrapper first |
| Server URL trailing slashes | `base_url + "/api/deals"` produces `https://example.com//api/deals` when base has trailing slash | Normalize the base URL on config load: trim trailing slashes |
| Date/time fields | Deserializing timestamps as strings | Use `chrono::DateTime<Utc>` with serde, but make it `Option<DateTime<Utc>>` since dates may be null |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Fetching all entities to filter client-side | Slow `list` commands, high memory usage | Use server-side filtering via query parameters; pass `--filter` flags to API params | >500 entities in a collection |
| Serializing entire response to string for logging then deserializing again | Double parse time, double memory | Log the raw bytes or use `serde_json::Value` for debug; deserialize once | Responses >1MB (large deal lists) |
| Creating a new reqwest::Client per request | TLS handshake and connection setup on every call | Create one `Client` at startup and reuse (it has an internal connection pool) | Any use; each request adds ~100ms overhead |
| Unbounded local cache with no eviction | Disk usage grows forever in `~/.pipelite/cache/` | TTL-based expiration (e.g., 5 minutes for lists, 1 hour for pipelines/stages); max cache size | Weeks of regular use |
| Table formatting with `format!` for wide data | Columns misalign with Unicode, terminal width overflow | Use a table library like `tabled` or `comfy-table` that handles Unicode width and terminal wrapping | Non-ASCII data (names with accents, CJK characters) |

## Security Mistakes

Domain-specific security issues for a CRM CLI tool.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Logging full HTTP request/response bodies at info level | API key in headers and sensitive CRM data (emails, phone numbers) appear in log files | Log headers and bodies only at debug/trace level; redact Authorization headers in all log levels |
| Storing API key in shell history via `--api-key` flag | Anyone with shell access can read `~/.bash_history` | Accept API key via env var `PIPELITE_API_KEY` or config file only; if a flag is needed, use `--api-key-stdin` that reads from stdin |
| Cache files containing full API responses in plaintext | Sensitive CRM data persisted on disk without protection | Set cache directory permissions to 0700; document that cache contains CRM data; provide `pipelite cache clear` command |
| No TLS certificate verification (disabling for dev) | MITM attacks intercept API key and CRM data | Never ship with TLS verification disabled; if needed for dev, require explicit `--insecure` flag with a warning printed to stderr |

## UX Pitfalls

Common user experience mistakes in CLI tools wrapping APIs.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Opaque "request failed" errors | User cannot diagnose whether it is a network issue, auth issue, or server bug | Show HTTP status code, URL called, and a human-readable hint (e.g., "401: check your API key") |
| No output on success for mutation commands | User unsure if `deal create` actually worked | Print the created/updated entity or at least its ID; follow Unix convention of silence only for reads piped elsewhere |
| Requiring exact entity IDs for every command | User must look up IDs constantly; breaks flow | Support name-based lookups with fuzzy matching for interactive mode; require IDs only in headless mode |
| `--help` text that just lists flags without examples | Users cannot figure out common workflows | Include 2-3 usage examples in each subcommand's help text using clap's `after_help` or `about` |
| Slow startup for simple commands like `--version` | Feels sluggish | Avoid initializing HTTP client or loading config for commands that do not need them; lazy initialization |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Authentication:** Supports API key in config -- verify it also works via env var, and errors clearly when no key is configured
- [ ] **CRUD operations:** Create/update works -- verify it handles server validation errors (422) with field-level messages, not just "request failed"
- [ ] **JSON output:** `--format json` works -- verify it outputs valid JSON when the list is empty (should be `[]`, not nothing), and when fields contain special characters
- [ ] **Table output:** Tables render correctly -- verify with Unicode names (accented characters, CJK), very long values, and empty fields
- [ ] **Pagination:** List commands work -- verify they fetch ALL pages, not just page 1, and handle the case where the collection is empty
- [ ] **Config loading:** Config loads from file -- verify it merges env vars > CLI flags > config file in correct precedence order
- [ ] **Shell completions:** Generated and installable -- verify they update when new subcommands are added (regeneration must be documented)
- [ ] **Cache:** Reads from cache work -- verify cache invalidation after a mutation (creating a deal should invalidate the deals list cache)
- [ ] **Error paths:** Happy path works -- verify timeout, DNS failure, invalid JSON response, expired API key, and server-down scenarios all produce useful errors

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| API keys leaked in config with bad permissions | LOW | Rotate key on server; fix file permissions; add permission check on startup that warns users |
| Blocking calls in async runtime | MEDIUM | Refactor to `spawn_blocking` or switch to sync reqwest; requires touching all call sites if async is pervasive |
| No TTY detection in output | HIGH | Requires adding an output abstraction layer; every `println!` must route through it; near-rewrite of output code |
| Missing retry/timeout logic | LOW | Add `reqwest-middleware` with retry policy to the centralized client; minimal code change if client is shared |
| Strict serde structs breaking on new API fields | MEDIUM | Add `Option<T>` and `#[serde(default)]` to all structs; requires touching every model but is mechanical |
| Hardcoded config paths | MEDIUM | Replace with `dirs` crate; need migration logic for users who already have files in the old location |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| API key in world-readable file | Phase 1: Auth & Config | `stat -c %a` on created config file shows 600; env var override works |
| Async runtime misuse | Phase 1: Project Scaffolding | CLI startup <50ms with no network; no hangs during interactive prompts |
| Interactive vs piped output | Phase 1: Output System | `pipelite deals list \| jq .` produces valid JSON; colors only appear in terminal |
| No timeouts or retries | Phase 1: HTTP Client | Kill the server mid-request; CLI errors cleanly within 30s, not hanging |
| Brittle serde deserialization | Phase 2: Entity CRUD | Add an unexpected field to mock response; CLI still works. Remove an optional field; CLI still works |
| Hardcoded config path | Phase 1: Configuration | `PIPELITE_CONFIG=/tmp/test.toml pipelite config show` uses the override |
| Pagination incomplete | Phase 2: List Commands | Mock API with 3 pages; `pipelite deals list --all` returns all entities |
| Cache never invalidated | Phase 3: Caching | Create a deal, then list deals; new deal appears without `--no-cache` |
| No useful error messages | Phase 2: Error Handling | Disconnect network; CLI shows "connection failed" not a Rust panic backtrace |
| Shell history API key leak | Phase 1: Auth | `--api-key` flag does not exist; only env var and config file are supported |

## Sources

- [Rust std::io::IsTerminal](https://doc.rust-lang.org/beta/std/io/trait.IsTerminal.html) -- TTY detection in standard library
- [Tokio runtime pitfalls](https://www.techbuddies.io/2026/03/21/top-5-tokio-runtime-mistakes-that-quietly-kill-your-async-rust/) -- Blocking the async runtime
- [Tokio bridging with sync code](https://tokio.rs/tokio/topics/bridging) -- Official guide on mixing sync/async
- [keyring-rs](https://github.com/hwchen/keyring-rs) -- Cross-platform credential storage for Rust
- [reqwest-retry](https://docs.rs/reqwest-retry) -- Retry middleware for reqwest
- [backoff crate](https://github.com/ihrwein/backoff) -- Exponential backoff for Rust
- [dirs crate](https://docs.rs/dirs) -- Platform-appropriate directory paths
- [Rust CLI config file discussion](https://github.com/rust-cli/team/issues/7) -- Community discussion on config management
- [XDG and Rust](https://zork.net/~st/jottings/Rust_and_the_XDG_Base_Directory_Specification.html) -- XDG compliance challenges
- [Caches in Rust](https://matklad.github.io/2022/06/11/caches-in-rust.html) -- Cache design patterns
- [reqwest Error handling](https://docs.rs/reqwest/latest/reqwest/struct.Error.html) -- Error types and checking methods
- [Rust error handling guide](https://www.sheshbabu.com/posts/rust-error-handling/) -- thiserror vs anyhow patterns

---
*Pitfalls research for: Rust CLI wrapping REST API (Pipelite CRM)*
*Researched: 2026-03-23*
