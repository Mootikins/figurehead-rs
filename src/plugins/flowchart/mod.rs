//! Flowchart diagram plugin
//!
//! This module contains the flowchart diagram implementation for Mermaid.js
//! flowchart syntax support.

use std::sync::Arc;
use crate::core::{Diagram, Detector};

mod detector;
mod parser;
mod database;
mod renderer;
mod layout;

pub use detector::*;
pub use parser::*;
pub use database::*;
pub use renderer::*;
pub use layout::*;

/// Flowchart diagram implementation
pub struct FlowchartDiagram;

impl Diagram for FlowchartDiagram {
    type Database = FlowchartDatabase;
    type Parser = FlowchartParser;
    type Renderer = FlowchartRenderer;

    fn detector() -> Arc<dyn Detector> {
        Arc::new(FlowchartDetector::new())
    }

    fn create_parser() -> Self::Parser {
        FlowchartParser::new()
    }

    fn create_database() -> Self::Database {
        FlowchartDatabase::new()
    }

    fn create_renderer() -> Self::Renderer {
        FlowchartRenderer::new()
    }

    fn name() -> &'static str {
        "flowchart"
    }

    fn version() -> &'static str {
        "0.1.0"
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::core::{Parser, Database, Renderer};

    #[test]
    fn test_full_pipeline() {
        // Integration test for the full flowchart pipeline

        // Test that we can create all components
        let detector = FlowchartDiagram::detector();
        let mut parser = FlowchartDiagram::create_parser();
        let mut database = FlowchartDiagram::create_database();
        let renderer = FlowchartDiagram::create_renderer();

        // Test detection
        let input = "graph TD\n    A --> B\n    B --> C";
        assert!(detector.detect(input));
        assert_eq!(detector.diagram_type(), "flowchart");

        // Test parsing
        parser.parse(input, &mut database).unwrap();
        assert_eq!(database.node_count(), 3);
        assert_eq!(database.edge_count(), 2);

        // Test rendering
        let output = renderer.render(&database).unwrap();
        assert!(output.contains("Flowchart Diagram"));
        assert!(output.len() > 0);
    }

    #[test]
    fn test_mermaid_compatibility() {
        // Test with real Mermaid.js syntax
        let input = r#"graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Process 1]
    B -->|No| D[Process 2]
    C --> E[End]
    D --> E"#;

        let detector = FlowchartDetector::new();
        assert!(detector.detect(input));

        let mut parser = FlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        // Should parse without panicking (even if implementation is basic)
        let result = parser.parse(input, &mut database);
        assert!(result.is_ok());
    }
}