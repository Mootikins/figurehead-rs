//! Core type definitions for diagram processing
//!
//! This module contains the fundamental types used throughout Figurehead:
//! node shapes, edge types, flow direction, and data structures.

use std::fmt;

/// Node shapes matching Mermaid.js syntax
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum NodeShape {
    /// Rectangle: `A[label]`
    #[default]
    Rectangle,
    /// Rounded rectangle (stadium): `A(label)`
    RoundedRect,
    /// Circle: `A((label))`
    Circle,
    /// Diamond (decision): `A{label}`
    Diamond,
    /// Hexagon: `A{{label}}`
    Hexagon,
    /// Subroutine: `A[[label]]`
    Subroutine,
    /// Cylinder (database): `A[(label)]`
    Cylinder,
    /// Asymmetric (flag): `A>label]`
    Asymmetric,
    /// Parallelogram: `A[/label/]`
    Parallelogram,
    /// Trapezoid: `A[/label\]`
    Trapezoid,
}

impl fmt::Display for NodeShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeShape::Rectangle => write!(f, "rectangle"),
            NodeShape::RoundedRect => write!(f, "rounded"),
            NodeShape::Circle => write!(f, "circle"),
            NodeShape::Diamond => write!(f, "diamond"),
            NodeShape::Hexagon => write!(f, "hexagon"),
            NodeShape::Subroutine => write!(f, "subroutine"),
            NodeShape::Cylinder => write!(f, "cylinder"),
            NodeShape::Asymmetric => write!(f, "asymmetric"),
            NodeShape::Parallelogram => write!(f, "parallelogram"),
            NodeShape::Trapezoid => write!(f, "trapezoid"),
        }
    }
}

/// Edge types matching Mermaid.js syntax
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum EdgeType {
    /// Solid arrow: `-->`
    #[default]
    Arrow,
    /// Solid line (no arrow): `---`
    Line,
    /// Dotted arrow: `-.->`
    DottedArrow,
    /// Dotted line: `-.-`
    DottedLine,
    /// Thick arrow: `==>`
    ThickArrow,
    /// Thick line: `===`
    ThickLine,
    /// Invisible edge: `~~~`
    Invisible,
    /// Open circle end: `--o`
    OpenArrow,
    /// Cross end: `--x`
    CrossArrow,
}

impl EdgeType {
    /// Returns true if this edge type has an arrowhead
    pub fn has_arrow(&self) -> bool {
        matches!(
            self,
            EdgeType::Arrow
                | EdgeType::DottedArrow
                | EdgeType::ThickArrow
                | EdgeType::OpenArrow
                | EdgeType::CrossArrow
        )
    }

    /// Returns true if this edge type uses dotted lines
    pub fn is_dotted(&self) -> bool {
        matches!(self, EdgeType::DottedArrow | EdgeType::DottedLine)
    }

    /// Returns true if this edge type uses thick lines
    pub fn is_thick(&self) -> bool {
        matches!(self, EdgeType::ThickArrow | EdgeType::ThickLine)
    }
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdgeType::Arrow => write!(f, "-->"),
            EdgeType::Line => write!(f, "---"),
            EdgeType::DottedArrow => write!(f, "-.->"),
            EdgeType::DottedLine => write!(f, "-.-"),
            EdgeType::ThickArrow => write!(f, "==>"),
            EdgeType::ThickLine => write!(f, "==="),
            EdgeType::Invisible => write!(f, "~~~"),
            EdgeType::OpenArrow => write!(f, "--o"),
            EdgeType::CrossArrow => write!(f, "--x"),
        }
    }
}

/// Flow direction for the diagram layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Direction {
    /// Top to bottom (TD or TB)
    #[default]
    TopDown,
    /// Left to right (LR)
    LeftRight,
    /// Right to left (RL)
    RightLeft,
    /// Bottom to top (BT)
    BottomUp,
}

impl Direction {
    /// Parse direction from mermaid syntax (TD, TB, LR, RL, BT)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TD" | "TB" => Some(Direction::TopDown),
            "LR" => Some(Direction::LeftRight),
            "RL" => Some(Direction::RightLeft),
            "BT" => Some(Direction::BottomUp),
            _ => None,
        }
    }

    /// Returns true if this is a vertical layout (TD or BT)
    pub fn is_vertical(&self) -> bool {
        matches!(self, Direction::TopDown | Direction::BottomUp)
    }

    /// Returns true if this is a horizontal layout (LR or RL)
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Direction::LeftRight | Direction::RightLeft)
    }

    /// Returns true if the flow is reversed (RL or BT)
    pub fn is_reversed(&self) -> bool {
        matches!(self, Direction::RightLeft | Direction::BottomUp)
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::TopDown => write!(f, "TD"),
            Direction::LeftRight => write!(f, "LR"),
            Direction::RightLeft => write!(f, "RL"),
            Direction::BottomUp => write!(f, "BT"),
        }
    }
}

