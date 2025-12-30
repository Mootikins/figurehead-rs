//! Sequence diagram ASCII renderer
//!
//! Renders sequence diagrams as ASCII art.

use anyhow::Result;

use super::database::{ArrowHead, LineStyle, SequenceDatabase};
use super::layout::{SequenceLayoutAlgorithm, SequenceLayoutResult};
use crate::core::CharacterSet;

/// ASCII canvas for sequence diagram rendering
struct Canvas {
    width: usize,
    height: usize,
    grid: Vec<Vec<char>>,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![' '; width.max(1)]; height.max(1)];
        Self { width, height, grid }
    }

    fn set_char(&mut self, x: usize, y: usize, c: char) {
        if y < self.height && x < self.width {
            self.grid[y][x] = c;
        }
    }

    fn draw_text_centered(&mut self, center_x: usize, y: usize, text: &str) {
        let start_x = center_x.saturating_sub(text.len() / 2);
        for (i, c) in text.chars().enumerate() {
            self.set_char(start_x + i, y, c);
        }
    }

    fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        for (i, c) in text.chars().enumerate() {
            self.set_char(x + i, y, c);
        }
    }

    fn draw_horizontal_line(&mut self, x1: usize, x2: usize, y: usize, solid: bool, unicode: bool) {
        let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        let line_char = if solid {
            if unicode { '─' } else { '-' }
        } else {
            if unicode { '╌' } else { '-' }
        };

        for x in start..=end {
            self.set_char(x, y, line_char);
        }
    }

    fn draw_vertical_line(&mut self, x: usize, y1: usize, y2: usize, unicode: bool) {
        let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        let line_char = if unicode { '│' } else { '|' };

        for y in start..=end {
            // Don't overwrite existing non-space characters (like arrows)
            if self.grid[y][x] == ' ' {
                self.set_char(x, y, line_char);
            }
        }
    }

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

/// Sequence diagram renderer
pub struct SequenceRenderer {
    style: CharacterSet,
}

impl SequenceRenderer {
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

    /// Draw a participant header box
    fn draw_participant(&self, canvas: &mut Canvas, x: usize, y: usize, label: &str, width: usize) {
        let unicode = self.is_unicode();

        // Draw box around label
        let left = x.saturating_sub(width / 2);
        let right = left + width - 1;

        if unicode {
            // Top border
            canvas.set_char(left, y, '┌');
            for i in (left + 1)..right {
                canvas.set_char(i, y, '─');
            }
            canvas.set_char(right, y, '┐');

            // Sides and label
            canvas.set_char(left, y + 1, '│');
            canvas.set_char(right, y + 1, '│');

            // Bottom border
            canvas.set_char(left, y + 2, '└');
            for i in (left + 1)..right {
                canvas.set_char(i, y + 2, '─');
            }
            canvas.set_char(right, y + 2, '┘');
        } else {
            // ASCII box
            canvas.set_char(left, y, '+');
            for i in (left + 1)..right {
                canvas.set_char(i, y, '-');
            }
            canvas.set_char(right, y, '+');

            canvas.set_char(left, y + 1, '|');
            canvas.set_char(right, y + 1, '|');

            canvas.set_char(left, y + 2, '+');
            for i in (left + 1)..right {
                canvas.set_char(i, y + 2, '-');
            }
            canvas.set_char(right, y + 2, '+');
        }

        // Center the label
        canvas.draw_text_centered(x, y + 1, label);
    }

