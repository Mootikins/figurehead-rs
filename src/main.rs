//! Figurehead - Convert Mermaid.js diagrams to ASCII art
//!
//! This is the main binary entry point for the figurehead CLI utility.

use clap::Parser;
use figurehead::cli::{Cli, FigureheadApp};
use std::process;

fn main() {
    let cli = Cli::parse();

    let app = FigureheadApp::new();

    if let Err(e) = app.run(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
