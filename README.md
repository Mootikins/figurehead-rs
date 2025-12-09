# Figurehead

A Rust utility to convert Mermaid.js diagram markup into ASCII diagrams, inspired by [mermaid-ascii](https://github.com/AlexanderGrooff/mermaid-ascii).

## Features

- 🎨 Convert Mermaid.js flowcharts to ASCII art
- 🔧 Modular, plugin-based architecture inspired by mermaid.js
- 🚀 Fast parsing using [chumsky](https://github.com/zesterer/chumsky)
- 🧪 Test-Driven Development approach
- 🌐 WASM-compatible core library (future browser support)
- 🎯 SOLID principles and Rust idioms

## Architecture

Figurehead follows a modular pipeline architecture:

```
Input → Detector → Parser → Database → Layout → Renderer → Output
```

### Core Components

- **Detector**: Identifies diagram types from markup patterns
- **Parser**: Parses markup into structured data using chumsky
- **Database**: Stores diagram nodes and edges
- **Layout**: Arranges elements in coordinate space (Dagre-inspired)
- **Renderer**: Generates ASCII output

### Diagram Types

Currently supported:
- Flowchart (basic implementation)

Planned:
- Sequence diagrams
- Class diagrams
- Subgraphs
- Styling with terminal colors

## Quick Start

### CLI Usage

```bash
# Basic conversion
echo "graph TD\n    A --> B" | figurehead

# Choose output character set (ascii|unicode|unicode-math|compact)
figurehead convert --style ascii -i input.mmd

# Output to file
figurehead -i input.mmd -o output.txt

# Respect FIGUREHEAD_STYLE environment variable
FIGUREHEAD_STYLE=compact figurehead convert -i input.mmd
```

### Library Usage

```rust
use figurehead::{plugins::flowchart::FlowchartDiagram, core::Diagram};

// Create a flowchart diagram
let diagram = FlowchartDiagram;

// Create parser and database
let mut parser = diagram.create_parser();
let mut database = diagram.create_database();

// Parse some markup
parser.parse("graph TD\n    A --> B", &mut database)?;

// Render to ASCII
let renderer = diagram.create_renderer();
let output = renderer.render(&database)?;
println!("{}", output);
```

## Development

### Building

```bash
cargo build
cargo test
```

### Running Tests

```bash
# All tests
cargo test

# Core trait tests
cargo test --test core_traits

# Run with colors
cargo test -- --nocapture
```

### Project Structure

```
src/
├── core/           # Core trait abstractions
├── plugins/        # Diagram type implementations
│   └── flowchart/  # Flowchart plugin
├── layout/         # Layout algorithms
├── rendering/      # Rendering implementations
└── cli/           # CLI interface
tests/
└── core_traits.rs  # Core functionality tests
```

## WASM Support

The core library is designed for WASM compilation:

```bash
# Build for WASM (requires wasm-pack)
cargo build --target wasm32-unknown-unknown
wasm-pack build --target web
```

Future versions will include browser APIs and web-based diagram rendering.

## Dependencies

- [chumsky](https://github.com/zesterer/chumsky) - Parsing library
- [clap](https://github.com/clap-rs/clap) - CLI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal handling
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling
- [thiserror](https://github.com/dtolnay/thiserror) - Error types
- [unicode-width](https://github.com/unicode-rs/unicode-width) - Text width calculation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

See [AGENTS.md](AGENTS.md) for development guidelines.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [mermaid-ascii](https://github.com/AlexanderGrooff/mermaid-ascii)
- Architecture inspired by [mermaid.js](https://github.com/mermaid-js/mermaid)
- Built with the Rust ecosystem
