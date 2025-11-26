# Architecture Documentation

This document describes the architecture of Figurehead, a Rust utility for converting Mermaid.js diagrams to ASCII art.

## Overview

Figurehead is inspired by the mermaid.js architecture but adapted for Rust with SOLID principles. It follows a modular, plugin-based design that separates concerns and enables extensibility.

## Core Philosophy

### 1. Plugin-Based Architecture
Like mermaid.js, each diagram type is a separate plugin that implements a common interface. This allows for easy addition of new diagram types without modifying core code.

### 2. Pipeline Processing
All diagram processing follows a consistent pipeline:
```
Input → Detector → Parser → Database → Layout → Renderer → Output
```

### 3. SOLID Principles
- **Single Responsibility**: Each component has one clear purpose
- **Open/Closed**: Extensible via plugins, closed to modification
- **Liskov Substitution**: All implementations are interchangeable
- **Interface Segregation**: Small, focused traits
- **Dependency Inversion**: Depends on abstractions

### 4. Rust Idioms
- Trait-based polymorphism
- Zero-cost abstractions
- Memory safety
- Error handling with `Result`

## Core Components

### 1. Diagram Trait (`src/core/diagram.rs`)

The main entry point for each diagram type:

```rust
pub trait Diagram: Send + Sync {
    type Database: Database + Send + Sync;
    type Parser: Parser<Self::Database> + Send + Sync;
    type Renderer: Renderer<Self::Database> + Send + Sync;

    fn detector() -> Arc<dyn Detector>;
    fn create_parser() -> Self::Parser;
    fn create_database() -> Self::Database;
    fn create_renderer() -> Self::Renderer;
    fn name() -> &'static str;
    fn version() -> &'static str;
}
```

**Purpose**: Factory pattern for creating all components of a specific diagram type.

### 2. Detector Trait (`src/core/detector.rs`)

Identifies diagram types from input patterns:

```rust
pub trait Detector: Send + Sync {
    fn detect(&self, input: &str) -> bool;
    fn confidence(&self, input: &str) -> f64;
    fn diagram_type(&self) -> &'static str;
    fn patterns(&self) -> Vec<&'static str>;
}
```

**Purpose**: Automatic diagram type detection using pattern matching.

### 3. Parser Trait (`src/core/parser.rs`)

Parses markup language into structured data:

```rust
pub trait Parser<D: Database>: Send + Sync {
    fn parse(&self, input: &str, database: &mut D) -> Result<()>;
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn can_parse(&self, input: &str) -> bool;
}
```

**Purpose**: Convert text markup into database records using chumsky.

### 4. Database Trait (`src/core/database.rs`)

Stores diagram nodes, edges, and metadata:

```rust
pub trait Database: Send + Sync {
    fn add_node(&mut self, id: &str, label: &str) -> Result<()>;
    fn add_edge(&mut self, from: &str, to: &str) -> Result<()>;
    fn get_node(&self, id: &str) -> Option<&str>;
    fn get_nodes(&self) -> Vec<(&str, &str)>;
    fn get_edges(&self) -> Vec<(&str, &str)>;
    // ... more methods
}
```

**Purpose**: Centralized data storage with CRUD operations.

### 5. Layout Algorithm Trait (`src/core/layout.rs`)

Arranges elements in coordinate space:

```rust
pub trait LayoutAlgorithm<D: Database>: Send + Sync {
    type Output;

    fn layout(&self, database: &D) -> Result<Self::Output>;
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn direction(&self) -> &'static str;
}
```

**Purpose**: Position nodes and route edges in ASCII coordinate system.

### 6. Renderer Trait (`src/core/renderer.rs`)

Generates final output in various formats:

```rust
pub trait Renderer<D: Database>: Send + Sync {
    type Output;

    fn render(&self, database: &D) -> Result<Self::Output>;
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn format(&self) -> &'static str;
}
```

**Purpose**: Convert positioned data to ASCII, SVG, or other formats.

## Plugin Implementation Example

### Flowchart Plugin Structure

