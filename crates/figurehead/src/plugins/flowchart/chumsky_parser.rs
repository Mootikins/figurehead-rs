//! Flowchart parser using chumsky
//!
//! Parses individual Mermaid.js flowchart statements into AST structures.

use super::whitespace::optional_whitespace;
use crate::core::{Direction, EdgeType, NodeShape};
use anyhow::Result;
use chumsky::prelude::*;
use chumsky::text::ident;

/// Chumsky-based flowchart parser
pub struct ChumskyFlowchartParser;

impl ChumskyFlowchartParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a single statement
    pub fn parse_statement(&self, input: &str) -> Result<Statement> {
        let parser = Self::statement_parser().then_ignore(end());

        parser
            .parse(input)
            .into_result()
            .map_err(|errors| anyhow::anyhow!("Parse errors: {:?}", errors))
    }

    /// Parse a graph declaration header (e.g., "graph TD" or "flowchart LR")
    pub fn parse_header(&self, input: &str) -> Option<Direction> {
        let trimmed = input.trim();

        // Handle semicolon-separated input like "graph LR; A-->B"
        let header_part = trimmed.split(';').next().unwrap_or(trimmed).trim();

        // Try to match "graph <dir>" or "flowchart <dir>"
        let parts: Vec<&str> = header_part.split_whitespace().collect();
        if parts.len() >= 2 {
            let keyword = parts[0].to_lowercase();
            if keyword == "graph" || keyword == "flowchart" {
                return Direction::from_str(parts[1]);
            }
        }

        // Default to TopDown if just "graph" or "flowchart"
        if header_part.eq_ignore_ascii_case("graph")
            || header_part.eq_ignore_ascii_case("flowchart")
        {
            return Some(Direction::TopDown);
        }

        None
    }

    fn statement_parser<'src>() -> impl Parser<'src, &'src str, Statement> + Clone {
        recursive(|statements| {
            Self::subgraph_parser(statements.clone())
                .or(Self::edge_parser().map(Statement::Edge))
                .or(Self::node_parser().map(Statement::Node))
        })
    }

    fn node_parser<'src>() -> impl Parser<'src, &'src str, Node> + Clone {
        let node_id = ident()
            .map(|s: &str| s.to_string())
            .labelled("node identifier");

        // A[label] - Rectangle
        let rectangular_node = node_id
            .clone()
            .then_ignore(just('['))
            .then(Self::label_parser())
            .then_ignore(just(']'))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Rectangle,
            });

        // A(label) - Rounded rectangle / stadium
        let rounded_node = node_id
            .clone()
            .then_ignore(just('('))
            .then(Self::label_parser())
            .then_ignore(just(')'))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::RoundedRect,
            });

        // A{label} - Diamond
        let diamond_node = node_id
            .clone()
            .then_ignore(just('{'))
            .then(Self::label_parser())
            .then_ignore(just('}'))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Diamond,
            });

        // A((label)) - Circle
        let circle_node = node_id
            .clone()
            .then_ignore(just("(("))
            .then(Self::label_parser())
            .then_ignore(just("))"))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Circle,
            });

        // A[[label]] - Subroutine
        let subroutine_node = node_id
            .clone()
            .then_ignore(just("[["))
            .then(Self::label_parser())
            .then_ignore(just("]]"))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Subroutine,
            });

        // A{{label}} - Hexagon
        let hexagon_node = node_id
            .clone()
            .then_ignore(just("{{"))
            .then(Self::label_parser())
            .then_ignore(just("}}"))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Hexagon,
            });

        // A[(label)] - Cylinder / database
        let cylinder_node = node_id
            .clone()
            .then_ignore(just("[("))
            .then(Self::label_parser())
            .then_ignore(just(")]"))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Cylinder,
            });

        // A[/label/] - Parallelogram (input/output)
        let parallelogram_node = node_id
            .clone()
            .then_ignore(just("[/"))
            .then(Self::label_parser_no_slash())
            .then_ignore(just("/]"))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Parallelogram,
            });

        // A[/label\] - Trapezoid
        let trapezoid_node = node_id
            .clone()
            .then_ignore(just("[/"))
            .then(Self::label_parser_no_slash())
            .then_ignore(just("\\]"))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Trapezoid,
            });

        // A>label] - Asymmetric (flag)
        let asymmetric_node = node_id
            .clone()
            .then_ignore(just('>'))
            .then(Self::label_parser())
            .then_ignore(just(']'))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Asymmetric,
            });

        // Order matters - try more specific patterns first
        hexagon_node
            .or(cylinder_node)
            .or(subroutine_node)
            .or(circle_node)
            .or(parallelogram_node)
            .or(trapezoid_node)
            .or(rectangular_node)
            .or(rounded_node)
            .or(diamond_node)
            .or(asymmetric_node)
            .labelled("node definition")
    }

    fn edge_parser<'src>() -> impl Parser<'src, &'src str, Edge> + Clone {
        let node_id = Self::node_reference();

        // Edge connectors - order by specificity (longer first)
        let thick_arrow = just("==>").to(EdgeType::ThickArrow);
        let thick_line = just("===").to(EdgeType::ThickLine);
        let dotted_arrow = just("-.->").to(EdgeType::DottedArrow);
        let dotted_line = just("-.-").to(EdgeType::DottedLine);
        let arrow = just("-->").to(EdgeType::Arrow);
        let line = just("---").to(EdgeType::Line);
        let open_arrow = just("--o").to(EdgeType::OpenArrow);
        let cross_arrow = just("--x").to(EdgeType::CrossArrow);
        let invisible = just("~~~").to(EdgeType::Invisible);

        let edge_connector = thick_arrow
            .or(thick_line)
            .or(dotted_arrow)
            .or(dotted_line)
            .or(arrow)
            .or(line)
            .or(open_arrow)
            .or(cross_arrow)
            .or(invisible)
            .then_ignore(optional_whitespace());

        // Edge label: |label|
        let edge_label = just('|')
            .ignore_then(Self::label_parser())
            .then_ignore(just('|'))
            .then_ignore(optional_whitespace())
            .or_not();

        node_id
            .clone()
            .then(edge_connector)
            .then(edge_label)
            .then(node_id)
            .map(|(((from_ref, edge_type), label), to_ref)| Edge {
                from: from_ref.id.clone(),
                to: to_ref.id.clone(),
                from_ref,
                to_ref,
                edge_type,
                label,
            })
            .labelled("edge definition")
    }

    fn node_reference<'src>() -> impl Parser<'src, &'src str, NodeRef> + Clone {
        ident()
            .map(|s: &str| s.to_string())
            .then(Self::label_suffix().or_not())
            .map(|(id, shape_info)| {
                if let Some((label, shape)) = shape_info {
                    NodeRef {
                        id,
                        label: Some(label),
                        shape: Some(shape),
                    }
                } else {
                    NodeRef {
                        id,
                        label: None,
                        shape: None,
                    }
                }
            })
            .then_ignore(optional_whitespace())
    }

    fn label_suffix<'src>() -> impl Parser<'src, &'src str, (String, NodeShape)> + Clone {
        // Match node shape suffixes and extract label + shape
        let double_bracket = just("[[")
            .ignore_then(Self::label_parser())
            .then_ignore(just("]]"))
            .map(|label| (label, NodeShape::Subroutine));

        let double_brace = just("{{")
            .ignore_then(Self::label_parser())
            .then_ignore(just("}}"))
            .map(|label| (label, NodeShape::Hexagon));

        let double_paren = just("((")
            .ignore_then(Self::label_parser())
            .then_ignore(just("))"))
            .map(|label| (label, NodeShape::Circle));

        let cylinder = just("[(")
            .ignore_then(Self::label_parser())
            .then_ignore(just(")]"))
            .map(|label| (label, NodeShape::Cylinder));

        let bracket = just('[')
            .ignore_then(Self::label_parser())
            .then_ignore(just(']'))
            .map(|label| (label, NodeShape::Rectangle));

        let paren = just('(')
            .ignore_then(Self::label_parser())
            .then_ignore(just(')'))
            .map(|label| (label, NodeShape::RoundedRect));

        let brace = just('{')
            .ignore_then(Self::label_parser())
            .then_ignore(just('}'))
            .map(|label| (label, NodeShape::Diamond));

        let asymmetric = just('>')
            .ignore_then(Self::label_parser())
            .then_ignore(just(']'))
            .map(|label| (label, NodeShape::Asymmetric));

        let parallelogram = just("[/")
            .ignore_then(Self::label_parser_no_slash())
            .then_ignore(just("/]"))
            .map(|label| (label, NodeShape::Parallelogram));

        let trapezoid = just("[/")
            .ignore_then(Self::label_parser_no_slash())
            .then_ignore(just("\\]"))
            .map(|label| (label, NodeShape::Trapezoid));

        // Order by specificity
        double_bracket
            .or(double_brace)
            .or(double_paren)
            .or(cylinder)
            .or(parallelogram)
            .or(trapezoid)
            .or(bracket)
            .or(paren)
            .or(brace)
            .or(asymmetric)
    }

    fn subgraph_parser<'src, Statements>(
        statements: Statements,
    ) -> impl Parser<'src, &'src str, Statement> + Clone
    where
        Statements: Parser<'src, &'src str, Statement> + Clone + 'src,
    {
        let subgraph_title = just("subgraph")
            .then_ignore(optional_whitespace())
            .ignore_then(
                just('"')
                    .ignore_then(Self::label_parser())
                    .then_ignore(just('"'))
                    .or(Self::label_parser()),
            )
            .then_ignore(optional_whitespace());

        let subgraph_end = just("end").then_ignore(optional_whitespace()).ignored();

        let subgraph_statements = statements
            .clone()
            .then_ignore(optional_whitespace())
            .repeated()
            .collect();

        subgraph_title
            .then(subgraph_statements)
            .then_ignore(subgraph_end)
            .map(|(title, statements)| Statement::Subgraph(title, statements))
            .labelled("subgraph")
    }

    fn label_parser<'src>() -> impl Parser<'src, &'src str, String> + Clone {
        none_of("[](){}|\"\n\r\t")
            .repeated()
            .at_least(1)
            .collect::<String>()
            .labelled("label")
    }

    fn label_parser_no_slash<'src>() -> impl Parser<'src, &'src str, String> + Clone {
        none_of("[](){}|\"/\\\n\r\t")
            .repeated()
            .at_least(1)
            .collect::<String>()
            .labelled("label")
    }
}

