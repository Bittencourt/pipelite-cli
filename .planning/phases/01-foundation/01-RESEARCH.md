# Phase 1: Foundation - Research

**Researched:** 2026-03-24
**Domain:** Rust CLI foundation -- authentication, configuration, HTTP client, error handling, TTY-aware output
**Confidence:** HIGH

## Summary

Phase 1 builds the foundation that every subsequent phase depends on: CLI argument parsing with clap derive, API key authentication (config file + env var), TOML configuration management, an HTTP client with timeouts, a structured error handling framework, and TTY-aware output. This is a greenfield Rust project (edition 2024, rustc 1.94.0) with only a hello-world `main.rs` and an empty `Cargo.toml` dependencies section.

The research is unambiguous: use clap 4.6 derive macros for CLI structure, reqwest 0.13 with rustls-tls for HTTP, tokio current_thread for async, serde + toml for config, anyhow + thiserror for errors, and dialoguer for the init wizard. All decisions are locked by the user in CONTEXT.md -- no alternatives to evaluate. The project research (STACK.md, ARCHITECTURE.md, PITFALLS.md) already covers this domain at HIGH confidence.

**Primary recommendation:** Build leaf dependencies first (error types, config loading), then the HTTP client and output abstraction, then wire everything through AppContext, and finally implement the four commands (init, ping, config show, config set). The init wizard (dialoguer) must run in synchronous code before entering the async runtime.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Guided wizard: step-by-step prompts for server URL, then API key, then test connection, then confirm and save
- API key stored plain in config.toml (file created with 0600 permissions)
- `PIPELITE_API_KEY` env var always wins over config file value -- standard for CI/scripts
- Single server in v1 -- no profiles. Profiles deferred to v2
- `pipelite init --url <url> --key <key>` also supported for headless/scripted init
- Noun-verb subcommands: `pipelite config show`, `pipelite config set <key> <value>` -- like git/gh
- Entity names are plural: `pipelite deals list`, `pipelite orgs get` -- like kubectl resources
- Short aliases for entities: `pipelite d ls` = `pipelite deals list`, `pipelite p ls` = `pipelite people list`
- Action aliases: `ls` = `list`, `rm` = `delete`
- Entity IDs passed as positional args: `pipelite deals get 123`
- Delete requires confirmation in interactive TTY unless `--force` flag is present; in headless mode (`--no-input`), `--force` is required or error
- `pipelite --version` shows rich info: `pipelite 0.1.0 (rustc 1.84, linux-x86_64)` with build metadata
- Clap default help + after_help examples section on every command
- Global flags: `--format json|table|csv|plain`, `--no-color`, `-q`/`--quiet`, `-v`/`--verbose`
- Structured error format: type + message + hint
- Red for errors, yellow for warnings, dim for hints
- All non-data output goes to stderr. Only data to stdout. Pipe-safe.
- Auth errors are detailed with recovery steps
- Config path: `~/.pipelite/config.toml` with `PIPELITE_CONFIG` env var override
- Grouped TOML sections: `[server]`, `[output]`, `[display]`
- Generated config is a commented template with defaults
- `pipelite config set` accepts dotted paths: `pipelite config set output.format json`
- Config precedence: CLI flags > env vars > config file

### Claude's Discretion
- Exact progress spinner implementation for `pipelite ping`
- Internal module structure (cli/ vs commands/ split)
- Error type hierarchy (thiserror enums vs anyhow contexts)
- HTTP client configuration details (exact timeout values, retry counts)
- TTY detection implementation details

