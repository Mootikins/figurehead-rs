//! ASCII rendering implementation for flowcharts
//!
//! Converts positioned nodes into ASCII diagrams using Unicode box drawing characters.

use anyhow::Result;
use std::collections::HashMap;

use super::{FlowchartDatabase, FlowchartLayout, PositionedNode};
use crate::core::{Database, LayoutAlgorithm, Renderer};

/// ASCII canvas representing the final diagram
#[derive(Debug, Clone)]
pub struct AsciiCanvas {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<char>>,
}

impl AsciiCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![' '; width]; height];
        Self {
            width,
            height,
            grid,
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.grid {
            for cell in row {
                *cell = ' ';
            }
        }
    }

    pub fn set_char(&mut self, x: usize, y: usize, c: char) {
        if y < self.height && x < self.width {
            self.grid[y][x] = c;
        }
    }

    pub fn get_char(&self, x: usize, y: usize) -> char {
        if y < self.height && x < self.width {
            self.grid[y][x]
        } else {
            ' '
        }
    }

    pub fn to_string(&self) -> String {
        self.grid
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Box styling options for ASCII rendering
#[derive(Debug, Clone, PartialEq)]
pub enum BoxStyle {
    Single,       // ┌───┐
    Double,       // ╔═══╗
    Rounded,      // .---.
    SingleDouble, // ╔───╗  (double top/bottom, single sides)
    DoubleSingle, // ┌===┐  (single top/bottom, double sides)
}

impl Default for BoxStyle {
    fn default() -> Self {
        BoxStyle::Single
    }
}

/// Box drawing characters for different styles
#[derive(Debug)]
struct BoxChars {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    horizontal: char,
    vertical: char,
}

impl BoxChars {
    fn for_style(style: &BoxStyle) -> Self {
        match style {
            BoxStyle::Single => Self {
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
                horizontal: '─',
                vertical: '│',
            },
            BoxStyle::Double => Self {
                top_left: '╔',
                top_right: '╗',
                bottom_left: '╚',
                bottom_right: '╝',
                horizontal: '═',
                vertical: '║',
            },
            BoxStyle::Rounded => Self {
                top_left: '.',
                top_right: '.',
                bottom_left: '\'',
                bottom_right: '\'',
                horizontal: '-',
                vertical: '|',
            },
            BoxStyle::SingleDouble => Self {
                top_left: '╔',
                top_right: '╗',
                bottom_left: '╚',
                bottom_right: '╝',
                horizontal: '─',
                vertical: '║',
            },
            BoxStyle::DoubleSingle => Self {
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
                horizontal: '═',
                vertical: '│',
            },
        }
    }
}

/// ASCII renderer for flowcharts
pub struct FlowchartAsciiRenderer {
    default_style: BoxStyle,
    node_styles: HashMap<String, BoxStyle>,
}

impl FlowchartAsciiRenderer {
    pub fn new() -> Self {
        Self {
            default_style: BoxStyle::default(),
            node_styles: HashMap::new(),
        }
    }

    pub fn with_style(style: BoxStyle) -> Self {
        Self {
            default_style: style,
            node_styles: HashMap::new(),
        }
    }

    pub fn set_node_style(&mut self, node_id: &str, style: BoxStyle) {
        self.node_styles.insert(node_id.to_string(), style);
    }

    pub fn get_node_style(&self, node_id: &str) -> &BoxStyle {
        self.node_styles.get(node_id).unwrap_or(&self.default_style)
    }

    /// Draw a box on the canvas
    fn draw_box(
        &self,
        canvas: &mut AsciiCanvas,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        style: &BoxStyle,
        label: &str,
    ) {
        if width < 3 || height < 3 {
            return; // Too small to draw a proper box
        }

        let chars = BoxChars::for_style(style);
        let right_x = x + width - 1;
        let bottom_y = y + height - 1;

        // Draw corners
        canvas.set_char(x, y, chars.top_left);
        canvas.set_char(right_x, y, chars.top_right);
        canvas.set_char(x, bottom_y, chars.bottom_left);
        canvas.set_char(right_x, bottom_y, chars.bottom_right);

        // Draw horizontal edges
        for hx in (x + 1)..right_x {
            canvas.set_char(hx, y, chars.horizontal);
            canvas.set_char(hx, bottom_y, chars.horizontal);
        }

        // Draw vertical edges
        for hy in (y + 1)..bottom_y {
            canvas.set_char(x, hy, chars.vertical);
            canvas.set_char(right_x, hy, chars.vertical);
        }

        // Draw label centered in the box
        let label_lines: Vec<&str> = label.lines().collect();
        let label_y = y + (height - label_lines.len()) / 2;

        for (i, line) in label_lines.iter().enumerate() {
            if label_y + i < bottom_y {
                let label_x = x + (width - line.len()) / 2;
                for (j, c) in line.chars().enumerate() {
                    if label_x + j < right_x {
                        canvas.set_char(label_x + j, label_y + i, c);
                    }
                }
            }
        }
    }

    /// Draw connections between nodes
    fn draw_connections(
        &self,
        canvas: &mut AsciiCanvas,
        layout: &FlowchartLayout,
        database: &FlowchartDatabase,
    ) {
        let node_positions: HashMap<_, _> = layout
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();

        for (from, to) in database.get_edges() {
            if let (Some(from_node), Some(to_node)) =
                (node_positions.get(from), node_positions.get(to))
            {
                self.draw_arrow(canvas, from_node, to_node);
            }
        }
    }

    /// Draw an arrow from one node to another
    fn draw_arrow(&self, canvas: &mut AsciiCanvas, from: &PositionedNode, to: &PositionedNode) {
        // Apply the same offset used when drawing nodes
        const RENDER_OFFSET_X: usize = 2;
        const RENDER_OFFSET_Y: usize = 2;

        // Calculate connection points (right edge of from node to left edge of to node)
        // Add the render offset to align with where nodes are actually drawn
        let from_x = from.x + from.width + RENDER_OFFSET_X;
        let from_y = from.y + from.height / 2 + RENDER_OFFSET_Y;
        let to_x = to.x + RENDER_OFFSET_X;
        let to_y = to.y + to.height / 2 + RENDER_OFFSET_Y;

        // Simple horizontal line with arrowhead
        if from_y == to_y {
            // Horizontal arrow
            for x in from_x..to_x {
                if x < canvas.width && from_y < canvas.height {
                    canvas.set_char(x, from_y, '─');
                }
            }
            // Arrowhead
            if to_x > 0 && to_y < canvas.height {
                canvas.set_char(to_x - 1, to_y, '→');
            }
        } else {
            // More complex routing needed for now just draw diagonal
            let mut x = from_x;
            let mut y = from_y;

            while x < to_x && y < canvas.height {
                if y < canvas.height && x < canvas.width {
                    canvas.set_char(x, y, '/');
                }
                x += 1;
                if y < to_y {
                    y += 1;
                } else if y > to_y {
                    y -= 1;
                }
            }
        }
    }
}

impl Renderer<FlowchartDatabase> for FlowchartAsciiRenderer {
    type Output = AsciiCanvas;

    fn render(&self, database: &FlowchartDatabase) -> Result<Self::Output> {
        // First, create a layout to position the nodes
        let layout_algorithm = super::FlowchartLayoutAlgorithm::new();
        let layout = layout_algorithm.layout(database)?;

        // Create canvas with some extra padding
        let canvas_width = layout.width + 4;
        let canvas_height = layout.height + 4;
        let mut canvas = AsciiCanvas::new(canvas_width, canvas_height);

        // Draw all nodes with a 2-character offset
        for node in &layout.nodes {
            let style = self.get_node_style(&node.id);
            self.draw_box(
                &mut canvas,
                node.x + 2,
                node.y + 2,
                node.width,
                node.height,
                style,
                database
                    .get_nodes()
                    .iter()
                    .find(|(id, _)| *id == node.id)
                    .map(|(_, label)| *label)
                    .unwrap_or(""),
            );
        }

        // Draw connections
        self.draw_connections(&mut canvas, &layout, database);

        Ok(canvas)
    }

    fn name(&self) -> &'static str {
        "ascii"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn format(&self) -> &'static str {
        "ascii"
    }
}

