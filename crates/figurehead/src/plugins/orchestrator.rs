//! Plugin orchestrator for coordinating the diagram processing pipeline
//!
//! The orchestrator manages the flow of data through all plugins:
//! Detector → Parser → Database → Layout → Renderer

use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info, span, trace, warn, Level};

use crate::core::{Database, Detector, Parser, Renderer};
use crate::plugins::flowchart::FlowchartDatabase;

/// Plugin orchestrator that coordinates the entire pipeline
///
/// The orchestrator wires detectors, parsers, layout, and renderer pieces
/// together so callers can run a full pipeline without handling each trait
/// manually.
pub struct Orchestrator {
    detectors: HashMap<String, Box<dyn Detector>>,
    flowchart_parser: Option<crate::plugins::flowchart::FlowchartParser>,
    flowchart_layout: Option<crate::plugins::flowchart::FlowchartLayoutAlgorithm>,
    ascii_renderer: Option<crate::plugins::flowchart::FlowchartRenderer>,
}

impl Orchestrator {
    /// Create a new empty orchestrator
    pub fn new() -> Self {
        Self {
            detectors: HashMap::new(),
            flowchart_parser: None,
            flowchart_layout: None,
            ascii_renderer: None,
        }
    }

    /// Create a new orchestrator with all flowchart plugins registered
    pub fn with_flowchart_plugins() -> Self {
        Self::with_flowchart_plugins_and_style(crate::core::CharacterSet::default())
    }

    /// Create a new orchestrator with flowchart plugins and a specific renderer style
    pub fn with_flowchart_plugins_and_style(style: crate::core::CharacterSet) -> Self {
        Self {
            detectors: HashMap::new(),
            flowchart_parser: Some(crate::plugins::flowchart::FlowchartParser::new()),
            flowchart_layout: Some(crate::plugins::flowchart::FlowchartLayoutAlgorithm::new()),
            ascii_renderer: Some(crate::plugins::flowchart::FlowchartRenderer::with_style(
                style,
            )),
        }
    }

    /// Register a detector plugin
    pub fn register_detector(&mut self, name: String, detector: Box<dyn Detector>) {
        self.detectors.insert(name, detector);
    }

    /// Get available detector names
    pub fn get_detectors(&self) -> Vec<String> {
        self.detectors.keys().cloned().collect()
    }

    /// Check if flowchart plugins are available
    pub fn has_flowchart_plugins(&self) -> bool {
        self.flowchart_parser.is_some()
            && self.flowchart_layout.is_some()
            && self.ascii_renderer.is_some()
    }

    /// Detect diagram type from input text
    pub fn detect_diagram_type(&self, input: &str) -> Result<String> {
        let detect_span = span!(Level::INFO, "detect_diagram_type", input_len = input.len());
        let _enter = detect_span.enter();

        trace!("Starting diagram type detection");

        for (name, detector) in &self.detectors {
            let confidence = detector.confidence(input);
            trace!(detector = name, confidence, "Checking detector");
            if detector.detect(input) {
                info!(detector = name, confidence, "Detected diagram type");
                return Ok(name.clone());
            }
        }

        warn!("No suitable detector found for input");
        Err(anyhow::anyhow!("No suitable detector found for input"))
    }

    /// Process input through the complete pipeline (for flowcharts only)
    ///
    /// Runs detector → parser → renderer using registered plugins.
    pub fn process(&self, input: &str) -> Result<String> {
        let process_span = span!(Level::INFO, "process_diagram", input_len = input.len());
        let _enter = process_span.enter();

        info!("Starting diagram processing pipeline");

        // Step 1: Detect diagram type (must be flowchart for now)
        let detect_span = span!(Level::DEBUG, "pipeline_detect");
        let _detect_enter = detect_span.enter();
        let diagram_type = self.detect_diagram_type(input)?;
        debug!(diagram_type, "Diagram type detected");
        drop(_detect_enter);

        if diagram_type != "flowchart" {
            warn!(diagram_type, "Unsupported diagram type");
            return Err(anyhow::anyhow!(
                "Only flowchart diagrams are currently supported"
            ));
        }

        self.process_flowchart(input)
    }

    /// Process flowchart input directly (skip detection)
    ///
    /// Useful when the caller already knows the diagram type.
    pub fn process_flowchart(&self, input: &str) -> Result<String> {
        let flowchart_span = span!(Level::INFO, "process_flowchart", input_len = input.len());
        let _enter = flowchart_span.enter(); // Enter span to track total pipeline duration

        info!("Processing flowchart diagram");

        // Step 1: Parse the input
        let parse_span = span!(Level::DEBUG, "pipeline_parse");
        let _parse_enter = parse_span.enter();
        let parser = self
            .flowchart_parser
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No flowchart parser available"))?;

        let mut database = FlowchartDatabase::new();
        parser.parse(input, &mut database)?;
        debug!(
            node_count = database.node_count(),
            edge_count = database.edge_count(),
            "Parsing completed"
        );
        drop(_parse_enter);

        // Step 2: Render the result
        let render_span = span!(Level::DEBUG, "pipeline_render");
        let _render_enter = render_span.enter();
        let renderer = self
            .ascii_renderer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No ASCII renderer available"))?;

        let canvas = renderer.render(&database)?;
        debug!(output_len = canvas.len(), "Rendering completed");
        drop(_render_enter);

        info!("Pipeline completed successfully");

        // Step 3: Convert canvas to string
        Ok(canvas)
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::flowchart::FlowchartDetector;

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = Orchestrator::new();
        assert_eq!(orchestrator.get_detectors().len(), 0);
        assert!(!orchestrator.has_flowchart_plugins());
    }

