//! State diagram ASCII renderer
//!
//! Renders state diagrams as ASCII art.

use super::database::StateDatabase;
use super::layout::{StateLayoutAlgorithm, StateLayoutResult};
use crate::core::{CharacterSet, NodeShape, Renderer};
use anyhow::Result;

/// ASCII canvas for rendering
struct Canvas {
    width: usize,
    height: usize,
    grid: Vec<Vec<char>>,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![' '; width.max(1)]; height.max(1)];
        Self {
            width,
            height,
            grid,
        }
    }

    fn set_char(&mut self, x: usize, y: usize, c: char) {
        if y < self.height && x < self.width {
            self.grid[y][x] = c;
        }
    }

    fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        for (i, c) in text.chars().enumerate() {
            self.set_char(x + i, y, c);
        }
    }

    fn draw_text_centered(&mut self, center_x: usize, y: usize, text: &str) {
        let start_x = center_x.saturating_sub(text.len() / 2);
        self.draw_text(start_x, y, text);
    }
}

impl std::fmt::Display for Canvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = self
            .grid
            .iter()
            .map(|row| {
                let s: String = row.iter().collect();
                s.trim_end().to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}", output.trim_end())
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
    fn unicode() -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
        }
    }

    fn ascii() -> Self {
        Self {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            horizontal: '-',
            vertical: '|',
        }
    }
}

/// State diagram renderer
pub struct StateRenderer {
    style: CharacterSet,
}

impl StateRenderer {
    pub fn new() -> Self {
        Self {
            style: CharacterSet::default(),
        }
    }

    pub fn with_style(style: CharacterSet) -> Self {
        Self { style }
    }

    fn is_unicode(&self) -> bool {
        !self.style.is_ascii()
    }

    fn box_chars(&self) -> BoxChars {
        if self.is_unicode() {
            BoxChars::unicode()
        } else {
            BoxChars::ascii()
        }
    }

    /// Draw a terminal state (start/end circle)
    fn draw_terminal(&self, canvas: &mut Canvas, x: usize, y: usize, width: usize, is_start: bool) {
        let center_x = x + width / 2;

        if self.is_unicode() {
            if is_start {
                canvas.draw_text_centered(center_x, y + 1, "(●)");
            } else {
                canvas.draw_text_centered(center_x, y + 1, "(○)");
            }
        } else if is_start {
            canvas.draw_text_centered(center_x, y + 1, "(*)");
        } else {
            canvas.draw_text_centered(center_x, y + 1, "(o)");
        }
    }

    /// Draw a state box
    fn draw_state_box(
        &self,
        canvas: &mut Canvas,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        label: &str,
    ) {
        let chars = self.box_chars();

        // Top border
        canvas.set_char(x, y, chars.top_left);
        for i in 1..width - 1 {
            canvas.set_char(x + i, y, chars.horizontal);
        }
        canvas.set_char(x + width - 1, y, chars.top_right);

        // Sides
        for row in 1..height - 1 {
            canvas.set_char(x, y + row, chars.vertical);
            canvas.set_char(x + width - 1, y + row, chars.vertical);
        }

        // Bottom border
        canvas.set_char(x, y + height - 1, chars.bottom_left);
        for i in 1..width - 1 {
            canvas.set_char(x + i, y + height - 1, chars.horizontal);
        }
        canvas.set_char(x + width - 1, y + height - 1, chars.bottom_right);

        // Center label
        let center_x = x + width / 2;
        let center_y = y + height / 2;
        canvas.draw_text_centered(center_x, center_y, label);
    }

