//! Tests for Display implementations in core types

use figurehead::core::{Direction, EdgeType, NodeShape};

#[test]
fn test_node_shape_display() {
    assert_eq!(NodeShape::Rectangle.to_string(), "rectangle");
    assert_eq!(NodeShape::RoundedRect.to_string(), "rounded");
    assert_eq!(NodeShape::Circle.to_string(), "circle");
    assert_eq!(NodeShape::Diamond.to_string(), "diamond");
    assert_eq!(NodeShape::Hexagon.to_string(), "hexagon");
    assert_eq!(NodeShape::Subroutine.to_string(), "subroutine");
    assert_eq!(NodeShape::Cylinder.to_string(), "cylinder");
    assert_eq!(NodeShape::Asymmetric.to_string(), "asymmetric");
    assert_eq!(NodeShape::Parallelogram.to_string(), "parallelogram");
    assert_eq!(NodeShape::Trapezoid.to_string(), "trapezoid");
}

#[test]
fn test_edge_type_display() {
    assert_eq!(EdgeType::Arrow.to_string(), "-->");
    assert_eq!(EdgeType::Line.to_string(), "---");
    assert_eq!(EdgeType::DottedArrow.to_string(), "-.->");
    assert_eq!(EdgeType::DottedLine.to_string(), "-.-");
    assert_eq!(EdgeType::ThickArrow.to_string(), "==>");
    assert_eq!(EdgeType::ThickLine.to_string(), "===");
    assert_eq!(EdgeType::Invisible.to_string(), "~~~");
    assert_eq!(EdgeType::OpenArrow.to_string(), "--o");
    assert_eq!(EdgeType::CrossArrow.to_string(), "--x");
}

#[test]
fn test_direction_display() {
    assert_eq!(Direction::TopDown.to_string(), "TD");
    assert_eq!(Direction::LeftRight.to_string(), "LR");
    assert_eq!(Direction::RightLeft.to_string(), "RL");
    assert_eq!(Direction::BottomUp.to_string(), "BT");
}

#[test]
fn test_all_edge_types_have_arrow_property() {
    assert!(EdgeType::Arrow.has_arrow());
    assert!(EdgeType::DottedArrow.has_arrow());
    assert!(EdgeType::ThickArrow.has_arrow());
    assert!(EdgeType::OpenArrow.has_arrow());
    assert!(EdgeType::CrossArrow.has_arrow());
    assert!(!EdgeType::Line.has_arrow());
    assert!(!EdgeType::DottedLine.has_arrow());
    assert!(!EdgeType::ThickLine.has_arrow());
    assert!(!EdgeType::Invisible.has_arrow());
}

#[test]
fn test_all_edge_types_dotted_property() {
    assert!(EdgeType::DottedArrow.is_dotted());
    assert!(EdgeType::DottedLine.is_dotted());
    assert!(!EdgeType::Arrow.is_dotted());
    assert!(!EdgeType::Line.is_dotted());
    assert!(!EdgeType::ThickArrow.is_dotted());
    assert!(!EdgeType::ThickLine.is_dotted());
    assert!(!EdgeType::Invisible.is_dotted());
    assert!(!EdgeType::OpenArrow.is_dotted());
    assert!(!EdgeType::CrossArrow.is_dotted());
}

#[test]
fn test_all_edge_types_thick_property() {
    assert!(EdgeType::ThickArrow.is_thick());
    assert!(EdgeType::ThickLine.is_thick());
    assert!(!EdgeType::Arrow.is_thick());
    assert!(!EdgeType::Line.is_thick());
    assert!(!EdgeType::DottedArrow.is_thick());
    assert!(!EdgeType::DottedLine.is_thick());
    assert!(!EdgeType::Invisible.is_thick());
    assert!(!EdgeType::OpenArrow.is_thick());
    assert!(!EdgeType::CrossArrow.is_thick());
}
