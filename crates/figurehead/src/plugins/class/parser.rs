//! Class diagram parser
//!
//! Parses class diagram syntax into the database.

use anyhow::Result;
use crate::core::Parser;
use super::database::{Class, ClassDatabase, Classifier, Member, Relationship, RelationshipKind, Visibility};

/// Class diagram parser
pub struct ClassParser;

impl ClassParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a relationship line like "Animal <|-- Dog" or "A --> B : label"
    fn parse_relationship(&self, line: &str) -> Option<(String, String, RelationshipKind, Option<String>)> {
        // Relationship patterns (longest first to avoid partial matches)
        let patterns = [
            ("<|--", RelationshipKind::Inheritance),
            ("..|>", RelationshipKind::Realization),
            ("*--", RelationshipKind::Composition),
            ("o--", RelationshipKind::Aggregation),
            ("..>", RelationshipKind::Dependency),
            ("-->", RelationshipKind::Association),
            ("..", RelationshipKind::DashedLink),
            ("--", RelationshipKind::Link),
        ];

        for (pattern, kind) in patterns {
            if let Some(pos) = line.find(pattern) {
                let from = line[..pos].trim().to_string();
                let rest = &line[pos + pattern.len()..];

                // Check for label after ":"
                let (to, label) = if let Some(colon_pos) = rest.find(':') {
                    let to = rest[..colon_pos].trim().to_string();
                    let label = rest[colon_pos + 1..].trim().to_string();
                    (to, if label.is_empty() { None } else { Some(label) })
                } else {
                    (rest.trim().to_string(), None)
                };

                if !from.is_empty() && !to.is_empty() {
                    return Some((from, to, kind, label));
                }
            }
        }
        None
    }

    /// Parse a class body line into a Member
    fn parse_member(&self, line: &str) -> Option<Member> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        let mut chars = line.chars().peekable();

        // Check for visibility prefix
        let visibility = chars.peek().and_then(|&c| Visibility::from_char(c));
        if visibility.is_some() {
            chars.next();
        }

        // Collect the rest
        let rest: String = chars.collect();
        let rest = rest.trim();

        if rest.is_empty() {
            return None;
        }

        // Check for classifier suffix (* or $)
        let (rest, classifier) = if rest.ends_with('*') {
            (rest.trim_end_matches('*').trim(), Some(Classifier::Abstract))
        } else if rest.ends_with('$') {
            (rest.trim_end_matches('$').trim(), Some(Classifier::Static))
        } else {
            (rest, None)
        };

        // Check if it's a method (has parentheses)
        let is_method = rest.contains('(');

        // Parse name and type
        if is_method {
            // Method: name() or name(): type or name(args): type
            let paren_pos = rest.find('(')?;
            let name = rest[..paren_pos].trim().to_string();

            // Check for return type after )
            let member_type = if let Some(close_paren) = rest.find(')') {
                let after = rest[close_paren + 1..].trim();
                if let Some(colon_pos) = after.find(':') {
                    Some(after[colon_pos + 1..].trim().to_string())
                } else if after.starts_with(':') {
                    Some(after[1..].trim().to_string())
                } else {
                    None
                }
            } else {
                None
            };

            Some(Member {
                visibility,
                name,
                member_type,
                classifier,
                is_method: true,
            })
        } else {
            // Attribute: name or name: type
            let (name, member_type) = if let Some(colon_pos) = rest.find(':') {
                (
                    rest[..colon_pos].trim().to_string(),
                    Some(rest[colon_pos + 1..].trim().to_string()),
                )
            } else {
                (rest.to_string(), None)
            };

            Some(Member {
                visibility,
                name,
                member_type,
                classifier,
                is_method: false,
            })
        }
    }
}

