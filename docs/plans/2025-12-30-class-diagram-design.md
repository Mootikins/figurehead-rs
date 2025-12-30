# Class Diagram Plugin Design

## Scope

**Phase A (Minimal)**: Classes with attributes and methods only
**Phase B (Core)**: Add relationships (inheritance, composition, aggregation, association)

## ASCII Rendering Target

```
┌─────────────────┐
│     Animal      │
├─────────────────┤
│ +name: string   │
│ -age: int       │
├─────────────────┤
│ +eat()          │
│ +sleep()        │
│ #digest()*      │
└─────────────────┘
```

## Data Model

```rust
pub struct Class {
    pub name: String,
    pub attributes: Vec<Member>,
    pub methods: Vec<Member>,
    pub annotation: Option<String>,  // <<interface>>, <<abstract>>
}

pub struct Member {
    pub visibility: Visibility,
    pub name: String,
    pub member_type: Option<String>,
    pub classifier: Option<Classifier>,
}

pub enum Visibility { Public, Private, Protected, Package }
pub enum Classifier { Abstract, Static }

// Phase B additions
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub kind: RelationshipKind,
    pub label: Option<String>,
    pub from_cardinality: Option<String>,
    pub to_cardinality: Option<String>,
}

pub enum RelationshipKind {
    Inheritance,    // <|--
    Composition,    // *--
    Aggregation,    // o--
    Association,    // -->
    Dependency,     // ..>
    Realization,    // ..|>
}
```

## Plugin Structure

```
src/plugins/class/
├── mod.rs           # ClassDiagram struct, trait impls
├── detector.rs      # Detects "classDiagram" keyword
├── parser.rs        # Chumsky parser for class syntax
├── database.rs      # ClassDatabase storage
├── layout.rs        # Grid positioning
└── renderer.rs      # ASCII box drawing
```

## Detection

- Primary: `classDiagram` keyword (confidence 1.0)
- Secondary: class patterns like `<|--`, `*--`, `class X {` (confidence 0.7)

## Layout Algorithm

**Single class**: Center on canvas
**Multiple classes**: Left-to-right grid with 2-char spacing

**Box sizing**:
- Width = max(class name, longest member) + 2
- Height = 3 (header) + attributes.len() + 1 + methods.len() + 1

## Testing Strategy (TDD)

1. Detector tests → implement detector
2. Parser tests (empty class) → basic parser
3. Parser tests (members) → extend parser
4. Database tests → implement database
5. Layout tests → implement layout
6. Renderer tests → implement renderer
7. Snapshot integration tests

## Snapshot Test Cases

- `class_single.mmd` → `class_single.txt`
- `class_with_members.mmd` → `class_with_members.txt`
- `class_multiple.mmd` → `class_multiple.txt`
- `class_visibility.mmd` → `class_visibility.txt`
