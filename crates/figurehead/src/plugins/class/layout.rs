//! Class diagram layout algorithm
//!
//! Calculates positions for class boxes in a grid layout.

use anyhow::Result;
use unicode_width::UnicodeWidthStr;

use super::database::{Class, ClassDatabase, Visibility, Classifier};

/// Positioned class box for rendering
#[derive(Debug, Clone)]
pub struct PositionedClass {
    pub name: String,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub annotation: Option<String>,
    pub attributes: Vec<String>,
    pub methods: Vec<String>,
}

/// Layout result containing all positioned elements
#[derive(Debug)]
pub struct ClassLayoutResult {
    pub classes: Vec<PositionedClass>,
    pub width: usize,
    pub height: usize,
}

/// Class diagram layout algorithm
pub struct ClassLayoutAlgorithm {
    box_padding: usize,
    box_spacing: usize,
    max_classes_per_row: usize,
}

impl ClassLayoutAlgorithm {
    pub fn new() -> Self {
        Self {
            box_padding: 1,
            box_spacing: 2,
            max_classes_per_row: 3,
        }
    }

    /// Format a class member for display
    fn format_member(
        visibility: Option<Visibility>,
        name: &str,
        member_type: Option<&str>,
        classifier: Option<Classifier>,
        is_method: bool,
    ) -> String {
        let vis = visibility.map(|v| v.to_char()).unwrap_or(' ');
        let suffix = classifier.map(|c| c.to_char()).unwrap_or(' ');
        let suffix_str = if suffix == ' ' { String::new() } else { suffix.to_string() };

        if is_method {
            if let Some(t) = member_type {
                format!("{}{}(): {}{}", vis, name, t, suffix_str)
            } else {
                format!("{}{}(){}", vis, name, suffix_str)
            }
        } else {
            if let Some(t) = member_type {
                format!("{}{}: {}{}", vis, name, t, suffix_str)
            } else {
                format!("{}{}{}", vis, name, suffix_str)
            }
        }
    }

    /// Calculate dimensions needed for a class box
    fn class_dimensions(&self, class: &Class) -> (usize, usize) {
        let mut max_width = UnicodeWidthStr::width(class.name.as_str());

        // Format and measure attributes
        let attrs: Vec<String> = class.attributes.iter().map(|m| {
            Self::format_member(
                m.visibility,
                &m.name,
                m.member_type.as_deref(),
                m.classifier,
                false,
            )
        }).collect();

        for attr in &attrs {
            max_width = max_width.max(UnicodeWidthStr::width(attr.as_str()));
        }

        // Format and measure methods
        let methods: Vec<String> = class.methods.iter().map(|m| {
            Self::format_member(
                m.visibility,
                &m.name,
                m.member_type.as_deref(),
                m.classifier,
                true,
            )
        }).collect();

        for method in &methods {
            max_width = max_width.max(UnicodeWidthStr::width(method.as_str()));
        }

        // Add padding
        let width = max_width + self.box_padding * 2 + 2; // +2 for borders

        // Calculate height:
        // - 1 for top border
        // - 1 for class name
        // - 1 for separator (if has attrs)
        // - N for attributes
        // - 1 for separator (if has methods)
        // - M for methods
        // - 1 for bottom border
        let mut height = 3; // top border, name, bottom border
        if !class.attributes.is_empty() {
            height += 1 + class.attributes.len(); // separator + attrs
        }
        if !class.methods.is_empty() {
            height += 1 + class.methods.len(); // separator + methods
        }

        (width, height)
    }

