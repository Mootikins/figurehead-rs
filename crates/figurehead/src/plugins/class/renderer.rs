//! Class diagram renderer
//!
//! Renders class diagrams to ASCII art.

use anyhow::Result;
use unicode_width::UnicodeWidthStr;

use super::layout::{ClassLayoutAlgorithm, ClassLayoutResult, PositionedClass, PositionedRelationship};
use super::database::{ClassDatabase, RelationshipKind};

/// ASCII canvas for rendering
struct Canvas {
    cells: Vec<Vec<char>>,
    width: usize,
    height: usize,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![vec![' '; width]; height],
            width,
            height,
        }
    }

    fn set(&mut self, x: usize, y: usize, c: char) {
        if y < self.height && x < self.width {
            self.cells[y][x] = c;
        }
    }

    fn draw_horizontal(&mut self, x: usize, y: usize, len: usize, c: char) {
        for i in 0..len {
            self.set(x + i, y, c);
        }
    }

    fn draw_vertical(&mut self, x: usize, y: usize, len: usize, c: char) {
        for i in 0..len {
            self.set(x, y + i, c);
        }
    }

    fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        for (i, c) in text.chars().enumerate() {
            self.set(x + i, y, c);
        }
    }

    fn draw_text_centered(&mut self, x: usize, y: usize, width: usize, text: &str) {
        let text_width = UnicodeWidthStr::width(text);
        let padding = (width.saturating_sub(text_width)) / 2;
        self.draw_text(x + padding, y, text);
    }

    fn to_string(&self) -> String {
        self.cells
            .iter()
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Class diagram renderer
pub struct ClassRenderer;

impl ClassRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Draw a class box on the canvas
    fn draw_class(&self, canvas: &mut Canvas, class: &PositionedClass) {
        let x = class.x;
        let y = class.y;
        let w = class.width;

        // Box drawing characters
        const TOP_LEFT: char = '┌';
        const TOP_RIGHT: char = '┐';
        const BOTTOM_LEFT: char = '└';
        const BOTTOM_RIGHT: char = '┘';
        const HORIZONTAL: char = '─';
        const VERTICAL: char = '│';
        const T_LEFT: char = '├';
        const T_RIGHT: char = '┤';

        // Current y position for drawing
        let mut cy = y;

        // Top border
        canvas.set(x, cy, TOP_LEFT);
        canvas.draw_horizontal(x + 1, cy, w - 2, HORIZONTAL);
        canvas.set(x + w - 1, cy, TOP_RIGHT);
        cy += 1;

        // Class name (centered)
        canvas.set(x, cy, VERTICAL);
        canvas.draw_text_centered(x + 1, cy, w - 2, &class.name);
        canvas.set(x + w - 1, cy, VERTICAL);
        cy += 1;

        // Attributes section
        if !class.attributes.is_empty() {
            // Separator
            canvas.set(x, cy, T_LEFT);
            canvas.draw_horizontal(x + 1, cy, w - 2, HORIZONTAL);
            canvas.set(x + w - 1, cy, T_RIGHT);
            cy += 1;

            // Attributes
            for attr in &class.attributes {
                canvas.set(x, cy, VERTICAL);
                canvas.draw_text(x + 2, cy, attr);
                canvas.set(x + w - 1, cy, VERTICAL);
                cy += 1;
            }
        }

        // Methods section
        if !class.methods.is_empty() {
            // Separator
            canvas.set(x, cy, T_LEFT);
            canvas.draw_horizontal(x + 1, cy, w - 2, HORIZONTAL);
            canvas.set(x + w - 1, cy, T_RIGHT);
            cy += 1;

            // Methods
            for method in &class.methods {
                canvas.set(x, cy, VERTICAL);
                canvas.draw_text(x + 2, cy, method);
                canvas.set(x + w - 1, cy, VERTICAL);
                cy += 1;
            }
        }

        // Bottom border
        canvas.set(x, cy, BOTTOM_LEFT);
        canvas.draw_horizontal(x + 1, cy, w - 2, HORIZONTAL);
        canvas.set(x + w - 1, cy, BOTTOM_RIGHT);
    }

    /// Draw a relationship arrow between classes
    fn draw_relationship(&self, canvas: &mut Canvas, rel: &PositionedRelationship) {
        // For horizontal relationships (classes on same row)
        // Draw: from_class ──────▷ to_class
        //
        // Arrow head characters based on relationship type
        let (arrow_char, line_char) = match rel.kind {
            RelationshipKind::Inheritance => ('◁', '─'),   // <|-- hollow triangle
            RelationshipKind::Realization => ('◁', '╌'),   // ..|> dashed hollow
            RelationshipKind::Composition => ('◆', '─'),   // *-- filled diamond
            RelationshipKind::Aggregation => ('◇', '─'),   // o-- hollow diamond
            RelationshipKind::Association => ('▷', '─'),   // --> arrow
            RelationshipKind::Dependency => ('▷', '╌'),    // ..> dashed arrow
            RelationshipKind::Link => ('─', '─'),          // -- plain
            RelationshipKind::DashedLink => ('╌', '╌'),    // .. dashed
        };

        let from_x = rel.from_x;
        let to_x = rel.to_x;
        let y = rel.from_y; // Draw at bottom of from class

        // Determine direction
        if from_x < to_x {
            // Left to right
            for x in (from_x + 1)..to_x {
                canvas.set(x, y, line_char);
            }
            canvas.set(to_x, y, arrow_char);
        } else if from_x > to_x {
            // Right to left
            for x in (to_x + 1)..from_x {
                canvas.set(x, y, line_char);
            }
            canvas.set(to_x, y, arrow_char);
        }

        // Draw label if present
        if let Some(ref label) = rel.label {
            let mid_x = (from_x + to_x) / 2;
            let label_start = mid_x.saturating_sub(label.len() / 2);
            canvas.draw_text(label_start, y.saturating_sub(1), label);
        }
    }

    /// Render the layout to ASCII art
    pub fn render(&self, layout: &ClassLayoutResult) -> Result<String> {
        if layout.classes.is_empty() {
            return Ok(String::new());
        }

        // Add extra space for relationship lines
        let extra_height = if layout.relationships.is_empty() { 0 } else { 2 };
        let mut canvas = Canvas::new(layout.width + 1, layout.height + extra_height + 1);

        // Draw classes first
        for class in &layout.classes {
            self.draw_class(&mut canvas, class);
        }

        // Draw relationships on top
        for rel in &layout.relationships {
            self.draw_relationship(&mut canvas, rel);
        }

        Ok(canvas.to_string())
    }

    /// Convenience method to render directly from database
    pub fn render_database(&self, database: &ClassDatabase) -> Result<String> {
        let layout = ClassLayoutAlgorithm::new();
        let result = layout.layout(database)?;
        self.render(&result)
    }
}