### Deferred Ideas (OUT OF SCOPE)
- Config profiles for multiple servers ([profiles.production], [profiles.staging]) -- v2 requirement (PROF-01, PROF-02)
- Activity logging shortcut (`pipelite log`) -- v2 requirement (ALOG-01)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AUTH-01 | User can initialize CLI with API key via `pipelite init` interactive flow | dialoguer 0.12 for wizard prompts; clap derive for `--url`/`--key` flags; sync code before async runtime |
| AUTH-02 | User can authenticate via `PIPELITE_API_KEY` environment variable for CI/scripts | clap `env` feature for env var binding; config precedence pattern in AppContext builder |
| AUTH-03 | API key is stored in config file with 0600 permissions (never as CLI flag) | `OpenOptionsExt::mode(0o600)` on Unix; `PermissionsExt` for verification |
| AUTH-04 | User can test connection with `pipelite ping` showing server status and latency | reqwest client with timeout; `std::time::Instant` for latency; indicatif spinner on stderr |
| CONF-01 | User can store persistent config in `~/.pipelite/config.toml` | toml 1.1 for ser/de; hardcoded `~/.pipelite/` per user decision with `PIPELITE_CONFIG` override |
| CONF-02 | User can view config with `pipelite config show` | toml serialization to pretty-printed string; mask API key in display |
| CONF-03 | User can set config values with `pipelite config set <key> <value>` | toml_edit crate for format-preserving edits with comments; dotted path resolution |
| CONF-04 | Config precedence follows flags > env vars > config file | Three-layer merge in AppContext builder; clap env feature for env var defaults |
| ERRH-01 | CLI returns non-zero exit codes on failure (1=runtime, 2=misuse) | `std::process::exit()` with distinct codes; clap returns 2 for misuse by default |
| ERRH-02 | Error messages are actionable with suggested next commands | thiserror enums with hint fields; structured error formatter on stderr |
| ERRH-03 | CLI shows `--help` with usage examples on every command | clap `after_help` attribute with example strings on every subcommand |
| UX-02 | Quiet mode (`-q`) suppresses non-essential output | Global `--quiet` flag on Cli struct; conditional stderr output in output abstraction |
| UX-03 | `pipelite --version` shows version string | clap `version` from Cargo.toml + build metadata via `env!()` macros |
</phase_requirements>

## Standard Stack

### Core (Phase 1 Dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.6 (derive, env) | CLI parsing, subcommands, help, version | Ecosystem standard; derive macros for subcommand routing; env feature for `PIPELITE_API_KEY` |
| tokio | 1.50 (rt, macros) | Async runtime | Only viable runtime; current_thread flavor for CLI; needed by reqwest |
| reqwest | 0.13 (json, rustls-tls) | HTTP client for CRM API | Ergonomic async HTTP; rustls for static binary; built-in timeout support |
| serde | 1.0 (derive) | Serialization framework | Non-negotiable for structured data |
| serde_json | 1.0 | JSON output format | API responses and `--format json` output |
| toml | 1.1 | Config file reading/writing | Reads/writes `config.toml`; serde integration |
| toml_edit | 0.22 | Format-preserving config edits | `config set` preserves comments and formatting |
| dialoguer | 0.12 | Interactive init wizard prompts | Stable, TTY-aware, simpler than inquire |
| anyhow | 1.0 | Error propagation with context | `.context("doing X")` pattern for actionable errors |
| thiserror | 2.0 | Typed error enum definitions | Structured errors: ApiError, ConfigError, AuthError |
| colored | 3.1 | Terminal colors | `.red()`, `.yellow()`, `.dimmed()`; respects `NO_COLOR` |
| indicatif | 0.18 | Spinner for ping command | Shows activity during network requests on stderr |

### Supporting (Phase 1)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| comfy-table | 7.2 | Table output | `pipelite config show` table display; foundation for Phase 2 |
| chrono | 0.4 (serde) | Timestamps | Not strictly needed in Phase 1 but worth including for foundation |

### Not Needed in Phase 1

| Library | Phase | Reason |
|---------|-------|--------|
| csv | Phase 2 | No CSV output commands in Phase 1 |
| clap_complete | Phase 4 | Shell completions are a later enhancement |
| directories | N/A | User locked config path to `~/.pipelite/`; use `PIPELITE_CONFIG` env var override instead |

### Installation

```toml
[dependencies]
clap = { version = "4.6", features = ["derive", "env"] }
tokio = { version = "1", features = ["rt", "macros"] }
reqwest = { version = "0.13", default-features = false, features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "1.1"
toml_edit = "0.22"
dialoguer = "0.12"
anyhow = "1.0"
thiserror = "2.0"
colored = "3.1"
indicatif = "0.18"
comfy-table = "7.2"
```

**Note on `toml_edit`:** The user requires `pipelite config set` to work with dotted paths and the generated config to be a commented template. The `toml` crate loses comments on round-trip. `toml_edit` preserves formatting and comments during edits. Use `toml` for reading/deserializing config and `toml_edit` for writing/modifying.

## Architecture Patterns

### Recommended Project Structure (Phase 1)

