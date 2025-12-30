//! Snapshot tests for ASCII rendering output
//!
//! These tests compare rendered output against golden files in tests/fixtures/.
//! To update fixtures after fixing rendering, run the tests with UPDATE_FIXTURES=1

use figurehead::render;
use std::fs;
use std::path::Path;

/// Compare rendered output to a fixture file
fn assert_fixture(name: &str, input: &str) {
    let output = render(input).expect("render should succeed");
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{}.txt", name));

    if std::env::var("UPDATE_FIXTURES").is_ok() {
        fs::write(&fixture_path, &output).expect("failed to write fixture");
        println!("Updated fixture: {}", fixture_path.display());
        return;
    }

    let expected = fs::read_to_string(&fixture_path).unwrap_or_else(|_| {
        panic!(
            "Fixture not found: {}\nRun with UPDATE_FIXTURES=1 to create it.\n\nActual output:\n{}",
            fixture_path.display(),
            output
        )
    });

    if output != expected {
        panic!(
            "Snapshot mismatch for '{}'!\n\n=== Expected ===\n{}\n=== Actual ===\n{}\n=== Diff ===\nRun with UPDATE_FIXTURES=1 to update.",
            name, expected, output
        );
    }
}

#[test]
fn test_simple_chain_lr() {
    assert_fixture("simple_chain_lr", "graph LR; A-->B-->C");
}

#[test]
fn test_simple_chain_td() {
    assert_fixture("simple_chain_td", "graph TD; A-->B-->C");
}

#[test]
fn test_diamond_decision_td() {
    assert_fixture(
        "diamond_decision_td",
        "graph TD; A[Start]-->B{Decision}-->C[End]",
    );
}

#[test]
fn test_diamond_decision_lr() {
    assert_fixture(
        "diamond_decision_lr",
        "graph LR; A[Start]-->B{Decision}-->C[End]",
    );
}

#[test]
fn test_complex_flowchart() {
    assert_fixture(
        "complex_flowchart",
        r#"graph LR
            A[Start] --> B{Decision}
            B -->|Yes| C[Process 1]
            B -->|No| D[Process 2]
            C --> E[End]
            D --> E"#,
    );
}

#[test]
fn test_all_shapes() {
    assert_fixture(
        "all_shapes",
        r#"graph TD
            A[Rectangle]
            B(Rounded)
            C{Diamond}
            D((Circle))
            E[[Subroutine]]
            F{{Hexagon}}
            G[(Cylinder)]
            H[/Parallelogram/]
            I[/Trapezoid\\]"#,
    );
}

#[test]
fn test_labeled_edges() {
    assert_fixture(
        "labeled_edges",
        "graph TD; A-->|yes|B; A-->|no|C",
    );
}

#[test]
fn test_asymmetric_shape() {
    assert_fixture("asymmetric_shape", "graph LR; A>Flag]");
}

#[test]
fn test_long_labels() {
    assert_fixture(
        "long_labels",
        "graph LR; A[This is a very long label]-->B[Another long label here]",
    );
}

#[test]
fn test_subgraph_td() {
    assert_fixture(
        "subgraph_td",
        r#"graph TD
            subgraph "Group"
                A --> B
                B --> C
            end
            C --> D"#,
    );
}

#[test]
fn test_subgraph_lr() {
    assert_fixture(
        "subgraph_lr",
        r#"graph LR
            subgraph "Services"
                API --> DB
            end
            Client --> API
            DB --> Backup"#,
    );
}

#[test]
fn test_subgraph_multiple() {
    assert_fixture(
        "subgraph_multiple",
        r#"graph TD
            subgraph "Alpha"
                A --> B
            end
            subgraph "Beta"
                C --> D
            end
            B --> C"#,
    );
}

// =============================================================================
// Git Graph Snapshots
// =============================================================================

#[test]
fn test_gitgraph_simple_td() {
    assert_fixture(
        "gitgraph_simple_td",
        r#"gitGraph
   commit
   commit
   commit"#,
    );
}

#[test]
fn test_gitgraph_simple_lr() {
    assert_fixture(
        "gitgraph_simple_lr",
        r#"gitGraph LR
   commit
   commit
   commit"#,
    );
}

#[test]
fn test_gitgraph_with_ids() {
    assert_fixture(
        "gitgraph_with_ids",
        r#"gitGraph
   commit id: "Initial"
   commit id: "Feature"
   commit id: "Release""#,
    );
}

#[test]
fn test_gitgraph_with_branch() {
    assert_fixture(
        "gitgraph_with_branch",
        r#"gitGraph
   commit
   branch develop
   checkout develop
   commit
   checkout main
   commit"#,
    );
}

// =============================================================================
// Sequence Diagram Snapshots
// =============================================================================

#[test]
fn test_sequence_simple() {
    assert_fixture(
        "sequence_simple",
        r#"sequenceDiagram
    Alice->>Bob: Hello
    Bob-->>Alice: Hi"#,
    );
}

#[test]
fn test_sequence_three_participants() {
    assert_fixture(
        "sequence_three_participants",
        r#"sequenceDiagram
    Alice->>Bob: Hello
    Bob->>Charlie: Hi there
    Charlie-->>Alice: Hey!"#,
    );
}

#[test]
fn test_sequence_with_aliases() {
    assert_fixture(
        "sequence_with_aliases",
        r#"sequenceDiagram
    participant A as Alice
    participant B as Bob
    A->>B: Hello Bob!
    B-->>A: Hi Alice!"#,
    );
}

#[test]
fn test_sequence_open_arrows() {
    assert_fixture(
        "sequence_open_arrows",
        r#"sequenceDiagram
    Alice->Bob: Sync call
    Bob-->Alice: Sync response"#,
    );
}

// =============================================================================
// Class Diagram Snapshots
// =============================================================================

#[test]
fn test_class_simple() {
    assert_fixture(
        "class_simple",
        r#"classDiagram
    class Animal"#,
    );
}

#[test]
fn test_class_with_attributes() {
    assert_fixture(
        "class_with_attributes",
        r#"classDiagram
    class Animal {
        +name: string
        -age: int
    }"#,
    );
}

#[test]
fn test_class_with_methods() {
    assert_fixture(
        "class_with_methods",
        r#"classDiagram
    class Animal {
        +name: string
        +eat()
        +sleep(): void
        #digest()*
    }"#,
    );
}

#[test]
fn test_class_multiple() {
    assert_fixture(
        "class_multiple",
        r#"classDiagram
    class Animal {
        +name
    }
    class Dog {
        +breed
    }"#,
    );
}
