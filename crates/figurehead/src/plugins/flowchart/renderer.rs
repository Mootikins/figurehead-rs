//! ASCII rendering implementation for flowcharts
//!
//! Converts positioned nodes into ASCII diagrams using Unicode box drawing characters.

use anyhow::Result;

use super::{FlowchartDatabase, FlowchartLayoutAlgorithm, PositionedNode};
use crate::core::{Database, EdgeType, LayoutAlgorithm, NodeShape, Renderer};

/// ASCII canvas representing the final diagram
#[derive(Debug, Clone)]
pub struct AsciiCanvas {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<char>>,
}

impl AsciiCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![' '; width.max(1)]; height.max(1)];
        Self {
            width,
            height,
            grid,
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

    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        for (i, c) in text.chars().enumerate() {
            self.set_char(x + i, y, c);
        }
    }
}

impl ToString for AsciiCanvas {
    fn to_string(&self) -> String {
        self.grid
            .iter()
            .map(|row| {
                let s: String = row.iter().collect();
                s.trim_end().to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string()
    }
}

/// Box drawing characters
struct BoxChars {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    horizontal: char,
    vertical: char,
}

impl BoxChars {
    fn rectangle() -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
        }
    }

    fn rounded() -> Self {
        Self {
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            horizontal: '─',
            vertical: '│',
        }
    }

    fn double() -> Self {
        Self {
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            horizontal: '═',
            vertical: '║',
        }
    }
}

/// Flowchart ASCII renderer
pub struct FlowchartRenderer;

impl FlowchartRenderer {
    pub fn new() -> Self {
        Self
    }

    fn draw_node(
        &self,
        canvas: &mut AsciiCanvas,
        node: &PositionedNode,
        shape: NodeShape,
        label: &str,
    ) {
        match shape {
            NodeShape::Rectangle | NodeShape::Subroutine => {
                self.draw_rectangle(canvas, node, label, BoxChars::rectangle())
            }
            NodeShape::RoundedRect => {
                self.draw_rectangle(canvas, node, label, BoxChars::rounded())
            }
            NodeShape::Diamond => self.draw_diamond(canvas, node, label),
            NodeShape::Circle => self.draw_circle(canvas, node, label),
            NodeShape::Hexagon => {
                self.draw_rectangle(canvas, node, label, BoxChars::double())
            }
            _ => self.draw_rectangle(canvas, node, label, BoxChars::rectangle()),
        }
    }

    fn draw_rectangle(
        &self,
        canvas: &mut AsciiCanvas,
        node: &PositionedNode,
        label: &str,
        chars: BoxChars,
    ) {
        let x = node.x;
        let y = node.y;
        let w = node.width;
        let h = node.height;

        // Top border
        canvas.set_char(x, y, chars.top_left);
        for i in 1..w - 1 {
            canvas.set_char(x + i, y, chars.horizontal);
        }
        canvas.set_char(x + w - 1, y, chars.top_right);

        // Sides and content
        for row in 1..h - 1 {
            canvas.set_char(x, y + row, chars.vertical);
            canvas.set_char(x + w - 1, y + row, chars.vertical);
        }

        // Label centered
        let label_y = y + h / 2;
        let label_x = x + (w.saturating_sub(label.len())) / 2;
        canvas.draw_text(label_x.max(x + 1), label_y, label);

        // Bottom border
        canvas.set_char(x, y + h - 1, chars.bottom_left);
        for i in 1..w - 1 {
            canvas.set_char(x + i, y + h - 1, chars.horizontal);
        }
        canvas.set_char(x + w - 1, y + h - 1, chars.bottom_right);
    }

    fn draw_diamond(&self, canvas: &mut AsciiCanvas, node: &PositionedNode, label: &str) {
        let x = node.x;
        let y = node.y;
        let w = node.width;
        let h = node.height;
        let mid_x = x + w / 2;
        let mid_y = y + h / 2;

        // Top point
        canvas.set_char(mid_x, y, '/');
        canvas.set_char(mid_x + 1, y, '\\');

        // Upper sides
        for i in 1..h / 2 {
            canvas.set_char(mid_x - i, y + i, '/');
            canvas.set_char(mid_x + 1 + i, y + i, '\\');
        }

        // Middle with label
        canvas.set_char(x, mid_y, '<');
        canvas.set_char(x + w - 1, mid_y, '>');
        let label_x = x + (w.saturating_sub(label.len())) / 2;
        canvas.draw_text(label_x.max(x + 1), mid_y, label);

        // Lower sides
        for i in 1..h / 2 {
            canvas.set_char(x + i, mid_y + i, '\\');
            canvas.set_char(x + w - 1 - i, mid_y + i, '/');
        }

        // Bottom point
        canvas.set_char(mid_x, y + h - 1, '\\');
        canvas.set_char(mid_x + 1, y + h - 1, '/');
    }

    fn draw_circle(&self, canvas: &mut AsciiCanvas, node: &PositionedNode, label: &str) {
        let x = node.x;
        let y = node.y;
        let w = node.width;
        let h = node.height;

        // Top
        canvas.set_char(x, y, '(');
        for i in 1..w - 1 {
            canvas.set_char(x + i, y, '-');
        }
        canvas.set_char(x + w - 1, y, ')');

        // Middle
        for row in 1..h - 1 {
            canvas.set_char(x, y + row, '(');
            canvas.set_char(x + w - 1, y + row, ')');
        }

        // Label
        let label_y = y + h / 2;
        let label_x = x + (w.saturating_sub(label.len())) / 2;
        canvas.draw_text(label_x.max(x + 1), label_y, label);

        // Bottom
        canvas.set_char(x, y + h - 1, '(');
        for i in 1..w - 1 {
            canvas.set_char(x + i, y + h - 1, '-');
        }
        canvas.set_char(x + w - 1, y + h - 1, ')');
    }