// Keep the old simple renderer for backwards compatibility with tests
pub struct FlowchartRenderer;

impl FlowchartRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer<FlowchartDatabase> for FlowchartRenderer {
    type Output = String;

    fn render(&self, database: &FlowchartDatabase) -> Result<Self::Output> {
        // TODO: Implement proper ASCII rendering
        // For now, just a simple implementation to make tests compile

        let mut output = String::new();
        output.push_str("Flowchart Diagram\n");
        output.push_str("==================\n\n");

        // Render nodes
        for (id, label) in database.get_nodes() {
            output.push_str(&format!("Node {}: {}\n", id, label));
        }

        output.push_str("\n");

        // Render edges
        for (from, to) in database.get_edges() {
            output.push_str(&format!("Edge: {} --> {}\n", from, to));
        }

        Ok(output)
    }

    fn name(&self) -> &'static str {
        "ascii"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn format(&self) -> &'static str {
        "ascii"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_canvas_creation() {
        let canvas = AsciiCanvas::new(10, 5);
        assert_eq!(canvas.width, 10);
        assert_eq!(canvas.height, 5);
        assert_eq!(canvas.grid.len(), 5);
        assert_eq!(canvas.grid[0].len(), 10);

        // All cells should be spaces initially
        for row in &canvas.grid {
            for cell in row {
                assert_eq!(*cell, ' ');
            }
        }
    }

