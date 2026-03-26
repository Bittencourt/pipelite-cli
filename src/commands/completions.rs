use clap::CommandFactory;
use clap_complete::aot::generate;

use crate::cli::Cli;
use crate::cli::completions::CompletionsArgs;

/// Generate shell completions and print to stdout.
///
/// Install instructions are printed to stderr so they don't
/// contaminate the completion script output.
pub fn run(args: &CompletionsArgs) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    generate(args.shell, &mut cmd, "pipelite", &mut std::io::stdout());

    let instruction = match args.shell {
        clap_complete::aot::Shell::Bash => {
            "Add to ~/.bashrc:\n  source <(pipelite completions bash)"
        }
        clap_complete::aot::Shell::Zsh => {
            "Add to ~/.zshrc:\n  source <(pipelite completions zsh)"
        }
        clap_complete::aot::Shell::Fish => {
            "Run once:\n  pipelite completions fish > ~/.config/fish/completions/pipelite.fish"
        }
        _ => "See your shell's documentation for completion installation.",
    };
    eprintln!(
        "\n# Install instructions:\n# {}",
        instruction.replace('\n', "\n# ")
    );
    eprintln!("#");
    eprintln!("# For dynamic entity ID completions, run:");
    eprintln!("#   pipelite cache refresh");
    Ok(())
}
