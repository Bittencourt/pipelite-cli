use clap::ValueEnum;
use std::io::IsTerminal;

/// Output format for command results.
#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
    Plain,
}

/// Determine the output format based on explicit user choice or TTY detection.
///
/// If the user explicitly provided a format, use it. Otherwise, default to
/// Table when stdout is a TTY, or Json when piped.
#[allow(dead_code)]
pub fn detect_format(explicit: Option<OutputFormat>) -> OutputFormat {
    match explicit {
        Some(fmt) => fmt,
        None => {
            if std::io::stdout().is_terminal() {
                OutputFormat::Table
            } else {
                OutputFormat::Json
            }
        }
    }
}
