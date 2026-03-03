mod cli;
mod format;

use clap::Parser;
use cli::Cli;

fn main() {
    let args = Cli::parse();

    if args.json || args.dry_run {
        eprintln!("Non-interactive mode not yet implemented");
        std::process::exit(1);
    } else {
        eprintln!("TUI mode not yet implemented");
        std::process::exit(1);
    }
}
