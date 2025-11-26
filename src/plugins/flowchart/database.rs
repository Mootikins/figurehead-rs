//! Flowchart database implementation
//!
//! Stores flowchart diagram data (nodes and edges).

use std::collections::HashMap;
use anyhow::Result;

use crate::core::Database;

/// Flowchart database implementation
#[derive(Debug, Default)]
pub struct FlowchartDatabase {
    nodes: HashMap<String, String>,
    edges: Vec<(String, String)>,
}

impl FlowchartDatabase {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Database for FlowchartDatabase {
    fn add_node(&mut self, id: &str, label: &str) -> Result<()> {
        self.nodes.insert(id.to_string(), label.to_string());
        Ok(())
    }

    fn add_edge(&mut self, from: &str, to: &str) -> Result<()> {
        self.edges.push((from.to_string(), to.to_string()));
        Ok(())
    }

    fn get_node(&self, id: &str) -> Option<&str> {
        self.nodes.get(id).map(|s| s.as_str())
    }

    fn get_nodes(&self) -> Vec<(&str, &str)> {
        self.nodes.iter()
            .map(|(id, label)| (id.as_str(), label.as_str()))
            .collect()
    }

    fn get_edges(&self) -> Vec<(&str, &str)> {
        self.edges.iter()
            .map(|(from, to)| (from.as_str(), to.as_str()))
            .collect()
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }
}