//! Figurehead CLI - Convert Mermaid.js diagrams to ASCII art

mod cli;

use clap::Parser;
use figurehead::core::logging::init_logging;

fn main() {
    // Parse CLI args first to get logging configuration
    let cli_args = cli::Cli::parse();

    // Initialize logging based on CLI flags or environment variables
    // Note: CLI flags will be handled in the app.run() method, but we need
    // to initialize here for early logging. The app will reinitialize if needed.
    if let Err(e) = init_logging(None, None) {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }

    let mut app = cli::FigureheadApp::new();

    if let Err(e) = app.run(cli_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
