//! Edge case tests for diagram parsing and rendering
//!
//! Tests for boundary conditions, unusual inputs, and error handling.

// =============================================================================
// Empty Input Tests
// =============================================================================

mod empty_inputs {
    use figurehead::core::Parser;
    use figurehead::plugins::class::*;
    use figurehead::plugins::sequence::*;

    #[test]
    fn test_sequence_empty_database() {
        let db = SequenceDatabase::new();
        let renderer = SequenceRenderer::new();
        let result = renderer.render(&db).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_sequence_parser_empty_input() {
        let mut db = SequenceDatabase::new();
        let parser = SequenceParser::new();
        let result = parser.parse("", &mut db);
        assert!(result.is_ok());
        assert!(db.participants().is_empty());
    }

    #[test]
    fn test_sequence_parser_whitespace_only() {
        let mut db = SequenceDatabase::new();
        let parser = SequenceParser::new();
        let result = parser.parse("   \n\n  \t  \n", &mut db);
        assert!(result.is_ok());
        assert!(db.participants().is_empty());
    }

    #[test]
    fn test_class_empty_database() {
        let db = ClassDatabase::new();
        let renderer = ClassRenderer::new();
        let result = renderer.render_database(&db).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_class_parser_empty_input() {
        let mut db = ClassDatabase::new();
        let parser = ClassParser::new();
        let result = parser.parse("", &mut db);
        assert!(result.is_ok());
        assert!(db.classes().is_empty());
    }

    #[test]
    fn test_class_parser_whitespace_only() {
        let mut db = ClassDatabase::new();
        let parser = ClassParser::new();
        let result = parser.parse("   \n\n  \t  \n", &mut db);
        assert!(result.is_ok());
        assert!(db.classes().is_empty());
    }
}

// =============================================================================
// Unicode and Special Character Tests
// =============================================================================

mod unicode_handling {
    use figurehead::core::{Database, Parser};
    use figurehead::plugins::class::*;
    use figurehead::plugins::flowchart::*;

    #[test]
    fn test_flowchart_emoji_labels() {
        let input = r#"
            graph LR
            A[🎉 Party] --> B[🚀 Launch]
        "#;
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        assert_eq!(db.nodes().count(), 2);
        let labels: Vec<_> = db.nodes().map(|n| n.label.as_str()).collect();
        assert!(labels.contains(&"🎉 Party"));
        assert!(labels.contains(&"🚀 Launch"));
    }

    #[test]
    fn test_flowchart_cjk_full_width() {
        let input = r#"
            graph TD
            A[中文标签] --> B[日本語]
            B --> C[한국어]
        "#;
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        assert_eq!(db.nodes().count(), 3);
    }

    #[test]
    fn test_flowchart_mixed_scripts() {
        let input = r#"
            graph LR
            A[Hello世界🌍] --> B[Привет мир]
        "#;
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        assert_eq!(db.nodes().count(), 2);
    }

    #[test]
    fn test_class_unicode_names() {
        let mut db = ClassDatabase::new();
        let mut class = Class::new("数据库管理器");
        class.add_method(Member::method("连接"));
        db.add_class(class).unwrap();

        let renderer = ClassRenderer::new();
        let output = renderer.render_database(&db).unwrap();

        assert!(output.contains("数据库管理器"));
    }

    #[test]
    fn test_flowchart_edge_label_unicode() {
        let input = r#"
            graph LR
            A -->|処理中| B
        "#;
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        assert_eq!(db.edges().count(), 1);
    }
}

// =============================================================================
// Very Long Labels Tests
// =============================================================================

mod long_labels {
    use figurehead::core::{Parser, Renderer};
    use figurehead::plugins::class::*;
    use figurehead::plugins::flowchart::*;

    #[test]
    fn test_flowchart_unwrappable_long_label() {
        // Label with no spaces - cannot be wrapped
        let input = r#"
            graph TD
            A[ThisIsAVeryLongLabelWithNoSpacesToWrapAtAllAndShouldStillWork]
        "#;
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        let renderer = FlowchartRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.contains("ThisIsAVeryLong"));
    }

    #[test]
    fn test_flowchart_very_long_edge_label() {
        let input = r#"
            graph LR
            A -->|This is a very long edge label that should still be handled| B
        "#;
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        let renderer = FlowchartRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.contains("A"));
        assert!(output.contains("B"));
    }

