use figurehead::core::{Direction, EdgeType, LayoutAlgorithm, Renderer};
use figurehead::plugins::flowchart::{
    FlowchartDatabase, FlowchartLayoutAlgorithm, FlowchartRenderer,
};

fn main() {
    let mut db = FlowchartDatabase::with_direction(Direction::LeftRight);
    db.add_simple_node("S", "Start").unwrap();
    db.add_simple_node("A", "A").unwrap();
    db.add_simple_node("B", "B").unwrap();
    db.add_labeled_edge("S", "A", EdgeType::Arrow, "Y").unwrap();
    db.add_labeled_edge("S", "B", EdgeType::Arrow, "N").unwrap();

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    println!("=== Node positions ===");
    for node in &result.nodes {
        println!(
            "{}: x={}, y={}, w={}, h={}",
            node.id, node.x, node.y, node.width, node.height
        );
    }

    println!("\n=== Edge positions ===");
    for edge in &result.edges {
        println!("{} -> {}: {:?}", edge.from_id, edge.to_id, edge.waypoints);
        if let Some(j) = edge.junction {
            println!("  junction: {:?}", j);
        }
    }

    let renderer = FlowchartRenderer::new();
    let output = renderer.render(&db).unwrap();
    println!("\n=== Output ===\n{}", output);
}
