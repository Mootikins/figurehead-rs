use figurehead::plugins::flowchart::{FlowchartDatabase, FlowchartRenderer};
use figurehead::core::{Direction, EdgeType, Renderer};

fn main() {
    println!("=== LR Split ===");
    let mut db = FlowchartDatabase::with_direction(Direction::LeftRight);
    db.add_simple_node("S", "Start").unwrap();
    db.add_simple_node("A", "A").unwrap();
    db.add_simple_node("B", "B").unwrap();
    db.add_labeled_edge("S", "A", EdgeType::Arrow, "Y").unwrap();
    db.add_labeled_edge("S", "B", EdgeType::Arrow, "N").unwrap();

    let renderer = FlowchartRenderer::new();
    let output = renderer.render(&db).unwrap();
    println!("{}", output);
    
    println!("\n=== TD Split ===");
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
