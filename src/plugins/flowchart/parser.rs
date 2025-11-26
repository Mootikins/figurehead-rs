//! Flowchart parser implementation
//!
//! Parses flowchart diagram markup into structured data.

use anyhow::Result;

use crate::core::{Parser, Database};
use super::FlowchartDatabase;

/// Flowchart parser implementation
pub struct FlowchartParser;

impl FlowchartParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser<FlowchartDatabase> for FlowchartParser {
    fn parse(&self, input: &str, database: &mut FlowchartDatabase) -> Result<()> {
        // TODO: Implement proper chumsky parser
        // For now, just a simple implementation to make tests compile

        let lines: Vec<&str> = input.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("%%"))
            .collect();

        for line in lines {
            // Simple edge parsing: "A --> B"
            if let Some((left, right)) = parse_simple_edge(line) {
                database.add_node(&left, &left)?;
                database.add_node(&right, &right)?;
                database.add_edge(&left, &right)?;
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "flowchart"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn can_parse(&self, input: &str) -> bool {
        input.contains("-->") || input.contains("---")
    }
}

/// Simple edge parser for basic syntax
fn parse_simple_edge(line: &str) -> Option<(String, String)> {
    if let Some(start) = line.find("-->") {
        let left = line[..start].trim().to_string();
        let right = line[start + 3..].trim().to_string();
        Some((left, right))
    } else if let Some(start) = line.find("---") {
        let left = line[..start].trim().to_string();
        let right = line[start + 3..].trim().to_string();
        Some((left, right))
    } else {
        None
    }
}