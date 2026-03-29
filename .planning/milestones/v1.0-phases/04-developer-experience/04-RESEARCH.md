# Phase 4: Developer Experience - Research

**Researched:** 2026-03-25
**Domain:** Interactive prompts, headless mode, shell completions, dry-run
**Confidence:** HIGH

## Summary

Phase 4 enhances existing CRUD commands with interactive prompts (using dialoguer FuzzySelect for entity dropdowns), explicit headless mode (`--no-input`), shell completions (via clap_complete), and `--dry-run` for previewing mutations. The codebase already uses dialoguer for `init` and has TTY detection throughout -- this phase extends those patterns to all create/update commands.

The primary refactoring is changing every `single_create`/`single_update` function from "fail if missing flag" to "prompt if TTY, fail if headless". The `--no-input` and `--dry-run` flags are global clap args added to `Cli` struct and carried through `AppContext`. Shell completions are a standalone `completions` subcommand using `clap_complete::aot::generate()`.

**Primary recommendation:** Build a shared `prompt` module (`src/prompt.rs`) with generic helpers (`prompt_text`, `prompt_select`, `prompt_optional`) that check `AppContext.no_input` and TTY state. Each entity's create/update calls these helpers instead of directly using `ok_or_else(CliError::Validation)`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Running create/update with no flags on a TTY opens interactive prompts for ALL fields (required first, then optional)
- Fields with known values (stage, pipeline, org, person) use dialoguer FuzzySelect -- type to filter, arrow keys to browse, options fetched from API on prompt
- Partial flags are honored: if some flags provided, pre-fill those and prompt for remaining fields
- Optional fields show "(optional, press Enter to skip)" -- empty input skips the field
- Prompts only activate when stdin is a TTY (consistent with existing init behavior)
- dialoguer already in use for init -- extend usage, no library switch needed
- `--no-input` flag guarantees no interactive prompts
- Non-TTY stdin automatically implies --no-input (auto-detect + explicit flag for control)
- Missing required fields in headless mode: fail with exit code 2, listing ALL missing fields at once (not one at a time)
- Delete in headless mode requires --force -- error without it (consistent with Phase 1 decision)
- `--stdin` and individual field flags are mutually exclusive -- error if both provided
- All mutations can be performed entirely via flags (HEAD-03 requirement)
- `pipelite completions bash|zsh|fish` subcommand -- outputs completion script to stdout
- Static completions only (clap_complete from clap definitions) -- commands, subcommands, flags
- Dynamic entity ID completion deferred to Phase 5 (needs caching layer)
- Show shell-specific install instructions on stderr after generating script
- `--dry-run` flag on all mutation commands: create, update, and delete
- Shows formatted output: HTTP method, endpoint URL, and JSON body that would be sent
- Delete dry-run shows "Would delete [entity] [id]"
- Dry-run works with interactive prompts: user fills prompts, then sees preview instead of executing
- Respects --format flag (e.g., --format json for machine-readable dry-run output)

### Claude's Discretion
- Exact prompt ordering and grouping for each entity's fields
- FuzzySelect display format for entity options (e.g., "name (id)" vs "id - name")
- Dry-run output styling (colors, separators)
- clap_complete shell integration details
- How --dry-run interacts with --stdin batch mode (show all payloads or summary)

### Deferred Ideas (OUT OF SCOPE)
- Dynamic shell completions for entity IDs/names -- Phase 5 (needs caching, CACH-03)
- Config profiles for --no-input defaults per environment -- v2 (PROF-01, PROF-02)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INTR-01 | Running create/update with no flags opens interactive prompts for required fields | Shared prompt module pattern, dialoguer Input/FuzzySelect |
| INTR-02 | Interactive prompts show dropdowns for known values (stages, pipelines) | dialoguer FuzzySelect with `fuzzy-select` feature flag |
| INTR-03 | Interactive prompts only activate when stdin is a TTY | `std::io::IsTerminal` already used throughout codebase |
| HEAD-01 | User can pass `--no-input` to guarantee no interactive prompts | Global clap flag on Cli struct, carried in AppContext |
| HEAD-02 | Headless mode fails with clear error if required input is missing | Collect ALL missing fields, single CliError::Validation with exit code 2 |
| HEAD-03 | All mutations can be performed entirely via flags (no prompts needed) | Already true -- all fields are Option flags, just need prompt-or-error logic |
| SHLL-01 | User can generate shell completions for bash | clap_complete 4.6 with Shell::Bash |
| SHLL-02 | User can generate shell completions for zsh | clap_complete 4.6 with Shell::Zsh |
| SHLL-03 | User can generate shell completions for fish | clap_complete 4.6 with Shell::Fish |
| UX-04 | `--dry-run` on mutations shows what would be sent without executing | Global flag, intercepted before API call to print request details |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| dialoguer | 0.12.0 | Interactive prompts (Input, FuzzySelect, Confirm) | Already in use for `init`, extend with FuzzySelect |
| clap_complete | 4.6.0 | Shell completion script generation | Official clap companion crate for bash/zsh/fish completions |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| clap | 4.6 | CLI framework (already in use) | Add `--no-input`, `--dry-run` global flags |
| std::io::IsTerminal | stable | TTY detection | Already imported throughout codebase |
| colored | 3.1 | Terminal colors (already in use) | Dry-run output styling |
| serde_json | 1.0 | JSON formatting (already in use) | Dry-run JSON body display |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| dialoguer FuzzySelect | inquire crate | inquire has better UX but would add a new dependency; dialoguer is already in use |
| clap_complete | Manual completion scripts | clap_complete auto-generates from clap definitions; manual scripts would drift |

