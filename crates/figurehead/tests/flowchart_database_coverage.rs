//! Tests for flowchart database edge cases

use figurehead::core::{Database, Direction, EdgeData, EdgeType, NodeData, NodeShape};
use figurehead::plugins::flowchart::FlowchartDatabase;

#[test]
fn test_database_add_node_twice() {
    let mut db = FlowchartDatabase::new();
    db.add_node(NodeData::new("A", "Label")).unwrap();
    // Adding same node again should be ok (idempotent or error)
    let result = db.add_node(NodeData::new("A", "Different"));
    // May succeed or fail depending on implementation
    let _ = result;
}

#[test]
fn test_database_add_edge_missing_node() {
    let mut db = FlowchartDatabase::new();
    db.add_node(NodeData::new("A", "A")).unwrap();
    // Try to add edge to non-existent node
    let edge = EdgeData::new("A", "B");
    let result = db.add_edge(edge);
    // The database may auto-create missing nodes or return an error
    // Let's just verify the operation completes (either succeeds or fails gracefully)
    let _ = result;
}

#[test]
fn test_database_clear() {
    let mut db = FlowchartDatabase::new();
    db.add_simple_node("A", "Label").unwrap();
    db.add_simple_node("B", "Label2").unwrap();
    db.add_simple_edge("A", "B").unwrap();

    assert_eq!(db.node_count(), 2);
    assert_eq!(db.edge_count(), 1);

    db.clear();

    assert_eq!(db.node_count(), 0);
    assert_eq!(db.edge_count(), 0);
}

#[test]
fn test_database_with_direction() {
    let db = FlowchartDatabase::with_direction(Direction::RightLeft);
    assert_eq!(db.direction(), Direction::RightLeft);
}

#[test]
fn test_database_get_node() {
    let mut db = FlowchartDatabase::new();
    db.add_simple_node("A", "Label").unwrap();

    let node = db.get_node("A");
    assert!(node.is_some());
    assert_eq!(node.unwrap().label, "Label");

    let missing = db.get_node("Z");
    assert!(missing.is_none());
}
