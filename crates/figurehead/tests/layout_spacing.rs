use figurehead::plugins::flowchart::{FlowchartDatabase, FlowchartLayoutAlgorithm};
use figurehead::core::{Direction, LayoutAlgorithm};

#[test]
fn test_compact_vertical_gap_is_four() {
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("A", "A").unwrap();
    db.add_simple_node("B", "B").unwrap();
    db.add_simple_edge("A", "B").unwrap();

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    let node_a = result.nodes.iter().find(|n| n.id == "A").unwrap();
    let node_b = result.nodes.iter().find(|n| n.id == "B").unwrap();

    // B should start 4 spaces after A ends (gap of 4 = rank_sep)
    assert_eq!(node_b.y, node_a.y + node_a.height + 4,
        "Vertical gap should be 4: B.y={} should equal A.y+A.height+4={}",
        node_b.y, node_a.y + node_a.height + 4);
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
fn test_td_layer_nodes_keep_natural_heights() {
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

    // TD direction keeps natural heights - no normalization
    // With Box diamond style (default), diamonds are 3 lines like rectangles
    assert_eq!(node_a.height, 3, "Rectangle should have natural height 3");
    assert_eq!(node_b.height, 3, "Diamond (Box style) should have natural height 3");
}

#[test]
fn test_lr_layer_nodes_have_same_width() {
    let mut db = FlowchartDatabase::with_direction(Direction::LeftRight);
    db.add_simple_node("A", "Hi").unwrap();
    db.add_simple_node("B", "Hello World").unwrap();
    // No edges - both in layer 0

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    let node_a = result.nodes.iter().find(|n| n.id == "A").unwrap();
    let node_b = result.nodes.iter().find(|n| n.id == "B").unwrap();

    assert_eq!(node_a.width, node_b.width,
        "Nodes in same layer should have same width: A={}, B={}",
        node_a.width, node_b.width);
}

#[test]
fn test_split_edges_are_grouped() {
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("D", "Decision").unwrap();
    db.add_simple_node("Y", "Yes").unwrap();
    db.add_simple_node("N", "No").unwrap();
    db.add_simple_edge("D", "Y").unwrap();
    db.add_simple_edge("D", "N").unwrap();

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    // Both edges from D should have group info
    let edges_from_d: Vec<_> = result.edges.iter()
        .filter(|e| e.from_id == "D")
        .collect();

    assert_eq!(edges_from_d.len(), 2);

    for edge in &edges_from_d {
        assert!(edge.junction.is_some(), "Edge to {} should have junction", edge.to_id);
        assert_eq!(edge.group_size, Some(2), "Group size should be 2");
    }

    // Group indices should be 0 and 1
    let indices: Vec<_> = edges_from_d.iter()
        .filter_map(|e| e.group_index)
        .collect();
    assert!(indices.contains(&0) && indices.contains(&1));
}

use figurehead::core::Renderer;
use figurehead::plugins::flowchart::FlowchartRenderer;

#[test]
fn test_split_renders_junction_character() {
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("D", "D").unwrap();
    db.add_simple_node("Y", "Y").unwrap();
    db.add_simple_node("N", "N").unwrap();
    db.add_simple_edge("D", "Y").unwrap();
    db.add_simple_edge("D", "N").unwrap();

    let renderer = FlowchartRenderer::new();
    let output = renderer.render(&db).unwrap();

    // Should contain a T-junction character
    assert!(
        output.contains('┬') || output.contains('┴') ||
        output.contains('├') || output.contains('┤') ||
        output.contains('+'),  // ASCII fallback
        "Output should contain junction character:\n{}", output
    );
}