```
src/
├── main.rs              # ~30 lines: parse CLI, build context, match command, handle errors
├── cli/
│   ├── mod.rs           # Cli struct (Parser), Commands enum (Subcommand), global flags
│   ├── config.rs        # ConfigCommands subcommand enum (show, set, get)
│   └── init.rs          # InitArgs struct (--url, --key flags)
├── commands/
│   ├── mod.rs           # Shared command utilities
│   ├── init.rs          # Init wizard logic (dialoguer prompts, config creation)
│   ├── ping.rs          # Connection test (HTTP request + latency measurement)
│   └── config.rs        # Config show/set/get handlers
├── api/
│   ├── mod.rs           # PipeliteClient struct (reqwest wrapper, auth, timeouts)
│   └── models.rs        # Shared API types (PingResponse, etc.)
├── output/
│   ├── mod.rs           # OutputFormat enum, output dispatch fn, TTY detection
│   └── table.rs         # Table formatter (comfy-table) for config show
├── config.rs            # AppConfig struct, TOML loading, env var merge, config file creation
├── context.rs           # AppContext: merged config + client + output settings
└── error.rs             # Error enums (thiserror) + formatted error display
```

### Pattern 1: AppContext as Single Source of Truth

**What:** Build a single `AppContext` struct at startup that merges config file, env vars, and CLI flags. Pass by reference to all command handlers.

**When to use:** Always -- this is the foundational pattern.

```rust
pub struct AppContext {
    pub config: AppConfig,
    pub client: PipeliteClient,
    pub output_format: OutputFormat,
    pub quiet: bool,
    pub verbose: bool,
    pub color: bool,
}

impl AppContext {
    pub fn build(cli: &Cli) -> Result<Self> {
        let config = AppConfig::load(cli.config_path.as_deref())?;  // file + env merge
        let client = PipeliteClient::new(&config)?;
        let is_tty = std::io::stdout().is_terminal();
        let output_format = cli.format.unwrap_or(if is_tty {
            OutputFormat::Table
        } else {
            OutputFormat::Json
        });
        let color = !cli.no_color && is_tty && std::env::var("NO_COLOR").is_err();
        Ok(Self { config, client, output_format, quiet: cli.quiet, verbose: cli.verbose, color })
    }
}
```

### Pattern 2: Lazy Context for Commands That Don't Need It

**What:** `--version` and `--help` must not load config or create HTTP client. Clap handles these before your code runs, but if you add custom version logic, guard it.

**When to use:** For `pipelite --version` with rich build info.

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        // These need full context
        Commands::Init(args) => {
            // Init is special: may not have config yet
            commands::init::run(args).await?;
        }
        Commands::Ping => {
            let ctx = AppContext::build(&cli)?;
            commands::ping::run(&ctx).await?;
        }
        Commands::Config(cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::config::run(&ctx, cmd)?;
        }
    }
    Ok(())
}
```

### Pattern 3: Structured Error Display on stderr

**What:** All errors go to stderr with structured format: type + message + hint. Colors applied only when stderr is a TTY.

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Authentication failed")]
    Auth {
        detail: String,
        hint: String,
    },
    #[error("Connection failed")]
    Connection {
        detail: String,
        hint: String,
    },
    #[error("Configuration error")]
    Config {
        detail: String,
        hint: String,
    },
}

pub fn display_error(err: &anyhow::Error, color: bool) {
    let stderr = std::io::stderr();
    if let Some(cli_err) = err.downcast_ref::<CliError>() {
        match cli_err {
            CliError::Auth { detail, hint } => {
                eprintln!("{}",  format_error("error", &cli_err.to_string(), detail, hint, color));
            }
            // ... other variants
        }
    } else {
        // Generic anyhow error with context chain
        eprintln!("error: {err:#}");
    }
}

fn format_error(level: &str, title: &str, detail: &str, hint: &str, color: bool) -> String {
    if color {
        format!("{}: {}\n  {}\n  {}: {}",
            "error".red().bold(), title, detail,
            "hint".dimmed(), hint)
    } else {
        format!("{level}: {title}\n  {detail}\n  hint: {hint}")
    }
}
```

### Pattern 4: Config File Creation with 0600 Permissions

**What:** Create config file atomically with restricted permissions on Unix.

```rust
use std::os::unix::fs::OpenOptionsExt;

pub fn write_config(path: &Path, config: &AppConfig) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = generate_commented_config(config);

    // Create file with 0600 permissions (owner read/write only)
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;

    file.write_all(content.as_bytes())?;
    Ok(())
}
```

