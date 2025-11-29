//! Integration tests for the public API

use figurehead::prelude::*;
use figurehead::{parse, render, Direction, NodeShape};

#[test]
fn test_render_simple_chain() {
    let ascii = render("graph LR; A-->B-->C").unwrap();
    assert!(ascii.contains('A'));
    assert!(ascii.contains('B'));
    assert!(ascii.contains('C'));
}

#[test]
fn test_render_with_labels() {
    let ascii = render("graph LR; A[Start]-->B[End]").unwrap();
    assert!(ascii.contains("Start"));
    assert!(ascii.contains("End"));
}

#[test]
fn test_render_top_down() {
    let ascii = render("graph TD; A-->B-->C").unwrap();
    // In TD layout, nodes should be stacked vertically
    assert!(ascii.contains('A'));
    assert!(ascii.contains('B'));
    assert!(ascii.contains('C'));
}

#[test]
fn test_render_with_diamond() {
    let ascii = render("graph LR; A-->B{Decision}-->C").unwrap();
    assert!(ascii.contains("Decision"));
    // Diamond shape uses < and > characters
    assert!(ascii.contains('<') || ascii.contains('>'));
}

#[test]
fn test_parse_node_count() {
    let db = parse("graph TD; A-->B-->C-->D").unwrap();
    assert_eq!(db.node_count(), 4);
    assert_eq!(db.edge_count(), 3);
}

#[test]
fn test_parse_direction_td() {
    let db = parse("graph TD; A-->B").unwrap();
    assert_eq!(db.direction(), Direction::TopDown);
}

#[test]
fn test_parse_direction_lr() {
    let db = parse("graph LR; A-->B").unwrap();
    assert_eq!(db.direction(), Direction::LeftRight);
}

#[test]
fn test_parse_direction_rl() {
    let db = parse("graph RL; A-->B").unwrap();
    assert_eq!(db.direction(), Direction::RightLeft);
}

#[test]
fn test_parse_direction_bt() {
    let db = parse("graph BT; A-->B").unwrap();
    assert_eq!(db.direction(), Direction::BottomUp);
}

#[test]
fn test_parse_node_shapes() {
    let db = parse(
        r#"graph TD
        A[Rectangle]
        B(Rounded)
        C{Diamond}
        D((Circle))"#,
    )
    .unwrap();

    assert_eq!(db.get_node("A").unwrap().shape, NodeShape::Rectangle);
    assert_eq!(db.get_node("B").unwrap().shape, NodeShape::RoundedRect);
    assert_eq!(db.get_node("C").unwrap().shape, NodeShape::Diamond);
    assert_eq!(db.get_node("D").unwrap().shape, NodeShape::Circle);
}

#[test]
fn test_parse_node_labels() {
    let db = parse("graph TD; A[Hello World]-->B[Goodbye]").unwrap();
    assert_eq!(db.get_node("A").unwrap().label, "Hello World");
    assert_eq!(db.get_node("B").unwrap().label, "Goodbye");
}

#[test]
fn test_parse_edge_labels() {
    let db = parse("graph TD; A-->|Yes|B; A-->|No|C").unwrap();
    let edges: Vec<_> = db.edges().collect();

    // Find edges and check labels
    let yes_edge = edges.iter().find(|e| e.label == Some("Yes".to_string()));
    let no_edge = edges.iter().find(|e| e.label == Some("No".to_string()));

    assert!(yes_edge.is_some());
    assert!(no_edge.is_some());
}

#[test]
fn test_parse_chained_edges() {
    let db = parse("graph LR; A-->B-->C-->D-->E").unwrap();
    assert_eq!(db.node_count(), 5);
    assert_eq!(db.edge_count(), 4);
}

#[test]
fn test_full_pipeline_complex() {
    let input = r#"graph TD
        A[Start] --> B{Is it working?}
        B -->|Yes| C[Great!]
        B -->|No| D[Debug]
        D --> B
        C --> E[End]"#;

    let db = parse(input).unwrap();
    assert_eq!(db.node_count(), 5);
    assert_eq!(db.edge_count(), 5);
    assert_eq!(db.direction(), Direction::TopDown);

    // Verify specific nodes
    assert_eq!(db.get_node("B").unwrap().shape, NodeShape::Diamond);
    assert_eq!(db.get_node("A").unwrap().label, "Start");
    assert_eq!(db.get_node("E").unwrap().label, "End");

    // Render should succeed
    let ascii = render(input).unwrap();
    assert!(!ascii.is_empty());
    assert!(ascii.contains("Start"));
    assert!(ascii.contains("End"));
}

#[test]
fn test_prelude_imports() {
    // Test that prelude provides everything needed
    let parser = FlowchartParser::new();
    let mut database = FlowchartDatabase::new();
    let renderer = FlowchartRenderer::new();
    let detector = FlowchartDetector::new();

    let input = "graph LR; A-->B";

    assert!(detector.detect(input));
    parser.parse(input, &mut database).unwrap();
    assert_eq!(database.node_count(), 2);

    let output = renderer.render(&database).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_empty_input() {
    let db = parse("").unwrap();
    assert_eq!(db.node_count(), 0);
    assert_eq!(db.edge_count(), 0);

    let ascii = render("").unwrap();
    assert!(ascii.is_empty());
}

#[test]
fn test_multiline_input() {
    let input = r#"
        graph TD
        A --> B
        B --> C
        C --> D
    "#;

    let db = parse(input).unwrap();
    assert_eq!(db.node_count(), 4);
    assert_eq!(db.edge_count(), 3);
}

#[test]
fn test_semicolon_separated() {
    let db = parse("graph LR; A-->B; B-->C; C-->D").unwrap();
    assert_eq!(db.node_count(), 4);
    assert_eq!(db.edge_count(), 3);
}