impl Default for ClassParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser<ClassDatabase> for ClassParser {
    fn parse(&self, input: &str, database: &mut ClassDatabase) -> Result<()> {
        let mut current_class: Option<Class> = None;
        let mut in_class_body = false;

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines and diagram declaration
            if line.is_empty() || line.to_lowercase().starts_with("classdiagram") {
                continue;
            }

            // Handle class definition with body: class Name {
            if line.starts_with("class ") {
                // Save previous class if any
                if let Some(class) = current_class.take() {
                    database.add_class(class)?;
                }

                let rest = line.strip_prefix("class ").unwrap().trim();

                if rest.ends_with('{') {
                    // Start of class body
                    let name = rest.trim_end_matches('{').trim();
                    current_class = Some(Class::new(name));
                    in_class_body = true;
                } else {
                    // Single-line class definition (no body)
                    database.add_class(Class::new(rest))?;
                }
                continue;
            }

            // Handle end of class body
            if line == "}" {
                if let Some(class) = current_class.take() {
                    database.add_class(class)?;
                }
                in_class_body = false;
                continue;
            }

            // Parse member if inside class body
            if in_class_body {
                if let Some(member) = self.parse_member(line) {
                    if let Some(ref mut class) = current_class {
                        if member.is_method {
                            class.add_method(member);
                        } else {
                            class.add_attribute(member);
                        }
                    }
                }
                continue;
            }

            // Try to parse as relationship (outside class body)
            if let Some((from, to, kind, label)) = self.parse_relationship(line) {
                // Ensure classes exist
                database.get_or_create_class(&from);
                database.get_or_create_class(&to);

                let mut rel = Relationship::new(from, to, kind);
                if let Some(lbl) = label {
                    rel = rel.with_label(lbl);
                }
                database.add_relationship(rel)?;
            }
        }

        // Handle class without closing brace
        if let Some(class) = current_class {
            database.add_class(class)?;
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "class"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn can_parse(&self, input: &str) -> bool {
        let lower = input.to_lowercase();
        lower.contains("classdiagram") || lower.contains("class ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_class() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        parser.parse("classDiagram\n    class Animal", &mut db).unwrap();

        assert_eq!(db.class_count(), 1);
        assert_eq!(db.classes()[0].name, "Animal");
    }

    #[test]
    fn test_parse_class_with_body() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        let input = r#"classDiagram
    class Animal {
        +name: string
        -age: int
    }"#;

        parser.parse(input, &mut db).unwrap();

        assert_eq!(db.class_count(), 1);
        let class = &db.classes()[0];
        assert_eq!(class.name, "Animal");
        assert_eq!(class.attributes.len(), 2);
        assert_eq!(class.attributes[0].name, "name");
        assert_eq!(class.attributes[0].visibility, Some(Visibility::Public));
        assert_eq!(class.attributes[0].member_type, Some("string".to_string()));
    }

    #[test]
    fn test_parse_methods() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        let input = r#"classDiagram
    class Animal {
        +eat()
        +sleep(): void
        #digest()*
    }"#;

        parser.parse(input, &mut db).unwrap();

        let class = &db.classes()[0];
        assert_eq!(class.methods.len(), 3);
        assert_eq!(class.methods[0].name, "eat");
        assert!(class.methods[0].is_method);
        assert_eq!(class.methods[1].member_type, Some("void".to_string()));
        assert_eq!(class.methods[2].classifier, Some(Classifier::Abstract));
    }

    #[test]
    fn test_parse_all_visibility() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        let input = r#"classDiagram
    class Test {
        +public
        -private
        #protected
        ~package
    }"#;

        parser.parse(input, &mut db).unwrap();

