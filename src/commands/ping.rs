use std::io::IsTerminal;
use std::time::Instant;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

use crate::context::AppContext;

/// Run the `pipelite ping` command.
///
/// Tests server connectivity, showing a spinner during the request and
/// reporting server URL, status, and latency on success.
pub async fn run(ctx: &AppContext) -> Result<()> {
    // Show spinner on stderr if interactive and not quiet
    let spinner = if !ctx.quiet && std::io::stderr().is_terminal() {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .expect("valid spinner template"),
        );
        pb.set_message("Connecting...");
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        Some(pb)
    } else {
        None
    };

    let start = Instant::now();
    let result = ctx.client.ping().await;
    let latency = start.elapsed();

    // Clear spinner before printing results
    if let Some(ref pb) = spinner {
        pb.finish_and_clear();
    }

    let status = result?;

    if !ctx.quiet {
        eprintln!("Server:  {}", ctx.client.base_url());
        eprintln!("Status:  {}", status);
        eprintln!("Latency: {}ms", latency.as_millis());
    }

    Ok(())
}
