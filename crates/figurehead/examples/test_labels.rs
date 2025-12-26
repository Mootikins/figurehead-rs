use figurehead::plugins::flowchart::{FlowchartDatabase, FlowchartRenderer};
use figurehead::core::{Direction, EdgeType, Renderer};

fn main() {
    println!("=== Labeled Edges TD ===");
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("A", "A").unwrap();
    db.add_simple_node("B", "B").unwrap();
    db.add_simple_node("C", "C").unwrap();
    db.add_labeled_edge("A", "B", EdgeType::Arrow, "yes").unwrap();
    db.add_labeled_edge("A", "C", EdgeType::Arrow, "no").unwrap();

    let renderer = FlowchartRenderer::new();
    let output = renderer.render(&db).unwrap();
    println!("{}", output);
}
