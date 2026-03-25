pub mod csv;
pub mod fields;
pub mod format;
pub mod json;
pub mod plain;
pub mod table;

use anyhow::Result;
use clap::ValueEnum;
use std::io::IsTerminal;

use crate::api::models::PaginationMeta;

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

/// Render a list of items in the specified output format.
///
/// Dispatches to the appropriate submodule. For JSON, uses field filtering.
/// For table/csv/plain, uses columns (overridden by fields if provided).
#[allow(dead_code)]
pub fn render_list(
    items: &[serde_json::Value],
    format: &OutputFormat,
    columns: &[String],
    fields: &Option<Vec<String>>,
    color: bool,
    meta: Option<&PaginationMeta>,
) -> Result<()> {
    let effective_columns = fields.as_deref().unwrap_or(columns);

    match format {
        OutputFormat::Json => json::render_list(items, fields),
        OutputFormat::Table => {
            table::render_list(items, &effective_columns.to_vec(), color, meta)
        }
        OutputFormat::Csv => csv::render_list(items, &effective_columns.to_vec()),
        OutputFormat::Plain => plain::render_list(items, &effective_columns.to_vec()),
    }
}

/// Render a single item in the specified output format.
#[allow(dead_code)]
pub fn render_single(
    item: &serde_json::Value,
    format: &OutputFormat,
    columns: &[String],
    fields: &Option<Vec<String>>,
    color: bool,
) -> Result<()> {
    let effective_columns = fields.as_deref().unwrap_or(columns);

    match format {
        OutputFormat::Json => json::render_single(item, fields),
        OutputFormat::Table => {
            table::render_single(item, &effective_columns.to_vec(), color)
        }
        OutputFormat::Csv => csv::render_list(&[item.clone()], &effective_columns.to_vec()),
        OutputFormat::Plain => plain::render_list(&[item.clone()], &effective_columns.to_vec()),
    }
}
