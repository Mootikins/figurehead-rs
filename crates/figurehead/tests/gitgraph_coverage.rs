//! Comprehensive tests for git graph plugin to improve coverage

use figurehead::plugins::gitgraph::*;
use figurehead::core::{Database, Direction, Parser, Renderer, SyntaxParser, Detector, LayoutAlgorithm};
use figurehead::CharacterSet;

#[test]
fn test_gitgraph_database_with_direction() {
    let db = GitGraphDatabase::with_direction(Direction::LeftRight);
    assert_eq!(db.direction(), Direction::LeftRight);
}

#[test]
fn test_gitgraph_database_source_nodes() {
    let mut db = GitGraphDatabase::new();
    db.add_commit("c1", Some("Initial")).unwrap();
    db.add_commit("c2", Some("Second")).unwrap();
    // add_parent_edge(child, parent) creates edge from child -> parent
    // So c2 -> c1 means c2 is a source (no incoming edges)
    db.add_parent_edge("c2", "c1").unwrap();
    
    let sources = db.source_nodes();
    // Sources are nodes with in_degree == 0 (not targets of any edge)
    assert_eq!(sources.len(), 1);
    assert!(sources.contains(&"c2"));
}

#[test]
fn test_gitgraph_database_sink_nodes() {
    let mut db = GitGraphDatabase::new();
    db.add_commit("c1", Some("Initial")).unwrap();
    db.add_commit("c2", Some("Second")).unwrap();
    // add_parent_edge(child, parent) creates edge from child -> parent
    // So c2 -> c1 means c1 is a sink (no outgoing edges)
    db.add_parent_edge("c2", "c1").unwrap();
    
    let sinks = db.sink_nodes();
    // Sinks are nodes with out_degree == 0 (not sources of any edge)
    assert_eq!(sinks.len(), 1);
    assert!(sinks.contains(&"c1"));
}

#[test]
fn test_gitgraph_database_predecessors() {
    let mut db = GitGraphDatabase::new();
    db.add_commit("c1", None::<String>).unwrap();
    db.add_commit("c2", None::<String>).unwrap();
    // add_parent_edge(child, parent) creates edge from child -> parent
    // So c2 -> c1 means c1's predecessor is c2
    db.add_parent_edge("c2", "c1").unwrap();
    
    // Predecessors of c1 are nodes with edges TO c1, so c2
    let preds_c1 = db.predecessors("c1");
    assert_eq!(preds_c1.len(), 1);
    assert!(preds_c1.contains(&"c2"));
    
    // c2 has no predecessors
    let preds_c2 = db.predecessors("c2");
    assert_eq!(preds_c2.len(), 0);
}

#[test]
fn test_gitgraph_database_successors() {
    let mut db = GitGraphDatabase::new();
    db.add_commit("c1", None::<String>).unwrap();
    db.add_commit("c2", None::<String>).unwrap();
    // add_parent_edge(child, parent) creates edge from child -> parent
    // So c2 -> c1 means c2's successor is c1
    db.add_parent_edge("c2", "c1").unwrap();
    
    // Successors of c2 are nodes with edges FROM c2, so c1
    let succs_c2 = db.successors("c2");
    assert_eq!(succs_c2.len(), 1);
    assert!(succs_c2.contains(&"c1"));
    
    // c1 has no successors
    let succs_c1 = db.successors("c1");
    assert_eq!(succs_c1.len(), 0);
}

#[test]
fn test_gitgraph_database_in_degree() {
    let mut db = GitGraphDatabase::new();
    db.add_commit("c1", None::<String>).unwrap();
    db.add_commit("c2", None::<String>).unwrap();
    db.add_commit("c3", None::<String>).unwrap();
    // add_parent_edge(child, parent) creates edge from child -> parent
    // So c2 -> c1 and c3 -> c1 means c1 has 2 incoming edges
    db.add_parent_edge("c2", "c1").unwrap();
    db.add_parent_edge("c3", "c1").unwrap();
    
    // c1 has 2 incoming edges (from c2 and c3)
    assert_eq!(db.in_degree("c1"), 2);
    // c2 and c3 have no incoming edges (they are sources)
    assert_eq!(db.in_degree("c2"), 0);
    assert_eq!(db.in_degree("c3"), 0);
}

#[test]
fn test_gitgraph_database_out_degree() {
    let mut db = GitGraphDatabase::new();
    db.add_commit("c1", None::<String>).unwrap();
    db.add_commit("c2", None::<String>).unwrap();
    db.add_commit("c3", None::<String>).unwrap();
    // add_parent_edge(child, parent) creates edge from child -> parent
    db.add_parent_edge("c2", "c1").unwrap();
    db.add_parent_edge("c3", "c1").unwrap();
    
    // c2 and c3 each have 1 outgoing edge (to c1)
    assert_eq!(db.out_degree("c2"), 1);
    assert_eq!(db.out_degree("c3"), 1);
    // c1 has no outgoing edges (it's a sink)
    assert_eq!(db.out_degree("c1"), 0);
}

#[test]
fn test_gitgraph_database_clear() {
    let mut db = GitGraphDatabase::new();
    db.add_commit("c1", Some("Test")).unwrap();
    db.add_commit("c2", None::<String>).unwrap();
    db.add_parent_edge("c2", "c1").unwrap();
    
    assert_eq!(db.node_count(), 2);
    assert_eq!(db.edge_count(), 1);
    
    db.clear();
    
    assert_eq!(db.node_count(), 0);
    assert_eq!(db.edge_count(), 0);
}

