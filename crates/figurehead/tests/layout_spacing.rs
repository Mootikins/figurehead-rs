use figurehead::plugins::flowchart::{FlowchartDatabase, FlowchartLayoutAlgorithm};
use figurehead::core::{Direction, LayoutAlgorithm};

#[test]
fn test_compact_vertical_gap_is_three() {
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("A", "A").unwrap();
    db.add_simple_node("B", "B").unwrap();
    db.add_simple_edge("A", "B").unwrap();

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    let node_a = result.nodes.iter().find(|n| n.id == "A").unwrap();
    let node_b = result.nodes.iter().find(|n| n.id == "B").unwrap();

    // B should start 3 spaces after A ends (gap of 3)
    assert_eq!(node_b.y, node_a.y + node_a.height + 3,
        "Vertical gap should be 3: B.y={} should equal A.y+A.height+3={}",
        node_b.y, node_a.y + node_a.height + 3);
}

#[test]
fn test_compact_horizontal_gap_is_one() {
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("A", "A").unwrap();
    db.add_simple_node("B", "B").unwrap();
    // No edges - both in same layer

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    let node_a = result.nodes.iter().find(|n| n.id == "A").unwrap();
    let node_b = result.nodes.iter().find(|n| n.id == "B").unwrap();

    // Horizontal gap should be 1
    let gap = if node_a.x < node_b.x {
        node_b.x - (node_a.x + node_a.width)
    } else {
        node_a.x - (node_b.x + node_b.width)
    };
    assert_eq!(gap, 1, "Horizontal gap should be 1, got {}", gap);
}

#[test]
fn test_td_layer_nodes_have_same_height() {
    use figurehead::core::NodeShape;

    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    // Rectangle has height 3, Diamond has height 5 (3 + extra_height of 2)
    db.add_shaped_node("A", "Rectangle", NodeShape::Rectangle).unwrap();
    db.add_shaped_node("B", "Diamond", NodeShape::Diamond).unwrap();
    // No edges - both in layer 0

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    let node_a = result.nodes.iter().find(|n| n.id == "A").unwrap();
    let node_b = result.nodes.iter().find(|n| n.id == "B").unwrap();

    assert_eq!(node_a.height, node_b.height,
        "Nodes in same layer should have same height: A={}, B={}",
        node_a.height, node_b.height);
}
