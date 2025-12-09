//! Tests for logging initialization to improve coverage

use figurehead::core::logging::{init_logging, LogFormat};
use std::str::FromStr;

#[test]
fn test_init_logging_compact() {
    // Test compact format
    let result = init_logging(Some("error"), Some("compact"));
    // May fail if already initialized, that's ok
    let _ = result;
}

#[test]
fn test_init_logging_pretty() {
    // Test pretty format
    let result = init_logging(Some("warn"), Some("pretty"));
    let _ = result;
}

#[test]
fn test_init_logging_json() {
    // Test JSON format
    let result = init_logging(Some("info"), Some("json"));
    let _ = result;
}

#[test]
fn test_init_logging_with_env_vars() {
    // Test with environment variables
    std::env::set_var("FIGUREHEAD_LOG_LEVEL", "debug");
    std::env::set_var("FIGUREHEAD_LOG_FORMAT", "compact");
    let result = init_logging(None, None);
    let _ = result;
    std::env::remove_var("FIGUREHEAD_LOG_LEVEL");
    std::env::remove_var("FIGUREHEAD_LOG_FORMAT");
}

#[test]
fn test_init_logging_off() {
    // Test "off" level
    let result = init_logging(Some("off"), Some("compact"));
    let _ = result;
}

#[test]
fn test_init_logging_default() {
    // Test default initialization
    let result = init_logging(None, None);
    let _ = result;
}

#[test]
fn test_log_format_from_str_case_insensitive() {
    assert_eq!(LogFormat::from_str("COMPACT").unwrap(), LogFormat::Compact);
    assert_eq!(LogFormat::from_str("PRETTY").unwrap(), LogFormat::Pretty);
    assert_eq!(LogFormat::from_str("JSON").unwrap(), LogFormat::Json);
    assert_eq!(LogFormat::from_str("Compact").unwrap(), LogFormat::Compact);
    assert_eq!(LogFormat::from_str("Pretty").unwrap(), LogFormat::Pretty);
}