**Installation:**
```bash
# Only new dependency is clap_complete
cargo add clap_complete@4.6
# Update dialoguer to enable fuzzy-select feature
# Change: dialoguer = "0.12" -> dialoguer = { version = "0.12", features = ["fuzzy-select"] }
```

## Architecture Patterns

### Recommended Module Structure
```
src/
├── prompt.rs            # NEW: shared prompt helpers (prompt_text, prompt_select, prompt_optional)
├── cli/
│   ├── mod.rs           # MODIFY: add --no-input, --dry-run global flags, Completions command
│   └── completions.rs   # NEW: completions subcommand args
├── commands/
│   ├── completions.rs   # NEW: completions command handler
│   ├── deals/
│   │   ├── create.rs    # MODIFY: add interactive prompt logic, dry-run intercept
│   │   ├── update.rs    # MODIFY: add interactive prompt logic, dry-run intercept
│   │   └── delete.rs    # MODIFY: add dry-run intercept
│   └── [other entities same pattern]
├── context.rs           # MODIFY: add no_input, dry_run fields to AppContext
└── main.rs              # MODIFY: add Completions dispatch
```

### Pattern 1: Prompt-or-Error for Required Fields
**What:** Replace `ok_or_else(CliError::Validation)` with a helper that prompts on TTY or collects missing fields for headless error.
**When to use:** Every required field in create/update commands.
**Example:**
```rust
// src/prompt.rs
use std::io::IsTerminal;
use dialoguer::{Input, FuzzySelect};
use crate::context::AppContext;
use crate::error::CliError;

/// Resolve a required text field: use flag value, prompt if TTY, or collect as missing.
pub fn require_text(
    flag_value: &Option<String>,
    field_name: &str,
    prompt_label: &str,
    missing: &mut Vec<String>,
    no_input: bool,
) -> Result<Option<String>, anyhow::Error> {
    if let Some(val) = flag_value {
        return Ok(Some(val.clone()));
    }
    if no_input || !std::io::stdin().is_terminal() {
        missing.push(format!("--{}", field_name));
        return Ok(None);
    }
    let val: String = Input::new()
        .with_prompt(prompt_label)
        .interact_text()?;
    Ok(Some(val))
}

/// Resolve a field by selecting from a list of API-fetched options.
pub fn require_select(
    flag_value: &Option<String>,
    field_name: &str,
    prompt_label: &str,
    options: &[(String, String)],  // (id, display_name)
    missing: &mut Vec<String>,
    no_input: bool,
) -> Result<Option<String>, anyhow::Error> {
    if let Some(val) = flag_value {
        return Ok(Some(val.clone()));
    }
    if no_input || !std::io::stdin().is_terminal() {
        missing.push(format!("--{}", field_name));
        return Ok(None);
    }
    let display: Vec<String> = options.iter()
        .map(|(id, name)| format!("{} ({})", name, id))
        .collect();
    let idx = FuzzySelect::new()
        .with_prompt(prompt_label)
        .items(&display)
        .interact()?;
    Ok(Some(options[idx].0.clone()))
}

/// After collecting all fields, check for missing required ones.
pub fn check_missing(missing: &[String], usage_hint: &str) -> Result<(), CliError> {
    if missing.is_empty() {
        return Ok(());
    }
    Err(CliError::Validation {
        detail: format!("Missing required flags: {}", missing.join(", ")),
        hint: usage_hint.to_string(),
    })
}
```