### Pattern 5: Init Wizard (Sync, Before Async)

**What:** The init wizard uses dialoguer which is synchronous. Keep it in sync code, then use async only for the test-connection step.

```rust
pub async fn run(args: &InitArgs) -> Result<()> {
    let (url, key) = if args.url.is_some() && args.key.is_some() {
        // Headless mode: use provided flags
        (args.url.clone().unwrap(), args.key.clone().unwrap())
    } else {
        // Interactive wizard (dialoguer is sync -- this is fine)
        let url: String = dialoguer::Input::new()
            .with_prompt("Pipelite server URL")
            .default("https://app.pipelite.io".into())
            .interact_text()?;

        let key: String = dialoguer::Password::new()
            .with_prompt("API key")
            .interact()?;

        (url, key)
    };

    // Test connection (async)
    eprintln!("Testing connection...");
    let client = PipeliteClient::from_credentials(&url, &key)?;
    client.ping().await?;
    eprintln!("Connected successfully!");

    // Confirm and save
    let config = AppConfig::new(url, key);
    let path = config_path()?;
    write_config(&path, &config)?;
    eprintln!("Configuration saved to {}", path.display());
    Ok(())
}
```

### Anti-Patterns to Avoid

- **Monolithic main.rs:** Keep main.rs to ~30 lines. Parse, build context, dispatch, handle errors.
- **HTTP logic in command handlers:** All HTTP goes through `PipeliteClient`. Commands never touch `reqwest` directly.
- **Format-specific code in commands:** Commands return data; output layer renders it.
- **Blocking dialoguer in async context:** Keep dialoguer calls in sync code or use `spawn_blocking`.
- **`unwrap()` on HTTP responses:** Always use `?` with `.context()` for actionable error messages.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI argument parsing | Custom arg parser | clap 4.6 derive | Subcommands, validation, help gen, env var binding |
| Interactive prompts | Custom stdin reader | dialoguer 0.12 | Password masking, validation, TTY detection |
| Terminal colors | Raw ANSI escape codes | colored 3.1 | NO_COLOR support, cross-platform |
| Table formatting | Manual column alignment | comfy-table 7.2 | Unicode width, terminal wrapping, customizable |
| Config file writing | `format!()` with TOML syntax | toml_edit 0.22 | Preserves comments, handles escaping |
| HTTP timeout/retry | Manual timeout logic | reqwest Client builder | Built-in connect_timeout, timeout, connection pooling |
| Progress spinner | Custom spinner thread | indicatif 0.18 | stderr-safe, multiple styles, async-compatible |
| Error type boilerplate | Manual Display/Error impls | thiserror 2.0 | Derive macro for Display + Error + From |

**Key insight:** Every "simple" hand-rolled solution above has edge cases that the library handles correctly and you will miss (Unicode width calculation, ANSI code stripping in non-TTY, TOML escaping for special characters, connection pool reuse).

## Common Pitfalls

### Pitfall 1: World-Readable Config File with API Key
**What goes wrong:** `std::fs::write` uses process umask (typically 0022), creating 0644 files readable by any local user.
**Why it happens:** Developers test on single-user machines and never check permissions.
**How to avoid:** Use `OpenOptions::new().mode(0o600)` on Unix for config file creation. Verify with `stat` in tests.
**Warning signs:** Config file creation code without explicit `mode()` call.

### Pitfall 2: Blocking Dialoguer in Async Runtime
**What goes wrong:** `dialoguer::Input::interact()` blocks the tokio thread. Other futures stall, timeouts stop firing.
**Why it happens:** Tutorials show `#[tokio::main]` and developers put everything inside async functions.
**How to avoid:** Use `current_thread` flavor. Keep dialoguer calls in the sync portion of command logic (before awaiting network calls). The init wizard flow is: sync prompts first, then `client.ping().await`.
**Warning signs:** Dialoguer calls inside `async fn` without `spawn_blocking`.

### Pitfall 3: TTY Detection Missing from Day One
**What goes wrong:** Colors and spinners appear in piped output, breaking `pipelite ping | jq .`.
**Why it happens:** Developers test only in terminals.
**How to avoid:** Use `std::io::IsTerminal` on stdout AND stderr independently at startup. Route all decorations (colors, spinners, progress) to stderr. Auto-switch to JSON when stdout is not a TTY.
**Warning signs:** `println!` with color codes and no TTY guard.

