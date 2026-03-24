# Technology Stack

**Project:** Pipelite CLI
**Researched:** 2026-03-23

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| clap | 4.6.0 | CLI argument parsing, subcommands | Industry standard for Rust CLIs. Derive macros eliminate boilerplate. Built-in help generation, validation, and subcommand routing. Used by ripgrep, cargo, and virtually every serious Rust CLI. | HIGH |
| clap_complete | 4.6.0 | Shell completion generation | First-party clap companion. Generates bash/zsh/fish completions from the same Command definition. Zero additional API design needed. | HIGH |
| tokio | 1.50.0 | Async runtime | The only viable async runtime in 2025+. async-std was officially discontinued March 2025. reqwest requires tokio. Use `rt-multi-thread` and `macros` features; `full` is overkill for a CLI. | HIGH |
| reqwest | 0.13.2 | HTTP client for CRM API | Ergonomic, async, built on hyper+tokio. Features: JSON support, custom headers, timeouts, connection pooling. The standard choice for REST API clients in Rust. Enable `json` and `rustls-tls` features. | HIGH |

### Serialization & Data

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| serde | 1.0.228 | Serialization framework | Non-negotiable for any Rust project handling structured data. Derive macros for automatic ser/de. Every other data crate builds on serde. | HIGH |
| serde_json | 1.0.149 | JSON parsing/generation | First-party serde JSON support. Handles API request/response bodies and `--format json` output. | HIGH |
| toml | 1.1.0 | TOML config file parsing | First-party TOML serde support. Reads/writes `~/.pipelite/config.toml`. Supports TOML spec 1.1. Human-readable config format that matches project requirement. | HIGH |
| csv | 1.4.0 | CSV output generation | Standard Rust CSV crate by BurntSushi. Handles `--format csv` output with proper escaping and quoting. Battle-tested. | HIGH |
| chrono | 0.4.44 | Date/time handling | CRM data includes timestamps, activity dates, due dates. chrono provides parsing, formatting, and timezone support. Use `serde` feature for automatic deserialization. | HIGH |

### User Interface

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| dialoguer | 0.12.0 | Interactive prompts | Mature, focused library for CLI prompts: confirmations, text input, selections, multi-select. Simpler API than inquire. Better for create/update workflows where you need structured input. Works well with headless mode (can detect non-TTY). | HIGH |
| comfy-table | 7.2.2 | Table output formatting | Automatic column width adjustment, ANSI color support, customizable borders. Considered "finished" by its author -- stable, well-tested, no unsafe code. Better than tabled for dynamic content where you build tables programmatically (not from struct derives). | HIGH |
| colored | 3.1.1 | Terminal colors/styles | Simple `.red()`, `.bold()` extension trait API. Supports `NO_COLOR` env var out of the box. Minimal learning curve. Slightly heavier than owo-colors but more ergonomic for application code (vs library code where zero-cost matters more). | MEDIUM |
| indicatif | 0.18.4 | Progress bars/spinners | Shows progress during API calls and bulk operations. Provides spinners for network requests and progress bars for batch operations. Integrates well with tokio async. | HIGH |

### Error Handling

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| anyhow | 1.0.102 | Application error handling | For application-level error propagation with context. `anyhow::Result` + `.context("doing X")` pattern. Simpler than eyre for a CLI that doesn't need customizable error reporters. color-eyre is overkill for a non-developer-facing tool. | HIGH |
| thiserror | 2.0.18 | Typed error definitions | For defining structured error enums (API errors, config errors, cache errors). Generates `Display` and `Error` impls via derive macro. Complements anyhow -- use thiserror for error types, anyhow for propagation. | HIGH |

### System & Configuration

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| directories | 6.0.0 | Platform-standard paths | Provides XDG-compliant paths on Linux, proper paths on macOS/Windows. Use `ProjectDirs` for `~/.pipelite/` config and cache locations. Does not create directories (you handle that). | HIGH |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Arg parsing | clap (derive) | argh, bpaf | clap is the ecosystem standard. argh is Google-opinionated and less flexible. bpaf has a smaller community. |
| HTTP client | reqwest | ureq, hyper | ureq is sync-only (limits concurrent API calls). hyper is too low-level for a CLI tool. |
| Async runtime | tokio | smol, async-std | async-std discontinued March 2025. smol is viable but reqwest requires tokio. No choice here. |
| Prompts | dialoguer | inquire, cliclack | inquire has more features (date picker, autocomplete) but dialoguer is simpler and more stable for basic CRUD prompts. inquire 0.9.x has had maintenance churn (multiple forks exist). cliclack is newer with less ecosystem adoption. |
| Table output | comfy-table | tabled, prettytable | tabled is derive-focused (less flexible for dynamic columns). prettytable is older and less maintained. comfy-table hits the sweet spot. |
| Colors | colored | owo-colors, yansi | owo-colors is better for libraries (zero-cost, no_std). For application code, colored's ergonomics win. yansi is fine but less popular. |
| Error handling | anyhow + thiserror | eyre + color-eyre, miette | eyre/color-eyre add fancy formatting aimed at developer tools. miette is for diagnostic-rich errors (compilers, linters). A CRM CLI needs clear messages, not fancy diagnostics. anyhow is simpler and sufficient. |
| Config format | TOML (toml crate) | figment, config-rs | Project requirement specifies TOML. figment adds multi-source config merging which is unnecessary complexity for a single config file. Direct toml crate is simpler. |
| Date/time | chrono | time | chrono has broader ecosystem support and better serde integration. time crate is lighter but less ergonomic for formatting. |

