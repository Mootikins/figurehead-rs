//! Terminal colorization for ASCII diagram output
//!
//! Applies ANSI escape codes based on style definitions in the diagram.
//! Only colorizes when explicit styles (classDef, style, :::) are present.

use crossterm::style::{Color, Stylize};
use std::collections::HashMap;

/// Style information extracted from diagram input
#[derive(Debug, Default)]
pub struct StyleInfo {
    /// Class definitions: className -> fill color
    pub class_defs: HashMap<String, String>,
    /// Node to class mappings: nodeId -> className
    pub node_classes: HashMap<String, String>,
    /// Inline styles: nodeId -> fill color
    pub node_styles: HashMap<String, String>,
}

impl StyleInfo {
    /// Check if any styles are defined
    pub fn has_styles(&self) -> bool {
        !self.class_defs.is_empty() || !self.node_classes.is_empty() || !self.node_styles.is_empty()
    }

    /// Get the fill color for a node (resolves class -> color)
    pub fn get_node_color(&self, node_id: &str) -> Option<&str> {
        // Check inline style first
        if let Some(color) = self.node_styles.get(node_id) {
            return Some(color.as_str());
        }
        // Then check class
        if let Some(class) = self.node_classes.get(node_id) {
            if let Some(color) = self.class_defs.get(class) {
                return Some(color.as_str());
            }
        }
        None
    }
}

/// Extract style information from diagram input text
pub fn extract_styles(input: &str) -> StyleInfo {
    let mut info = StyleInfo::default();

    for line in input.lines() {
        let trimmed = line.trim();

        // classDef className fill:#color
        if trimmed.starts_with("classDef ") {
            if let Some((name, color)) = parse_classdef(trimmed) {
                info.class_defs.insert(name, color);
            }
        }
        // style nodeId fill:#color
        else if trimmed.starts_with("style ") {
            if let Some((node_id, color)) = parse_style(trimmed) {
                info.node_styles.insert(node_id, color);
            }
        }
        // class nodeId className
        else if trimmed.starts_with("class ") {
            if let Some((node_id, class_name)) = parse_class(trimmed) {
                info.node_classes.insert(node_id, class_name);
            }
        }
        // Inline :::className syntax
        else if trimmed.contains(":::") {
            for (node_id, class_name) in parse_inline_classes(trimmed) {
                info.node_classes.insert(node_id, class_name);
            }
        }
    }

    info
}

/// Parse `classDef className fill:#color` -> (className, color)
fn parse_classdef(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 {
        let class_name = parts[1].to_string();
        let style_str = parts[2..].join(" ");
        if let Some(color) = extract_fill_color(&style_str) {
            return Some((class_name, color));
        }
    }
    None
}

/// Parse `style nodeId fill:#color` -> (nodeId, color)
fn parse_style(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 {
        let node_id = parts[1].trim_matches(',').to_string();
        let style_str = parts[2..].join(" ");
        if let Some(color) = extract_fill_color(&style_str) {
            return Some((node_id, color));
        }
    }
    None
}

/// Parse `class nodeId className` -> (nodeId, className)
fn parse_class(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 {
        let node_id = parts[1].trim_matches(',').to_string();
        let class_name = parts[2].to_string();
        return Some((node_id, class_name));
    }
    None
}

/// Parse inline `A[Label]:::className` syntax from a line
fn parse_inline_classes(line: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut remaining = line;

    while let Some(pos) = remaining.find(":::") {
        // Find the node ID before :::
        let before = &remaining[..pos];
        if let Some(node_id) = extract_node_id_before(before) {
            // Find the class name after :::
            let after = &remaining[pos + 3..];
            if let Some(class_name) = extract_class_name_after(after) {
                results.push((node_id, class_name));
            }
        }
        remaining = &remaining[pos + 3..];
    }

    results
}

