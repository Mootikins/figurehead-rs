//! Flowchart database implementation
//!
//! Stores flowchart diagram data including nodes with shapes,
//! edges with types and labels, and the flow direction.

use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, trace};

use crate::core::{Database, Direction, EdgeData, EdgeType, NodeData, NodeShape};

/// Flowchart database implementation
///
/// Stores nodes, edges, and metadata for flowchart diagrams.
/// Maintains insertion order for deterministic layout.
#[derive(Debug, Default)]
pub struct FlowchartDatabase {
    /// Flow direction for the diagram
    direction: Direction,
    /// Nodes indexed by ID
    nodes: HashMap<String, NodeData>,
    /// Edges in insertion order
    edges: Vec<EdgeData>,
    /// Node IDs in insertion order (for deterministic iteration)
    node_order: Vec<String>,
}

impl FlowchartDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new database with a specific direction
    pub fn with_direction(direction: Direction) -> Self {
        Self {
            direction,
            ..Default::default()
        }
    }

    /// Set the flow direction
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    /// Get the flow direction
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Check if a node exists
    pub fn has_node(&self, id: &str) -> bool {
        self.nodes.contains_key(id)
    }

    /// Get in-degree (number of incoming edges) for a node
    pub fn in_degree(&self, node_id: &str) -> usize {
        self.edges.iter().filter(|e| e.to == node_id).count()
    }

    /// Get out-degree (number of outgoing edges) for a node
    pub fn out_degree(&self, node_id: &str) -> usize {
        self.edges.iter().filter(|e| e.from == node_id).count()
    }

    /// Get IDs of nodes that this node points to
    pub fn successors(&self, node_id: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|e| e.from == node_id)
            .map(|e| e.to.as_str())
            .collect()
    }

    /// Get IDs of nodes that point to this node
    pub fn predecessors(&self, node_id: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|e| e.to == node_id)
            .map(|e| e.from.as_str())
            .collect()
    }

    /// Get source nodes (no incoming edges)
    pub fn source_nodes(&self) -> Vec<&str> {
        self.node_order
            .iter()
            .filter(|id| self.in_degree(id) == 0)
            .map(|id| id.as_str())
            .collect()
    }

    /// Get sink nodes (no outgoing edges)
    pub fn sink_nodes(&self) -> Vec<&str> {
        self.node_order
            .iter()
            .filter(|id| self.out_degree(id) == 0)
            .map(|id| id.as_str())
            .collect()
    }

    /// Topological sort using Kahn's algorithm
    /// Returns nodes in topological order, or all nodes if graph has cycles
    pub fn topological_sort(&self) -> Vec<&str> {
        trace!(node_count = self.node_count(), edge_count = self.edge_count(), "Starting topological sort");
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        // Initialize
        for id in &self.node_order {
            in_degree.insert(id.as_str(), 0);
            adjacency.insert(id.as_str(), Vec::new());
        }

        // Build adjacency and in-degree
        for edge in &self.edges {
            if let Some(deg) = in_degree.get_mut(edge.to.as_str()) {
                *deg += 1;
            }
            if let Some(adj) = adjacency.get_mut(edge.from.as_str()) {
                adj.push(edge.to.as_str());
            }
        }

        // Process nodes with in-degree 0
        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        // Sort for determinism
        queue.sort();

        let mut result = Vec::new();

        while let Some(node) = queue.pop() {
            result.push(node);

            if let Some(neighbors) = adjacency.get(node) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(neighbor);
                            queue.sort();
                        }
                    }
                }
            }
        }

        // If we didn't get all nodes, there's a cycle
        // Return what we have plus remaining nodes
        if result.len() < self.node_order.len() {
            debug!(sorted_count = result.len(), total_nodes = self.node_order.len(), "Cycle detected in graph");
            for id in &self.node_order {
                if !result.contains(&id.as_str()) {
                    result.push(id.as_str());
                }
            }
        }

        debug!(sorted_count = result.len(), "Topological sort completed");
        result
    }

    /// Get edges between two specific nodes
    pub fn edges_between(&self, from: &str, to: &str) -> Vec<&EdgeData> {
        self.edges
            .iter()
            .filter(|e| e.from == from && e.to == to)
            .collect()
    }
}