```rust
// src/plugins/flowchart/mod.rs
pub struct FlowchartDiagram;

impl Diagram for FlowchartDiagram {
    type Database = FlowchartDatabase;
    type Parser = FlowchartParser;
    type Renderer = FlowchartRenderer;

    fn detector() -> Arc<dyn Detector> {
        Arc::new(FlowchartDetector::new())
    }

    fn create_parser() -> Self::Parser {
        FlowchartParser::new()
    }

    // ... other methods
}
```

### Component Relationships

```
FlowchartDiagram
├── FlowchartDetector (pattern matching)
├── FlowchartParser (chumsky implementation)
├── FlowchartDatabase (HashMap storage)
├── FlowchartLayoutAlgorithm (grid layout)
└── FlowchartRenderer (ASCII output)
```

## Data Flow

### 1. Input Processing
```rust
let input = "graph TD\n    A --> B\n    B --> C";
```

### 2. Detection
```rust
let detector = FlowchartDiagram::detector();
if detector.detect(input) {
    // It's a flowchart
}
```

### 3. Parsing
```rust
let parser = FlowchartDiagram::create_parser();
let mut database = FlowchartDiagram::create_database();
parser.parse(input, &mut database)?;
```

### 4. Layout
```rust
let layout = FlowchartLayoutAlgorithm::new();
let positioned = layout.layout(&database)?;
```

### 5. Rendering
```rust
let renderer = FlowchartDiagram::create_renderer();
let output = renderer.render(&database)?;
println!("{}", output);
```

## Coordinate System

Figurehead uses an ASCII coordinate system:
- Origin (0,0) at top-left
- X increases to the right
- Y increases downward
- All coordinates are integers
- Text width calculated with unicode-width

## Error Handling

### Error Types
```rust
#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("Parse error: {message} at line {line}, column {column}")]
    ParseError { message: String, line: usize, column: usize },

    #[error("Layout error: {message}")]
    LayoutError { message: String },

    #[error("Render error: {message}")]
    RenderError { message: String },

    // ... more variants
}
```

### Error Propagation
- Use `Result<T, E>` throughout the pipeline
- Provide context and location information
- Handle recoverable errors gracefully
- Use `anyhow` for application-level error chaining

## Performance Considerations

### 1. Memory Management
- Use `HashMap` for O(1) node lookup
- Avoid unnecessary string allocations
- Leverage Rust's ownership system

### 2. Parsing Performance
- Chumsky provides linear-time parsing
- Minimal memory overhead
- Good error recovery

### 3. Layout Optimization
- Incremental layout algorithms
- Spatial partitioning for large diagrams
- Edge routing optimization

## WASM Compatibility

### Core Library (WASM-Compatible)
- Parser logic (chumsky)
- Layout algorithms
- Database operations
- Basic rendering

### CLI Wrapper (Native Only)
- clap for argument parsing
- crossterm for terminal handling
- File I/O operations

### WASM Integration Strategy
```rust
#[wasm_bindgen]
pub struct DiagramRenderer {
    parser: FlowchartParser,
    database: FlowchartDatabase,
}

#[wasm_bindgen]
impl DiagramRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DiagramRenderer { /* ... */ }

    #[wasm_bindgen]
    pub fn render_to_ascii(&mut self, input: &str) -> String { /* ... */ }
}
```

## Future Extensions

### 1. Additional Diagram Types
- Sequence diagrams
- Class diagrams
- Gantt charts
- State machines

### 2. Advanced Rendering
- SVG output
- Canvas/WebGL rendering
- Terminal colors with crossterm
- Interactive web interface

### 3. Performance Features
- Parallel layout
- Incremental updates
- Large diagram optimization
- Memory pooling

### 4. Plugin Ecosystem
- Third-party plugins
- Layout algorithm plugins
- Custom renderers
- Domain-specific features

## Testing Strategy

### 1. Unit Tests
- Individual trait implementations
- Error conditions
- Edge cases

### 2. Integration Tests
- End-to-end pipeline
- Real Mermaid.js examples
- Performance benchmarks

### 3. Property-Based Testing
- Parser correctness
- Layout invariants
- Rendering consistency

This architecture provides a solid foundation for a robust, extensible diagram processing system while maintaining Rust's performance and safety guarantees.