### Pattern 2: Dry-Run Intercept
**What:** Check `ctx.dry_run` after building the request payload but before calling the API. Print the would-be request instead.
**When to use:** Every mutation command (create, update, delete).
**Example:**
```rust
// In a create command, after building the DealCreate struct:
if ctx.dry_run {
    return render_dry_run(
        "POST",
        &format!("{}/api/v1/deals", ctx.client.base_url()),
        &serde_json::to_value(&data)?,
        &ctx.output_format,
        ctx.color,
    );
}
let deal = ctx.client.create_deal(&data).await?;
```

### Pattern 3: Shell Completions Subcommand
**What:** A `completions` subcommand that generates static shell completion scripts.
**When to use:** Single implementation, standalone command.
**Example:**
```rust
// src/commands/completions.rs
use clap::CommandFactory;
use clap_complete::aot::{generate, Shell};
use crate::cli::Cli;

pub fn run(shell: Shell) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "pipelite", &mut std::io::stdout());
    // Show install instructions on stderr
    let instruction = match shell {
        Shell::Bash => "Add to ~/.bashrc:\n  source <(pipelite completions bash)",
        Shell::Zsh => "Add to ~/.zshrc:\n  source <(pipelite completions zsh)",
        Shell::Fish => "Run once:\n  pipelite completions fish > ~/.config/fish/completions/pipelite.fish",
        _ => "See your shell's documentation for completion installation.",
    };
    eprintln!("\n# Install instructions:\n# {}", instruction.replace('\n', "\n# "));
    Ok(())
}
```

### Pattern 4: Global Flags on AppContext
**What:** Add `no_input` and `dry_run` as fields on AppContext, resolved from CLI flags and TTY state.
**When to use:** Built once in `AppContext::build()`, consumed everywhere.
**Example:**
```rust
// In Cli struct:
/// Disable interactive prompts
#[arg(long, global = true)]
pub no_input: bool,

/// Preview mutations without executing
#[arg(long, global = true)]
pub dry_run: bool,

// In AppContext::build():
let no_input = cli.no_input || !std::io::stdin().is_terminal();
// ...
Self { ..., no_input, dry_run: cli.dry_run }
```

### Anti-Patterns to Avoid
- **Prompting in headless mode:** Never call dialoguer without checking `no_input` first -- dialoguer will panic or hang on non-TTY
- **One error per missing field:** Headless mode must collect ALL missing fields and report them together, not fail on the first one
- **Fetching API data for prompts when not needed:** Only call list endpoints (for FuzzySelect options) if the field was not provided via flag and we are in interactive mode
- **Conflating --stdin and interactive:** `--stdin` reads JSON from pipe, interactive reads from TTY prompts -- these are different input modes

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Shell completion scripts | Custom bash/zsh/fish scripts | clap_complete | Auto-generates from clap definitions, stays in sync with commands |
| Fuzzy filtering in terminal | Custom terminal UI | dialoguer FuzzySelect | Handles terminal rendering, key events, search matching |
| TTY detection | Custom isatty check | std::io::IsTerminal | Standard library, already in use |

**Key insight:** The interactive prompt logic is the main custom code; everything else leverages existing libraries.

## Common Pitfalls

### Pitfall 1: FuzzySelect Panics on Non-TTY
**What goes wrong:** Calling `FuzzySelect::interact()` when stdin is not a TTY causes a panic.
**Why it happens:** dialoguer expects terminal control codes and cursor movement.
**How to avoid:** Always gate FuzzySelect/Input behind `std::io::stdin().is_terminal()` AND `!ctx.no_input` checks.
**Warning signs:** Tests that call create without TTY will panic instead of error.

### Pitfall 2: Forgetting to Enable fuzzy-select Feature
**What goes wrong:** `FuzzySelect` is not available, compilation fails.
**Why it happens:** dialoguer 0.12 has `fuzzy-select` as an optional feature flag, not included by default.
**How to avoid:** Update Cargo.toml: `dialoguer = { version = "0.12", features = ["fuzzy-select"] }`.
**Warning signs:** "cannot find struct FuzzySelect" compiler error.

### Pitfall 3: Headless Exit Code Wrong
**What goes wrong:** Missing fields return exit code 1 instead of 2.
**Why it happens:** CliError::Validation goes through the default error path returning 1.
**How to avoid:** The `exit_code()` function in error.rs currently returns 2 only for clap errors. Add a `MissingInput` variant or adjust the exit code logic for validation errors in headless mode.
**Warning signs:** Scripts checking `$?` for exit code 2 get unexpected results.

