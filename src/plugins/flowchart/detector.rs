//! Flowchart detector implementation
//!
//! Detects flowchart diagram syntax patterns.

use crate::core::Detector;

/// Flowchart detector implementation
pub struct FlowchartDetector;

impl FlowchartDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Detector for FlowchartDetector {
    fn detect(&self, input: &str) -> bool {
        input.contains("graph") ||
        input.contains("flowchart") ||
        input.contains("-->") ||
        input.contains("---")
    }

    fn confidence(&self, input: &str) -> f64 {
        let mut score: f64 = 0.0;

        if input.contains("graph") { score += 0.3; }
        if input.contains("flowchart") { score += 0.3; }
        if input.contains("-->") { score += 0.2; }
        if input.contains("---") { score += 0.2; }

        score.min(1.0)
    }

    fn diagram_type(&self) -> &'static str {
        "flowchart"
    }

    fn patterns(&self) -> Vec<&'static str> {
        vec!["graph", "flowchart", "-->", "---"]
    }
}