    /// Draw a vertical arrow with optional label
    fn draw_vertical_arrow(
        &self,
        canvas: &mut Canvas,
        x: usize,
        from_y: usize,
        to_y: usize,
        label: Option<&str>,
    ) {
        if from_y >= to_y {
            return;
        }

        let arrow = if self.is_unicode() { '▼' } else { 'v' };
        let line = if self.is_unicode() { '│' } else { '|' };

        // Draw vertical line
        for y in from_y..to_y {
            canvas.set_char(x, y, line);
        }

        // Draw arrow head
        canvas.set_char(x, to_y, arrow);

        // Draw label if present
        if let Some(lbl) = label {
            if !lbl.is_empty() {
                // Place label to the right of the line
                let label_y = from_y + (to_y - from_y) / 2;
                canvas.draw_text(x + 2, label_y, lbl);
            }
        }
    }

    /// Render the layout result
    fn render_layout(&self, layout: &StateLayoutResult) -> String {
        if layout.states.is_empty() {
            return String::new();
        }

        // Calculate canvas size with extra space for arrows
        let extra_height = layout.transitions.len() * 2;
        let width = layout.width + 20; // Extra space for labels
        let height = layout.height + extra_height + 2;

        let mut canvas = Canvas::new(width, height);

        // Track which [*] states have incoming vs outgoing edges
        let mut terminal_has_outgoing: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut terminal_has_incoming: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for trans in &layout.transitions {
            if trans.from_id == "[*]" {
                terminal_has_outgoing.insert(format!("{}_{}", trans.from_x, trans.from_y));
            }
            if trans.to_id == "[*]" {
                terminal_has_incoming.insert(format!("{}_{}", trans.to_x, trans.to_y));
            }
        }

        // Draw states
        for state in &layout.states {
            match state.shape {
                NodeShape::Terminal => {
                    // Determine if this is a start or end terminal based on position
                    let key = format!("{}_{}", state.x + state.width / 2, state.y + state.height);
                    let is_start = state.rank == 0 || terminal_has_outgoing.contains(&key);
                    self.draw_terminal(&mut canvas, state.x, state.y, state.width, is_start);
                }
                _ => {
                    self.draw_state_box(
                        &mut canvas,
                        state.x,
                        state.y,
                        state.width,
                        state.height,
                        &state.label,
                    );
                }
            }
        }

        // Draw transitions
        for trans in &layout.transitions {
            self.draw_vertical_arrow(
                &mut canvas,
                trans.from_x,
                trans.from_y,
                trans.to_y.saturating_sub(1),
                trans.label.as_deref(),
            );
        }

        canvas.to_string()
    }

    /// Render the database to ASCII
    pub fn render(&self, database: &StateDatabase) -> Result<String> {
        let layout_algo = StateLayoutAlgorithm::new();
        let layout = layout_algo.layout(database)?;

        Ok(self.render_layout(&layout))
    }
}

impl Default for StateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer<StateDatabase> for StateRenderer {
    type Output = String;

    fn render(&self, database: &StateDatabase) -> Result<Self::Output> {
        self.render(database)
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
    use crate::core::{EdgeData, NodeData};

    #[test]
    fn test_render_empty() {
        let db = StateDatabase::new();
        let renderer = StateRenderer::new();
        let output = renderer.render(&db).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_render_single_state() {
        let mut db = StateDatabase::new();
        db.add_state(NodeData::new("Idle", "Idle")).unwrap();

        let renderer = StateRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.contains("Idle"));
        assert!(output.contains('┌') || output.contains('+'));
    }

    #[test]
    fn test_render_with_terminal() {
        let mut db = StateDatabase::new();
        db.add_transition(EdgeData::new("[*]", "Idle")).unwrap();

        let renderer = StateRenderer::new();
        let output = renderer.render(&db).unwrap();

        // Should have terminal marker and state
        assert!(output.contains("●") || output.contains("*"));
        assert!(output.contains("Idle"));
    }

    #[test]
    fn test_render_ascii_mode() {
        let mut db = StateDatabase::new();
        db.add_state(NodeData::new("Test", "Test")).unwrap();

        let renderer = StateRenderer::with_style(CharacterSet::Ascii);
        let output = renderer.render(&db).unwrap();

        // Should use ASCII characters
        assert!(output.contains('+'));
        assert!(output.contains('-'));
    }
}