### Pitfall 4: No HTTP Timeout
**What goes wrong:** `reqwest::Client::new()` has no default timeout. CLI hangs forever on unresponsive servers.
**Why it happens:** Developers test on localhost where timeouts never trigger.
**How to avoid:** Configure timeouts on Client construction: `connect_timeout(5s)`, `timeout(30s)`.
**Warning signs:** `reqwest::Client::new()` or `Client::builder().build()` with no timeout calls.

### Pitfall 5: Slow Startup for --version
**What goes wrong:** Building AppContext (loading config, creating HTTP client) for `--version` adds 50-100ms.
**Why it happens:** All commands go through the same context-building path.
**How to avoid:** Clap handles `--version` before your code runs if you use `#[command(version)]`. For rich version info, use `#[command(version = build_version())]` where `build_version()` is a const/static function. No context needed.
**Warning signs:** AppContext built unconditionally before command dispatch.

### Pitfall 6: Config Set Losing Comments
**What goes wrong:** Reading config with `toml::from_str`, modifying the struct, and writing with `toml::to_string` strips all comments.
**Why it happens:** The `toml` crate serializer does not preserve comments.
**How to avoid:** Use `toml_edit` for `config set` operations. Parse as `toml_edit::DocumentMut`, navigate to the key, modify in place, write back.
**Warning signs:** `config set` round-trips through serde serialize/deserialize.

## Code Examples

### Main Entry Point

```rust
// src/main.rs
use anyhow::Result;
use clap::Parser;

mod api;
mod cli;
mod commands;
mod config;
mod context;
mod error;
mod output;

use cli::Cli;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli).await {
        error::display_error(&err, std::io::stderr().is_terminal());
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        cli::Commands::Init(args) => commands::init::run(&args).await,
        cli::Commands::Ping => {
            let ctx = context::AppContext::build(&cli)?;
            commands::ping::run(&ctx).await
        }
        cli::Commands::Config(cmd) => {
            let ctx = context::AppContext::build(&cli)?;
            commands::config::run(&ctx, &cmd)
        }
    }
}
```

### CLI Definition with Global Flags

```rust
// src/cli/mod.rs
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "pipelite",
    version = build_version(),
    about = "Manage your Pipelite CRM from the terminal",
    after_help = "Get started:\n  pipelite init          Configure server connection\n  pipelite ping          Test connectivity\n  pipelite config show   View current configuration"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format
    #[arg(long, global = true, value_enum)]
    pub format: Option<OutputFormat>,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Show verbose debug information
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize connection to a Pipelite server
    #[command(after_help = "Examples:\n  pipelite init\n  pipelite init --url https://crm.example.com --key pk_live_abc123")]
    Init(InitArgs),

    /// Test server connectivity
    #[command(after_help = "Examples:\n  pipelite ping")]
    Ping,

    /// Manage configuration
    #[command(subcommand, after_help = "Examples:\n  pipelite config show\n  pipelite config set output.format json")]
    Config(ConfigCommands),
}

#[derive(clap::Args)]
pub struct InitArgs {
    /// Server URL (prompted if not provided)
    #[arg(long)]
    pub url: Option<String>,

    /// API key (prompted if not provided)
    #[arg(long)]
    pub key: Option<String>,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Display current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key (dotted path, e.g., output.format)
        key: String,
        /// Value to set
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key (dotted path)
        key: String,
    },
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
    Plain,
}

fn build_version() -> &'static str {
    concat!(
        env!("CARGO_PKG_VERSION"),
        " (rustc ",
        env!("RUSTC_VERSION", "unknown"),
        ", ",
        std::env::consts::OS,
        "-",
        std::env::consts::ARCH,
        ")"
    )
}
```

**Note on build_version:** `env!("RUSTC_VERSION")` requires a build script (`build.rs`) to set this. Alternatively, use the `built` crate or a simpler approach with `option_env!`.

### Config File Structure

```toml
# Pipelite CLI Configuration
# Generated by `pipelite init`

[server]
# Pipelite CRM server URL
url = "https://app.pipelite.io"
# API key for authentication (keep this secret!)
api_key = "pk_live_abc123"

[output]
# Default output format: json, table, csv, plain
# format = "table"

[display]
# Disable colored output (also: NO_COLOR env var or --no-color flag)
# no_color = false
```

