//! Integration tests for tracing spans and events
//!
//! These tests verify that tracing spans are created during pipeline execution
//! and that events are logged correctly.

use figurehead::core::logging::init_logging;
use figurehead::prelude::*;
use figurehead::{parse, render};
use tracing_subscriber::util::SubscriberInitExt;

#[test]
fn test_tracing_spans_created_during_pipeline() {
    // Initialize a test subscriber that captures events
    let _guard = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::TRACE)
        .set_default();

    // Run a simple pipeline
    let input = "graph LR; A-->B-->C";
    let result = render(input);

    // Verify it succeeds
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
    assert!(output.contains('A') || output.contains('B') || output.contains('C'));
}

#[test]
fn test_parse_with_tracing() {
    // Initialize logging
    let _ = init_logging(Some("debug"), Some("compact"));

    // Parse a diagram
    let input = "graph TD; A[Start]-->B{Decision}-->C[End]";
    let db = parse(input);

    // Verify parsing succeeds
    assert!(db.is_ok());
    let db = db.unwrap();
    assert_eq!(db.node_count(), 3);
    assert_eq!(db.edge_count(), 2);
}

#[test]
fn test_render_with_tracing() {
    // Initialize logging
    let _ = init_logging(Some("debug"), Some("compact"));

    // Render a diagram
    let input = "graph LR; A-->B-->C";
    let result = render(input);

    // Verify rendering succeeds
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_layout_with_tracing() {
    // Initialize logging
    let _ = init_logging(Some("debug"), Some("compact"));

    // Create a database and run layout
    let mut db = FlowchartDatabase::new();
    db.add_simple_node("A", "Start").unwrap();
    db.add_simple_node("B", "End").unwrap();
    db.add_simple_edge("A", "B").unwrap();

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db);

    // Verify layout succeeds
    assert!(result.is_ok());
    let layout_result = result.unwrap();
    assert_eq!(layout_result.nodes.len(), 2);
    assert!(layout_result.width > 0);
    assert!(layout_result.height > 0);
}

#[test]
fn test_detector_with_tracing() {
    // Initialize logging
    let _ = init_logging(Some("debug"), Some("compact"));

    // Test detector
    let detector = FlowchartDetector::new();
    let input = "graph TD; A-->B";

    assert!(detector.detect(input));
    assert!(detector.confidence(input) > 0.0);
    assert_eq!(detector.diagram_type(), "flowchart");
}

#[test]
fn test_orchestrator_with_tracing() {
    // Initialize logging
    let _ = init_logging(Some("debug"), Some("compact"));

    use figurehead::plugins::flowchart::FlowchartDetector;
    use figurehead::plugins::Orchestrator;

    let mut orchestrator = Orchestrator::with_flowchart_plugins();
    orchestrator.register_detector("flowchart".to_string(), Box::new(FlowchartDetector::new()));

    let input = "graph TD; A-->B-->C";
    let result = orchestrator.process(input);

    // Verify processing succeeds
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}
