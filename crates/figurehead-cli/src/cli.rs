//! Command-line interface for the figurehead utility
//!
//! Provides a CLI to convert Mermaid.js diagram markup into ASCII diagrams.

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use figurehead::plugins::Orchestrator;

/// Figurehead - Convert Mermaid.js diagrams to ASCII art
#[derive(Parser)]
#[command(name = "figurehead")]
#[command(about = "A Rust utility to convert Mermaid.js diagrams to ASCII diagrams")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert a Mermaid.js diagram to ASCII
    Convert {
        /// Input file containing Mermaid.js diagram (use - for stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output file for ASCII diagram (use - for stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Skip diagram type detection (treat as flowchart)
        #[arg(long)]
        skip_detection: bool,
    },

    /// Detect diagram type in input
    Detect {
        /// Input file to analyze (use - for stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,
    },

    /// Show supported diagram types
    Types {
        /// Show in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Validate Mermaid.js syntax
    Validate {
        /// Input file to validate (use - for stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,
    },
}

/// Main CLI application
pub struct FigureheadApp {
    orchestrator: Orchestrator,
}

impl FigureheadApp {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            orchestrator: Orchestrator::with_flowchart_plugins(),
        }
    }

    /// Run the application with the given CLI arguments
    pub fn run(&self, cli: Cli) -> Result<()> {
        if cli.verbose {
            eprintln!("Figurehead v{}", env!("CARGO_PKG_VERSION"));
        }

        match cli.command {
            Commands::Convert {
                input,
                output,
                skip_detection,
            } => self.convert_command(input, output, skip_detection, cli.verbose),
            Commands::Detect { input } => self.detect_command(input, cli.verbose),
            Commands::Types { json } => self.types_command(json, cli.verbose),
            Commands::Validate { input } => self.validate_command(input, cli.verbose),
        }
    }

    /// Handle the convert command
    fn convert_command(
        &self,
        input: Option<PathBuf>,
        output: Option<PathBuf>,
        skip_detection: bool,
        verbose: bool,
    ) -> Result<()> {
        // Read input
        let content = self.read_input(input)?;

        if verbose {
            eprintln!("Read {} bytes of input", content.len());
        }

        // Process the diagram
        let result = if skip_detection {
            self.orchestrator.process_flowchart(&content)
        } else {
            self.orchestrator.process(&content)
        };

        match result {
            Ok(ascii_output) => {
                if verbose {
                    eprintln!("Successfully converted diagram to ASCII");
                }
                self.write_output(output, &ascii_output)?;
                Ok(())
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(e)
            }
        }
    }

    /// Handle the detect command
    fn detect_command(&self, input: Option<PathBuf>, verbose: bool) -> Result<()> {
        let content = self.read_input(input)?;

        if verbose {
            eprintln!("Read {} bytes of input", content.len());
        }

        match self.orchestrator.detect_diagram_type(&content) {
            Ok(diagram_type) => {
                println!("{}", diagram_type);
                Ok(())
            }
            Err(e) => {
                eprintln!("Could not detect diagram type: {}", e);
                Err(e)
            }
        }
    }

    /// Handle the types command
    fn types_command(&self, json: bool, verbose: bool) -> Result<()> {
        if verbose {
            eprintln!("Listing supported diagram types");
        }

        if json {
            // JSON output
            let types = serde_json::json!({
                "supported_types": [
                    {
                        "name": "flowchart",
                        "description": "Flowchart diagrams with nodes and edges",
                        "status": "supported"
                    }
                ],
                "total": 1
            });
            println!("{}", serde_json::to_string_pretty(&types)?);
        } else {
            // Human-readable output
            println!("Supported diagram types:");
            println!("  flowchart  - Flowchart diagrams with nodes and edges");
            println!();
            println!("Total: 1 diagram type supported");
        }

        Ok(())
    }

    /// Handle the validate command
    fn validate_command(&self, input: Option<PathBuf>, verbose: bool) -> Result<()> {
        let content = self.read_input(input)?;

        if verbose {
            eprintln!("Read {} bytes of input", content.len());
        }

        // Try to detect diagram type first
        let detection_result = self.orchestrator.detect_diagram_type(&content);

        match detection_result {
            Ok(diagram_type) => {
                if verbose {
                    eprintln!("Detected diagram type: {}", diagram_type);
                }

                // Try to process it
                match self.orchestrator.process(&content) {
                    Ok(_) => {
                        println!("✓ Valid {} diagram", diagram_type);
                        Ok(())
                    }
                    Err(e) => {
                        println!("✗ Invalid {} diagram: {}", diagram_type, e);
                        Err(e)
                    }
                }
            }
            Err(_) => {
                println!("✗ Could not detect diagram type");
                Err(anyhow!("Unknown diagram type"))
            }
        }
    }

    /// Read input from file or stdin
    pub fn read_input(&self, input: Option<PathBuf>) -> Result<String> {
        match input {
            Some(path) => {
                if path.to_string_lossy() == "-" {
                    // Read from stdin
                    let mut content = String::new();
                    io::stdin().read_to_string(&mut content)?;
                    Ok(content)
                } else {
                    // Read from file
                    fs::read_to_string(&path).map_err(|e| {
                        anyhow!("Failed to read input file '{}': {}", path.display(), e)
                    })
                }
            }
            None => {
                // No input file specified, read from stdin
                let mut content = String::new();
                io::stdin().read_to_string(&mut content)?;
                Ok(content)
            }
        }
    }

    /// Write output to file or stdout
    pub fn write_output(&self, output: Option<PathBuf>, content: &str) -> Result<()> {
        match output {
            Some(path) => {
                if path.to_string_lossy() == "-" {
                    // Write to stdout
                    print!("{}", content);
                    io::stdout().flush()?;
                } else {
                    // Write to file
                    fs::write(&path, content).map_err(|e| {
                        anyhow!("Failed to write output file '{}': {}", path.display(), e)
                    })?;
                }
            }
            None => {
                // No output file specified, write to stdout
                print!("{}", content);
                io::stdout().flush()?;
            }
        }
        Ok(())
    }

    /// Get a reference to the orchestrator (for testing)
    #[cfg(test)]
    pub fn orchestrator(&self) -> &Orchestrator {
        &self.orchestrator
    }

    /// Get a mutable reference to the orchestrator (for testing)
    #[cfg(test)]
    pub fn orchestrator_mut(&mut self) -> &mut Orchestrator {
        &mut self.orchestrator
    }
}

