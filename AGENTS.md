# AI Development Guidelines

This document provides guidelines for AI agents working on the Figurehead project.

## Architecture Overview

Figurehead follows a modular, plugin-based architecture inspired by mermaid.js:

```
Input → Detector → Parser → Database → Layout → Renderer → Output
```

### Core Components

1. **Core Traits** (`src/core/`): Define interfaces for all components
   - `Diagram`: Main entry point for each diagram type
   - `Detector`: Identifies diagram types from markup
   - `Parser`: Parses markup using chumsky
   - `Database`: Stores nodes, edges, and diagram data
   - `LayoutAlgorithm`: Arranges elements in coordinate space
   - `Renderer`: Generates output (ASCII, SVG, etc.)

2. **Plugins** (`src/plugins/`): Implementations for specific diagram types
   - Each plugin implements all core traits
   - Currently: flowchart (basic implementation)
   - Future: sequence, class, gantt charts

3. **Pipeline**: Data flows through stages using the traits

## Development Principles

### SOLID Principles
- **Single Responsibility**: Each trait/component has one purpose
- **Open/Closed**: Extensible through plugins, closed for modification
- **Liskov Substitution**: All implementations can replace their interfaces
- **Interface Segregation**: Small, focused traits
- **Dependency Inversion**: Depend on abstractions, not concretions

### Rust Idioms
- Use `match`, iterators, and `&[Type]` instead of `Vec` where possible
- Prefer `Result<T, E>` over exceptions
- Use trait objects (`dyn Trait`) for runtime polymorphism
- Leverage zero-cost abstractions and compile-time guarantees

### Test-Driven Development (TDD)
1. Write failing tests first
2. Implement minimal code to make tests pass
3. Refactor while maintaining test coverage
4. Use property-based testing for complex logic

## Implementation Guidelines

### When Adding New Diagram Types

1. **Create Plugin Structure**:
   ```rust
   // src/plugins/newtype/mod.rs
   pub struct NewTypeDiagram;

   impl Diagram for NewTypeDiagram {
       type Database = NewTypeDatabase;
       type Parser = NewTypeParser;
       type Renderer = NewTypeRenderer;
       // ... implement all methods
   }
   ```

2. **Implement Core Traits**: All 5 traits must be implemented
3. **Add Detector**: Pattern matching for markup identification
4. **Parser**: Use chumsky for parsing (human-readable, good error handling)
5. **Layout**: Coordinate system arrangement (inspired by Dagre)
6. **Renderer**: ASCII output (extendable to other formats)

### Parser Implementation
- Use chumsky for all parsing (not nom, pest)
- Handle errors gracefully with clear messages
- Support incremental parsing where possible
- Provide line/column information for errors

### Layout Algorithm
- Use ASCII coordinate system (not pixels)
- Implement basic grid layout, then optimize
- Consider edge routing and node spacing
- Handle text width with unicode-width crate

### Rendering
- Start with basic ASCII output
- Consider terminal colors with crossterm
- Plan for future SVG/WebGL support
- Handle different terminal sizes

## Code Quality

### Error Handling
- Use `thiserror` for custom error types
- Provide context and location information
- Handle recoverable errors gracefully
- Use `anyhow` for application-level error propagation

### Performance
- Avoid unnecessary allocations
- Use iterators and lazy evaluation
- Profile before optimizing
- Consider WASM target constraints

### Documentation
- Document all public APIs with examples
- Include architectural decision records
- Maintain this file for development guidelines
- Add inline comments for complex logic

## Testing Strategy

### Unit Tests
- Test each trait implementation independently
- Use property-based testing for complex functions
- Test error conditions and edge cases
- Mock dependencies where appropriate

### Integration Tests
- Test the full pipeline: Input → Output
- Use real Mermaid.js examples
- Test performance with large diagrams
- Verify WASM compilation when applicable

### Test Organization
- `tests/core_traits.rs`: Core functionality tests
- `tests/flowchart/`: Flowchart-specific tests
- `tests/integration/`: End-to-end tests
- Use descriptive test names

## WASM Considerations

The core library should be WASM-compatible:
- Separate CLI from core logic
- Avoid terminal-specific dependencies in core
- Use `wasm-bindgen` for browser APIs
- Test with `wasm-pack build --target web`

## Tool Usage Guidelines

### When to Use Agents
- **Code Generation**: For repetitive patterns and boilerplate
- **Refactoring**: For systematic improvements
- **Testing**: For comprehensive test coverage
- **Documentation**: For consistent doc generation

### Development Workflow
1. Explore current state first
2. Plan changes before implementing
3. Use TDD for new features
4. Run tests frequently
5. Update documentation

### Best Practices
- Read existing code before modifying
- Maintain consistent style and patterns
- Ask for clarification when requirements are ambiguous
- Provide context for architectural decisions

## Dependencies

### Core Dependencies
- **chumsky**: Parsing (WASM-compatible)
- **anyhow**: Error handling (WASM-compatible)
- **thiserror**: Custom error types (WASM-compatible)
- **unicode-width**: Text width (WASM-compatible)

### CLI Dependencies
- **clap**: Command-line interface (NOT WASM-compatible)
- **crossterm**: Terminal handling (NOT WASM-compatible)

### When Adding Dependencies
- Check WASM compatibility
- Prefer minimal, focused crates
- Consider maintenance status
- Update documentation

## Release Process

1. Ensure all tests pass
2. Update documentation
3. Check WASM compilation
4. Update version in Cargo.toml
5. Create git tag
6. Update CHANGELOG.md

## Future Directions

- Browser-based mermaid.js alternative
- SVG and canvas rendering
- Real-time collaborative editing
- Plugin ecosystem
- Performance optimizations
- Additional diagram types