    /// Layout the diagram
    pub fn layout(&self, database: &ClassDatabase) -> Result<ClassLayoutResult> {
        let classes = database.classes();

        if classes.is_empty() {
            return Ok(ClassLayoutResult {
                classes: Vec::new(),
                width: 0,
                height: 0,
            });
        }

        // Pre-calculate all dimensions and formatted content
        let class_info: Vec<_> = classes.iter().map(|c| {
            let (width, height) = self.class_dimensions(c);
            let attrs: Vec<String> = c.attributes.iter().map(|m| {
                Self::format_member(
                    m.visibility,
                    &m.name,
                    m.member_type.as_deref(),
                    m.classifier,
                    false,
                )
            }).collect();
            let methods: Vec<String> = c.methods.iter().map(|m| {
                Self::format_member(
                    m.visibility,
                    &m.name,
                    m.member_type.as_deref(),
                    m.classifier,
                    true,
                )
            }).collect();
            (c, width, height, attrs, methods)
        }).collect();

        // Arrange in rows
        let mut positioned = Vec::new();
        let mut x = 0;
        let mut y = 0;
        let mut row_height = 0;
        let mut max_width = 0;
        let mut classes_in_row = 0;

        for (class, width, height, attrs, methods) in class_info {
            // Start new row if needed
            if classes_in_row >= self.max_classes_per_row {
                y += row_height + self.box_spacing;
                x = 0;
                row_height = 0;
                classes_in_row = 0;
            }

            positioned.push(PositionedClass {
                name: class.name.clone(),
                x,
                y,
                width,
                height,
                annotation: class.annotation.clone(),
                attributes: attrs,
                methods,
            });

            x += width + self.box_spacing;
            max_width = max_width.max(x);
            row_height = row_height.max(height);
            classes_in_row += 1;
        }

        let total_width = max_width;
        let total_height = y + row_height;

        Ok(ClassLayoutResult {
            classes: positioned,
            width: total_width,
            height: total_height,
        })
    }
}

impl Default for ClassLayoutAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::database::{Member, Visibility};

    #[test]
    fn test_empty_layout() {
        let db = ClassDatabase::new();
        let layout = ClassLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        assert_eq!(result.classes.len(), 0);
        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
    }

    #[test]
    fn test_single_class() {
        let mut db = ClassDatabase::new();
        db.add_class(Class::new("Animal")).unwrap();

        let layout = ClassLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        assert_eq!(result.classes.len(), 1);
        assert_eq!(result.classes[0].name, "Animal");
        assert_eq!(result.classes[0].x, 0);
        assert_eq!(result.classes[0].y, 0);
    }

    #[test]
    fn test_class_with_members() {
        let mut db = ClassDatabase::new();
        let mut class = Class::new("Person");
        class.add_attribute(
            Member::attribute("name")
                .with_visibility(Visibility::Public)
                .with_type("string"),
        );
        class.add_method(Member::method("greet").with_visibility(Visibility::Public));
        db.add_class(class).unwrap();

        let layout = ClassLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        assert_eq!(result.classes.len(), 1);
        // Has 2 attributes, 1 method after formatting
        assert_eq!(result.classes[0].attributes.len(), 1);
        assert_eq!(result.classes[0].methods.len(), 1);
    }

    #[test]
    fn test_multiple_classes_grid() {
        let mut db = ClassDatabase::new();
        db.add_class(Class::new("A")).unwrap();
        db.add_class(Class::new("B")).unwrap();
        db.add_class(Class::new("C")).unwrap();
        db.add_class(Class::new("D")).unwrap();

        let layout = ClassLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        assert_eq!(result.classes.len(), 4);
        // First 3 on first row
        assert_eq!(result.classes[0].y, result.classes[1].y);
        assert_eq!(result.classes[1].y, result.classes[2].y);
        // 4th on second row
        assert!(result.classes[3].y > result.classes[0].y);
    }

    #[test]
    fn test_class_width_accommodates_members() {
        let mut db = ClassDatabase::new();
        let mut class = Class::new("X");
        class.add_attribute(
            Member::attribute("veryLongAttributeName")
                .with_type("VeryLongTypeName"),
        );
        db.add_class(class).unwrap();

        let layout = ClassLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        // Width should accommodate the long member line
        assert!(result.classes[0].width > 20);
    }

    #[test]
    fn test_format_member() {
        let formatted = ClassLayoutAlgorithm::format_member(
            Some(Visibility::Public),
            "name",
            Some("string"),
            None,
            false,
        );
        assert_eq!(formatted, "+name: string");

        let method = ClassLayoutAlgorithm::format_member(
            Some(Visibility::Private),
            "calc",
            None,
            Some(Classifier::Abstract),
            true,
        );
        assert_eq!(method, "-calc()*");
    }
}
