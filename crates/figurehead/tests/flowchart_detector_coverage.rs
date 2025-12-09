//! Tests for flowchart detector edge cases

use figurehead::plugins::flowchart::FlowchartDetector;
use figurehead::core::Detector;

#[test]
fn test_detector_empty_input() {
    let detector = FlowchartDetector::new();
    assert!(!detector.detect(""));
    assert_eq!(detector.confidence(""), 0.0);
}

#[test]
fn test_detector_whitespace_only() {
    let detector = FlowchartDetector::new();
    assert!(!detector.detect("   \n\t  "));
}

#[test]
fn test_detector_patterns() {
    let detector = FlowchartDetector::new();
    let patterns = detector.patterns();
    assert!(patterns.contains(&"graph"));
    assert!(patterns.contains(&"flowchart"));
    assert!(patterns.contains(&"-->"));
}