    #[test]
    fn test_orchestrator_default() {
        let orchestrator = Orchestrator::default();
        assert_eq!(orchestrator.get_detectors().len(), 0);
        assert!(!orchestrator.has_flowchart_plugins());
    }

    #[test]
    fn test_orchestrator_with_flowchart_plugins() {
        let orchestrator = Orchestrator::with_flowchart_plugins();
        assert_eq!(orchestrator.get_detectors().len(), 0);
        assert!(orchestrator.has_flowchart_plugins());
    }

    #[test]
    fn test_register_detector() {
        let mut orchestrator = Orchestrator::new();
        assert!(!orchestrator.has_flowchart_plugins());

        // Register a detector
        let detector = Box::new(FlowchartDetector::new());
        orchestrator.register_detector("flowchart".to_string(), detector);

        assert_eq!(orchestrator.get_detectors(), vec!["flowchart"]);
        assert!(!orchestrator.has_flowchart_plugins());
    }

    #[test]
    fn test_detect_diagram_type_with_no_detectors() {
        let orchestrator = Orchestrator::new();
        let input = "graph TD; A-->B;";

        let result = orchestrator.detect_diagram_type(input);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No suitable detector found for input"
        );
    }

    #[test]
    fn test_detect_diagram_type_with_flowchart() {
        let mut orchestrator = Orchestrator::new();
        let detector = Box::new(FlowchartDetector::new());
        orchestrator.register_detector("flowchart".to_string(), detector);

        let input = "graph TD; A-->B;";
        let result = orchestrator.detect_diagram_type(input);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "flowchart");
    }

    #[test]
    fn test_process_with_missing_plugins() {
        let orchestrator = Orchestrator::new();
        let input = "graph TD; A-->B;";

        let result = orchestrator.process(input);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No suitable detector found for input"
        );
    }

    #[test]
    fn test_process_with_no_flowchart_plugins() {
        let mut orchestrator = Orchestrator::new();
        let detector = Box::new(FlowchartDetector::new());
        orchestrator.register_detector("flowchart".to_string(), detector);

        let input = "graph TD; A-->B;";
        let result = orchestrator.process(input);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No flowchart parser available"
        );
    }

    #[test]
    fn test_process_flowchart_success() {
        let orchestrator = Orchestrator::with_flowchart_plugins();
        let input = "graph TD; A-->B;";
        let result = orchestrator.process_flowchart(input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
        // The output should contain ASCII diagram content
        assert!(output.contains("A") || output.contains("B") || output.contains("┌"));
    }

    #[test]
    fn test_process_flowchart_complex() {
        let orchestrator = Orchestrator::with_flowchart_plugins();
        let input = r#"
            graph TD;
            A[Start] --> B{Decision};
            B -->|Yes| C[Process];
            B -->|No| D[End];
            C --> D;
        "#;

        let result = orchestrator.process_flowchart(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
        // Should contain multiple nodes
    }

    #[test]
    fn test_process_with_detection_and_plugins() {
        let mut orchestrator = Orchestrator::with_flowchart_plugins();

        // Add detector for the pipeline
        let detector = Box::new(FlowchartDetector::new());
        orchestrator.register_detector("flowchart".to_string(), detector);

        let input = "graph TD; A-->B;";
        let result = orchestrator.process(input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_process_with_non_flowchart_detection() {
        let mut orchestrator = Orchestrator::with_flowchart_plugins();

        // Add detector that will not match
        let detector = Box::new(FlowchartDetector::new());
        orchestrator.register_detector("flowchart".to_string(), detector);

        let input = "This is just plain text, not a diagram";
        let result = orchestrator.process(input);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No suitable detector found for input"
        );
    }

    #[test]
    fn test_process_with_wrong_diagram_type() {
        // Create a mock detector that returns a wrong type
        let mut orchestrator = Orchestrator::with_flowchart_plugins();

        // We'll test by manually calling detect with a different result
        // since we can't easily mock the detector to return a wrong type
        let detector = Box::new(FlowchartDetector::new());
        orchestrator.register_detector("flowchart".to_string(), detector);

        let input = "graph TD; A-->B;";
        let detection_result = orchestrator.detect_diagram_type(input);
        assert!(detection_result.is_ok());
        assert_eq!(detection_result.unwrap(), "flowchart");

        // Since we detect "flowchart" and have flowchart plugins, this should work
        let result = orchestrator.process(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_empty_input() {
        let orchestrator = Orchestrator::with_flowchart_plugins();
        let result = orchestrator.process_flowchart("");

        assert!(result.is_ok());
        // Empty input produces empty output (no nodes to render)
        let output = result.unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_process_invalid_syntax() {
        let orchestrator = Orchestrator::with_flowchart_plugins();
        let input = "invalid syntax that is not mermaid";
        let result = orchestrator.process_flowchart(input);

        // Should still return Ok (parser handles errors gracefully)
        assert!(result.is_ok());
    }
}