### Pitfall 4: --dry-run With --stdin Batch
**What goes wrong:** Unclear behavior when dry-running a batch of items from stdin.
**Why it happens:** No defined behavior for showing multiple payloads.
**How to avoid:** Show each individual payload that would be sent. For large batches, show count + first item as sample.
**Warning signs:** User runs `cat deals.json | pipelite deals create --stdin --dry-run` and gets confusing output.

### Pitfall 5: clap_complete Needs CommandFactory Trait
**What goes wrong:** Cannot get a `Command` from the derive-based `Cli` struct.
**Why it happens:** `clap_complete::generate()` needs `&mut Command`, not the parsed struct.
**How to avoid:** Use `Cli::command()` from the `CommandFactory` trait (auto-derived by clap's Parser derive).
**Warning signs:** "no method named command" if CommandFactory is not in scope.

### Pitfall 6: Prompt Order Affects UX
**What goes wrong:** Required fields prompted after optional fields, confusing users.
**Why it happens:** Fields prompted in struct definition order, not importance order.
**How to avoid:** Explicitly order prompts: required fields first, then optional fields.
**Warning signs:** User has to fill 5 optional fields before reaching required ones.

## Code Examples

### Dry-Run Output Rendering
```rust
// src/dry_run.rs or within prompt.rs
use crate::output::OutputFormat;
use colored::Colorize;

pub fn render_dry_run(
    method: &str,
    url: &str,
    body: &serde_json::Value,
    format: &OutputFormat,
    color: bool,
) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "dry_run": true,
                "method": method,
                "url": url,
                "body": body,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            if color {
                println!("{} {}", method.yellow().bold(), url);
            } else {
                println!("{} {}", method, url);
            }
            println!("{}", serde_json::to_string_pretty(body)?);
        }
    }
    Ok(())
}

pub fn render_dry_run_delete(
    entity: &str,
    id: &str,
    url: &str,
    format: &OutputFormat,
    color: bool,
) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "dry_run": true,
                "method": "DELETE",
                "url": url,
                "entity": entity,
                "id": id,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            if color {
                println!("Would delete {} {}", entity.bold(), id.cyan());
            } else {
                println!("Would delete {} {}", entity, id);
            }
        }
    }
    Ok(())
}
```

### Complete Create Command with Prompts and Dry-Run
```rust
// Pattern for deals/create.rs single_create refactor
async fn single_create(ctx: &AppContext, args: &DealsCreateArgs) -> Result<()> {
    let mut missing = Vec::new();

    // Required fields - prompt or collect missing
    let title = prompt::require_text(
        &args.title, "title", "Deal title", &mut missing, ctx.no_input
    )?;

    // For stage: fetch stages from API if we need to prompt
    let stage_id = if args.stage.is_some() {
        args.stage.clone()
    } else if ctx.no_input || !std::io::stdin().is_terminal() {
        missing.push("--stage".to_string());
        None
    } else {
        // Fetch pipelines first, then stages for selected pipeline
        let pipelines = fetch_pipeline_options(ctx).await?;
        let pl_id = prompt::require_select(
            &None, "pipeline", "Select pipeline", &pipelines, &mut missing, ctx.no_input
        )?;
        if let Some(pl_id) = &pl_id {
            let stages = fetch_stage_options(ctx, pl_id).await?;
            prompt::require_select(
                &None, "stage", "Select stage", &stages, &mut missing, ctx.no_input
            )?
        } else { None }
    };

    prompt::check_missing(&missing, "Usage: pipelite deals create --title <title> --stage <stage_id>")?;

    // Optional fields - prompt with skip option
    let value = prompt::optional_number(&args.value, "value", "Deal value", ctx.no_input)?;
    // ... other optional fields ...

    let data = DealCreate {
        title: title.unwrap(),
        stage_id: stage_id.unwrap(),
        value,
        // ...
    };

    // Dry-run intercept
    if ctx.dry_run {
        return dry_run::render_dry_run(
            "POST",
            &format!("{}/api/v1/deals", ctx.client.base_url()),
            &serde_json::to_value(&data)?,
            &ctx.output_format,
            ctx.color,
        );
    }

    let deal = ctx.client.create_deal(&data).await?;
    // ... render output ...
}
```

### FuzzySelect Display Format
```rust
// Recommended: "Name (id)" format for readability
// e.g., "Sales Pipeline (pl_abc123)"
let display: Vec<String> = items.iter()
    .map(|(id, name)| format!("{} ({})", name, id))
    .collect();

let idx = FuzzySelect::new()
    .with_prompt("Select stage")
    .items(&display)
    .highlight_matches(true)
    .interact()?;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| clap_generate (separate crate) | clap_complete (merged into clap workspace) | clap 4.0 | Use clap_complete, not clap_generate |
| dialoguer 0.10 (no fuzzy-select) | dialoguer 0.12 with fuzzy-select feature | 2023 | Must enable `fuzzy-select` feature flag |
| clap_complete `generate()` at top level | `clap_complete::aot::generate()` in aot module | clap_complete 4.5+ | Import from `aot` submodule |

**Deprecated/outdated:**
- `clap_generate` crate: replaced by `clap_complete`
- `clap_complete::generate()` without `aot::` prefix: still works but `aot::` is the current recommended path

## Open Questions

1. **Exit code 2 for missing input in headless mode**
   - What we know: Current `exit_code()` returns 2 only for clap errors, 1 for everything else
   - What's unclear: Whether to add a new `CliError::MissingInput` variant or reuse `Validation` with special handling
   - Recommendation: Add a `MissingInput` variant with exit code 2, keeping `Validation` at exit code 1

2. **FuzzySelect for entities with many items**
   - What we know: FuzzySelect works well for <100 items, API fetches all with limit 100
   - What's unclear: Performance if an org has 500+ stages or pipelines
   - Recommendation: Cap FuzzySelect at 100 items, show message if more exist suggesting --flag direct use

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | assert_cmd 2.0 + predicates 3.0 (integration), cargo test (unit) |
| Config file | Cargo.toml [dev-dependencies] |
| Quick run command | `cargo test --test cli_skeleton` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INTR-01 | Create with no flags prompts (needs TTY) | manual-only | Manual: run `pipelite deals create` in terminal | N/A |
| INTR-02 | FuzzySelect for known values | manual-only | Manual: run create and verify dropdown appears | N/A |
| INTR-03 | Prompts only on TTY | integration | `cargo test --test headless_test` | Wave 0 |
| HEAD-01 | --no-input prevents prompts | integration | `cargo test --test headless_test::no_input_flag` | Wave 0 |
| HEAD-02 | Headless fails listing all missing fields | integration | `cargo test --test headless_test::missing_fields` | Wave 0 |
| HEAD-03 | All mutations via flags only | integration | `cargo test --test headless_test::flags_only` | Wave 0 |
| SHLL-01 | Bash completions generated | integration | `cargo test --test completions_test::bash` | Wave 0 |
| SHLL-02 | Zsh completions generated | integration | `cargo test --test completions_test::zsh` | Wave 0 |
| SHLL-03 | Fish completions generated | integration | `cargo test --test completions_test::fish` | Wave 0 |
| UX-04 | --dry-run shows request preview | integration | `cargo test --test dry_run_test` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --test cli_skeleton`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before verification

### Wave 0 Gaps
- [ ] `tests/headless_test.rs` -- covers INTR-03, HEAD-01, HEAD-02, HEAD-03
- [ ] `tests/completions_test.rs` -- covers SHLL-01, SHLL-02, SHLL-03
- [ ] `tests/dry_run_test.rs` -- covers UX-04
- [ ] Note: INTR-01 and INTR-02 are manual-only (require real TTY with interactive terminal input)

## Sources

### Primary (HIGH confidence)
- [dialoguer docs.rs](https://docs.rs/dialoguer/latest/dialoguer/struct.FuzzySelect.html) - FuzzySelect API, version 0.12.0, feature flags
- [docs.rs/crate/dialoguer/0.12.0/features](https://docs.rs/crate/dialoguer/0.12.0/features) - Feature flags: fuzzy-select is opt-in
- [clap_complete docs.rs](https://docs.rs/clap_complete/latest/clap_complete/index.html) - generate() API, version 4.6.0
- [clap_complete shells module](https://docs.rs/clap_complete/latest/clap_complete/shells/index.html) - Shell enum (Bash, Zsh, Fish)
- Local codebase analysis - src/commands/init.rs, src/context.rs, src/cli/mod.rs, src/error.rs

### Secondary (MEDIUM confidence)
- [clap completion-derive example](https://github.com/clap-rs/clap/blob/master/clap_complete/examples/completion-derive.rs) - Derive API integration pattern
- [Kevin K's blog on CLI completions](https://kbknapp.dev/shell-completions/) - Best practices for shell completion generation

### Tertiary (LOW confidence)
- None -- all findings verified with official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - dialoguer 0.12 and clap_complete 4.6 verified via cargo search and docs.rs
- Architecture: HIGH - patterns derived from actual codebase analysis (init.rs, create.rs, context.rs)
- Pitfalls: HIGH - FuzzySelect feature flag and non-TTY panic verified in official docs

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable libraries, no breaking changes expected)
