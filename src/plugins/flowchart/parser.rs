//! Flowchart parser implementation
//!
//! Parses flowchart diagram markup into structured data by delegating to the chumsky-based
//! statement parser for each logical statement in the input.

use super::chumsky_parser::{ChumskyFlowchartParser, Statement};
use super::FlowchartDatabase;
use crate::{Database, Parser};
use anyhow::Result;

/// Flowchart parser implementation
pub struct FlowchartParser;

impl FlowchartParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser<FlowchartDatabase> for FlowchartParser {
    fn parse(&self, input: &str, database: &mut FlowchartDatabase) -> Result<()> {
        let parser = ChumskyFlowchartParser::new();

        let mut skipped_statements = Vec::new();

        for statement_text in extract_statements(input) {
            match parser.parse_statement(&statement_text) {
                Ok(statement) => {
                    apply_statement(&statement, database)?;
                }
                Err(_) => skipped_statements.push(statement_text),
            }
        }

        if !skipped_statements.is_empty() {
            eprintln!(
                "FlowchartParser skipped {} invalid statement(s): {:?}",
                skipped_statements.len(),
                skipped_statements,
            );
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "flowchart"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn can_parse(&self, input: &str) -> bool {
        input.contains("-->") || input.contains("---") || input.contains("==>")
    }
}

fn extract_statements(input: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current_subgraph: Vec<String> = Vec::new();
    let mut in_subgraph = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        if in_subgraph {
            current_subgraph.push(trimmed.to_string());
            if trimmed.eq_ignore_ascii_case("end") {
                statements.push(current_subgraph.join(" "));
                current_subgraph.clear();
                in_subgraph = false;
            }
            continue;
        }

        for segment in trimmed.split(';') {
            let segment = segment.trim();
            if segment.is_empty() {
                continue;
            }

            if segment.to_lowercase().starts_with("subgraph") {
                in_subgraph = true;
                current_subgraph.push(segment.to_string());
                break;
            }

            if is_graph_declaration(segment) {
                continue;
            }

            statements.push(segment.to_string());
        }
    }

    statements
}

fn apply_statement(statement: &Statement, database: &mut FlowchartDatabase) -> Result<()> {
    match statement {
        Statement::Node(node) => {
            database.add_node(&node.id, &node.label)?;
        }
        Statement::Edge(edge) => {
            ensure_node(database, &edge.from)?;
            ensure_node(database, &edge.to)?;
            database.add_edge(&edge.from, &edge.to)?;
        }
        Statement::Subgraph(_, children) => {
            for child in children {
                apply_statement(child, database)?;
            }
        }
    }

    Ok(())
}

fn ensure_node(database: &mut FlowchartDatabase, node_id: &str) -> Result<()> {
    if database.get_node(node_id).is_none() {
        database.add_node(node_id, node_id)?;
    }
    Ok(())
}

fn is_graph_declaration(line: &str) -> bool {
    let trimmed = line.trim();
    let without_semicolon = trimmed.trim_end_matches(';');

    without_semicolon.starts_with("graph ")
        || without_semicolon.starts_with("flowchart ")
        || without_semicolon == "graph"
        || without_semicolon == "flowchart"
        || without_semicolon.starts_with("flowchart TB")
        || without_semicolon.starts_with("flowchart TD")
        || without_semicolon.starts_with("flowchart LR")
        || without_semicolon.starts_with("flowchart RL")
        || without_semicolon.starts_with("flowchart BT")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_statements_basics() {
        let input = "graph TD; A-->B; B-->C";
        let statements = extract_statements(input);
        assert_eq!(statements, vec!["A-->B", "B-->C"]);
    }

    #[test]
    fn test_apply_statement_with_nodes_and_edges() {
        let parser = ChumskyFlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        let stmt = parser.parse_statement("A[Start]").unwrap();
        apply_statement(&stmt, &mut database).unwrap();
        assert_eq!(database.get_node("A"), Some("Start"));

        let stmt = parser.parse_statement("A --> B").unwrap();
        apply_statement(&stmt, &mut database).unwrap();
        assert_eq!(database.get_node("B"), Some("B"));
        assert_eq!(database.edge_count(), 1);
    }

    #[test]
    fn test_subgraph_population() {
        let parser = ChumskyFlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        let stmt = parser
            .parse_statement(
                r#"subgraph "Group"
                    A --> B
                    B --> C
                end"#,
            )
            .unwrap();

        apply_statement(&stmt, &mut database).unwrap();
        assert_eq!(database.edge_count(), 2);
        assert_eq!(database.node_count(), 3);
    }
}