## Feature Flags to Enable

```toml
[dependencies]
clap = { version = "4.6", features = ["derive", "env"] }
clap_complete = "4.6"
tokio = { version = "1.50", features = ["rt-multi-thread", "macros", "fs"] }
reqwest = { version = "0.13", features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "1.1"
csv = "1.4"
chrono = { version = "0.4", features = ["serde"] }
dialoguer = "0.12"
comfy-table = "7.2"
colored = "3.1"
indicatif = "0.18"
anyhow = "1.0"
thiserror = "2.0"
directories = "6.0"
```

## Key Design Decisions

### Why `rustls-tls` over `native-tls` for reqwest

Use `rustls-tls` (pure Rust TLS) instead of `native-tls` (OpenSSL bindings). Reasons:
1. No system OpenSSL dependency -- simplifies cross-compilation and distribution
2. Single static binary with no shared library requirements
3. Slightly smaller attack surface (Rust memory safety)
4. Avoids OpenSSL version conflicts on various Linux distros

### Why `derive` feature for clap

The derive macro approach (`#[derive(Parser)]`) is strongly preferred over the builder API for CLIs with many subcommands. Reasons:
1. Subcommand hierarchy maps naturally to enum variants
2. Argument validation via type system
3. Less boilerplate, more maintainable as commands grow
4. Shell completions auto-generated from the same types

### Why dialoguer over inquire for prompts

Both are capable, but dialoguer is the safer choice for this project:
1. Stable release (0.12) vs inquire's maintenance uncertainty (multiple community forks exist as of 2025)
2. Simpler API for the prompts we need: text, confirm, select, multi-select
3. We don't need inquire's extras (date picker, editor integration, autocomplete)
4. dialoguer can detect non-interactive terminals, important for headless mode

### Why NOT a TUI framework

The project explicitly excludes TUI frameworks (ratatui, crossterm for full-screen). This is a CLI tool like `gh` or `kubectl` -- it prints output and exits. The "status dashboard" requirement should be a formatted table print, not a live-updating TUI.

## Crate Dependency Graph (simplified)

```
pipelite-cli
  +-- clap (derive) -----> clap_complete (shell completions)
  +-- tokio (async runtime)
  |     +-- reqwest (HTTP) --> serde_json (API payloads)
  +-- serde (derive) -----> toml (config), serde_json (API), csv (output)
  +-- dialoguer (prompts)
  +-- comfy-table (table output)
  +-- colored (terminal colors)
  +-- indicatif (progress/spinners)
  +-- anyhow + thiserror (errors)
  +-- directories (XDG paths)
  +-- chrono (dates/times)
```

## What NOT to Install

| Crate | Why Not |
|-------|---------|
| async-std | Discontinued March 2025. Dead project. |
| hyper (directly) | Too low-level. reqwest wraps it properly. |
| ratatui / crossterm | This is a CLI, not a TUI. Out of scope per PROJECT.md. |
| figment | Over-engineered for a single TOML config file. |
| color-eyre / miette | Fancy error formatting aimed at dev tools, not end-user CLIs. |
| prettytable-rs | Unmaintained. comfy-table supersedes it. |
| structopt | Predecessor to clap derive. Merged into clap v3+. Do not use. |
| serde_yaml | No YAML in this project. TOML for config, JSON for API. |

## Sources

- [clap on crates.io](https://crates.io/crates/clap) - v4.6.0 verified 2026-03-23
- [reqwest on crates.io](https://crates.io/crates/reqwest) - v0.13.2 verified 2026-03-23
- [tokio on crates.io](https://crates.io/crates/tokio) - v1.50.0 verified 2026-03-23
- [dialoguer on crates.io](https://crates.io/crates/dialoguer) - v0.12.0 verified 2026-03-23
- [comfy-table on GitHub](https://github.com/Nukesor/comfy-table) - "finished" status noted
- [colored on crates.io](https://crates.io/crates/colored) - v3.1.1 verified 2026-03-23
- [directories on crates.io](https://crates.io/crates/directories) - v6.0.0 verified 2026-03-23
- [async-std discontinuation](https://corrode.dev/blog/async/) - March 2025
- [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html) - color management guidance
- [Rust CLI Patterns 2026](https://dasroot.net/posts/2026/02/rust-cli-patterns-clap-cargo-configuration/) - clap + config patterns
- [Comparison of Rust CLI Prompts](https://fadeevab.com/comparison-of-rust-cli-prompts/) - dialoguer vs inquire vs cliclack
