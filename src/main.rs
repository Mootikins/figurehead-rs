//! Figurehead - Convert Mermaid.js diagrams to ASCII art
//!
//! This is the main entry point for the figurehead CLI application.

use clap::Parser as ClapParser;

#[derive(ClapParser)]
#[command(name = "figurehead")]
#[command(about = "Convert Mermaid.js diagrams to ASCII art")]
#[command(version)]
struct Cli {
    /// Enable colors in output
    #[arg(long = "colors")]
    colors: bool,

    /// Input file containing Mermaid diagram
    #[arg(short, long)]
    input: Option<String>,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // TODO: Implement the full processing pipeline
    println!("Figurehead - Convert Mermaid.js diagrams to ASCII art");
    println!("Colors: {}", cli.colors);

    if let Some(input) = &cli.input {
        println!("Input file: {}", input);
    }

    if let Some(output) = &cli.output {
        println!("Output file: {}", output);
    }

    Ok(())
}