/// A node in the diagram with all its metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeData {
    /// Unique identifier for the node
    pub id: String,
    /// Display label (may differ from id)
    pub label: String,
    /// Visual shape of the node
    pub shape: NodeShape,
}

impl NodeData {
    /// Create a new node with default rectangle shape
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shape: NodeShape::Rectangle,
        }
    }

    /// Create a new node with a specific shape
    pub fn with_shape(id: impl Into<String>, label: impl Into<String>, shape: NodeShape) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shape,
        }
    }
}

/// An edge connecting two nodes with metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeData {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Visual type of the edge
    pub edge_type: EdgeType,
    /// Optional label on the edge
    pub label: Option<String>,
}

impl EdgeData {
    /// Create a new edge with default arrow type
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            edge_type: EdgeType::Arrow,
            label: None,
        }
    }

    /// Create a new edge with a specific type
    pub fn with_type(
        from: impl Into<String>,
        to: impl Into<String>,
        edge_type: EdgeType,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            edge_type,
            label: None,
        }
    }

    /// Create a new edge with a label
    pub fn with_label(
        from: impl Into<String>,
        to: impl Into<String>,
        edge_type: EdgeType,
        label: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            edge_type,
            label: Some(label.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_parsing() {
        assert_eq!(Direction::from_str("TD"), Some(Direction::TopDown));
        assert_eq!(Direction::from_str("tb"), Some(Direction::TopDown));
        assert_eq!(Direction::from_str("LR"), Some(Direction::LeftRight));
        assert_eq!(Direction::from_str("RL"), Some(Direction::RightLeft));
        assert_eq!(Direction::from_str("BT"), Some(Direction::BottomUp));
        assert_eq!(Direction::from_str("invalid"), None);
    }

    #[test]
    fn test_direction_properties() {
        assert!(Direction::TopDown.is_vertical());
        assert!(Direction::BottomUp.is_vertical());
        assert!(!Direction::LeftRight.is_vertical());

        assert!(Direction::LeftRight.is_horizontal());
        assert!(Direction::RightLeft.is_horizontal());
        assert!(!Direction::TopDown.is_horizontal());

        assert!(Direction::RightLeft.is_reversed());
        assert!(Direction::BottomUp.is_reversed());
        assert!(!Direction::TopDown.is_reversed());
    }

    #[test]
    fn test_edge_type_properties() {
        assert!(EdgeType::Arrow.has_arrow());
        assert!(EdgeType::DottedArrow.has_arrow());
        assert!(!EdgeType::Line.has_arrow());

        assert!(EdgeType::DottedArrow.is_dotted());
        assert!(EdgeType::DottedLine.is_dotted());
        assert!(!EdgeType::Arrow.is_dotted());

        assert!(EdgeType::ThickArrow.is_thick());
        assert!(EdgeType::ThickLine.is_thick());
        assert!(!EdgeType::Arrow.is_thick());
    }

    #[test]
    fn test_node_data_constructors() {
        let node = NodeData::new("A", "Label A");
        assert_eq!(node.id, "A");
        assert_eq!(node.label, "Label A");
        assert_eq!(node.shape, NodeShape::Rectangle);

        let diamond = NodeData::with_shape("B", "Decision", NodeShape::Diamond);
        assert_eq!(diamond.shape, NodeShape::Diamond);
    }

    #[test]
    fn test_edge_data_constructors() {
        let edge = EdgeData::new("A", "B");
        assert_eq!(edge.from, "A");
        assert_eq!(edge.to, "B");
        assert_eq!(edge.edge_type, EdgeType::Arrow);
        assert!(edge.label.is_none());

        let labeled = EdgeData::with_label("A", "B", EdgeType::DottedArrow, "Yes");
        assert_eq!(labeled.label, Some("Yes".to_string()));
        assert_eq!(labeled.edge_type, EdgeType::DottedArrow);
    }
}