    /// Draw a message arrow with label
    fn draw_message(
        &self,
        canvas: &mut Canvas,
        from_x: usize,
        to_x: usize,
        y: usize,
        label: &str,
        solid: bool,
        head: ArrowHead,
    ) {
        let unicode = self.is_unicode();
        let going_right = to_x > from_x;

        // Determine arrow head characters
        let (arrow_char, arrow_offset) = match head {
            ArrowHead::Arrow => {
                if unicode {
                    if going_right { ('▶', 0) } else { ('◀', 0) }
                } else {
                    if going_right { ('>', 0) } else { ('<', 0) }
                }
            }
            ArrowHead::Open => {
                if going_right { (')', 0) } else { ('(', 0) }
            }
            ArrowHead::None => {
                (' ', 1) // No arrow, just line
            }
        };

        // Draw the line (leaving space for arrow)
        let (line_start, line_end) = if going_right {
            (from_x + 1, to_x.saturating_sub(1 - arrow_offset))
        } else {
            (to_x + 1 + arrow_offset, from_x.saturating_sub(1))
        };

        if line_start < line_end {
            canvas.draw_horizontal_line(line_start, line_end, y, solid, unicode);
        }

        // Draw arrow head
        if head != ArrowHead::None {
            canvas.set_char(to_x, y, arrow_char);
        }

        // Draw label centered on the line
        if !label.is_empty() {
            let center_x = (from_x + to_x) / 2;
            canvas.draw_text_centered(center_x, y, label);
        }
    }

    /// Render the database to ASCII
    pub fn render(&self, database: &SequenceDatabase) -> Result<String> {
        let layout_algo = SequenceLayoutAlgorithm::new();
        let layout = layout_algo.layout(database)?;

        if layout.participants.is_empty() {
            return Ok(String::new());
        }

        let mut canvas = Canvas::new(layout.width, layout.height);
        let unicode = self.is_unicode();

        // Draw participant headers
        for participant in &layout.participants {
            self.draw_participant(
                &mut canvas,
                participant.x,
                0,
                &participant.label,
                participant.width,
            );
        }

        // Draw lifelines
        for participant in &layout.participants {
            canvas.draw_vertical_line(
                participant.x,
                layout.lifeline_start_y,
                layout.height - 1,
                unicode,
            );
        }

        // Draw messages
        for msg in &layout.messages {
            let solid = msg.arrow.line == LineStyle::Solid;
            self.draw_message(
                &mut canvas,
                msg.from_x,
                msg.to_x,
                msg.y,
                &msg.label,
                solid,
                msg.arrow.head,
            );
        }

        Ok(canvas.to_string())
    }
}

impl Default for SequenceRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::core::Renderer<SequenceDatabase> for SequenceRenderer {
    type Output = String;

    fn render(&self, database: &SequenceDatabase) -> Result<Self::Output> {
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
    use super::super::database::{Message, Participant, ArrowType};

    #[test]
    fn test_render_single_message() {
        let mut db = SequenceDatabase::new();
        db.add_message(Message::new("Alice", "Bob", "Hello")).unwrap();

        let renderer = SequenceRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(!output.is_empty());
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_render_multiple_messages() {
        let mut db = SequenceDatabase::new();
        db.add_message(Message::new("Alice", "Bob", "Hello")).unwrap();
        db.add_message(Message::new("Bob", "Alice", "Hi")).unwrap();

        let renderer = SequenceRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.contains("Hello"));
        assert!(output.contains("Hi"));
    }

    #[test]
    fn test_render_with_alias() {
        let mut db = SequenceDatabase::new();
        db.add_participant(Participant::with_label("A", "Alice")).unwrap();
        db.add_participant(Participant::with_label("B", "Bob")).unwrap();
        db.add_message(Message::new("A", "B", "Hi")).unwrap();

        let renderer = SequenceRenderer::new();
        let output = renderer.render(&db).unwrap();

        // Should show labels, not ids
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
    }

    #[test]
    fn test_render_empty_database() {
        let db = SequenceDatabase::new();
        let renderer = SequenceRenderer::new();
        let output = renderer.render(&db).unwrap();

        assert!(output.is_empty());
    }

    #[test]
    fn test_render_dotted_arrow() {
        let mut db = SequenceDatabase::new();
        let msg = Message::new("Alice", "Bob", "Response")
            .with_arrow(ArrowType::dotted_arrow());
        db.add_message(msg).unwrap();

        let renderer = SequenceRenderer::new();
        let output = renderer.render(&db).unwrap();

        // Should contain dotted line character
        assert!(output.contains('╌') || output.contains('-'));
    }
}
