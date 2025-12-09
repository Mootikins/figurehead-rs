//! Tests for core error types

use figurehead::core::DiagramError;

#[test]
fn test_parse_error() {
    let error = DiagramError::parse_error("Invalid syntax".to_string(), 5, 10);
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Parse error"));
    assert!(error_msg.contains("Invalid syntax"));
    assert!(error_msg.contains("line 5"));
    assert!(error_msg.contains("column 10"));
}

#[test]
fn test_layout_error() {
    let error = DiagramError::layout_error("Layout failed".to_string());
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Layout error"));
    assert!(error_msg.contains("Layout failed"));
}

#[test]
fn test_render_error() {
    let error = DiagramError::render_error("Render failed".to_string());
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Render error"));
    assert!(error_msg.contains("Render failed"));
}

#[test]
fn test_database_error() {
    let error = DiagramError::database_error("Database error".to_string());
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Database error"));
}

#[test]
fn test_detection_error() {
    let error = DiagramError::detection_error("Detection failed".to_string());
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Detection error"));
    assert!(error_msg.contains("Detection failed"));
}

#[test]
fn test_unknown_diagram_type() {
    let error = DiagramError::UnknownDiagramType {
        diagram_type: "unknown".to_string(),
    };
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Unknown diagram type"));
    assert!(error_msg.contains("unknown"));
}

#[test]
fn test_io_error() {
    use std::io;
    let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error: DiagramError = io_err.into();
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("IO error"));
    assert!(error_msg.contains("File not found"));
}
