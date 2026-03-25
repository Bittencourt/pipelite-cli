use clap::Args;
use clap_complete::aot::Shell;

/// Generate shell completions for bash, zsh, or fish.
#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    pub shell: Shell,
}
