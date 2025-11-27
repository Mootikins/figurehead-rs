//! Flowchart parser implementation
//!
//! Parses flowchart diagram markup into structured data by delegating to the chumsky-based
//! statement parser for each logical statement in the input.

use super::chumsky_parser::{ChumskyFlowchartParser, Statement};
use super::FlowchartDatabase;
use crate::{Database, Parser};
use anyhow::Result;

const CONNECTORS: [&str; 4] = ["-->", "==>", "---", "-.-"];

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
    let mut i = 0;

    while i < len {
        if let Some((replacement, next_index)) = match_inline_pattern(input, i) {
            if last_index < i {
                result.push_str(&input[last_index..i]);
            }
            result.push_str(&replacement);
            i = next_index;
            last_index = i;
            continue;
        }
        i += 1;
    }

    if last_index < len {
        result.push_str(&input[last_index..]);
    }

    result
}

fn match_inline_pattern(input: &str, idx: usize) -> Option<(String, usize)> {
    let len = input.len();
    if idx + 2 >= len {
        return None;
    }

    const PATTERNS: [(&str, &str); 2] = [("--", "-->"), ("--", "---")];

    for &(prefix, suffix) in PATTERNS.iter() {
        if input[idx..].starts_with(prefix) {
            let mut pos = idx + prefix.len();
            while pos < len && input.as_bytes()[pos].is_ascii_whitespace() {
                pos += 1;
            }

            if pos < len && input[pos..].starts_with('|') {
                let label_start = pos + 1;
                if label_start < len {
                    if let Some(label_end_rel) = input[label_start..].find('|') {
                        let label_end = label_start + label_end_rel;
                        let label = &input[label_start..label_end];
                        let mut after_label = label_end + 1;
                        while after_label < len && input.as_bytes()[after_label].is_ascii_whitespace()
                        {
                            after_label += 1;
                        }

                        if input[after_label..].starts_with(suffix) {
                            let replacement = format!("{}|{}|", suffix, label);
                            let next_index = after_label + suffix.len();
                            return Some((replacement, next_index));
                        }
                    }
                }
            }
        }
    }

    None
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

    #[test]
    fn test_flowchart_parser_handles_chained_edges() {
        let parser = FlowchartParser::new();
        let mut database = FlowchartDatabase::new();

        parser.parse("graph TD\n    A-->B-->C", &mut database).unwrap();
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
        assert!(database.get_node("A").is_some());
        assert!(database.get_node("B").is_some());
    }
}
