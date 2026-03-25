use clap::Args;

/// Arguments for the dashboard command.
///
/// The dashboard uses only global flags (--format, --no-color, etc.).
/// No entity-specific arguments are needed.
#[derive(Debug, Args)]
pub struct DashboardArgs {}
