//! Tests for chumsky parser edge cases

// Note: chumsky_parser is private, so we test through the public parser interface
use figurehead::plugins::flowchart::FlowchartParser;
use figurehead::core::{Database, Parser};

#[test]
fn test_parse_edge_with_open_arrow() {
    let parser = FlowchartParser::new();
    let mut db = figurehead::plugins::flowchart::FlowchartDatabase::new();
    parser.parse("graph TD\n    A --o B", &mut db).unwrap();
    let edges: Vec<_> = db.edges().collect();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].edge_type, figurehead::core::EdgeType::OpenArrow);
}

#[test]
fn test_parse_edge_with_cross_arrow() {
    let parser = FlowchartParser::new();
    let mut db = figurehead::plugins::flowchart::FlowchartDatabase::new();
    parser.parse("graph TD\n    A --x B", &mut db).unwrap();
    let edges: Vec<_> = db.edges().collect();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].edge_type, figurehead::core::EdgeType::CrossArrow);
}

#[test]
fn test_parse_edge_with_thick_line() {
    let parser = FlowchartParser::new();
    let mut db = figurehead::plugins::flowchart::FlowchartDatabase::new();
    parser.parse("graph TD\n    A === B", &mut db).unwrap();
    let edges: Vec<_> = db.edges().collect();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].edge_type, figurehead::core::EdgeType::ThickLine);
}