### Ping Command with Spinner and Latency

```rust
// src/commands/ping.rs
use anyhow::{Context, Result};
use std::time::Instant;

pub async fn run(ctx: &crate::context::AppContext) -> Result<()> {
    let spinner = if !ctx.quiet && std::io::stderr().is_terminal() {
        let sp = indicatif::ProgressBar::new_spinner();
        sp.set_message("Connecting...");
        sp.enable_steady_tick(std::time::Duration::from_millis(80));
        Some(sp)
    } else {
        None
    };

    let start = Instant::now();
    let status = ctx.client.ping().await
        .context("Failed to reach Pipelite server")?;
    let latency = start.elapsed();

    if let Some(sp) = spinner {
        sp.finish_and_clear();
    }

    if !ctx.quiet {
        eprintln!("Server: {}", ctx.config.server.url);
        eprintln!("Status: {}", status);
        eprintln!("Latency: {}ms", latency.as_millis());
    }

    Ok(())
}
```

### Config Set with toml_edit (Preserving Comments)

```rust
// In src/commands/config.rs
use toml_edit::DocumentMut;

pub fn set_value(config_path: &Path, key: &str, value: &str) -> Result<()> {
    let content = std::fs::read_to_string(config_path)
        .context("Could not read config file")?;
    let mut doc = content.parse::<DocumentMut>()
        .context("Config file has invalid TOML")?;

    // Parse dotted key: "output.format" -> ["output", "format"]
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        [section, field] => {
            doc[section][field] = toml_edit::value(value);
        }
        [field] => {
            doc[field] = toml_edit::value(value);
        }
        _ => anyhow::bail!("Invalid key format: {key}. Use section.key (e.g., output.format)"),
    }

    // Write back preserving comments and formatting
    std::fs::write(config_path, doc.to_string())
        .context("Could not write config file")?;
    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| async-std runtime | tokio only | March 2025 (async-std discontinued) | No choice -- use tokio |
| `atty` crate for TTY detection | `std::io::IsTerminal` | Rust 1.70 (stable) | No external crate needed |
| `structopt` for CLI | `clap` 4 derive | clap v3+ absorbed structopt | Use clap directly |
| `reqwest` native-tls default | `reqwest` 0.13 rustls default | reqwest 0.13 | rustls is default; no OpenSSL needed |
| `dirs` / `directories` crate | Still current (6.0) | N/A | User chose hardcoded `~/.pipelite/` path |
| Rust edition 2021 | Rust edition 2024 | Late 2024 | Project already uses edition 2024 |

## Open Questions

1. **Rich version string implementation**
   - What we know: User wants `pipelite 0.1.0 (rustc X.Y.Z, linux-x86_64)` format
   - What's unclear: Whether to use a `build.rs` script, the `built` crate, or `vergen` crate for build metadata
   - Recommendation: Use a minimal `build.rs` that sets `RUSTC_VERSION` env var via `rustc --version`. Simpler than adding a dependency. Fallback: `option_env!("RUSTC_VERSION").unwrap_or("unknown")`.

2. **Config path: `~/.pipelite/` vs XDG**
   - What we know: User explicitly chose `~/.pipelite/config.toml` in CONTEXT.md. REQUIREMENTS.md mentions "XDG-compliant paths" for CONF-01.
   - What's unclear: Whether `~/.pipelite/` satisfies the XDG requirement since it is not under `~/.config/`
   - Recommendation: Use `~/.pipelite/` as the user decided. The `PIPELITE_CONFIG` env var override satisfies users who want XDG compliance. Document the path clearly.

3. **Commented config template approach**
   - What we know: User wants generated config with comments explaining each key
   - What's unclear: Whether to use `toml_edit` to build the template programmatically or use a hardcoded template string
   - Recommendation: Use a hardcoded template string for the initial `pipelite init` generation (simpler, full control over formatting). Use `toml_edit` only for `config set` operations.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test framework (`#[cfg(test)]` + `#[test]`) |
