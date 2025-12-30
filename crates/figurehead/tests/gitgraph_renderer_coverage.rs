//! Tests for git graph renderer edge cases

use figurehead::core::{CharacterSet, Database, Direction, LayoutAlgorithm, Parser, Renderer};
use figurehead::plugins::gitgraph::*;

#[test]
fn test_gitgraph_renderer_horizontal_layout() {
    let mut db = GitGraphDatabase::with_direction(Direction::LeftRight);
    db.add_commit("c1", Some("First")).unwrap();
    db.add_commit("c2", Some("Second")).unwrap();
    db.add_parent_edge("c2", "c1").unwrap();

    let renderer = GitGraphRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_gitgraph_renderer_bottom_up_layout() {
    let mut db = GitGraphDatabase::with_direction(Direction::BottomUp);
    db.add_commit("c1", Some("First")).unwrap();
    db.add_commit("c2", Some("Second")).unwrap();
    db.add_parent_edge("c2", "c1").unwrap();

    let renderer = GitGraphRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_gitgraph_renderer_right_left_layout() {
    let mut db = GitGraphDatabase::with_direction(Direction::RightLeft);
    db.add_commit("c1", Some("First")).unwrap();
    db.add_commit("c2", Some("Second")).unwrap();
    db.add_parent_edge("c2", "c1").unwrap();

    let renderer = GitGraphRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_gitgraph_renderer_multiple_commits() {
    let mut db = GitGraphDatabase::new();
    db.add_commit("c1", Some("Initial")).unwrap();
    db.add_commit("c2", Some("Feature")).unwrap();
    db.add_commit("c3", Some("Fix")).unwrap();
    db.add_parent_edge("c2", "c1").unwrap();
    db.add_parent_edge("c3", "c2").unwrap();

    let renderer = GitGraphRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_gitgraph_renderer_renderer_properties() {
    let renderer = GitGraphRenderer::new();
    assert_eq!(renderer.name(), "ascii");
    assert_eq!(renderer.version(), "0.1.0");
    assert_eq!(renderer.format(), "ascii");
}

#[test]
fn test_gitgraph_canvas_edge_cases() {
    // Test canvas with edge drawing that covers different paths
    let mut db = GitGraphDatabase::new();
    db.add_commit("c1", Some("A")).unwrap();
    db.add_commit("c2", Some("B")).unwrap();
    db.add_commit("c3", Some("C")).unwrap();
    db.add_parent_edge("c2", "c1").unwrap();
    db.add_parent_edge("c3", "c2").unwrap();

    let renderer = GitGraphRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(!result.is_empty());
}
