//! ASCII rendering implementation for flowcharts
//!
//! Converts positioned nodes into ASCII diagrams using various character sets.

use anyhow::Result;

use super::{FlowchartDatabase, FlowchartLayoutAlgorithm, PositionedNode};
use crate::core::{CharacterSet, Database, EdgeType, LayoutAlgorithm, NodeShape, Renderer};

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
pub struct FlowchartRenderer {
    style: CharacterSet,
}

impl FlowchartRenderer {
    /// Create a new renderer with default Unicode style
    pub fn new() -> Self {
        Self {
            style: CharacterSet::default(),
        }
    }

    /// Create a new renderer with a specific character set
    pub fn with_style(style: CharacterSet) -> Self {
        Self { style }
    }

    /// Get the current character set
    pub fn style(&self) -> CharacterSet {
        self.style
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

        // Diamond structure for height 5:
        //     /\        row 0: top point
        //    /  \       row 1: expanding
        //   <text>      row 2: middle (widest, with label)
        //    \  /       row 3: contracting
        //     \/        row 4: bottom point

        let mid_y = y + h / 2;
        let half_h = h / 2;
        let center_x = x + w / 2;

        // Top point
        canvas.set_char(center_x, y, '/');
        canvas.set_char(center_x + 1, y, '\\');

        // Upper expanding rows (between top point and middle)
        for row in 1..half_h {
            let left_x = center_x.saturating_sub(row);
            let right_x = center_x + 1 + row;
            canvas.set_char(left_x, y + row, '/');
            canvas.set_char(right_x, y + row, '\\');
        }

        // Middle row with label
        canvas.set_char(x, mid_y, '<');
        canvas.set_char(x + w - 1, mid_y, '>');
        let label_x = x + (w.saturating_sub(label.len())) / 2;
        canvas.draw_text(label_x.max(x + 1), mid_y, label);

        // Lower contracting rows (between middle and bottom point)
        for row in 1..half_h {
            let left_x = center_x.saturating_sub(half_h - row);
            let right_x = center_x + 1 + (half_h - row);
            canvas.set_char(left_x, mid_y + row, '\\');
            canvas.set_char(right_x, mid_y + row, '/');
        }

        // Bottom point
        canvas.set_char(center_x, y + h - 1, '\\');
        canvas.set_char(center_x + 1, y + h - 1, '/');
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

        let chars = EdgeChars::for_type(edge_type);
        if chars.is_invisible() {
            return;
        }

        let (x1, y1) = waypoints[0];
        let (x2, y2) = waypoints[waypoints.len() - 1];

        // Shorten endpoint by 1 to leave room for arrowhead
        let has_arrow = edge_type.has_arrow();

        // Determine if we need orthogonal routing
        if y1 == y2 {
            // Pure horizontal - adjust endpoint for arrow
            let end_x = if has_arrow {
                if x2 > x1 { x2.saturating_sub(1) } else { x2 + 1 }
            } else {
                x2
            };
            self.draw_horizontal_line(canvas, y1, x1, end_x, &chars);
            if has_arrow {
                let arrow = if x2 > x1 { chars.arrow_right } else { chars.arrow_left };
                canvas.set_char(end_x, y1, arrow);
            }
        } else if x1 == x2 {
            // Pure vertical - adjust endpoint for arrow
            let end_y = if has_arrow {
                if y2 > y1 { y2.saturating_sub(1) } else { y2 + 1 }
            } else {
                y2
            };
            self.draw_vertical_line(canvas, x1, y1, end_y, &chars);
            if has_arrow {
                let arrow = if y2 > y1 { chars.arrow_down } else { chars.arrow_up };
                canvas.set_char(x1, end_y, arrow);
            }
        } else {
            // Orthogonal routing: horizontal first, then vertical
            let mid_y = y1;
            let turn_x = x2;

            // Horizontal segment (full length to turn)
            self.draw_horizontal_line(canvas, mid_y, x1, turn_x, &chars);

            // Corner at turn point
            let corner = if x2 > x1 {
                if y2 > y1 { '┐' } else { '┘' }
            } else {
                if y2 > y1 { '┌' } else { '└' }
            };
            canvas.set_char(turn_x, mid_y, corner);

            // Vertical segment - adjust endpoint for arrow
            let end_y = if has_arrow {
                if y2 > y1 { y2.saturating_sub(1) } else { y2 + 1 }
            } else {
                y2
            };
            self.draw_vertical_line(canvas, turn_x, mid_y, end_y, &chars);
            if has_arrow {
                let arrow = if y2 > y1 { chars.arrow_down } else { chars.arrow_up };
                canvas.set_char(turn_x, end_y, arrow);
            }
        }
    }