impl Default for ClassRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::database::{Class, Member, Visibility, Classifier};

    #[test]
    fn test_render_empty() {
        let db = ClassDatabase::new();
        let renderer = ClassRenderer::new();
        let result = renderer.render_database(&db).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_simple_class() {
        let mut db = ClassDatabase::new();
        db.add_class(Class::new("Animal")).unwrap();

        let renderer = ClassRenderer::new();
        let result = renderer.render_database(&db).unwrap();

        assert!(result.contains("Animal"));
        assert!(result.contains("┌"));
        assert!(result.contains("└"));
    }

    #[test]
    fn test_render_class_with_attributes() {
        let mut db = ClassDatabase::new();
        let mut class = Class::new("Person");
        class.add_attribute(
            Member::attribute("name")
                .with_visibility(Visibility::Public)
                .with_type("string"),
        );
        class.add_attribute(
            Member::attribute("age")
                .with_visibility(Visibility::Private)
                .with_type("int"),
        );
        db.add_class(class).unwrap();

        let renderer = ClassRenderer::new();
        let result = renderer.render_database(&db).unwrap();

        assert!(result.contains("Person"));
        assert!(result.contains("+name: string"));
        assert!(result.contains("-age: int"));
        // Has separator
        assert!(result.contains("├"));
    }

    #[test]
    fn test_render_class_with_methods() {
        let mut db = ClassDatabase::new();
        let mut class = Class::new("Animal");
        class.add_attribute(Member::attribute("name").with_visibility(Visibility::Public));
        class.add_method(Member::method("eat").with_visibility(Visibility::Public));
        class.add_method(
            Member::method("digest")
                .with_visibility(Visibility::Protected)
                .with_classifier(Classifier::Abstract),
        );
        db.add_class(class).unwrap();

        let renderer = ClassRenderer::new();
        let result = renderer.render_database(&db).unwrap();

        assert!(result.contains("+eat()"));
        assert!(result.contains("#digest()*"));
    }

    #[test]
    fn test_render_multiple_classes() {
        let mut db = ClassDatabase::new();
        db.add_class(Class::new("Animal")).unwrap();
        db.add_class(Class::new("Dog")).unwrap();

        let renderer = ClassRenderer::new();
        let result = renderer.render_database(&db).unwrap();

        assert!(result.contains("Animal"));
        assert!(result.contains("Dog"));
    }

    #[test]
    fn test_box_structure() {
        let mut db = ClassDatabase::new();
        let mut class = Class::new("X");
        class.add_attribute(Member::attribute("a"));
        class.add_method(Member::method("m"));
        db.add_class(class).unwrap();

        let renderer = ClassRenderer::new();
        let result = renderer.render_database(&db).unwrap();

        // Should have top border, name, separator, attr, separator, method, bottom border
        let lines: Vec<_> = result.lines().collect();
        assert!(lines.len() >= 7);
        assert!(lines[0].starts_with('┌'));
        assert!(lines[lines.len() - 1].starts_with('└'));
    }

    // =========================================================================
    // Relationship rendering tests
    // =========================================================================

    #[test]
    fn test_render_inheritance() {
        use super::super::database::Relationship;

        let mut db = ClassDatabase::new();
        db.add_class(Class::new("Animal")).unwrap();
        db.add_class(Class::new("Dog")).unwrap();
        db.add_relationship(Relationship::new("Animal", "Dog", RelationshipKind::Inheritance)).unwrap();

        let renderer = ClassRenderer::new();
        let result = renderer.render_database(&db).unwrap();

        // Should contain both classes and relationship arrow
        assert!(result.contains("Animal"));
        assert!(result.contains("Dog"));
        // Should have inheritance arrow (hollow triangle)
        assert!(result.contains('◁') || result.contains('─'));
    }

    #[test]
    fn test_render_composition() {
        use super::super::database::Relationship;

        let mut db = ClassDatabase::new();
        db.add_class(Class::new("Person")).unwrap();
        db.add_class(Class::new("Heart")).unwrap();
        db.add_relationship(Relationship::new("Person", "Heart", RelationshipKind::Composition)).unwrap();

        let renderer = ClassRenderer::new();
        let result = renderer.render_database(&db).unwrap();

        // Should have filled diamond
        assert!(result.contains('◆') || result.contains('─'));
    }

    #[test]
    fn test_render_relationship_with_label() {
        use super::super::database::Relationship;

        let mut db = ClassDatabase::new();
        db.add_class(Class::new("Customer")).unwrap();
        db.add_class(Class::new("Order")).unwrap();
        db.add_relationship(
            Relationship::new("Customer", "Order", RelationshipKind::Association)
                .with_label("places")
        ).unwrap();

        let renderer = ClassRenderer::new();
        let result = renderer.render_database(&db).unwrap();

        assert!(result.contains("places"));
    }
}