| Config file | None -- Rust tests need no config file |
| Quick run command | `cargo test` |
| Full suite command | `cargo test -- --include-ignored` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AUTH-01 | Init wizard creates config file | integration | `cargo test --test init_test` | No -- Wave 0 |
| AUTH-02 | Env var auth overrides config file | unit | `cargo test config::tests::env_var_precedence` | No -- Wave 0 |
| AUTH-03 | Config file created with 0600 perms | unit | `cargo test config::tests::file_permissions` | No -- Wave 0 |
| AUTH-04 | Ping shows status and latency | integration | `cargo test --test ping_test` | No -- Wave 0 |
| CONF-01 | Config persists to ~/.pipelite/config.toml | unit | `cargo test config::tests::load_and_save` | No -- Wave 0 |
| CONF-02 | Config show displays current config | integration | `cargo test --test config_show_test` | No -- Wave 0 |
| CONF-03 | Config set modifies value | unit | `cargo test config::tests::set_dotted_path` | No -- Wave 0 |
| CONF-04 | Precedence: flags > env > file | unit | `cargo test config::tests::precedence_order` | No -- Wave 0 |
| ERRH-01 | Non-zero exit codes on failure | integration | `cargo test --test exit_code_test` | No -- Wave 0 |
| ERRH-02 | Actionable error messages | unit | `cargo test error::tests::error_formatting` | No -- Wave 0 |
| ERRH-03 | Help with examples on every cmd | integration | `cargo test --test help_examples_test` | No -- Wave 0 |
| UX-02 | Quiet mode suppresses output | integration | `cargo test --test quiet_mode_test` | No -- Wave 0 |
| UX-03 | Version shows build info | integration | `cargo test --test version_test` | No -- Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test -- --include-ignored`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/` directory -- does not exist yet
- [ ] Integration test harness for CLI binary (`assert_cmd` crate recommended for testing CLI output and exit codes)
- [ ] `assert_cmd` + `predicates` in `[dev-dependencies]` for integration tests
- [ ] Unit test modules in each source file (`#[cfg(test)] mod tests`)
- [ ] Test fixtures: sample config.toml files for config loading tests
- [ ] Temporary directory setup for config file creation tests (`tempfile` crate in dev-dependencies)

**Recommended dev-dependencies:**

```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.0"
```

## Sources

### Primary (HIGH confidence)
- [clap docs.rs -- derive tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) -- subcommand patterns, after_help, global args
- [clap GitHub -- git-derive.rs example](https://github.com/clap-rs/clap/blob/master/examples/git-derive.rs) -- real-world subcommand structure
- [reqwest docs.rs -- ClientBuilder](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html) -- timeout, rustls-tls configuration
- [reqwest v0.13 announcement](https://seanmonstar.com/blog/reqwest-v013-rustls-default/) -- rustls as default TLS
- [tokio docs.rs -- runtime](https://docs.rs/tokio/latest/tokio/runtime/index.html) -- current_thread flavor documentation
- [std::os::unix::fs::OpenOptionsExt](https://doc.rust-lang.org/std/os/unix/fs/trait.OpenOptionsExt.html) -- mode() for file permissions
- [std::io::IsTerminal](https://doc.rust-lang.org/std/io/trait.IsTerminal.html) -- TTY detection (stable since Rust 1.70)
- [dialoguer docs.rs](https://docs.rs/dialoguer/latest/dialoguer/) -- Input, Password, Confirm prompts
- [directories crate](https://docs.rs/directories) -- ProjectDirs for XDG paths (noted but not used per user decision)

### Secondary (MEDIUM confidence)
- [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/handling-arguments.html) -- argument handling patterns
- [Tokio bridging with sync code](https://tokio.rs/tokio/topics/bridging) -- mixing sync dialoguer with async reqwest
- [Top 5 Tokio runtime mistakes](https://www.techbuddies.io/2026/03/21/top-5-tokio-runtime-mistakes-that-quietly-kill-your-async-rust/) -- blocking runtime pitfalls

### Tertiary (LOW confidence)
- `toml_edit` version 0.22 -- inferred from crates.io; exact latest version should be verified at implementation time
- `build.rs` approach for RUSTC_VERSION -- standard practice but exact implementation varies

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all crates verified in project research (2026-03-23), versions confirmed on crates.io
- Architecture: HIGH -- layered pattern sourced from clap author and Rust CLI Book; matches established project research
- Pitfalls: HIGH -- all six critical pitfalls documented in PITFALLS.md with prevention strategies; verified against Rust stdlib docs

**Research date:** 2026-03-24
**Valid until:** 2026-04-24 (stable ecosystem, no fast-moving dependencies)
