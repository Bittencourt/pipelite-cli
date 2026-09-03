use colored::Colorize;

/// Structured CLI error types with detail and hint fields.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Authentication failed")]
    Auth { detail: String, hint: String },

    #[error("Connection failed")]
    Connection { detail: String, hint: String },

    #[allow(dead_code)]
    #[error("Configuration error")]
    Config { detail: String, hint: String },

    #[error("Not found")]
    NotFound { detail: String, hint: String },

    #[error("Validation error")]
    Validation { detail: String, hint: String },

    #[error("API error")]
    Api {
        status: u16,
        detail: String,
        hint: String,
    },

    #[error("Missing input")]
    MissingInput { detail: String, hint: String },

    /// Structurally invalid user input (invalid JSON, empty list, mutually
    /// exclusive flags). Maps to exit code 2 so scripts can distinguish
    /// "whole input broken, nothing ran" from per-item failures (exit 1).
    #[error("Invalid input")]
    InvalidInput { detail: String, hint: String },
}

/// Display a structured error message on stderr.
///
/// For `CliError` variants, shows: type + detail + hint.
/// For generic anyhow errors, shows the full error chain.
pub fn display_error(err: &anyhow::Error, color: bool) {
    if let Some(cli_err) = err.downcast_ref::<CliError>() {
        let (title, detail, hint) = match cli_err {
            CliError::Auth { detail, hint } => (cli_err.to_string(), detail, hint),
            CliError::Connection { detail, hint } => (cli_err.to_string(), detail, hint),
            CliError::Config { detail, hint } => (cli_err.to_string(), detail, hint),
            CliError::NotFound { detail, hint } => (cli_err.to_string(), detail, hint),
            CliError::Validation { detail, hint } => (cli_err.to_string(), detail, hint),
            CliError::Api {
                status: _,
                detail,
                hint,
            } => (cli_err.to_string(), detail, hint),
            CliError::MissingInput { detail, hint } => (cli_err.to_string(), detail, hint),
            CliError::InvalidInput { detail, hint } => (cli_err.to_string(), detail, hint),
        };
        eprintln!("{}", format_error(&title, detail, hint, color));
    } else {
        if color {
            eprintln!("{}: {err:#}", "error".red().bold());
        } else {
            eprintln!("error: {err:#}");
        }
    }
}

/// Determine the exit code for an error.
///
/// Returns 2 for clap usage errors (misuse) and structural input errors
/// (MissingInput, InvalidInput), 1 for everything else — so scripts can
/// tell "whole input was broken, nothing ran" (2) apart from "some items
/// failed" (1).
pub fn exit_code(err: &anyhow::Error) -> i32 {
    if err.downcast_ref::<clap::Error>().is_some() {
        2
    } else if err.downcast_ref::<CliError>().is_some_and(|e| {
        matches!(
            e,
            CliError::MissingInput { .. } | CliError::InvalidInput { .. }
        )
    }) {
        2
    } else {
        1
    }
}

fn format_error(title: &str, detail: &str, hint: &str, color: bool) -> String {
    if color {
        format!(
            "{}: {}\n  {}\n  {}: {}",
            "error".red().bold(),
            title,
            detail,
            "hint".dimmed(),
            hint
        )
    } else {
        format!("error: {title}\n  {detail}\n  hint: {hint}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_error_formats_without_color() {
        let err = CliError::Auth {
            detail: "invalid API key".to_string(),
            hint: "run `pipelite init` to reconfigure".to_string(),
        };
        let anyhow_err: anyhow::Error = err.into();

        // Capture via format_error directly
        let cli_err = anyhow_err.downcast_ref::<CliError>().unwrap();
        match cli_err {
            CliError::Auth { detail, hint } => {
                let output = format_error(&cli_err.to_string(), detail, hint, false);
                assert_eq!(
                    output,
                    "error: Authentication failed\n  invalid API key\n  hint: run `pipelite init` to reconfigure"
                );
            }
            _ => panic!("Expected Auth variant"),
        }
    }

    #[test]
    fn config_error_formats_without_color() {
        let err = CliError::Config {
            detail: "missing server.url".to_string(),
            hint: "run `pipelite config set server.url <url>`".to_string(),
        };
        let anyhow_err: anyhow::Error = err.into();
        let cli_err = anyhow_err.downcast_ref::<CliError>().unwrap();
        match cli_err {
            CliError::Config { detail, hint } => {
                let output = format_error(&cli_err.to_string(), detail, hint, false);
                assert!(output.contains("error: Configuration error"));
                assert!(output.contains("missing server.url"));
                assert!(output.contains("hint:"));
            }
            _ => panic!("Expected Config variant"),
        }
    }

    #[test]
    fn connection_error_formats_without_color() {
        let err = CliError::Connection {
            detail: "timeout after 30s".to_string(),
            hint: "check your network connection".to_string(),
        };
        let anyhow_err: anyhow::Error = err.into();
        let cli_err = anyhow_err.downcast_ref::<CliError>().unwrap();
        match cli_err {
            CliError::Connection { detail, hint } => {
                let output = format_error(&cli_err.to_string(), detail, hint, false);
                assert!(output.contains("error: Connection failed"));
                assert!(output.contains("timeout after 30s"));
            }
            _ => panic!("Expected Connection variant"),
        }
    }

    #[test]
    fn not_found_error_formats_without_color() {
        let err = CliError::NotFound {
            detail: "deal 123 does not exist".to_string(),
            hint: "run `pipelite deals list` to see available deals".to_string(),
        };
        let anyhow_err: anyhow::Error = err.into();
        let cli_err = anyhow_err.downcast_ref::<CliError>().unwrap();
        match cli_err {
            CliError::NotFound { detail, hint } => {
                let output = format_error(&cli_err.to_string(), detail, hint, false);
                assert!(output.contains("error: Not found"));
                assert!(output.contains("deal 123 does not exist"));
            }
            _ => panic!("Expected NotFound variant"),
        }
    }

    #[test]
    fn color_true_produces_different_output_than_color_false() {
        // Force colored output even in non-TTY test environment
        colored::control::set_override(true);

        let with_color = format_error("Test error", "detail", "hint text", true);
        let without_color = format_error("Test error", "detail", "hint text", false);

        // Both should contain the content
        assert!(with_color.contains("hint text"));
        assert!(with_color.contains("detail"));
        assert!(without_color.contains("hint text"));

        // Color version should contain ANSI escape codes (different from plain)
        assert_ne!(with_color, without_color);

        // Plain version should start with "error:"
        assert!(without_color.starts_with("error:"));

        // Reset colored override
        colored::control::unset_override();
    }

    #[test]
    fn exit_code_returns_1_for_runtime_errors() {
        let err = anyhow::anyhow!("runtime error");
        assert_eq!(exit_code(&err), 1);
    }

    #[test]
    fn exit_code_returns_2_for_structural_input_errors() {
        let invalid_input = anyhow::Error::new(CliError::InvalidInput {
            detail: "Empty update list".to_string(),
            hint: "Provide at least one object.".to_string(),
        });
        assert_eq!(exit_code(&invalid_input), 2);

        let missing_input = anyhow::Error::new(CliError::MissingInput {
            detail: "missing value".to_string(),
            hint: "provide a value".to_string(),
        });
        assert_eq!(exit_code(&missing_input), 2);

        // Per-item batch failures stay exit 1 (Validation), never 2.
        let validation = anyhow::Error::new(CliError::Validation {
            detail: "1 of 2 update operations failed".to_string(),
            hint: "Review the errors above.".to_string(),
        });
        assert_eq!(exit_code(&validation), 1);
    }
}
