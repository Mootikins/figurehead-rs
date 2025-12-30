//! Tests for public API functions in lib.rs

use figurehead::prelude::Database;
use figurehead::{parse, render, render_with_style, CharacterSet, Direction};

#[test]
fn test_render_flowchart() {
    let input = "graph TD\n    A --> B";
    let result = render(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_render_gitgraph() {
    let input = "gitGraph\n   commit\n   commit";
    let result = render(input);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_render_with_style_ascii() {
    let input = "graph LR\n    A --> B";
    let result = render_with_style(input, CharacterSet::Ascii);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
    // ASCII should use + and - characters
    assert!(output.contains('+') || output.contains('-') || output.contains('>'));
}

#[test]
fn test_render_with_style_unicode() {
    let input = "graph TD\n    A --> B";
    let result = render_with_style(input, CharacterSet::Unicode);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_render_with_style_compact() {
    let input = "graph LR\n    A --> B";
    let result = render_with_style(input, CharacterSet::Compact);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_render_with_style_unicode_math() {
    let input = "graph TD\n    A --> B";
    let result = render_with_style(input, CharacterSet::UnicodeMath);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_parse_flowchart() {
    let input = "graph TD\n    A --> B --> C";
    let result = parse(input);
    assert!(result.is_ok());
    let db = result.unwrap();
    assert_eq!(db.node_count(), 3);
    assert_eq!(db.edge_count(), 2);
    assert_eq!(db.direction(), Direction::TopDown);
}

#[test]
fn test_parse_flowchart_lr() {
    let input = "graph LR\n    A --> B";
    let result = parse(input);
    assert!(result.is_ok());
    let db = result.unwrap();
    assert_eq!(db.direction(), Direction::LeftRight);
}
