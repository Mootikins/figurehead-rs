//! State diagram database implementation
//!
//! Stores states and transitions for state diagrams using core types.

use crate::core::{Database, EdgeData, NodeData, NodeShape};
use anyhow::Result;

/// State diagram database using core NodeData and EdgeData
#[derive(Debug, Default)]
pub struct StateDatabase {
    states: Vec<NodeData>,
    transitions: Vec<EdgeData>,
}

impl StateDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a state
    pub fn add_state(&mut self, state: NodeData) -> Result<()> {
        // Don't add duplicates
        if !self.states.iter().any(|s| s.id == state.id) {
            self.states.push(state);
        }
        Ok(())
    }

    /// Ensure a state exists (creates implicit state if needed)
    pub fn ensure_state(&mut self, id: &str) -> Result<()> {
        if !self.states.iter().any(|s| s.id == id) {
            let shape = if id == "[*]" {
                NodeShape::Terminal
            } else {
                NodeShape::Rectangle
            };
            self.states.push(NodeData::with_shape(id, id, shape));
        }
        Ok(())
    }

    /// Add a transition
    pub fn add_transition(&mut self, transition: EdgeData) -> Result<()> {
        // Ensure states exist
        self.ensure_state(&transition.from)?;
        self.ensure_state(&transition.to)?;
        self.transitions.push(transition);
        Ok(())
    }

    /// Get all states
    pub fn states(&self) -> &[NodeData] {
        &self.states
    }

    /// Get all transitions
    pub fn transitions(&self) -> &[EdgeData] {
        &self.transitions
    }

    /// Get state count
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    /// Get transition count
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    /// Get state index (for layout)
    pub fn state_index(&self, id: &str) -> Option<usize> {
        self.states.iter().position(|s| s.id == id)
    }

    /// Clear all data
    pub fn clear_all(&mut self) {
        self.states.clear();
        self.transitions.clear();
    }
}

impl Database for StateDatabase {
    type Node = NodeData;
    type Edge = EdgeData;

    fn add_node(&mut self, node: Self::Node) -> Result<()> {
        self.add_state(node)
    }

    fn add_edge(&mut self, edge: Self::Edge) -> Result<()> {
        self.add_transition(edge)
    }

    fn get_node(&self, id: &str) -> Option<&Self::Node> {
        self.states.iter().find(|s| s.id == id)
    }

    fn nodes(&self) -> impl Iterator<Item = &Self::Node> {
        self.states.iter()
    }

    fn edges(&self) -> impl Iterator<Item = &Self::Edge> {
        self.transitions.iter()
    }

    fn clear(&mut self) {
        self.clear_all()
    }

    fn node_count(&self) -> usize {
        self.state_count()
    }

    fn edge_count(&self) -> usize {
        self.transition_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EdgeType;

    #[test]
    fn test_add_state() {
        let mut db = StateDatabase::new();
        db.add_state(NodeData::new("Idle", "Idle")).unwrap();
        db.add_state(NodeData::new("Running", "Running")).unwrap();
        assert_eq!(db.state_count(), 2);
    }

    #[test]
    fn test_no_duplicate_states() {
        let mut db = StateDatabase::new();
        db.add_state(NodeData::new("Idle", "Idle")).unwrap();
        db.add_state(NodeData::new("Idle", "Idle")).unwrap();
        assert_eq!(db.state_count(), 1);
    }

    #[test]
    fn test_add_transition_creates_implicit_states() {
        let mut db = StateDatabase::new();
        db.add_transition(EdgeData::new("Idle", "Running")).unwrap();
        assert_eq!(db.state_count(), 2);
        assert_eq!(db.transition_count(), 1);
    }

    #[test]
    fn test_terminal_state_shape() {
        let mut db = StateDatabase::new();
        db.ensure_state("[*]").unwrap();
        db.ensure_state("Normal").unwrap();

        let terminal = db.get_node("[*]").unwrap();
        let normal = db.get_node("Normal").unwrap();

        assert_eq!(terminal.shape, NodeShape::Terminal);
        assert_eq!(normal.shape, NodeShape::Rectangle);
    }

    #[test]
    fn test_transition_with_label() {
        let mut db = StateDatabase::new();
        let edge = EdgeData::with_label("Idle", "Running", EdgeType::Arrow, "start");
        db.add_transition(edge).unwrap();

        let transition = &db.transitions()[0];
        assert_eq!(transition.label, Some("start".to_string()));
    }
}
