use std::io::IsTerminal;

use anyhow::Result;
use clap::Parser;

mod api;
mod batch;
mod cache;
mod cli;
mod commands;
mod config;
mod context;
mod dry_run;
mod error;
mod output;
mod prompt;
mod splash;

use cli::{Cli, Commands};
use context::AppContext;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Check for splash screen before clap parsing: bare `pipelite` or `pipelite --no-color`
    let args: Vec<String> = std::env::args().collect();
    let is_tty = std::io::stdout().is_terminal();

    if is_tty {
        let show_splash = match args.len() {
            1 => true,
            2 if args[1] == "--no-color" => true,
            _ => false,
        };

        if show_splash {
            let no_color_env = std::env::var("NO_COLOR").is_ok();
            let no_color_flag = args.len() == 2 && args[1] == "--no-color";
            let color = !no_color_env && !no_color_flag;

            let configured = config::load_config(None)
                .ok()
                .map(|c| !c.server.api_key.is_empty())
                .unwrap_or(false);

            splash::print_splash(configured, color);
            return;
        }
    }

    let cli = Cli::parse();

    if let Err(err) = run(cli).await {
        let color = std::io::stderr().is_terminal();
        error::display_error(&err, color);
        std::process::exit(error::exit_code(&err));
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init(ref args) => commands::init::run(args, cli.quiet).await,
        Commands::Ping => {
            let ctx = AppContext::build(&cli)?;
            commands::ping::run(&ctx).await
        }
        Commands::Config(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::config::run(&ctx, cmd)
        }
        Commands::Deals(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::deals::run(&ctx, cmd).await
        }
        Commands::Orgs(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::orgs::run(&ctx, cmd).await
        }
        Commands::People(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::people::run(&ctx, cmd).await
        }
        Commands::Activities(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::activities::run(&ctx, cmd).await
        }
        Commands::Pipelines(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::pipelines::run(&ctx, cmd).await
        }
        Commands::Stages(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::stages::run(&ctx, cmd).await
        }
        Commands::Workflows(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::workflows::run(&ctx, cmd).await
        }
        Commands::Templates(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::templates::run(&ctx, cmd).await
        }
        Commands::Notes(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::notes::run(&ctx, cmd).await
        }
        Commands::Webhooks(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::webhooks::run(&ctx, cmd).await
        }
        Commands::Trash(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::trash::run(&ctx, cmd).await
        }
        Commands::Audit(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::audit::run(&ctx, cmd).await
        }
        Commands::CustomFields(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::custom_fields::run(&ctx, cmd).await
        }
        Commands::Docs(ref args) => {
            let ctx = AppContext::build(&cli)?;
            commands::docs::run(&ctx, args).await
        }
        Commands::Cache(ref cmd) => {
            let ctx = AppContext::build(&cli)?;
            commands::cache::run(&ctx, cmd).await
        }
        Commands::Completions(ref args) => commands::completions::run(args),
        Commands::Dashboard(ref args) => {
            let ctx = AppContext::build(&cli)?;
            commands::dashboard::run(&ctx, args).await
        }
    }
}
