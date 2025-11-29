//! Flowchart parser implementation
//!
//! Parses flowchart diagram markup into structured data by delegating to the chumsky-based
//! statement parser for each logical statement in the input.

use super::chumsky_parser::{ChumskyFlowchartParser, NodeRef, Statement};
use super::FlowchartDatabase;
use crate::core::{Database, EdgeData, NodeData, Parser};
use anyhow::Result;

const CONNECTORS: [&str; 6] = ["-->", "==>", "---", "-.-", "-.->", "~~~"];

/// Flowchart parser implementation
pub struct FlowchartParser;

impl FlowchartParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FlowchartParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser<FlowchartDatabase> for FlowchartParser {
    fn parse(&self, input: &str, database: &mut FlowchartDatabase) -> Result<()> {
        let chumsky = ChumskyFlowchartParser::new();

        // First, try to extract the direction from the header
        for line in input.lines() {
            let trimmed = line.trim();
            if let Some(direction) = chumsky.parse_header(trimmed) {
                database.set_direction(direction);
                break;
            }
        }

        let mut skipped_statements = Vec::new();

        for statement_text in extract_statements(input) {
            match chumsky.parse_statement(&statement_text) {
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

    let normalized_input = normalize_inline_labels(input);

    for line in normalized_input.lines() {
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

            statements.extend(split_chained_edges(segment));
        }
    }

    statements
}

fn split_chained_edges(statement: &str) -> Vec<String> {
    let trimmed = statement.trim();
    let mut connectors = Vec::new();
    let mut nodes = Vec::new();
    let mut cursor = 0;

    while cursor < trimmed.len() {
        if let Some((pos, conn)) = find_next_connector(trimmed, cursor) {
            let node = trimmed[cursor..pos].trim();
            if !node.is_empty() {
                nodes.push(node.to_string());
            }
            connectors.push(conn);
            cursor = pos + conn.len();
            continue;
        }
        break;
    }

    if cursor <= trimmed.len() {
        let node = trimmed[cursor..].trim();
        if !node.is_empty() {
            nodes.push(node.to_string());
        }
    }

    if connectors.is_empty() || nodes.len() <= 1 {
        return vec![trimmed.to_string()];
    }

    let mut edges = Vec::new();
    for i in 0..connectors.len() {
        if let (Some(from), Some(to)) = (nodes.get(i), nodes.get(i + 1)) {
            edges.push(format!("{}{}{}", from, connectors[i], to));
        }
    }

    edges
}

fn find_next_connector(statement: &str, start: usize) -> Option<(usize, &'static str)> {
    CONNECTORS
        .iter()
        .filter_map(|&conn| statement[start..].find(conn).map(|pos| (start + pos, conn)))
        .min_by_key(|(pos, _)| *pos)
}

fn normalize_inline_labels(input: &str) -> String {
    let mut result = String::new();
    let mut last_index = 0;
    let len = input.len();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'|' {
            if let Some(label_end_rel) = input[i + 1..].find('|') {
                let label_end = i + 1 + label_end_rel;
                let label = &input[i + 1..label_end];
                let mut suffix_idx = label_end + 1;
                while suffix_idx < len && bytes[suffix_idx].is_ascii_whitespace() {
                    suffix_idx += 1;
                }

                if let Some(&connector) = CONNECTORS.iter().find(|&&conn| {
                    suffix_idx + conn.len() <= len && input[suffix_idx..].starts_with(conn)
                }) {
                    let suffix_end = suffix_idx + connector.len();
                    let mut prefix_idx = i;
                    while prefix_idx > 0 {
                        let c = bytes[prefix_idx - 1];
                        if c == b'-' || c == b'=' {
                            prefix_idx -= 1;
                            continue;
                        }
                        break;
                    }

                    result.push_str(&input[last_index..prefix_idx]);
                    result.push_str(connector);
                    result.push('|');
                    result.push_str(label);
                    result.push('|');

                    i = suffix_end;
                    last_index = suffix_end;
                    continue;
                }
            }
        }
        i += 1;
    }

    if last_index < len {
        result.push_str(&input[last_index..]);
    }

    result
}

fn apply_statement(statement: &Statement, database: &mut FlowchartDatabase) -> Result<()> {
    match statement {
        Statement::Node(node) => {
            database.add_node(NodeData::with_shape(&node.id, &node.label, node.shape))?;
        }
        Statement::Edge(edge) => {
            // Ensure both nodes exist with their shape info if available
            ensure_node_from_ref(database, &edge.from_ref)?;
            ensure_node_from_ref(database, &edge.to_ref)?;

            // Add the edge with full metadata
            let edge_data = if let Some(label) = &edge.label {
                EdgeData::with_label(&edge.from, &edge.to, edge.edge_type, label)
            } else {
                EdgeData::with_type(&edge.from, &edge.to, edge.edge_type)
            };
            database.add_edge(edge_data)?;
        }
        Statement::Subgraph(_, children) => {
            for child in children {
                apply_statement(child, database)?;
            }
        }
    }

    Ok(())
}

/// Ensure a node exists, using shape info from the reference if available
fn ensure_node_from_ref(database: &mut FlowchartDatabase, node_ref: &NodeRef) -> Result<()> {
    if database.has_node(&node_ref.id) {
        return Ok(());
    }

    let label = node_ref.label.as_deref().unwrap_or(&node_ref.id);
    let shape = node_ref.shape.unwrap_or_default();
    database.add_node(NodeData::with_shape(&node_ref.id, label, shape))?;
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
    use crate::core::{Database, Direction, EdgeType, NodeShape};

    #[test]
    fn test_extract_statements_basics() {
        let input = "graph TD; A-->B; B-->C";
        let statements = extract_statements(input);
        assert_eq!(statements, vec!["A-->B", "B-->C"]);
    }

    #[test]
    fn test_extract_statements_handles_chains_and_comments() {
        let input = r#"
            graph TB
            A-->B-->C
            %% ignore
            D-->E"#;

        let statements = extract_statements(input);
        assert_eq!(statements, vec!["A-->B", "B-->C", "D-->E"]);
    }

    #[test]
    fn test_split_chained_edges() {
        let edges = split_chained_edges("A-->B-->C-->D");
        assert_eq!(edges, vec!["A-->B", "B-->C", "C-->D"]);
    }

    #[test]
    fn test_normalize_inline_labels() {
        let statement = "A--|Yes|-->B; C--|No|---D";
        let normalized = normalize_inline_labels(statement);
        assert!(normalized.contains("-->|Yes|"));
        assert!(normalized.contains("---|No|"));
    }

    #[test]
    fn test_parser_sets_direction() {
        let parser = FlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        parser.parse("graph LR\n    A-->B", &mut database).unwrap();
        assert_eq!(database.direction(), Direction::LeftRight);
    }

    #[test]
    fn test_parser_stores_node_shapes() {
        let parser = FlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        parser
            .parse("graph TD\n    A[Rectangle]\n    B{Diamond}", &mut database)
            .unwrap();

        assert_eq!(database.get_node("A").unwrap().shape, NodeShape::Rectangle);
        assert_eq!(database.get_node("B").unwrap().shape, NodeShape::Diamond);
    }

    #[test]
    fn test_parser_stores_edge_types() {
        let parser = FlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        parser
            .parse("graph TD\n    A --> B\n    B ==> C", &mut database)
            .unwrap();

        let edges: Vec<_> = database.edges().collect();
        assert_eq!(edges[0].edge_type, EdgeType::Arrow);
        assert_eq!(edges[1].edge_type, EdgeType::ThickArrow);
    }

    #[test]
    fn test_parser_stores_edge_labels() {
        let parser = FlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        parser
            .parse("graph TD\n    A -->|Yes| B", &mut database)
            .unwrap();

        let edges: Vec<_> = database.edges().collect();
        assert_eq!(edges[0].label, Some("Yes".to_string()));
    }

    #[test]
    fn test_flowchart_parser_handles_chained_edges() {
        let parser = FlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        parser
            .parse("graph TD\n    A-->B-->C", &mut database)
            .unwrap();
        assert_eq!(database.edge_count(), 2);
        assert_eq!(database.node_count(), 3);
    }

    #[test]
    fn test_flowchart_parser_handles_inline_label_connectors() {
        let parser = FlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        parser
            .parse("graph TD\n    A --|Yes|--> B", &mut database)
            .unwrap();
        assert_eq!(database.edge_count(), 1);
        assert!(database.has_node("A"));
        assert!(database.has_node("B"));
    }

    #[test]
    fn test_subgraph_population() {
        let parser = FlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        parser
            .parse(
                r#"graph TD
                subgraph "Group"
                    A --> B
                    B --> C
                end"#,
                &mut database,
            )
            .unwrap();

        assert_eq!(database.edge_count(), 2);
        assert_eq!(database.node_count(), 3);
    }
}
