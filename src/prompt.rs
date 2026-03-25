use std::io::IsTerminal;

use anyhow::Result;
use dialoguer::{FuzzySelect, Input};

use crate::error::CliError;

/// Prompt for a required text field interactively, or collect it from flag value.
///
/// If `flag_value` is Some, returns it immediately. Otherwise, if stdin is a TTY
/// and `no_input` is false, prompts the user interactively. If neither, pushes
/// `--{field_name}` to the missing list for batch error reporting.
pub fn require_text(
    flag_value: &Option<String>,
    field_name: &str,
    prompt_label: &str,
    missing: &mut Vec<String>,
    no_input: bool,
) -> Result<Option<String>> {
    if let Some(val) = flag_value {
        return Ok(Some(val.clone()));
    }

    if std::io::stdin().is_terminal() && !no_input {
        let input: String = Input::new()
            .with_prompt(prompt_label)
            .interact_text()?;
        if input.is_empty() {
            missing.push(format!("--{}", field_name));
            return Ok(None);
        }
        return Ok(Some(input));
    }

    missing.push(format!("--{}", field_name));
    Ok(None)
}

/// Prompt for a required selection from a list of options via FuzzySelect.
///
/// `options` is a slice of `(id, display_name)` tuples. The display format is
/// "name (id)". Returns the selected ID.
pub fn require_select(
    flag_value: &Option<String>,
    field_name: &str,
    prompt_label: &str,
    options: &[(String, String)],
    missing: &mut Vec<String>,
    no_input: bool,
) -> Result<Option<String>> {
    if let Some(val) = flag_value {
        return Ok(Some(val.clone()));
    }

    if std::io::stdin().is_terminal() && !no_input {
        if options.is_empty() {
            return Err(CliError::Validation {
                detail: format!("No options available for {}", field_name),
                hint: format!("Create a {} first, then retry.", field_name),
            }
            .into());
        }

        let display: Vec<String> = options
            .iter()
            .map(|(id, name)| format!("{} ({})", name, id))
            .collect();

        let selection = FuzzySelect::new()
            .with_prompt(prompt_label)
            .items(&display)
            .default(0)
            .interact()?;

        return Ok(Some(options[selection].0.clone()));
    }

    missing.push(format!("--{}", field_name));
    Ok(None)
}

/// Prompt for an optional text field interactively.
///
/// Returns flag value if present, prompts if TTY and not no_input,
/// or returns None in headless mode.
pub fn optional_text(
    flag_value: &Option<String>,
    prompt_label: &str,
    no_input: bool,
) -> Result<Option<String>> {
    if let Some(val) = flag_value {
        return Ok(Some(val.clone()));
    }

    if std::io::stdin().is_terminal() && !no_input {
        let input: String = Input::new()
            .with_prompt(format!("{} (optional, press Enter to skip)", prompt_label))
            .allow_empty(true)
            .interact_text()?;
        if input.is_empty() {
            return Ok(None);
        }
        return Ok(Some(input));
    }

    Ok(None)
}

/// Prompt for an optional numeric field interactively.
///
/// Same pattern as optional_text but parses the input as f64.
pub fn optional_number(
    flag_value: &Option<f64>,
    prompt_label: &str,
    no_input: bool,
) -> Result<Option<f64>> {
    if let Some(val) = flag_value {
        return Ok(Some(*val));
    }

    if std::io::stdin().is_terminal() && !no_input {
        let input: String = Input::new()
            .with_prompt(format!("{} (optional, press Enter to skip)", prompt_label))
            .allow_empty(true)
            .interact_text()?;
        if input.is_empty() {
            return Ok(None);
        }
        let num: f64 = input.parse().map_err(|_| CliError::Validation {
            detail: format!("Invalid number: '{}'", input),
            hint: "Enter a valid number (e.g., 50000.00).".to_string(),
        })?;
        return Ok(Some(num));
    }

    Ok(None)
}

/// Check if any required fields are missing and return a batch error.
///
/// If `missing` is non-empty, returns a `CliError::MissingInput` listing all
/// missing flags at once, so the user can fix them in a single retry.
pub fn check_missing(missing: &[String], usage_hint: &str) -> Result<(), CliError> {
    if missing.is_empty() {
        return Ok(());
    }

    Err(CliError::MissingInput {
        detail: format!("Missing required flags: {}", missing.join(", ")),
        hint: usage_hint.to_string(),
    })
}
