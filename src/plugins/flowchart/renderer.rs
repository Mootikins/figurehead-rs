//! Flowchart renderer implementation
//!
//! Renders flowchart diagram data into ASCII art.

use anyhow::Result;

use crate::core::{Renderer, Database};
use super::FlowchartDatabase;

/// Flowchart renderer implementation
pub struct FlowchartRenderer;

impl FlowchartRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer<FlowchartDatabase> for FlowchartRenderer {
    type Output = String;

    fn render(&self, database: &FlowchartDatabase) -> Result<Self::Output> {
        // TODO: Implement proper ASCII rendering
        // For now, just a simple implementation to make tests compile

        let mut output = String::new();
        output.push_str("Flowchart Diagram\n");
        output.push_str("==================\n\n");

        // Render nodes
        for (id, label) in database.get_nodes() {
            output.push_str(&format!("Node {}: {}\n", id, label));
        }

        output.push_str("\n");

        // Render edges
        for (from, to) in database.get_edges() {
            output.push_str(&format!("Edge: {} --> {}\n", from, to));
        }

        Ok(output)
    }

    fn name(&self) -> &'static str {
        "ascii"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn format(&self) -> &'static str {
        "ascii"
    }
}