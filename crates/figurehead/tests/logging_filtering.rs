//! Example test demonstrating log filtering by component
//!
//! This test shows how to filter logs to see only specific components.

#[test]
fn test_log_filtering_example() {
    // This test demonstrates the concept of log filtering
    // In practice, you would set RUST_LOG environment variable:
    //
    // # Show only parser logs at debug level
    // RUST_LOG="figurehead::plugins::flowchart::parser=debug" cargo test
    //
    // # Show all logs at info level, but layout at trace level
    // RUST_LOG="info,figurehead::plugins::flowchart::layout=trace" cargo test
    //
    // # Show only errors
    // RUST_LOG="error" cargo test
    //
    // # Show parser and renderer, but not layout
    // RUST_LOG="figurehead::plugins::flowchart::parser=debug,figurehead::plugins::flowchart::renderer=debug" cargo test

    use figurehead::render;

    // This test just verifies the code works - actual filtering
    // would be done via environment variable
    let input = "graph LR; A-->B-->C";
    let result = render(input);
    assert!(result.is_ok());
}
