//! Core error types for diagram processing
//!
//! This module defines common error types used throughout the diagram processing pipeline.

use thiserror::Error;

/// Core error types for diagram processing
#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("Parse error: {message} at line {line}, column {column}")]
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("Layout error: {message}")]
    LayoutError { message: String },

    #[error("Render error: {message}")]
    RenderError { message: String },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Detection error: {message}")]
    DetectionError { message: String },

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Unknown diagram type: {diagram_type}")]
    UnknownDiagramType { diagram_type: String },
}

impl DiagramError {
    /// Create a new parse error
    pub fn parse_error(message: String, line: usize, column: usize) -> Self {
        Self::ParseError {
            message,
            line,
            column,
        }
    }

    /// Create a new layout error
    pub fn layout_error(message: String) -> Self {
        Self::LayoutError { message }
    }

    /// Create a new render error
    pub fn render_error(message: String) -> Self {
        Self::RenderError { message }
    }

    /// Create a new database error
    pub fn database_error(message: String) -> Self {
        Self::DatabaseError { message }
    }

    /// Create a new detection error
    pub fn detection_error(message: String) -> Self {
        Self::DetectionError { message }
    }
}