    #[test]
    fn test_class_long_method_signature() {
        let mut db = ClassDatabase::new();
        let mut class = Class::new("Service");
        class.add_method(
            Member::method("processRequestWithVeryLongMethodNameAndManyParameters")
                .with_type("ResponseWithVeryLongTypeName"),
        );
        db.add_class(class).unwrap();

        let renderer = ClassRenderer::new();
        let output = renderer.render_database(&db).unwrap();

        assert!(output.contains("processRequest"));
    }
}

// =============================================================================
// Malformed Syntax Tests
// =============================================================================

mod malformed_syntax {
    use figurehead::core::Parser;
    use figurehead::plugins::class::*;
    use figurehead::plugins::gitgraph::*;
    use figurehead::plugins::sequence::*;

    #[test]
    fn test_gitgraph_missing_commit_id_value() {
        let input = "gitgraph\ncommit id:\n";
        let mut db = GitGraphDatabase::new();
        let parser = GitGraphParser::new();
        // Should not panic, even if invalid
        let _ = parser.parse(input, &mut db);
    }

    #[test]
    fn test_gitgraph_branch_without_name() {
        let input = "gitgraph\nbranch\n";
        let mut db = GitGraphDatabase::new();
        let parser = GitGraphParser::new();
        let _ = parser.parse(input, &mut db);
    }

    #[test]
    fn test_gitgraph_checkout_nonexistent() {
        let input = "gitgraph\ncheckout nonexistent_branch\n";
        let mut db = GitGraphDatabase::new();
        let parser = GitGraphParser::new();
        let _ = parser.parse(input, &mut db);
    }

    #[test]
    fn test_gitgraph_merge_without_branch() {
        let input = "gitgraph\nmerge\n";
        let mut db = GitGraphDatabase::new();
        let parser = GitGraphParser::new();
        let _ = parser.parse(input, &mut db);
    }

    #[test]
    fn test_sequence_missing_target() {
        let input = "sequenceDiagram\nAlice->>:\n";
        let mut db = SequenceDatabase::new();
        let parser = SequenceParser::new();
        // Should not panic
        let _ = parser.parse(input, &mut db);
    }

    #[test]
    fn test_sequence_participant_without_name() {
        let input = "sequenceDiagram\nparticipant\n";
        let mut db = SequenceDatabase::new();
        let parser = SequenceParser::new();
        let _ = parser.parse(input, &mut db);
    }

    #[test]
    fn test_class_missing_name() {
        let input = "classDiagram\nclass {\n}\n";
        let mut db = ClassDatabase::new();
        let parser = ClassParser::new();
        let _ = parser.parse(input, &mut db);
    }

    #[test]
    fn test_class_unclosed_body() {
        let input = "classDiagram\nclass Foo {\n+method()\n";
        let mut db = ClassDatabase::new();
        let parser = ClassParser::new();
        let _ = parser.parse(input, &mut db);
    }
}

// =============================================================================
// Deeply Nested Structure Tests
// =============================================================================

mod deep_nesting {
    use figurehead::core::{Database, Parser, Renderer};
    use figurehead::plugins::flowchart::*;
    use figurehead::plugins::gitgraph::*;

    #[test]
    fn test_flowchart_subgraph() {
        // Subgraph syntax uses quoted names
        let input = r#"
            graph TD
            subgraph "Group"
                A --> B
            end
            B --> C
        "#;
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        assert_eq!(db.subgraphs().count(), 1);
        assert!(db.nodes().count() >= 3);
    }

