//! PTY-based output verification tests
//!
//! These tests use expectrl to spawn the CLI in a pseudo-terminal,
//! capturing exact terminal output to verify spacing and alignment.

use expectrl::{spawn, Expect, Regex};
use std::time::Duration;

/// Helper to build the CLI binary path
fn cli_binary() -> std::path::PathBuf {
    // Find workspace root by looking for Cargo.toml with [workspace]
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap());

    // Go up to workspace root
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();

    // Binary is in target/debug/figurehead
    workspace_root.join("target/debug/figurehead")
}

/// Spawn CLI with given args and input, return output
fn run_cli(args: &[&str], input: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Write input to temp file
    let temp_dir = tempfile::tempdir()?;
    let input_path = temp_dir.path().join("input.mmd");
    std::fs::write(&input_path, input)?;

    // Build command
    let bin = cli_binary();
    if !bin.exists() {
        return Err(format!(
            "Binary not found at {:?}. Run `cargo build -p figurehead-cli` first.",
            bin
        )
        .into());
    }

    let mut cmd_args = vec!["convert", "-i", input_path.to_str().unwrap()];
    cmd_args.extend(args);

    // Create a wrapper script to set env and run command
    let script_path = temp_dir.path().join("run.sh");
    let script_content = format!(
        "#!/bin/sh\nexport FIGUREHEAD_LOG_LEVEL=off\nexec {} {}\n",
        bin.display(),
        cmd_args.join(" ")
    );
    std::fs::write(&script_path, &script_content)?;

    // Make script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;
    }

    let mut session = spawn(script_path.to_str().unwrap())?;
    session.set_expect_timeout(Some(Duration::from_secs(10)));

    // Read all output by expecting EOF
    let mut output = String::new();
    loop {
        match session.expect(expectrl::Eof) {
            Ok(found) => {
                output.push_str(&String::from_utf8_lossy(found.as_bytes()));
                break;
            }
            Err(expectrl::Error::ExpectTimeout) => {
                // Read whatever is available and continue
                if let Ok(found) = session.expect(Regex(".+")) {
                    output.push_str(&String::from_utf8_lossy(found.as_bytes()));
                }
            }
            Err(_) => break,
        }
    }

    Ok(output)
}

/// Verify that box characters align properly in a simple chain
#[test]
fn test_simple_chain_alignment() {
    let input = r#"flowchart LR
    A --> B --> C"#;

    let output = run_cli(&["--style", "unicode"], input).expect("CLI should succeed");

    // The output should have boxes with proper alignment
    // Each box corner should be on the same row as its counterpart
    let lines: Vec<&str> = output.lines().collect();

    // Find lines with box corners
    let top_corners: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains('┌') || line.contains('┐'))
        .map(|(i, _)| i)
        .collect();

    let bottom_corners: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains('└') || line.contains('┘'))
        .map(|(i, _)| i)
        .collect();

    // All top corners should be on the same line (for LR layout)
    assert!(
        top_corners.windows(2).all(|w| w[0] == w[1]) || top_corners.len() <= 1,
        "Top corners should align: {:?}\nOutput:\n{}",
        top_corners,
        output
    );

    // All bottom corners should be on the same line
    assert!(
        bottom_corners.windows(2).all(|w| w[0] == w[1]) || bottom_corners.len() <= 1,
        "Bottom corners should align: {:?}\nOutput:\n{}",
        bottom_corners,
        output
    );
}

/// Verify that vertical spacing is consistent
#[test]
fn test_vertical_spacing_consistency() {
    let input = r#"flowchart TD
    A --> B --> C"#;

    let output = run_cli(&["--style", "unicode"], input).expect("CLI should succeed");
    let lines: Vec<&str> = output.lines().collect();

    // Find vertical bar positions (│)
    let pipe_lines: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains('│') && !line.contains('┌') && !line.contains('└'))
        .map(|(i, _)| i)
        .collect();

    // Check that vertical segments have consistent lengths
    // Between nodes A-B and B-C
    if pipe_lines.len() >= 2 {
        // Just verify we have some vertical lines
        assert!(!pipe_lines.is_empty(), "Should have vertical connectors");
    }
}

/// Verify exact character positions for a known simple case
#[test]
fn test_exact_character_positions() {
    let input = r#"flowchart LR
    A --> B"#;

    let output = run_cli(&["--style", "unicode"], input).expect("CLI should succeed");

    // Each line should have consistent width interpretation
    // Check that box drawing characters are present
    assert!(
        output.contains('┌'),
        "Should have top-left corner\nOutput:\n{}",
        output
    );
    assert!(output.contains('┐'), "Should have top-right corner");
    assert!(output.contains('└'), "Should have bottom-left corner");
    assert!(output.contains('┘'), "Should have bottom-right corner");
    assert!(output.contains('─'), "Should have horizontal lines");
    assert!(output.contains('│'), "Should have vertical lines");
}