    fn draw_edge(
        &self,
        canvas: &mut AsciiCanvas,
        waypoints: &[(usize, usize)],
        edge_type: EdgeType,
    ) {
        if waypoints.len() < 2 {
            return;
        }

        let (h_char, v_char) = match edge_type {
            EdgeType::Arrow | EdgeType::Line | EdgeType::OpenArrow | EdgeType::CrossArrow => {
                ('─', '│')
            }
            EdgeType::DottedArrow | EdgeType::DottedLine => ('┄', '┆'),
            EdgeType::ThickArrow | EdgeType::ThickLine => ('═', '║'),
            EdgeType::Invisible => return,
        };

        // Draw path between waypoints
        for window in waypoints.windows(2) {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];

            if y1 == y2 {
                // Horizontal line
                let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
                for x in start..=end {
                    if canvas.get_char(x, y1) == ' ' {
                        canvas.set_char(x, y1, h_char);
                    }
                }
            } else if x1 == x2 {
                // Vertical line
                let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
                for y in start..=end {
                    if canvas.get_char(x1, y) == ' ' {
                        canvas.set_char(x1, y, v_char);
                    }
                }
            } else {
                // Diagonal - use orthogonal routing
                // First go horizontal, then vertical
                let mid_x = x2;
                for x in x1.min(mid_x)..=x1.max(mid_x) {
                    if canvas.get_char(x, y1) == ' ' {
                        canvas.set_char(x, y1, h_char);
                    }
                }
                for y in y1.min(y2)..=y1.max(y2) {
                    if canvas.get_char(mid_x, y) == ' ' {
                        canvas.set_char(mid_x, y, v_char);
                    }
                }
            }
        }

        // Draw arrowhead at the end
        if edge_type.has_arrow() && waypoints.len() >= 2 {
            let (x2, y2) = waypoints[waypoints.len() - 1];
            let (x1, y1) = waypoints[waypoints.len() - 2];

            let arrow = if x2 > x1 {
                '→'
            } else if x2 < x1 {
                '←'
            } else if y2 > y1 {
                '↓'
            } else {
                '↑'
            };

            // Place arrow one step before the target
            if x2 != x1 {
                let ax = if x2 > x1 { x2 - 1 } else { x2 + 1 };
                canvas.set_char(ax, y2, arrow);
            } else if y2 != y1 {
                let ay = if y2 > y1 { y2 - 1 } else { y2 + 1 };
                canvas.set_char(x2, ay, arrow);
            }
        }
    }
}

impl Default for FlowchartRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer<FlowchartDatabase> for FlowchartRenderer {
    type Output = String;

    fn render(&self, database: &FlowchartDatabase) -> Result<Self::Output> {
        // First, compute the layout
        let layout_algo = FlowchartLayoutAlgorithm::new();
        let layout = layout_algo.layout(database)?;

        if layout.nodes.is_empty() {
            return Ok(String::new());
        }

        // Create canvas
        let mut canvas = AsciiCanvas::new(layout.width, layout.height);

        // Draw edges first (so nodes overlay them)
        for edge in &layout.edges {
            let edge_data = database
                .edges()
                .find(|e| e.from == edge.from_id && e.to == edge.to_id);
            let edge_type = edge_data.map(|e| e.edge_type).unwrap_or(EdgeType::Arrow);
            self.draw_edge(&mut canvas, &edge.waypoints, edge_type);
        }

        // Draw nodes
        for node in &layout.nodes {
            if let Some(node_data) = database.get_node(&node.id) {
                self.draw_node(&mut canvas, node, node_data.shape, &node_data.label);
            }
        }

        Ok(canvas.to_string())
    }

    fn name(&self) -> &'static str {
        "ascii"
    }

    fn version(&self) -> &'static str {
        "0.2.0"
    }

    fn format(&self) -> &'static str {
        "ascii"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Direction;

    #[test]
    fn test_basic_rendering() {
        let mut db = FlowchartDatabase::with_direction(Direction::LeftRight);
        db.add_simple_node("A", "Start").unwrap();
        db.add_simple_node("B", "End").unwrap();
        db.add_simple_edge("A", "B").unwrap();

        let renderer = FlowchartRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(!output.is_empty());
        assert!(output.contains("Start"));
        assert!(output.contains("End"));
    }

    #[test]
    fn test_empty_database() {
        let db = FlowchartDatabase::new();
        let renderer = FlowchartRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.is_empty());
    }

    #[test]
    fn test_diamond_shape() {
        let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
        db.add_shaped_node("A", "Yes?", crate::core::NodeShape::Diamond)
            .unwrap();

        let renderer = FlowchartRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.contains("Yes?"));
        // Diamond should have < and > characters
        assert!(output.contains('<') || output.contains('>'));
    }

    #[test]
    fn test_renderer_properties() {
        let renderer = FlowchartRenderer::new();
        assert_eq!(renderer.name(), "ascii");
        assert_eq!(renderer.format(), "ascii");
    }
}