impl Default for FigureheadApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use figurehead::plugins::flowchart::FlowchartDetector;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_cli_parsing_convert_command() {
        let args = vec![
            "figurehead",
            "convert",
            "--input",
            "test.mmd",
            "--output",
            "output.txt",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Convert {
                input,
                output,
                skip_detection,
            } => {
                assert_eq!(input.unwrap().to_string_lossy(), "test.mmd");
                assert_eq!(output.unwrap().to_string_lossy(), "output.txt");
                assert!(!skip_detection);
            }
            _ => panic!("Expected Convert command"),
        }
    }

    #[test]
    fn test_cli_parsing_detect_command() {
        let args = vec!["figurehead", "detect", "--input", "test.mmd"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Detect { input } => {
                assert_eq!(input.unwrap().to_string_lossy(), "test.mmd");
            }
            _ => panic!("Expected Detect command"),
        }
    }

    #[test]
    fn test_cli_parsing_types_command() {
        let args = vec!["figurehead", "types", "--json"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Types { json } => {
                assert!(json);
            }
            _ => panic!("Expected Types command"),
        }
    }

    #[test]
    fn test_cli_parsing_validate_command() {
        let args = vec!["figurehead", "validate"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Validate { input } => {
                assert!(input.is_none());
            }
            _ => panic!("Expected Validate command"),
        }
    }

    #[test]
    fn test_figurehead_app_creation() {
        let _app = FigureheadApp::new();
        assert!(true);
    }

    #[test]
    fn test_figurehead_app_default() {
        let _app = FigureheadApp::default();
        assert!(true);
    }

    #[test]
    fn test_read_input_from_string() {
        let app = FigureheadApp::new();
        let input = "graph TD; A-->B;";

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.mmd");
        fs::write(&file_path, input).unwrap();

        let content = app.read_input(Some(file_path)).unwrap();
        assert_eq!(content, input);
    }

    #[test]
    fn test_write_output_to_string() {
        let app = FigureheadApp::new();
        let output = "Test output";

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("output.txt");

        app.write_output(Some(file_path.clone()), output).unwrap();

        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, output);
    }

    #[test]
    fn test_detect_command_with_flowchart() {
        let mut app = FigureheadApp::new();
        app.orchestrator_mut().register_detector(
            "flowchart".to_string(),
            Box::new(FlowchartDetector::new()),
        );

        let input = "graph TD; A-->B;";
        let result = app.orchestrator().detect_diagram_type(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "flowchart");
    }

    #[test]
    fn test_detect_command_with_non_flowchart() {
        let app = FigureheadApp::new();
        let input = "This is not a diagram";

        let result = app.orchestrator().detect_diagram_type(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_command_simple_flowchart() {
        let app = FigureheadApp::new();
        let input = "graph TD; A-->B;";

        let result = app.orchestrator().process_flowchart(input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_validate_command_valid_flowchart() {
        let mut app = FigureheadApp::new();
        app.orchestrator_mut().register_detector(
            "flowchart".to_string(),
            Box::new(FlowchartDetector::new()),
        );

        let input = "graph TD; A-->B;";

        let detection_result = app.orchestrator().detect_diagram_type(input);
        assert!(detection_result.is_ok());

        let process_result = app.orchestrator().process(input);
        assert!(process_result.is_ok());
    }

    #[test]
    fn test_validate_command_invalid_diagram() {
        let app = FigureheadApp::new();
        let input = "This is not a diagram";

        let detection_result = app.orchestrator().detect_diagram_type(input);
        assert!(detection_result.is_err());
    }

    #[test]
    fn test_types_command_json_format() {
        let app = FigureheadApp::new();
        let result = app.types_command(true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_types_command_human_format() {
        let app = FigureheadApp::new();
        let result = app.types_command(false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_skip_detection_flag() {
        let args = vec!["figurehead", "convert", "--skip-detection"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Convert { skip_detection, .. } => {
                assert!(skip_detection);
            }
            _ => panic!("Expected Convert command"),
        }
    }

    #[test]
    fn test_verbose_flag() {
        let args = vec!["figurehead", "--verbose", "convert"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
    }
}
