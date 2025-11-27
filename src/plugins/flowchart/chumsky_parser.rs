//! Enhanced flowchart parser using chumsky
//! This module demonstrates how to use `chumsky` for parsing Mermaid.js markup.

use super::whitespace::optional_whitespace;
use anyhow::Result;
use chumsky::prelude::*;
use chumsky::text::ident;

/// Enhanced flowchart parser using chumsky
pub struct ChumskyFlowchartParser;

impl ChumskyFlowchartParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a single statement using chumsky
    pub fn parse_statement(&self, input: &str) -> Result<Statement> {
        let parser = Self::statement_parser().then_ignore(end());

        parser
            .parse(input)
            .into_result()
            .map_err(|errors| anyhow::anyhow!("Parse errors: {:?}", errors))
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

        let rectangular_node = node_id
            .clone()
            .then_ignore(just('['))
            .then(Self::label_parser())
            .then_ignore(just(']'))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Rectangular,
            });

        let circular_node = node_id
            .clone()
            .then_ignore(just('('))
            .then(Self::label_parser())
            .then_ignore(just(')'))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Circular,
            });

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

        let redirect_node = node_id
            .clone()
            .then_ignore(just('>'))
            .then(Self::label_parser())
            .then_ignore(just('>'))
            .map(|(id, label)| Node {
                id,
                label,
                shape: NodeShape::Redirect,
            });

        rectangular_node
            .or(circular_node)
            .or(diamond_node)
            .or(redirect_node)
            .labelled("node definition")
    }

    fn edge_parser<'src>() -> impl Parser<'src, &'src str, Edge> + Clone {
        let node_id = Self::node_reference();

        let arrow_edge = just("-->").to(EdgeType::Arrow);
        let line_edge = just("---").to(EdgeType::Line);
        let dotted_edge = just("-.-").to(EdgeType::Dotted);
        let thick_edge = just("==>").to(EdgeType::Thick);

        let edge_connector = arrow_edge
            .or(line_edge)
            .or(dotted_edge)
            .or(thick_edge)
            .then_ignore(optional_whitespace());

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
            .map(|(((from, edge_type), label), to)| Edge {
                from,
                to,
                edge_type,
                label,
            })
            .labelled("edge definition")
    }

    fn node_reference<'src>() -> impl Parser<'src, &'src str, String> + Clone {
        ident()
            .map(|s: &str| s.to_string())
            .then(Self::label_suffix().or_not())
            .map(|(id, _)| id)
            .then_ignore(optional_whitespace())
    }

    fn label_suffix<'src>() -> impl Parser<'src, &'src str, ()> + Clone {
        let bracket = just('[')
            .ignore_then(Self::label_parser())
            .then_ignore(just(']'))
            .ignored();

        let paren = just('(')
            .ignore_then(Self::label_parser())
            .then_ignore(just(')'))
            .ignored();

        let brace = just('{')
            .ignore_then(Self::label_parser())
            .then_ignore(just('}'))
            .ignored();

        let redirect = just('>')
            .ignore_then(Self::label_parser())
            .then_ignore(just('>'))
            .ignored();

        bracket.or(paren).or(brace).or(redirect)
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
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NodeShape {
    Rectangular,
    Circular,
    Diamond,
    Redirect,
    Stadium,
    Hexagon,
    Parallelogram,
    Trapezoid,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Arrow,
    Line,
    Dotted,
    Thick,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub label: Option<String>,
}

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
    fn test_basic_node_parsing() {
        let parser = ChumskyFlowchartParser::new();

        let node = parser.parse_statement("A[Start]").unwrap();
        if let Statement::Node(node) = node {
            assert_eq!(node.id, "A");
            assert_eq!(node.label, "Start");
            assert_eq!(node.shape, NodeShape::Rectangular);
        } else {
            panic!("Expected node statement");
        }
    }

    #[test]
    fn test_edge_parsing() {
        let parser = ChumskyFlowchartParser::new();

        let arrow = parser.parse_statement("A --> B").unwrap();
        if let Statement::Edge(edge) = arrow {
            assert_eq!(edge.from, "A");
            assert_eq!(edge.to, "B");
            assert_eq!(edge.edge_type, EdgeType::Arrow);
        } else {
            panic!("Expected edge statement");
        }

        let line = parser.parse_statement("A --- B").unwrap();
        if let Statement::Edge(edge) = line {
            assert_eq!(edge.edge_type, EdgeType::Line);
        } else {
            panic!("Expected edge statement");
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
        if let Statement::Subgraph(_, children) = statement {
            assert_eq!(children.len(), 2);
            assert!(children
                .iter()
                .all(|stmt| matches!(stmt, Statement::Edge(_))));
        } else {
            panic!("Expected subgraph statement");
        }
    }
}