    fn draw_horizontal_line(
        &self,
        canvas: &mut AsciiCanvas,
        y: usize,
        x1: usize,
        x2: usize,
        chars: &EdgeChars,
    ) {
        let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        let going_right = x2 > x1;

        for x in start..=end {
            let existing = canvas.get_char(x, y);
            let is_start = x == start;
            let is_end = x == end;

            let new_char = match existing {
                ' ' => chars.horizontal,
                '│' | '┆' | '║' => {
                    // T-junction or crossing
                    if is_start {
                        if going_right { '├' } else { '┤' }
                    } else if is_end {
                        if going_right { '┤' } else { '├' }
                    } else {
                        '┼' // True crossing in the middle
                    }
                }
                '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼' => existing, // Keep existing junctions
                _ => chars.horizontal,
            };
            canvas.set_char(x, y, new_char);
        }
    }

    fn draw_vertical_line(
        &self,
        canvas: &mut AsciiCanvas,
        x: usize,
        y1: usize,
        y2: usize,
        chars: &EdgeChars,
    ) {
        let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        let going_down = y2 > y1;

        for y in start..=end {
            let existing = canvas.get_char(x, y);
            let is_start = y == start;
            let is_end = y == end;

            let new_char = match existing {
                ' ' => chars.vertical,
                '─' | '┄' | '═' => {
                    // T-junction or crossing
                    if is_start {
                        if going_down { '┬' } else { '┴' }
                    } else if is_end {
                        if going_down { '┴' } else { '┬' }
                    } else {
                        '┼' // True crossing in the middle
                    }
                }
                '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼' => existing, // Keep existing junctions
                _ => chars.vertical,
            };
            canvas.set_char(x, y, new_char);
        }
    }

}

/// Edge drawing characters
struct EdgeChars {
    horizontal: char,
    vertical: char,
    arrow_right: char,
    arrow_left: char,
    arrow_down: char,
    arrow_up: char,
    invisible: bool,
}

impl EdgeChars {
    fn for_type(edge_type: EdgeType) -> Self {
        match edge_type {
            EdgeType::Arrow | EdgeType::Line | EdgeType::OpenArrow | EdgeType::CrossArrow => {
                Self {
                    horizontal: '─',
                    vertical: '│',
                    arrow_right: '▶',
                    arrow_left: '◀',
                    arrow_down: '▼',
                    arrow_up: '▲',
                    invisible: false,
                }
            }
            EdgeType::DottedArrow | EdgeType::DottedLine => Self {
                horizontal: '┄',
                vertical: '┆',
                arrow_right: '▷',
                arrow_left: '◁',
                arrow_down: '▽',
                arrow_up: '△',
                invisible: false,
            },
            EdgeType::ThickArrow | EdgeType::ThickLine => Self {
                horizontal: '═',
                vertical: '║',
                arrow_right: '▶',
                arrow_left: '◀',
                arrow_down: '▼',
                arrow_up: '▲',
                invisible: false,
            },
            EdgeType::Invisible => Self {
                horizontal: ' ',
                vertical: ' ',
                arrow_right: ' ',
                arrow_left: ' ',
                arrow_down: ' ',
                arrow_up: ' ',
                invisible: true,
            },
        }
    }

    fn is_invisible(&self) -> bool {
        self.invisible
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