#[test]
fn test_gitgraph_detector_edge_cases() {
    let detector = GitGraphDetector::new();
    
    // Test empty input
    assert!(!detector.detect(""));
    assert_eq!(detector.confidence(""), 0.0);
    
    // Test with just commit keyword (needs branch/merge/checkout too)
    assert!(!detector.detect("commit"));
    
    // Test with commit and branch (should detect)
    assert!(detector.detect("commit\n   branch develop"));
    
    // Test with mixed case
    assert!(detector.detect("GITGRAPH"));
    assert!(detector.detect("GitGraph"));
}

#[test]
fn test_gitgraph_parser_direction_parsing() {
    let parser = GitGraphParser::new();
    let mut db = GitGraphDatabase::new();
    
    // Test TD direction
    let input = "gitGraph TD\n   commit\n   commit";
    parser.parse(input, &mut db).unwrap();
    assert_eq!(db.direction(), Direction::TopDown);
    
    // Test LR direction
    let mut db2 = GitGraphDatabase::new();
    let input2 = "gitGraph LR\n   commit\n   commit";
    parser.parse(input2, &mut db2).unwrap();
    assert_eq!(db2.direction(), Direction::LeftRight);
}

#[test]
fn test_gitgraph_layout_all_directions() {
    let directions = [Direction::TopDown, Direction::BottomUp, Direction::LeftRight, Direction::RightLeft];
    
    for direction in directions {
        let mut db = GitGraphDatabase::with_direction(direction);
        db.add_commit("c1", Some("First")).unwrap();
        db.add_commit("c2", Some("Second")).unwrap();
        db.add_parent_edge("c2", "c1").unwrap();
        
        let layout = GitGraphLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();
        
        assert_eq!(result.commits.len(), 2);
        assert_eq!(result.edges.len(), 1);
        assert!(result.width > 0);
        assert!(result.height > 0);
    }
}

#[test]
fn test_gitgraph_layout_empty() {
    let db = GitGraphDatabase::new();
    let layout = GitGraphLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();
    
    assert_eq!(result.commits.len(), 0);
    assert_eq!(result.edges.len(), 0);
    assert_eq!(result.width, 0);
    assert_eq!(result.height, 0);
}

#[test]
fn test_gitgraph_renderer_all_styles() {
    let mut db = GitGraphDatabase::new();
    db.add_commit("c1", Some("Initial")).unwrap();
    db.add_commit("c2", Some("Feature")).unwrap();
    db.add_parent_edge("c2", "c1").unwrap();
    
    let styles = [CharacterSet::Ascii, CharacterSet::Unicode, CharacterSet::Compact, CharacterSet::UnicodeMath];
    
    for style in styles {
        let renderer = GitGraphRenderer::with_style(style);
        let result = renderer.render(&db);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }
}

#[test]
fn test_gitgraph_renderer_empty() {
    let db = GitGraphDatabase::new();
    let renderer = GitGraphRenderer::new();
    let result = renderer.render(&db).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_gitgraph_syntax_parser_commit_with_all_attributes() {
    let parser = GitGraphSyntaxParser::new();
    
    let input = r#"gitGraph
   commit id: "Alpha" type: HIGHLIGHT tag: "v1.0"
   commit id: "Beta" type: REVERSE tag: "v2.0"
   commit id: "Gamma" type: NORMAL"#;
    
    let nodes = parser.parse(input).unwrap();
    assert!(!nodes.is_empty());
}

#[test]
fn test_gitgraph_syntax_parser_complex_branching() {
    let parser = GitGraphSyntaxParser::new();
    
    let input = r#"gitGraph
   commit
   branch feature
   checkout feature
   commit
   commit
   checkout main
   commit
   merge feature"#;
    
    let nodes = parser.parse(input).unwrap();
    // Should have commits and edges
    assert!(!nodes.is_empty());
}

#[test]
fn test_gitgraph_syntax_parser_switch_keyword() {
    let parser = GitGraphSyntaxParser::new();
    
    let input = r#"gitGraph
   commit
   branch develop
   switch develop
   commit"#;
    
    let nodes = parser.parse(input).unwrap();
    assert!(!nodes.is_empty());
}

#[test]
fn test_gitgraph_syntax_parser_comments() {
    let parser = GitGraphSyntaxParser::new();
    
    let input = r#"gitGraph
   // This is a comment
   commit
   # Another comment
   commit"#;
    
    let nodes = parser.parse(input).unwrap();
    // Should parse commits, ignoring comments
    assert!(!nodes.is_empty());
}

#[test]
fn test_gitgraph_syntax_parser_empty_input() {
    let parser = GitGraphSyntaxParser::new();
    let nodes = parser.parse("").unwrap();
    assert!(nodes.is_empty());
}

#[test]
fn test_gitgraph_syntax_parser_just_gitgraph_keyword() {
    let parser = GitGraphSyntaxParser::new();
    let nodes = parser.parse("gitGraph").unwrap();
    // Should return empty (no commits)
    assert!(nodes.is_empty());
}

#[test]
fn test_gitgraph_syntax_parser_can_parse() {
    let parser = GitGraphSyntaxParser::new();
    assert!(parser.can_parse("gitGraph\n   commit"));
    assert!(!parser.can_parse("graph TD"));
}