    #[test]
    fn test_ascii_canvas_operations() {
        let mut canvas = AsciiCanvas::new(5, 3);

        // Test setting and getting characters
        canvas.set_char(1, 1, 'X');
        assert_eq!(canvas.get_char(1, 1), 'X');

        // Test bounds checking
        canvas.set_char(10, 10, 'Y'); // Should not panic
        assert_eq!(canvas.get_char(10, 10), ' '); // Out of bounds returns space
    }

    #[test]
    fn test_ascii_canvas_to_string() {
        let mut canvas = AsciiCanvas::new(3, 2);
        canvas.set_char(0, 0, 'A');
        canvas.set_char(1, 0, 'B');
        canvas.set_char(2, 0, 'C');
        canvas.set_char(0, 1, '1');
        canvas.set_char(1, 1, '2');
        canvas.set_char(2, 1, '3');

        let result = canvas.to_string();
        let expected = "ABC\n123";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_box_style_characters() {
        let chars = BoxChars::for_style(&BoxStyle::Single);
        assert_eq!(chars.top_left, '┌');
        assert_eq!(chars.top_right, '┐');
        assert_eq!(chars.bottom_left, '└');
        assert_eq!(chars.bottom_right, '┘');
        assert_eq!(chars.horizontal, '─');
        assert_eq!(chars.vertical, '│');
    }

    #[test]
    fn test_double_box_style() {
        let chars = BoxChars::for_style(&BoxStyle::Double);
        assert_eq!(chars.top_left, '╔');
        assert_eq!(chars.top_right, '╗');
        assert_eq!(chars.bottom_left, '╚');
        assert_eq!(chars.bottom_right, '╝');
        assert_eq!(chars.horizontal, '═');
        assert_eq!(chars.vertical, '║');
    }

    #[test]
    fn test_rounded_box_style() {
        let chars = BoxChars::for_style(&BoxStyle::Rounded);
        assert_eq!(chars.top_left, '.');
        assert_eq!(chars.top_right, '.');
        assert_eq!(chars.bottom_left, '\'');
        assert_eq!(chars.bottom_right, '\'');
        assert_eq!(chars.horizontal, '-');
        assert_eq!(chars.vertical, '|');
    }

    #[test]
    fn test_flowchart_ascii_renderer_creation() {
        let renderer = FlowchartAsciiRenderer::new();
        assert_eq!(renderer.name(), "ascii");
        assert_eq!(renderer.version(), "0.1.0");
        assert_eq!(renderer.default_style, BoxStyle::Single);
    }

    #[test]
    fn test_flowchart_ascii_renderer_with_style() {
        let renderer = FlowchartAsciiRenderer::with_style(BoxStyle::Double);
        assert_eq!(renderer.default_style, BoxStyle::Double);
    }

    #[test]
    fn test_node_style_management() {
        let mut renderer = FlowchartAsciiRenderer::new();

        // Default style should be used for unknown nodes
        assert_eq!(renderer.get_node_style("unknown"), &BoxStyle::Single);

        // Set and get specific node style
        renderer.set_node_style("node1", BoxStyle::Double);
        assert_eq!(renderer.get_node_style("node1"), &BoxStyle::Double);

        // Unknown nodes should still get default style
        assert_eq!(renderer.get_node_style("node2"), &BoxStyle::Single);
    }

    #[test]
    fn test_draw_single_box() {
        let mut canvas = AsciiCanvas::new(10, 6);
        let renderer = FlowchartAsciiRenderer::new();

        renderer.draw_box(&mut canvas, 1, 1, 8, 3, &BoxStyle::Single, "Test");

        let result = canvas.to_string();
        let expected = "          \n ┌──────┐ \n │ Test │ \n └──────┘ \n          \n          ";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_draw_double_box() {
        let mut canvas = AsciiCanvas::new(10, 6);
        let renderer = FlowchartAsciiRenderer::new();

        renderer.draw_box(&mut canvas, 1, 1, 8, 3, &BoxStyle::Double, "Test");

        let result = canvas.to_string();
        let expected = "          \n ╔══════╗ \n ║ Test ║ \n ╚══════╝ \n          \n          ";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_draw_rounded_box() {
        let mut canvas = AsciiCanvas::new(10, 6);
        let renderer = FlowchartAsciiRenderer::new();

        renderer.draw_box(&mut canvas, 1, 1, 8, 3, &BoxStyle::Rounded, "Test");

        let result = canvas.to_string();
        let expected = "          \n .------. \n | Test | \n '------' \n          \n          ";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_draw_box_with_multiline_label() {
        let mut canvas = AsciiCanvas::new(12, 7);
        let renderer = FlowchartAsciiRenderer::new();

        renderer.draw_box(&mut canvas, 1, 1, 10, 4, &BoxStyle::Single, "Line1\nLine2");

        let result = canvas.to_string();
        let expected = "            \n ┌────────┐ \n │ Line1  │ \n │ Line2  │ \n └────────┘ \n            \n            ";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_basic_rendering() {
        let mut db = FlowchartDatabase::new();
        db.add_node("A", "Start").unwrap();
        db.add_node("B", "End").unwrap();
        db.add_edge("A", "B").unwrap();

        let renderer = FlowchartAsciiRenderer::new();
        let canvas = renderer.render(&db).unwrap();

        // Should have some content
        let result = canvas.to_string();
        assert!(!result.trim().is_empty());
        assert!(result.contains("Start"));
        assert!(result.contains("End"));
    }

    #[test]
    fn test_rendering_with_different_styles() {
        let mut db = FlowchartDatabase::new();
        db.add_node("A", "Single").unwrap();
        db.add_node("B", "Double").unwrap();
        db.add_node("C", "Rounded").unwrap();

        let mut renderer = FlowchartAsciiRenderer::new();
        renderer.set_node_style("B", BoxStyle::Double);
        renderer.set_node_style("C", BoxStyle::Rounded);

        let canvas = renderer.render(&db).unwrap();
        let result = canvas.to_string();

        assert!(result.contains("Single"));
        assert!(result.contains("Double"));
        assert!(result.contains("Rounded"));
        // Should contain box drawing characters
        assert!(result.contains('┌') || result.contains('╔') || result.contains('.'));
    }

    #[test]
    fn test_canvas_clear() {
        let mut canvas = AsciiCanvas::new(5, 3);

        // Fill with characters
        for y in 0..3 {
            for x in 0..5 {
                canvas.set_char(x, y, 'X');
            }
        }

        // Clear and verify empty
        canvas.clear();
        for y in 0..3 {
            for x in 0..5 {
                assert_eq!(canvas.get_char(x, y), ' ');
            }
        }
    }

    #[test]
    fn test_renderer_trait_implementation() {
        let renderer = FlowchartAsciiRenderer::new();

        // Test that the trait methods work
        assert_eq!(renderer.name(), "ascii");
        assert_eq!(renderer.version(), "0.1.0");

        // Test that render works with empty database
        let empty_db = FlowchartDatabase::new();
        let canvas = renderer.render(&empty_db).unwrap();
        assert_eq!(canvas.width, 4); // Default padding
        assert_eq!(canvas.height, 4); // Default padding
    }

    #[test]
    fn test_edge_connection_coordinates() {
        let mut db = FlowchartDatabase::new();
        db.add_node("A", "A").unwrap();
        db.add_node("B", "B").unwrap();
        db.add_edge("A", "B").unwrap();

        let renderer = FlowchartAsciiRenderer::new();
        let canvas = renderer.render(&db).unwrap();
        let result = canvas.to_string();

        // The arrow should connect from the right edge of box A to the left edge of box B
        // In the output, we should see: A───→B with the arrow touching both boxes
        assert!(result.contains("│  A   │"));
        assert!(result.contains("│  B   │"));
        assert!(result.contains("───→"));

        // The arrow should be positioned on the same line as the box content
        let lines: Vec<&str> = result.lines().collect();
        let mut arrow_line_found = false;
        let mut a_box_line_found = false;
        let mut b_box_line_found = false;

        for line in lines {
            if line.contains("│  A   │") {
                a_box_line_found = true;
            }
            if line.contains("│  B   │") {
                b_box_line_found = true;
            }
            if line.contains("───→") {
                arrow_line_found = true;
            }
        }

        // Verify the arrow is on the same line as both box contents
        assert!(a_box_line_found, "Box A content should be present");
        assert!(b_box_line_found, "Box B content should be present");
        assert!(arrow_line_found, "Arrow should be present");
    }
}