impl Database for FlowchartDatabase {
    type Node = NodeData;
    type Edge = EdgeData;

    fn add_node(&mut self, node: NodeData) -> Result<()> {
        trace!(node_id = %node.id, node_label = %node.label, node_shape = ?node.shape, "Adding node to database");
        if !self.nodes.contains_key(&node.id) {
            self.node_order.push(node.id.clone());
        }
        self.nodes.insert(node.id.clone(), node);
        debug!(node_count = self.node_count(), "Node added");
        Ok(())
    }

    fn add_edge(&mut self, edge: EdgeData) -> Result<()> {
        trace!(
            edge_from = %edge.from,
            edge_to = %edge.to,
            edge_type = ?edge.edge_type,
            edge_label = ?edge.label,
            "Adding edge to database"
        );
        self.edges.push(edge);
        debug!(edge_count = self.edge_count(), "Edge added");
        Ok(())
    }

    fn get_node(&self, id: &str) -> Option<&NodeData> {
        self.nodes.get(id)
    }

    fn nodes(&self) -> impl Iterator<Item = &NodeData> {
        self.node_order.iter().filter_map(|id| self.nodes.get(id))
    }

    fn edges(&self) -> impl Iterator<Item = &EdgeData> {
        self.edges.iter()
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.node_order.clear();
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

// Convenience methods for adding nodes/edges with less boilerplate
impl FlowchartDatabase {
    /// Add a simple node with default rectangle shape
    pub fn add_simple_node(&mut self, id: &str, label: &str) -> Result<()> {
        self.add_node(NodeData::new(id, label))
    }

    /// Add a node with a specific shape
    pub fn add_shaped_node(&mut self, id: &str, label: &str, shape: NodeShape) -> Result<()> {
        self.add_node(NodeData::with_shape(id, label, shape))
    }

    /// Add a simple edge with default arrow type
    pub fn add_simple_edge(&mut self, from: &str, to: &str) -> Result<()> {
        self.add_edge(EdgeData::new(from, to))
    }

    /// Add an edge with a specific type
    pub fn add_typed_edge(&mut self, from: &str, to: &str, edge_type: EdgeType) -> Result<()> {
        self.add_edge(EdgeData::with_type(from, to, edge_type))
    }

    /// Add an edge with a label
    pub fn add_labeled_edge(
        &mut self,
        from: &str,
        to: &str,
        edge_type: EdgeType,
        label: &str,
    ) -> Result<()> {
        self.add_edge(EdgeData::with_label(from, to, edge_type, label))
    }

    /// Ensure a node exists, creating it with default shape if not
    pub fn ensure_node(&mut self, id: &str) -> Result<()> {
        if !self.has_node(id) {
            self.add_simple_node(id, id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_basic_operations() {
        let mut db = FlowchartDatabase::new();

        db.add_simple_node("A", "Start").unwrap();
        db.add_simple_node("B", "Process").unwrap();
        db.add_simple_node("C", "End").unwrap();

        assert_eq!(db.node_count(), 3);
        assert!(db.has_node("A"));
        assert!(!db.has_node("Z"));

        let node_a = db.get_node("A").unwrap();
        assert_eq!(node_a.label, "Start");
        assert_eq!(node_a.shape, NodeShape::Rectangle);

        db.add_simple_edge("A", "B").unwrap();
        db.add_simple_edge("B", "C").unwrap();

        assert_eq!(db.edge_count(), 2);
    }

    #[test]
    fn test_node_shapes() {
        let mut db = FlowchartDatabase::new();

        db.add_shaped_node("A", "Decision", NodeShape::Diamond)
            .unwrap();
        db.add_shaped_node("B", "Process", NodeShape::RoundedRect)
            .unwrap();

        assert_eq!(db.get_node("A").unwrap().shape, NodeShape::Diamond);
        assert_eq!(db.get_node("B").unwrap().shape, NodeShape::RoundedRect);
    }

    #[test]
    fn test_edge_types_and_labels() {
        let mut db = FlowchartDatabase::new();

        db.add_simple_node("A", "A").unwrap();
        db.add_simple_node("B", "B").unwrap();
        db.add_simple_node("C", "C").unwrap();

        db.add_typed_edge("A", "B", EdgeType::DottedArrow).unwrap();
        db.add_labeled_edge("B", "C", EdgeType::ThickArrow, "Yes")
            .unwrap();

        let edges: Vec<_> = db.edges().collect();
        assert_eq!(edges[0].edge_type, EdgeType::DottedArrow);
        assert_eq!(edges[1].edge_type, EdgeType::ThickArrow);
        assert_eq!(edges[1].label, Some("Yes".to_string()));
    }

    #[test]
    fn test_direction() {
        let mut db = FlowchartDatabase::with_direction(Direction::LeftRight);
        assert_eq!(db.direction(), Direction::LeftRight);

        db.set_direction(Direction::TopDown);
        assert_eq!(db.direction(), Direction::TopDown);
    }

    #[test]
    fn test_graph_analysis() {
        let mut db = FlowchartDatabase::new();

        // A -> B -> C
        //      |
        //      v
        //      D
        db.add_simple_node("A", "A").unwrap();
        db.add_simple_node("B", "B").unwrap();
        db.add_simple_node("C", "C").unwrap();
        db.add_simple_node("D", "D").unwrap();

        db.add_simple_edge("A", "B").unwrap();
        db.add_simple_edge("B", "C").unwrap();
        db.add_simple_edge("B", "D").unwrap();

        assert_eq!(db.in_degree("A"), 0);
        assert_eq!(db.in_degree("B"), 1);
        assert_eq!(db.out_degree("B"), 2);

        assert_eq!(db.source_nodes(), vec!["A"]);
        assert!(db.sink_nodes().contains(&"C"));
        assert!(db.sink_nodes().contains(&"D"));

        assert_eq!(db.successors("B"), vec!["C", "D"]);
        assert_eq!(db.predecessors("B"), vec!["A"]);
    }

    #[test]
    fn test_topological_sort() {
        let mut db = FlowchartDatabase::new();

        db.add_simple_node("A", "A").unwrap();
        db.add_simple_node("B", "B").unwrap();
        db.add_simple_node("C", "C").unwrap();

        db.add_simple_edge("A", "B").unwrap();
        db.add_simple_edge("B", "C").unwrap();

        let sorted = db.topological_sort();
        let a_pos = sorted.iter().position(|&x| x == "A").unwrap();
        let b_pos = sorted.iter().position(|&x| x == "B").unwrap();
        let c_pos = sorted.iter().position(|&x| x == "C").unwrap();

        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_ensure_node() {
        let mut db = FlowchartDatabase::new();

        db.ensure_node("A").unwrap();
        assert!(db.has_node("A"));
        assert_eq!(db.get_node("A").unwrap().label, "A");

        // Second call should not create duplicate
        db.ensure_node("A").unwrap();
        assert_eq!(db.node_count(), 1);
    }

    #[test]
    fn test_iteration_order() {
        let mut db = FlowchartDatabase::new();

        db.add_simple_node("C", "C").unwrap();
        db.add_simple_node("A", "A").unwrap();
        db.add_simple_node("B", "B").unwrap();

        // Should iterate in insertion order
        let ids: Vec<_> = db.nodes().map(|n| n.id.as_str()).collect();
        assert_eq!(ids, vec!["C", "A", "B"]);
    }
}
