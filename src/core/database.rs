//! Core database trait for diagram data storage
//!
//! This trait defines the interface for storing and managing diagram data.
//! Inspired by mermaid.js's database system but adapted for Rust.

use anyhow::Result;

/// Core trait for diagram databases
///
/// This trait represents the data storage layer for diagram information.
/// Each diagram type has its own database implementation that stores
/// nodes, edges, and other diagram-specific data.
///
/// # Example
/// ```
/// use figurehead::core::Database;
/// use figurehead::plugins::flowchart::FlowchartDatabase;
///
/// let mut db = FlowchartDatabase::new();
/// db.add_node("A", "Node A");
/// db.add_node("B", "Node B");
/// db.add_edge("A", "B");
/// ```
pub trait Database: Send + Sync {
    /// Add a node to the database
    fn add_node(&mut self, id: &str, label: &str) -> Result<()>;

    /// Add an edge to the database
    fn add_edge(&mut self, from: &str, to: &str) -> Result<()>;

    /// Get a node by ID
    fn get_node(&self, id: &str) -> Option<&str>;

    /// Get all nodes in the database
    fn get_nodes(&self) -> Vec<(&str, &str)>;

    /// Get all edges in the database
    fn get_edges(&self) -> Vec<(&str, &str)>;

    /// Clear all data from the database
    fn clear(&mut self);

    /// Get the number of nodes
    fn node_count(&self) -> usize;

    /// Get the number of edges
    fn edge_count(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::flowchart::*;

    #[test]
    fn test_diagram_database_trait_exists() {
        // This test ensures we have the DiagramDatabase trait
        let db = FlowchartDatabase::new();
        assert_eq!(db.node_count(), 0);
        assert_eq!(db.edge_count(), 0);
    }

    #[test]
    fn test_database_operations() {
        let mut db = FlowchartDatabase::new();

        // Test adding nodes
        db.add_node("A", "Node A").unwrap();
        db.add_node("B", "Node B").unwrap();
        assert_eq!(db.node_count(), 2);

        // Test adding edges
        db.add_edge("A", "B").unwrap();
        assert_eq!(db.edge_count(), 1);

        // Test node retrieval
        assert_eq!(db.get_node("A"), Some("Node A"));
        assert_eq!(db.get_node("C"), None);

        // Test clearing
        db.clear();
        assert_eq!(db.node_count(), 0);
        assert_eq!(db.edge_count(), 0);
    }
}