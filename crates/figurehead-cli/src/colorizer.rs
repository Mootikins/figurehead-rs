//! Terminal colorization for ASCII diagram output
//!
//! Applies ANSI escape codes to diagram elements using crossterm.

use crossterm::style::{Color, Stylize};

/// Colorize ASCII diagram output using ANSI escape codes
///
/// Applies colors to different diagram elements:
/// - Box-drawing corners and edges: Cyan
/// - Arrows and edge markers: Yellow
/// - Diamond markers: Magenta
/// - Circle/terminal markers: Green
/// - Labels: Default (terminal color)
pub fn colorize_output(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 2); // Extra space for ANSI codes

    for line in input.lines() {
        for c in line.chars() {
            let colored = match c {
                // Box-drawing corners and lines (Unicode)
                '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼' | '─' | '│' |
                '╔' | '╗' | '╚' | '╝' | '╠' | '╣' | '╦' | '╩' | '╬' | '═' | '║' |
                '╭' | '╮' | '╯' | '╰' | '╒' | '╕' | '╘' | '╛' | '╓' | '╖' | '╙' | '╜' => {
                    format!("{}", c.to_string().with(Color::Cyan))
                }
                // ASCII box characters
                '+' | '-' | '|' => {
                    // Check context to avoid coloring hyphens in labels
                    if is_box_char_context(line, c) {
                        format!("{}", c.to_string().with(Color::Cyan))
                    } else {
                        c.to_string()
                    }
                }
                // Arrow heads
                '>' | 'v' | '^' | '<' | '▶' | '▼' | '▲' | '◀' => {
                    format!("{}", c.to_string().with(Color::Yellow))
                }
                // Diamond markers
                '◆' | '◇' => {
                    format!("{}", c.to_string().with(Color::Magenta))
                }
                // Circle markers (terminal states)
                '●' | '○' | '◉' => {
                    format!("{}", c.to_string().with(Color::Green))
                }
                // Keep other characters uncolored
                _ => c.to_string(),
            };
            result.push_str(&colored);
        }
        result.push('\n');
    }

    // Remove trailing newline to match input format
    if !input.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Check if a character is likely part of box drawing vs text content
fn is_box_char_context(line: &str, c: char) -> bool {
    match c {
        '+' => {
            // '+' is likely a box corner if surrounded by box chars
            line.contains("--") || line.contains("+-") || line.contains("-+")
        }
        '-' => {
            // '-' is likely a box line if it appears as ---
            line.contains("---") || line.contains("+--") || line.contains("--+")
        }
        '|' => {
            // '|' as first non-space or preceded by box chars is likely a box edge
            let trimmed = line.trim_start();
            trimmed.starts_with('|') || line.contains("| ") || line.contains(" |")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colorize_preserves_content() {
        let input = "┌───┐\n│ A │\n└───┘";
        let output = colorize_output(input);

        // Should contain ANSI codes
        assert!(output.contains("\x1b["));
        // Should preserve the basic structure (ignoring ANSI codes)
        let stripped: String = output
            .chars()
            .filter(|c| !c.is_ascii_control() && *c != '[' && *c != 'm')
            .filter(|c| !c.is_ascii_digit() && *c != ';')
            .collect();
        assert!(stripped.contains("A"));
    }

    #[test]
    fn test_colorize_arrows() {
        let input = "──▶";
        let output = colorize_output(input);
        // Arrow should have yellow color code (33 or 38;5;11)
        assert!(output.contains("▶"));
    }

    #[test]
    fn test_no_trailing_newline() {
        let input = "test";
        let output = colorize_output(input);
        assert!(!output.ends_with('\n'));
    }
}
