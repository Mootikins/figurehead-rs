//! Core type definitions for diagram processing
//!
//! This module contains the fundamental types used throughout Figurehead:
//! node shapes, edge types, flow direction, and data structures.

use std::fmt;

/// Character set for rendering output
///
/// Controls which characters are used for drawing shapes and edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum CharacterSet {
    /// Pure ASCII characters only: / \ < > - | +
    /// Maximum compatibility but limited visual quality
    Ascii,
    /// Unicode box-drawing characters: ┌ ┐ └ ┘ ─ │ ╭ ╮
    /// Good balance of appearance and compatibility
    #[default]
    Unicode,
    /// Unicode with mathematical diagonal symbols: ⟋ ⟍
    /// Better diamond shapes, requires font support
    UnicodeMath,
    /// Single-glyph compact mode: ◇ ○ □
    /// Minimal output, nodes are single characters
    Compact,
}

impl CharacterSet {
    /// Returns true if this character set uses only ASCII
    pub fn is_ascii(&self) -> bool {
        matches!(self, CharacterSet::Ascii)
    }

    /// Returns true if this is the compact single-glyph mode
    pub fn is_compact(&self) -> bool {
        matches!(self, CharacterSet::Compact)
    }
}

impl fmt::Display for CharacterSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CharacterSet::Ascii => write!(f, "ascii"),
            CharacterSet::Unicode => write!(f, "unicode"),
            CharacterSet::UnicodeMath => write!(f, "unicode-math"),
            CharacterSet::Compact => write!(f, "compact"),
        }
    }
}

/// Style for rendering diamond (decision) nodes
///
/// Controls the visual appearance of diamond shapes in flowcharts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum DiamondStyle {
    /// Traditional tall diamond with diagonal lines:
    /// ```text
    ///     /\
    ///    /  \
    ///   < ok >
    ///    \  /
    ///     \/
    /// ```
    #[default]
    Tall,
    /// Compact 3-line box with diamond corners:
    /// ```text
    /// ◆─────────◆
    /// │ decide  │
    /// ◆─────────◆
    /// ```
    Box,
    /// Minimal single-line inline style:
    /// ```text
    /// ◆ decide ◆
    /// ```
    Inline,
}

impl DiamondStyle {
    /// Returns the height in rows needed for this style
    pub fn height(&self, label_lines: usize) -> usize {
        match self {
            DiamondStyle::Tall => 5.max(label_lines + 4), // At least 5 rows
            DiamondStyle::Box => 3,                       // Always 3 rows
            DiamondStyle::Inline => 1,                    // Single row
        }
    }
}

impl fmt::Display for DiamondStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiamondStyle::Tall => write!(f, "tall"),
            DiamondStyle::Box => write!(f, "box"),
            DiamondStyle::Inline => write!(f, "inline"),
        }
    }
}

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

    #[test]
    fn test_character_set_properties() {
        assert!(CharacterSet::Ascii.is_ascii());
        assert!(!CharacterSet::Unicode.is_ascii());
        assert!(!CharacterSet::UnicodeMath.is_ascii());
        assert!(!CharacterSet::Compact.is_ascii());

        assert!(CharacterSet::Compact.is_compact());
        assert!(!CharacterSet::Ascii.is_compact());
        assert!(!CharacterSet::Unicode.is_compact());
    }

    #[test]
    fn test_character_set_default() {
        assert_eq!(CharacterSet::default(), CharacterSet::Unicode);
    }

    #[test]
    fn test_character_set_display() {
        assert_eq!(CharacterSet::Ascii.to_string(), "ascii");
        assert_eq!(CharacterSet::Unicode.to_string(), "unicode");
        assert_eq!(CharacterSet::UnicodeMath.to_string(), "unicode-math");
        assert_eq!(CharacterSet::Compact.to_string(), "compact");
    }

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
}