/// Extract node ID from text ending at a position (e.g., "A[Label]" -> "A")
fn extract_node_id_before(text: &str) -> Option<String> {
    // Find the last identifier before any shape delimiters
    let text = text.trim_end();

    // Skip shape suffix like [Label], (Label), {Label}
    let text = if text.ends_with(']') || text.ends_with(')') || text.ends_with('}') {
        // Find matching opener
        let closer = text.chars().last()?;
        let opener = match closer {
            ']' => '[',
            ')' => '(',
            '}' => '{',
            _ => return None,
        };
        if let Some(open_pos) = text.rfind(opener) {
            &text[..open_pos]
        } else {
            text
        }
    } else {
        text
    };

    // Extract trailing identifier
    let id: String = text
        .chars()
        .rev()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

/// Extract class name after ::: (until whitespace or delimiter)
fn extract_class_name_after(text: &str) -> Option<String> {
    let name: String = text
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();

    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Extract fill color from style string like "fill:#f9f,stroke:#333"
fn extract_fill_color(style: &str) -> Option<String> {
    for part in style.split(',') {
        let part = part.trim();
        if part.starts_with("fill:") {
            return Some(part[5..].trim().to_string());
        }
    }
    None
}

/// Convert a color string (hex or named) to crossterm Color
pub fn parse_color(color_str: &str) -> Option<Color> {
    let color_str = color_str.trim();

    // Hex color
    if color_str.starts_with('#') {
        let hex = &color_str[1..];
        return parse_hex_color(hex);
    }

    // Named colors (basic set)
    match color_str.to_lowercase().as_str() {
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "blue" => Some(Color::Blue),
        "yellow" => Some(Color::Yellow),
        "cyan" => Some(Color::Cyan),
        "magenta" => Some(Color::Magenta),
        "white" => Some(Color::White),
        "black" => Some(Color::Black),
        "grey" | "gray" => Some(Color::Grey),
        "darkred" => Some(Color::DarkRed),
        "darkgreen" => Some(Color::DarkGreen),
        "darkblue" => Some(Color::DarkBlue),
        "darkyellow" => Some(Color::DarkYellow),
        "darkcyan" => Some(Color::DarkCyan),
        "darkmagenta" => Some(Color::DarkMagenta),
        "darkgrey" | "darkgray" => Some(Color::DarkGrey),
        _ => None,
    }
}

/// Parse hex color (#RGB or #RRGGBB) to crossterm RGB
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');

    match hex.len() {
        // #RGB -> #RRGGBB
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color::Rgb { r, g, b })
        }
        // #RRGGBB
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::Rgb { r, g, b })
        }
        _ => None,
    }
}

/// Colorize output based on extracted styles
///
/// Only applies colors when styles are explicitly defined.
/// Returns input unchanged if no styles are present.
pub fn colorize_output(input: &str, output: &str, styles: &StyleInfo) -> String {
    // No styles defined - return unchanged
    if !styles.has_styles() {
        return output.to_string();
    }

    // Build a map of labels to colors for nodes with styles
    let mut label_colors: HashMap<String, Color> = HashMap::new();

    // Extract node labels from input and map to colors
    for line in input.lines() {
        for (node_id, label) in extract_node_labels(line) {
            if let Some(color_str) = styles.get_node_color(&node_id) {
                if let Some(color) = parse_color(color_str) {
                    label_colors.insert(label, color);
                }
            }
        }
    }

    // If no labels have colors, return unchanged
    if label_colors.is_empty() {
        return output.to_string();
    }

    // Apply colors to output where labels appear
    colorize_by_labels(output, &label_colors)
}

/// Extract (nodeId, label) pairs from a line
fn extract_node_labels(line: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current_id = String::new();

    while let Some(c) = chars.next() {
        if c.is_alphanumeric() || c == '_' {
            current_id.push(c);
        } else if (c == '[' || c == '(' || c == '{') && !current_id.is_empty() {
            // Found shape opener after ID
            let closer = match c {
                '[' => ']',
                '(' => ')',
                '{' => '}',
                _ => continue,
            };

            // Collect label until closer
            let mut label = String::new();
            let mut depth = 1;

            while let Some(&next) = chars.peek() {
                chars.next();
                if next == c {
                    depth += 1;
                } else if next == closer {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                label.push(next);
            }

            if !label.is_empty() {
                results.push((current_id.clone(), label));
            }
            current_id.clear();
        } else {
            current_id.clear();
        }
    }

    results
}

/// Apply colors to output where label text appears
fn colorize_by_labels(output: &str, label_colors: &HashMap<String, Color>) -> String {
    let mut result = output.to_string();

    for (label, color) in label_colors {
        // Simple replacement - find label and colorize it
        // This is imperfect but handles the common case
        if result.contains(label.as_str()) {
            let colored = format!("{}", label.clone().with(*color));
            result = result.replace(label.as_str(), &colored);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_styles_classdef() {
        let input = "classDef red fill:#f00";
        let styles = extract_styles(input);
        assert_eq!(styles.class_defs.get("red"), Some(&"#f00".to_string()));
    }

    #[test]
    fn test_extract_styles_inline() {
        let input = "A[Start]:::highlight --> B";
        let styles = extract_styles(input);
        assert_eq!(styles.node_classes.get("A"), Some(&"highlight".to_string()));
    }

    #[test]
    fn test_no_styles_returns_unchanged() {
        let input = "graph LR\nA --> B";
        let output = "┌─┐\n│A│\n└─┘";
        let styles = extract_styles(input);
        let result = colorize_output(input, output, &styles);
        assert_eq!(result, output);
    }

    #[test]
    fn test_parse_hex_color_short() {
        let color = parse_color("#f00").unwrap();
        assert!(matches!(color, Color::Rgb { r: 255, g: 0, b: 0 }));
    }

    #[test]
    fn test_parse_hex_color_long() {
        let color = parse_color("#ff8800").unwrap();
        assert!(matches!(color, Color::Rgb { r: 255, g: 136, b: 0 }));
    }

    #[test]
    fn test_parse_named_color() {
        assert!(matches!(parse_color("red"), Some(Color::Red)));
        assert!(matches!(parse_color("Blue"), Some(Color::Blue)));
    }
}
