mod cli;
mod format;
mod non_interactive;
mod tui;

use clap::Parser;
use cli::Cli;

fn main() {
    let args = Cli::parse();
    let config = args.to_scan_config();

    let result = if args.json {
        non_interactive::run_json(&config)
    } else if args.dry_run {
        non_interactive::run_dry_run(&config)
    } else {
        tui::run(config)
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
