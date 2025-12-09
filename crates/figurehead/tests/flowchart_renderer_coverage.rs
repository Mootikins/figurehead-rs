//! Tests for flowchart renderer edge cases to improve coverage

use figurehead::plugins::flowchart::*;
use figurehead::core::{CharacterSet, Database, Direction, EdgeType, NodeShape, Parser, Renderer, LayoutAlgorithm};

#[test]
fn test_renderer_all_node_shapes() {
    let mut db = FlowchartDatabase::new();
    db.add_shaped_node("R", "Rect", NodeShape::Rectangle).unwrap();
    db.add_shaped_node("RR", "Rounded", NodeShape::RoundedRect).unwrap();
    db.add_shaped_node("D", "Diamond", NodeShape::Diamond).unwrap();
    db.add_shaped_node("C", "Circle", NodeShape::Circle).unwrap();
    db.add_shaped_node("S", "Subroutine", NodeShape::Subroutine).unwrap();
    db.add_shaped_node("H", "Hexagon", NodeShape::Hexagon).unwrap();
    db.add_shaped_node("Cy", "Cylinder", NodeShape::Cylinder).unwrap();
    db.add_shaped_node("P", "Parallelogram", NodeShape::Parallelogram).unwrap();
    db.add_shaped_node("T", "Trapezoid", NodeShape::Trapezoid).unwrap();
    db.add_shaped_node("A", "Asymmetric", NodeShape::Asymmetric).unwrap();
    
    let renderer = FlowchartRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_renderer_all_edge_types() {
    let mut db = FlowchartDatabase::new();
    db.add_simple_node("A", "A").unwrap();
    db.add_simple_node("B", "B").unwrap();
    db.add_simple_node("C", "C").unwrap();
    db.add_simple_node("D", "D").unwrap();
    db.add_simple_node("E", "E").unwrap();
    db.add_simple_node("F", "F").unwrap();
    db.add_simple_node("G", "G").unwrap();
    db.add_simple_node("H", "H").unwrap();
    db.add_simple_node("I", "I").unwrap();
    
    db.add_typed_edge("A", "B", EdgeType::Arrow).unwrap();
    db.add_typed_edge("B", "C", EdgeType::Line).unwrap();
    db.add_typed_edge("C", "D", EdgeType::DottedArrow).unwrap();
    db.add_typed_edge("D", "E", EdgeType::DottedLine).unwrap();
    db.add_typed_edge("E", "F", EdgeType::ThickArrow).unwrap();
    db.add_typed_edge("F", "G", EdgeType::ThickLine).unwrap();
    db.add_typed_edge("G", "H", EdgeType::Invisible).unwrap();
    db.add_typed_edge("H", "I", EdgeType::OpenArrow).unwrap();
    
    let renderer = FlowchartRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_renderer_all_directions() {
    let directions = [Direction::TopDown, Direction::BottomUp, Direction::LeftRight, Direction::RightLeft];
    
    for direction in directions {
        let mut db = FlowchartDatabase::with_direction(direction);
        db.add_simple_node("A", "Start").unwrap();
        db.add_simple_node("B", "End").unwrap();
        db.add_simple_edge("A", "B").unwrap();
        
        let renderer = FlowchartRenderer::new();
        let result = renderer.render(&db).unwrap();
        assert!(!result.is_empty());
    }
}

#[test]
fn test_renderer_canvas_edge_cases() {
    let mut db = FlowchartDatabase::new();
    db.add_simple_node("A", "Very Long Label That Might Overflow").unwrap();
    db.add_simple_node("B", "B").unwrap();
    db.add_simple_edge("A", "B").unwrap();
    
    let renderer = FlowchartRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_renderer_orthogonal_edge_routing() {
    // Create a graph that requires orthogonal routing
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("A", "Top").unwrap();
    db.add_simple_node("B", "Left").unwrap();
    db.add_simple_node("C", "Right").unwrap();
    db.add_simple_node("D", "Bottom").unwrap();
    
    db.add_simple_edge("A", "B").unwrap();
    db.add_simple_edge("A", "C").unwrap();
    db.add_simple_edge("B", "D").unwrap();
    db.add_simple_edge("C", "D").unwrap();
    
    let renderer = FlowchartRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_renderer_edge_label_positioning() {
    let mut db = FlowchartDatabase::new();
    db.add_simple_node("A", "Start").unwrap();
    db.add_simple_node("B", "End").unwrap();
    db.add_labeled_edge("A", "B", EdgeType::Arrow, "Label").unwrap();
    
    let renderer = FlowchartRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(!result.is_empty());
    assert!(result.contains("Label"));
}

#[test]
fn test_renderer_style_properties() {
    let renderer = FlowchartRenderer::new();
    assert_eq!(renderer.style(), CharacterSet::Unicode);
    
    let ascii_renderer = FlowchartRenderer::with_style(CharacterSet::Ascii);
    assert_eq!(ascii_renderer.style(), CharacterSet::Ascii);
}
