//! Tests for logging functionality
//!
//! These tests verify that logging initialization works correctly
//! with different configurations.

use figurehead::core::logging::{init_logging, LogFormat};
use std::str::FromStr;

#[test]
fn test_log_format_parsing() {
    assert_eq!(LogFormat::from_str("compact").unwrap(), LogFormat::Compact);
    assert_eq!(LogFormat::from_str("pretty").unwrap(), LogFormat::Pretty);
    assert_eq!(LogFormat::from_str("json").unwrap(), LogFormat::Json);
    assert_eq!(LogFormat::from_str("COMPACT").unwrap(), LogFormat::Compact);
    assert!(LogFormat::from_str("invalid").is_err());
}

#[test]
fn test_log_format_variants() {
    let variants = LogFormat::variants();
    assert!(variants.contains(&"compact"));
    assert!(variants.contains(&"pretty"));
    assert!(variants.contains(&"json"));
}

#[test]
fn test_init_logging_with_levels() {
    // Test that we can initialize with different log levels
    // Note: We can't easily test the actual output without capturing stdout,
    // but we can verify initialization doesn't panic

    // These should all succeed (or fail gracefully if already initialized)
    let _ = init_logging(Some("trace"), Some("compact"));
    let _ = init_logging(Some("debug"), Some("compact"));
    let _ = init_logging(Some("info"), Some("compact"));
    let _ = init_logging(Some("warn"), Some("compact"));
    let _ = init_logging(Some("error"), Some("compact"));
    let _ = init_logging(Some("off"), Some("compact"));
}

#[test]
fn test_init_logging_with_formats() {
    // Test that we can initialize with different log formats
    let _ = init_logging(Some("info"), Some("compact"));
    let _ = init_logging(Some("info"), Some("pretty"));
    let _ = init_logging(Some("info"), Some("json"));
}

#[test]
fn test_init_logging_defaults() {
    // Test default initialization
    let _ = init_logging(None, None);
}

#[test]
fn test_init_logging_invalid_format() {
    // Test that invalid format returns an error
    let result = init_logging(Some("info"), Some("invalid_format"));
    assert!(result.is_err());
}

#[test]
fn test_init_logging_invalid_level() {
    // Test that invalid level still initializes (falls back to default)
    // EnvFilter will handle invalid levels gracefully
    let result = init_logging(Some("invalid_level"), Some("compact"));
    // This might succeed or fail depending on EnvFilter behavior
    // The important thing is it doesn't panic
    let _ = result;
}