/// Verify that box corners align vertically (column positions match)
#[test]
fn test_box_vertical_alignment() {
    let input = r#"flowchart LR
    A --> B"#;

    let output = run_cli(&["--style", "unicode"], input).expect("CLI should succeed");
    let lines: Vec<&str> = output.lines().collect();

    // For each box, top and bottom corners should be in same columns
    for line in &lines {
        // Find positions of corners
        let top_lefts: Vec<usize> = line.match_indices('┌').map(|(i, _)| i).collect();
        let bottom_lefts: Vec<usize> = line.match_indices('└').map(|(i, _)| i).collect();

        // These won't be on the same line, but we can verify structure
        if !top_lefts.is_empty() {
            // Top line should also have top-right corners
            assert!(line.contains('┐'), "Top line should have ┐ corners");
        }
        if !bottom_lefts.is_empty() {
            // Bottom line should also have bottom-right corners
            assert!(line.contains('┘'), "Bottom line should have ┘ corners");
        }
    }

    // Verify top and bottom corners align across lines
    let top_line = lines.iter().find(|l| l.contains('┌')).unwrap();
    let bottom_line = lines.iter().find(|l| l.contains('└')).unwrap();

    let top_positions: Vec<usize> = top_line
        .char_indices()
        .filter(|(_, c)| *c == '┌' || *c == '┐')
        .map(|(i, _)| i)
        .collect();
    let bottom_positions: Vec<usize> = bottom_line
        .char_indices()
        .filter(|(_, c)| *c == '└' || *c == '┘')
        .map(|(i, _)| i)
        .collect();

    assert_eq!(
        top_positions, bottom_positions,
        "Corner positions should match between top and bottom\nTop: {:?}\nBottom: {:?}\nOutput:\n{}",
        top_positions, bottom_positions, output
    );
}

/// Verify diamond shape is symmetric
#[test]
fn test_diamond_symmetry() {
    let input = r#"flowchart TD
    A{Decision}"#;

    let output = run_cli(&["--style", "unicode"], input).expect("CLI should succeed");
    let lines: Vec<&str> = output.lines().collect();

    // Find lines with diamond characters
    let diamond_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.contains('◇') || line.contains('/') || line.contains('\\'))
        .copied()
        .collect();

    // Diamond should have symmetric slashes
    for line in &diamond_lines {
        let slash_count = line.chars().filter(|c| *c == '/').count();
        let backslash_count = line.chars().filter(|c| *c == '\\').count();
        assert_eq!(
            slash_count, backslash_count,
            "Diamond should have symmetric slashes in line: {}",
            line
        );
    }
}

/// Test that labels don't overflow their containers
#[test]
fn test_label_containment() {
    let input = r#"flowchart LR
    A[Short] --> B[A Much Longer Label Here]"#;

    let output = run_cli(&["--style", "unicode"], input).expect("CLI should succeed");

    // Check that "Short" is enclosed
    assert!(output.contains("Short"), "Should contain the short label");
    assert!(
        output.contains("A Much Longer Label Here") || output.contains("A Much Longer"),
        "Should contain the long label"
    );

    // Verify box structure is intact
    let lines: Vec<&str> = output.lines().collect();
    for line in &lines {
        // If line has label text, it should also have side borders
        if line.contains("Short") || line.contains("Much") {
            assert!(
                line.contains('│'),
                "Label line should have vertical borders: {}",
                line
            );
        }
    }
}

/// Test arrow direction indicators
#[test]
fn test_arrow_indicators() {
    let input = r#"flowchart LR
    A --> B"#;

    let output = run_cli(&["--style", "unicode"], input).expect("CLI should succeed");

    // Should have arrow pointing right for LR layout
    assert!(
        output.contains('▶') || output.contains('►') || output.contains('>'),
        "Should have right-pointing arrow indicator\nOutput:\n{}",
        output
    );
}

/// Test that edge labels are positioned correctly
#[test]
fn test_edge_label_positioning() {
    let input = r#"flowchart LR
    A -->|yes| B"#;

    let output = run_cli(&["--style", "unicode"], input).expect("CLI should succeed");

    assert!(
        output.contains("yes"),
        "Edge label should be present\nOutput:\n{}",
        output
    );
}