impl Default for ChumskyFlowchartParser {
    fn default() -> Self {
        Self::new()
    }
}

/// A parsed node from the diagram
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
}

/// Node reference in an edge (ID + optional shape/label)
#[derive(Debug, Clone, PartialEq)]
pub struct NodeRef {
    pub id: String,
    pub label: Option<String>,
    pub shape: Option<NodeShape>,
}

/// A parsed edge from the diagram
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub from_ref: NodeRef,
    pub to_ref: NodeRef,
    pub edge_type: EdgeType,
    pub label: Option<String>,
}

/// A parsed statement from the diagram
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Node(Node),
    Edge(Edge),
    Subgraph(String, Vec<Statement>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let parser = ChumskyFlowchartParser::new();

        assert_eq!(parser.parse_header("graph TD"), Some(Direction::TopDown));
        assert_eq!(parser.parse_header("graph TB"), Some(Direction::TopDown));
        assert_eq!(parser.parse_header("graph LR"), Some(Direction::LeftRight));
        assert_eq!(parser.parse_header("graph RL"), Some(Direction::RightLeft));
        assert_eq!(parser.parse_header("graph BT"), Some(Direction::BottomUp));
        assert_eq!(
            parser.parse_header("flowchart LR"),
            Some(Direction::LeftRight)
        );
        assert_eq!(parser.parse_header("graph"), Some(Direction::TopDown));
        assert_eq!(parser.parse_header("not a graph"), None);
    }

    #[test]
    fn test_basic_node_parsing() {
        let parser = ChumskyFlowchartParser::new();

        let node = parser.parse_statement("A[Start]").unwrap();
        if let Statement::Node(node) = node {
            assert_eq!(node.id, "A");
            assert_eq!(node.label, "Start");
            assert_eq!(node.shape, NodeShape::Rectangle);
        } else {
            panic!("Expected node statement");
        }
    }

    #[test]
    fn test_node_shapes() {
        let parser = ChumskyFlowchartParser::new();

        // Rectangle
        let stmt = parser.parse_statement("A[Label]").unwrap();
        assert!(matches!(stmt, Statement::Node(Node { shape: NodeShape::Rectangle, .. })));

        // Rounded
        let stmt = parser.parse_statement("B(Label)").unwrap();
        assert!(matches!(stmt, Statement::Node(Node { shape: NodeShape::RoundedRect, .. })));

        // Diamond
        let stmt = parser.parse_statement("C{Label}").unwrap();
        assert!(matches!(stmt, Statement::Node(Node { shape: NodeShape::Diamond, .. })));

        // Circle
        let stmt = parser.parse_statement("D((Label))").unwrap();
        assert!(matches!(stmt, Statement::Node(Node { shape: NodeShape::Circle, .. })));

        // Subroutine
        let stmt = parser.parse_statement("E[[Label]]").unwrap();
        assert!(matches!(stmt, Statement::Node(Node { shape: NodeShape::Subroutine, .. })));

        // Hexagon
        let stmt = parser.parse_statement("F{{Label}}").unwrap();
        assert!(matches!(stmt, Statement::Node(Node { shape: NodeShape::Hexagon, .. })));

        // Cylinder
        let stmt = parser.parse_statement("G[(Label)]").unwrap();
        assert!(matches!(stmt, Statement::Node(Node { shape: NodeShape::Cylinder, .. })));

        // Parallelogram
        let stmt = parser.parse_statement("H[/Label/]").unwrap();
        assert!(matches!(stmt, Statement::Node(Node { shape: NodeShape::Parallelogram, .. })));

        // Trapezoid
        let stmt = parser.parse_statement("I[/Label\\]").unwrap();
        assert!(matches!(stmt, Statement::Node(Node { shape: NodeShape::Trapezoid, .. })));
    }

    #[test]
    fn test_edge_types() {
        let parser = ChumskyFlowchartParser::new();

        // Arrow
        let stmt = parser.parse_statement("A --> B").unwrap();
        if let Statement::Edge(edge) = stmt {
            assert_eq!(edge.edge_type, EdgeType::Arrow);
        } else {
            panic!("Expected edge");
        }

        // Line
        let stmt = parser.parse_statement("A --- B").unwrap();
        if let Statement::Edge(edge) = stmt {
            assert_eq!(edge.edge_type, EdgeType::Line);
        } else {
            panic!("Expected edge");
        }

        // Dotted arrow
        let stmt = parser.parse_statement("A -.-> B").unwrap();
        if let Statement::Edge(edge) = stmt {
            assert_eq!(edge.edge_type, EdgeType::DottedArrow);
        } else {
            panic!("Expected edge");
        }

        // Thick arrow
        let stmt = parser.parse_statement("A ==> B").unwrap();
        if let Statement::Edge(edge) = stmt {
            assert_eq!(edge.edge_type, EdgeType::ThickArrow);
        } else {
            panic!("Expected edge");
        }
    }

    #[test]
    fn test_edge_with_label() {
        let parser = ChumskyFlowchartParser::new();

        let stmt = parser.parse_statement("A -->|Yes| B").unwrap();
        if let Statement::Edge(edge) = stmt {
            assert_eq!(edge.from, "A");
            assert_eq!(edge.to, "B");
            assert_eq!(edge.label, Some("Yes".to_string()));
        } else {
            panic!("Expected edge");
        }
    }

    #[test]
    fn test_subgraph_parsing() {
        let parser = ChumskyFlowchartParser::new();

        let input = r#"subgraph "Process Group"
            A --> B
            B --> C
        end"#;

        let statement = parser.parse_statement(input).unwrap();
        if let Statement::Subgraph(title, children) = statement {
            assert_eq!(title, "Process Group");
            assert_eq!(children.len(), 2);
            assert!(children
                .iter()
                .all(|stmt| matches!(stmt, Statement::Edge(_))));
        } else {
            panic!("Expected subgraph statement");
        }
    }

    #[test]
    fn test_all_edge_connector_types() {
        let parser = ChumskyFlowchartParser::new();

        // Arrow
        let stmt = parser.parse_statement("A --> B").unwrap();
        assert!(matches!(stmt, Statement::Edge(Edge { edge_type: EdgeType::Arrow, .. })));

        // Thick arrow
        let stmt = parser.parse_statement("A ==> B").unwrap();
        assert!(matches!(stmt, Statement::Edge(Edge { edge_type: EdgeType::ThickArrow, .. })));

        // Line
        let stmt = parser.parse_statement("A --- B").unwrap();
        assert!(matches!(stmt, Statement::Edge(Edge { edge_type: EdgeType::Line, .. })));

        // Dotted line
        let stmt = parser.parse_statement("A -.- B").unwrap();
        assert!(matches!(stmt, Statement::Edge(Edge { edge_type: EdgeType::DottedLine, .. })));

        // Dotted arrow
        let stmt = parser.parse_statement("A -.-> B").unwrap();
        assert!(matches!(stmt, Statement::Edge(Edge { edge_type: EdgeType::DottedArrow, .. })));

        // Invisible
        let stmt = parser.parse_statement("A ~~~ B").unwrap();
        assert!(matches!(stmt, Statement::Edge(Edge { edge_type: EdgeType::Invisible, .. })));

        // Open arrow
        let stmt = parser.parse_statement("A --o B").unwrap();
        assert!(matches!(stmt, Statement::Edge(Edge { edge_type: EdgeType::OpenArrow, .. })));

        // Cross arrow
        let stmt = parser.parse_statement("A --x B").unwrap();
        assert!(matches!(stmt, Statement::Edge(Edge { edge_type: EdgeType::CrossArrow, .. })));

        // Thick line
        let stmt = parser.parse_statement("A === B").unwrap();
        assert!(matches!(stmt, Statement::Edge(Edge { edge_type: EdgeType::ThickLine, .. })));
    }

    #[test]
    fn test_edge_with_inline_label_variations() {
        let parser = ChumskyFlowchartParser::new();

        // Standard inline label
        let stmt = parser.parse_statement("A -->|Yes| B").unwrap();
        if let Statement::Edge(edge) = stmt {
            assert_eq!(edge.label, Some("Yes".to_string()));
        } else {
            panic!("Expected edge");
        }

        // Label with spaces
        let stmt = parser.parse_statement("A -->|Hello World| B").unwrap();
        if let Statement::Edge(edge) = stmt {
            assert_eq!(edge.label, Some("Hello World".to_string()));
        } else {
            panic!("Expected edge");
        }

        // Label with special characters (allowed in labels)
        let stmt = parser.parse_statement("A -->|Yes/No| B").unwrap();
        if let Statement::Edge(edge) = stmt {
            assert_eq!(edge.label, Some("Yes/No".to_string()));
        } else {
            panic!("Expected edge");
        }
    }

    #[test]
    fn test_node_with_empty_label() {
        let parser = ChumskyFlowchartParser::new();

        // Node ID alone should work (no label)
        // But node with empty label brackets should fail (label parser requires at least 1 char)
        assert!(parser.parse_statement("A[]").is_err());
        
        // Node ID without brackets should work (implicit label = ID)
        // However, parse_statement expects a full statement, so just ID won't parse as a node
        // Let's test that a node with a valid label works
        let stmt = parser.parse_statement("A[Label]").unwrap();
        if let Statement::Node(node) = stmt {
            assert_eq!(node.id, "A");
            assert_eq!(node.label, "Label");
        } else {
            panic!("Expected node");
        }
    }

    #[test]
    fn test_node_ids_with_numbers() {
        let parser = ChumskyFlowchartParser::new();

        // Node IDs can contain numbers after first char
        let stmt = parser.parse_statement("A1[Label]").unwrap();
        if let Statement::Node(node) = stmt {
            assert_eq!(node.id, "A1");
            assert_eq!(node.label, "Label");
        } else {
            panic!("Expected node");
        }

        let stmt = parser.parse_statement("Node123[Test]").unwrap();
        if let Statement::Node(node) = stmt {
            assert_eq!(node.id, "Node123");
        } else {
            panic!("Expected node");
        }
    }

    #[test]
    fn test_node_labels_with_special_chars() {
        let parser = ChumskyFlowchartParser::new();

        // Labels can contain various characters except delimiters
        let stmt = parser.parse_statement("A[Hello-World_123]").unwrap();
        if let Statement::Node(node) = stmt {
            assert_eq!(node.label, "Hello-World_123");
        } else {
            panic!("Expected node");
        }

        // Unicode characters
        let stmt = parser.parse_statement("A[こんにちは]").unwrap();
        if let Statement::Node(node) = stmt {
            assert_eq!(node.label, "こんにちは");
        } else {
            panic!("Expected node");
        }
    }

    #[test]
    fn test_subgraph_without_quotes() {
        let parser = ChumskyFlowchartParser::new();

        let input = r#"subgraph ProcessGroup
            A --> B
        end"#;

        let statement = parser.parse_statement(input).unwrap();
        if let Statement::Subgraph(title, _) = statement {
            assert_eq!(title, "ProcessGroup");
        } else {
            panic!("Expected subgraph statement");
        }
    }

    #[test]
    fn test_subgraph_with_nested_content() {
        let parser = ChumskyFlowchartParser::new();

        let input = r#"subgraph "Outer"
            A --> B
            subgraph "Inner"
                C --> D
            end
            B --> C
        end"#;

        let statement = parser.parse_statement(input).unwrap();
        if let Statement::Subgraph(title, children) = statement {
            assert_eq!(title, "Outer");
            assert!(children.len() >= 2);
        } else {
            panic!("Expected subgraph statement");
        }
    }

    #[test]
    fn test_whitespace_variations() {
        let parser = ChumskyFlowchartParser::new();

        // Extra whitespace
        assert!(parser.parse_statement("A  -->  B").is_ok());
        assert!(parser.parse_statement("A\t-->\tB").is_ok());
        assert!(parser.parse_statement("A\n-->\nB").is_ok());

        // No whitespace
        assert!(parser.parse_statement("A-->B").is_ok());
    }

    #[test]
    fn test_malformed_syntax_errors() {
        let parser = ChumskyFlowchartParser::new();

        // Missing closing bracket
        assert!(parser.parse_statement("A[Label").is_err());

        // Missing node ID
        assert!(parser.parse_statement("[Label]").is_err());

        // Incomplete edge
        assert!(parser.parse_statement("A -->").is_err());

        // Invalid connector
        assert!(parser.parse_statement("A ----> B").is_err());
    }

    #[test]
    fn test_edge_with_node_shapes() {
        let parser = ChumskyFlowchartParser::new();

        // Edge with shaped nodes
        let stmt = parser.parse_statement("A[Start] --> B{Decision}").unwrap();
        if let Statement::Edge(edge) = stmt {
            assert_eq!(edge.from_ref.shape, Some(NodeShape::Rectangle));
            assert_eq!(edge.to_ref.shape, Some(NodeShape::Diamond));
        } else {
            panic!("Expected edge");
        }
    }

    #[test]
    fn test_multiple_statements_on_one_line() {
        use crate::core::{Database, Parser};

        // This tests the extract_statements function in parser.rs
        // Multiple statements separated by semicolons
        let input = "A --> B; B --> C; C --> D";
        // Note: parse_statement only parses one statement, so we test the full parser
        let full_parser = super::super::parser::FlowchartParser::new();
        let mut db = super::super::database::FlowchartDatabase::new();
        assert!(full_parser.parse(input, &mut db).is_ok());
        assert_eq!(db.edge_count(), 3);
    }
}