    #[test]
    fn test_flowchart_long_chain() {
        // Create a chain of 20 nodes
        let mut chain = "graph TD\n".to_string();
        for i in 0..19 {
            chain.push_str(&format!("N{} --> N{}\n", i, i + 1));
        }

        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(&chain, &mut db).unwrap();

        assert_eq!(db.nodes().count(), 20);

        let renderer = FlowchartRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.contains("N0"));
        assert!(output.contains("N19"));
    }

    #[test]
    fn test_flowchart_wide_graph() {
        // Create a fan-out pattern: A -> B1, B2, B3, ... B10
        let mut input = "graph TD\n".to_string();
        for i in 1..=10 {
            input.push_str(&format!("A --> B{}\n", i));
        }

        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(&input, &mut db).unwrap();

        assert_eq!(db.nodes().count(), 11); // A + B1..B10
    }

    #[test]
    fn test_gitgraph_many_commits() {
        let input = r#"
            gitgraph
            commit
            branch feature1
            commit
            checkout main
            branch feature2
            commit
            checkout main
            branch feature3
            commit
            checkout main
            merge feature1
            merge feature2
        "#;

        let mut db = GitGraphDatabase::new();
        let parser = GitGraphParser::new();
        parser.parse(input, &mut db).unwrap();

        // Should have multiple nodes (commits)
        assert!(db.nodes().count() >= 4);
    }

    #[test]
    fn test_gitgraph_deep_history() {
        let mut input = "gitgraph\n".to_string();
        for i in 0..20 {
            input.push_str(&format!("commit id: \"c{}\"\n", i));
        }

        let mut db = GitGraphDatabase::new();
        let parser = GitGraphParser::new();
        parser.parse(&input, &mut db).unwrap();

        assert_eq!(db.nodes().count(), 20);
    }
}

// =============================================================================
// Layout Boundary Tests
// =============================================================================

mod layout_boundaries {
    use figurehead::core::{Database, Direction, Parser, Renderer};
    use figurehead::plugins::flowchart::*;

    #[test]
    fn test_single_node_layout() {
        let input = "graph TD\nA[Solo Node]";
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        let renderer = FlowchartRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.contains("Solo Node"));
    }

    #[test]
    fn test_empty_label_node() {
        // Note: Empty labels like A[] are not currently supported.
        // Use a space as minimal label.
        let input = "graph TD\nA[ ] --> B[Has Label]";
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        let renderer = FlowchartRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.contains("Has Label"));
    }

    #[test]
    fn test_disconnected_multiple_nodes() {
        let input = "graph TD\nA[One]\nB[Two]\nC[Three]";
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        let renderer = FlowchartRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.contains("One"));
        assert!(output.contains("Two"));
        assert!(output.contains("Three"));
    }

    #[test]
    fn test_self_loop_rendering() {
        let input = "graph TD\nA[Loop] --> A";
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        let renderer = FlowchartRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.contains("Loop"));
    }

    #[test]
    fn test_bidirectional_edge() {
        let input = "graph LR\nA --> B\nB --> A";
        let mut db = FlowchartDatabase::new();
        let parser = FlowchartParser::new();
        parser.parse(input, &mut db).unwrap();

        assert_eq!(db.edges().count(), 2);
    }

    #[test]
    fn test_all_directions() {
        for (dir, direction) in [
            ("TD", Direction::TopDown),
            ("LR", Direction::LeftRight),
            ("BT", Direction::BottomUp),
            ("RL", Direction::RightLeft),
        ] {
            let input = format!("graph {}\nA --> B", dir);
            let mut db = FlowchartDatabase::new();
            let parser = FlowchartParser::new();
            parser.parse(&input, &mut db).unwrap();

            assert_eq!(db.direction(), direction);

            let renderer = FlowchartRenderer::new();
            let output = renderer.render(&db).unwrap();
            assert!(!output.is_empty());
        }
    }
}
