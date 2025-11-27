//! Flowchart database implementation
//!
//! Stores flowchart diagram data (nodes and edges).

use anyhow::Result;
use std::collections::HashMap;

use crate::core::Database;

#[derive(Debug, Clone)]
struct FlowchartEdge {
    from: String,
    to: String,
    label: Option<String>,
}

/// Flowchart database implementation
#[derive(Debug, Default)]
pub struct FlowchartDatabase {
    nodes: HashMap<String, String>,
    edges: Vec<FlowchartEdge>,
}

impl FlowchartDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_edge_with_label(
        &mut self,
        from: &str,
        to: &str,
        label: Option<&str>,
    ) -> Result<()> {
        self.edges.push(FlowchartEdge {
            from: from.to_string(),
            to: to.to_string(),
            label: label.map(|value| value.to_string()),
        });
        Ok(())
    }

    pub fn get_edges_with_labels(&self) -> Vec<(&str, &str, Option<&str>)> {
        self.edges
            .iter()
            .map(|edge| (edge.from.as_str(), edge.to.as_str(), edge.label.as_deref()))
            .collect()
    }
}

impl Database for FlowchartDatabase {
    fn add_node(&mut self, id: &str, label: &str) -> Result<()> {
        self.nodes.insert(id.to_string(), label.to_string());
        Ok(())
    }

    fn add_edge(&mut self, from: &str, to: &str) -> Result<()> {
        self.add_edge_with_label(from, to, None)?;
        Ok(())
    }

    fn get_node(&self, id: &str) -> Option<&str> {
        self.nodes.get(id).map(|s| s.as_str())
    }

    fn get_nodes(&self) -> Vec<(&str, &str)> {
        self.nodes
            .iter()
            .map(|(id, label)| (id.as_str(), label.as_str()))
            .collect()
    }

    fn get_edges(&self) -> Vec<(&str, &str)> {
        self.edges
            .iter()
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Database;

    #[test]
    fn test_database_basic_functionality() {
        let mut db = FlowchartDatabase::new();

        assert!(db.add_node("A", "Start").is_ok());
        assert!(db.add_node("B", "Process").is_ok());
        assert!(db.add_node("C", "End").is_ok());

        assert_eq!(db.node_count(), 3);
        assert_eq!(db.get_node("A"), Some("Start"));
        assert_eq!(db.get_node("B"), Some("Process"));
        assert_eq!(db.get_node("C"), Some("End"));

        assert!(db.add_edge("A", "B").is_ok());
        assert!(db.add_edge("B", "C").is_ok());

        assert_eq!(db.edge_count(), 2);
        let edges = db.get_edges();
        assert!(edges.contains(&("A", "B")));
        assert!(edges.contains(&("B", "C")));
    }

    #[test]
    fn test_edges_with_labels() {
        let mut db = FlowchartDatabase::new();
        db.add_node("A", "Start").unwrap();
        db.add_node("B", "End").unwrap();

        db.add_edge_with_label("A", "B", Some("Yes")).unwrap();
        let edges = db.get_edges_with_labels();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0], ("A", "B", Some("Yes")));
    }
}

impl FlowchartDatabase {
    pub fn in_degree(&self, node_id: &str) -> usize {
        self.edges.iter().filter(|edge| edge.to == node_id).count()
    }

    pub fn out_degree(&self, node_id: &str) -> usize {
        self.edges.iter().filter(|edge| edge.from == node_id).count()
    }

    pub fn get_neighbors(&self, node_id: &str) -> Vec<&str> {
        let mut neighbors = Vec::new();
        for edge in &self.edges {
            if edge.from == node_id {
                neighbors.push(edge.to.as_str());
            }
            if edge.to == node_id {
                neighbors.push(edge.from.as_str());
            }
        }
        neighbors
    }

    pub fn get_outgoing_edges(&self, node_id: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|edge| edge.from == node_id)
            .map(|edge| edge.to.as_str())
            .collect()
    }

    pub fn get_incoming_edges(&self, node_id: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|edge| edge.to == node_id)
            .map(|edge| edge.from.as_str())
            .collect()
    }

    pub fn has_node(&self, node_id: &str) -> bool {
        self.nodes.contains_key(node_id)
    }

    pub fn topological_sort(&self) -> Vec<&str> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        for node_id in self.nodes.keys() {
            in_degree.insert(node_id, 0);
            adjacency.insert(node_id, Vec::new());
        }

        for edge in &self.edges {
            if let Some(degree) = in_degree.get_mut(edge.to.as_str()) {
                *degree += 1;
            }
            if let Some(neighbors) = adjacency.get_mut(edge.from.as_str()) {
                neighbors.push(edge.to.as_str());
            }
        }

        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(node, _)| *node)
            .collect();
        let mut result = Vec::new();

        while let Some(node) = queue.pop() {
            result.push(node);

            if let Some(neighbors) = adjacency.get(node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor);
                        }
                    }
                }
            }
        }

        result
    }

    pub fn get_source_nodes(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .filter(|(id, _)| self.in_degree(id) == 0)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    pub fn get_sink_nodes(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .filter(|(id, _)| self.out_degree(id) == 0)
            .map(|(id, _)| id.as_str())
            .collect()
    }
}
