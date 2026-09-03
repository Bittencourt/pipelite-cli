use std::io::IsTerminal;

use anyhow::Result;

use crate::error::CliError;

/// Resolve the note body from the locked source precedence.
///
/// Order of operations:
/// 1. Count explicit sources FIRST — `--body` and `--stdin` each count; two
///    or more is InvalidInput (exit 2) BEFORE any read, so the stdin-backed
///    pair `--body @-` + `--stdin` can never consume each other's data
///    (Pitfall 7 / open question 3).
/// 2. `--body`: a leading `@` reads from a file (`@-` = stdin to end);
///    an unreadable file is InvalidInput exit 2 with a check-the-path hint
///    (InvalidInput per the Phase 10 CONTEXT lock — NOT Validation).
/// 3. `--stdin`: read stdin to end.
/// 4. Neither: interactive prompt when stdin is a TTY and prompting is
///    allowed, else MissingInput exit 2. `--dry-run` callers pass
///    `no_input || dry_run` so a dry-run with no explicit source previews
///    the MissingInput rejection instead of prompting.
///
/// No trimming, no length check — the server owns content validation
/// (whitespace-only bodies 422 server-side and flow through the Phase 8
/// error layer untouched).
///
/// `prompt_context` is `(action, label)`: the action prefixes the dialoguer
/// prompt; the label carries the question (for edit, the locked
/// no-single-GET wording).
pub fn resolve_body(
    body: &Option<String>,
    stdin: bool,
    no_input: bool,
    prompt_context: Option<(&str, &str)>,
) -> Result<String> {
    // 1. XOR source check precedes EVERY read.
    let explicit_sources = usize::from(body.is_some()) + usize::from(stdin);
    if explicit_sources > 1 {
        return Err(CliError::InvalidInput {
            detail: "Multiple note body sources given (--body and --stdin)".to_string(),
            hint: "Pass exactly one: --body <text|@file>, --body @- (reads stdin), or --stdin."
                .to_string(),
        }
        .into());
    }

    // 2. --body flag: literal text, @file, or @-.
    if let Some(value) = body {
        return match value.strip_prefix('@') {
            Some("-") => read_stdin_to_end(),
            Some(path) => std::fs::read_to_string(path).map_err(|e| {
                CliError::InvalidInput {
                    detail: format!("Failed to read file '{path}': {e}"),
                    hint: "Check that the file path is correct and the file exists.".to_string(),
                }
                .into()
            }),
            None => Ok(value.clone()),
        };
    }

    // 3. --stdin flag.
    if stdin {
        return read_stdin_to_end();
    }

    // 4. Prompt fallback (TTY only) or MissingInput.
    if !no_input && std::io::stdin().is_terminal() {
        if let Some((action, label)) = prompt_context {
            let input: String = dialoguer::Input::new()
                .with_prompt(format!("{action}: {label}"))
                .interact_text()?;
            return Ok(input);
        }
    }

    Err(CliError::MissingInput {
        detail: "No note body provided".to_string(),
        hint: "Provide --body <text>, --body @file, --body @-, or --stdin (or run interactively without --no-input)."
            .to_string(),
    }
    .into())
}

/// Read stdin to end, mapping read/UTF-8 failures to a hinted error.
fn read_stdin_to_end() -> Result<String> {
    std::io::read_to_string(std::io::stdin()).map_err(|e| {
        CliError::InvalidInput {
            detail: format!("Failed to read stdin: {e}"),
            hint: "Check that stdin is providing readable UTF-8 text.".to_string(),
        }
        .into()
    })
}
