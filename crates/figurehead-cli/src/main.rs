//! Figurehead CLI - Convert Mermaid.js diagrams to ASCII art

mod cli;

use clap::Parser;

fn main() {
    let cli_args = cli::Cli::parse();
    let mut app = cli::FigureheadApp::new();

    if let Err(e) = app.run(cli_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