        let attrs = &db.classes()[0].attributes;
        assert_eq!(attrs[0].visibility, Some(Visibility::Public));
        assert_eq!(attrs[1].visibility, Some(Visibility::Private));
        assert_eq!(attrs[2].visibility, Some(Visibility::Protected));
        assert_eq!(attrs[3].visibility, Some(Visibility::Package));
    }

    #[test]
    fn test_parse_static_method() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        let input = r#"classDiagram
    class Util {
        +getInstance()$
    }"#;

        parser.parse(input, &mut db).unwrap();

        let method = &db.classes()[0].methods[0];
        assert_eq!(method.classifier, Some(Classifier::Static));
    }

    #[test]
    fn test_parse_multiple_classes() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        let input = r#"classDiagram
    class Animal {
        +name
    }
    class Dog {
        +breed
    }"#;

        parser.parse(input, &mut db).unwrap();

        assert_eq!(db.class_count(), 2);
        assert_eq!(db.classes()[0].name, "Animal");
        assert_eq!(db.classes()[1].name, "Dog");
    }

    #[test]
    fn test_parse_member_line() {
        let parser = ClassParser::new();

        let member = parser.parse_member("+name: string").unwrap();
        assert_eq!(member.name, "name");
        assert_eq!(member.visibility, Some(Visibility::Public));
        assert_eq!(member.member_type, Some("string".to_string()));
        assert!(!member.is_method);

        let method = parser.parse_member("-calculate()*").unwrap();
        assert_eq!(method.name, "calculate");
        assert_eq!(method.visibility, Some(Visibility::Private));
        assert_eq!(method.classifier, Some(Classifier::Abstract));
        assert!(method.is_method);
    }

    // =========================================================================
    // Relationship parsing tests
    // =========================================================================

    #[test]
    fn test_parse_inheritance() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        parser.parse("classDiagram\n    Animal <|-- Dog", &mut db).unwrap();

        assert_eq!(db.class_count(), 2);
        assert_eq!(db.relationship_count(), 1);
        let rel = &db.relationships()[0];
        assert_eq!(rel.from, "Animal");
        assert_eq!(rel.to, "Dog");
        assert_eq!(rel.kind, RelationshipKind::Inheritance);
    }

    #[test]
    fn test_parse_composition() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        parser.parse("classDiagram\n    Person *-- Heart", &mut db).unwrap();

        assert_eq!(db.relationship_count(), 1);
        let rel = &db.relationships()[0];
        assert_eq!(rel.kind, RelationshipKind::Composition);
    }

    #[test]
    fn test_parse_aggregation() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        parser.parse("classDiagram\n    Library o-- Book", &mut db).unwrap();

        let rel = &db.relationships()[0];
        assert_eq!(rel.kind, RelationshipKind::Aggregation);
    }

    #[test]
    fn test_parse_association() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        parser.parse("classDiagram\n    Student --> Course", &mut db).unwrap();

        let rel = &db.relationships()[0];
        assert_eq!(rel.kind, RelationshipKind::Association);
    }

    #[test]
    fn test_parse_dependency() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        parser.parse("classDiagram\n    Client ..> Service", &mut db).unwrap();

        let rel = &db.relationships()[0];
        assert_eq!(rel.kind, RelationshipKind::Dependency);
    }

    #[test]
    fn test_parse_realization() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        parser.parse("classDiagram\n    Shape ..|> Drawable", &mut db).unwrap();

        let rel = &db.relationships()[0];
        assert_eq!(rel.kind, RelationshipKind::Realization);
    }

    #[test]
    fn test_parse_relationship_with_label() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        parser.parse("classDiagram\n    Customer --> Order : places", &mut db).unwrap();

        let rel = &db.relationships()[0];
        assert_eq!(rel.label, Some("places".to_string()));
    }

    #[test]
    fn test_parse_mixed_classes_and_relationships() {
        let parser = ClassParser::new();
        let mut db = ClassDatabase::new();

        let input = r#"classDiagram
    class Animal {
        +name: string
    }
    class Dog {
        +breed: string
    }
    Animal <|-- Dog"#;

        parser.parse(input, &mut db).unwrap();

        assert_eq!(db.class_count(), 2);
        assert_eq!(db.relationship_count(), 1);
    }
}
