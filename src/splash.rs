use colored::Colorize;

/// ASCII art logo for the splash screen.
const LOGO: &str = r#"
 ____  _            _ _ _
|  _ \(_)_ __   ___| (_) |_ ___
| |_) | | '_ \ / _ \ | | __/ _ \
|  __/| | |_) |  __/ | | ||  __/
|_|   |_| .__/ \___|_|_|\__\___|
        |_|
"#;

/// Print the branded splash screen with ASCII art logo and quick-start hints.
///
/// Shows colored output when `color` is true. Displays different hints
/// depending on whether the CLI is configured (has API key) or not.
pub fn print_splash(configured: bool, color: bool) {
    if color {
        print!("{}", LOGO.cyan().bold());
    } else {
        print!("{}", LOGO);
    }

    if configured {
        println!("  Try:  pipelite deals list");
        println!("        pipelite dashboard");
        println!("        pipelite --help");
    } else {
        println!("  Get started: pipelite init");
    }
    println!();
}
