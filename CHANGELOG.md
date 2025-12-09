# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **WebAssembly Support**: Full WASM compatibility with browser-friendly APIs
  - WASM module with `render_flowchart()` and `render_flowchart_with_style()` functions
  - `parse_flowchart()` function to inspect diagram structure without rendering
  - Two web examples:
    - **Live Editor** (`editor.html`): Minimal split-pane editor with real-time updates
    - **Interactive Editor** (`index.html`): Full-featured editor with examples and controls
  - Rust-based HTTP server for serving web examples
  - Justfile commands: `just wasm-build`, `just web-server`, `just web`
  - Conditional compilation for WASM logging (uses `tracing-wasm` for browser console)
  - Comprehensive documentation
  - Integration tests for WASM builds
- **Structured Logging**: Comprehensive logging infrastructure using the `tracing` crate
  - Configurable log levels (trace, debug, info, warn, error)
  - Multiple log formats (compact, pretty, json)
  - CLI flags `--log-level` and `--log-format` for controlling logging
  - Environment variable support (`FIGUREHEAD_LOG_LEVEL`, `FIGUREHEAD_LOG_FORMAT`)
  - Detailed tracing spans and events throughout the processing pipeline:
    - Detection: Diagram type detection with confidence scores
    - Parsing: Parsing stages, node/edge counts, error tracking
    - Layout: Layer assignment, positioning, edge routing, canvas dimensions
    - Rendering: Canvas creation, node/edge drawing, output metrics
  - Performance tracking: Automatic duration measurement for parse, layout, and render operations
  - Structured fields: Rich metadata in log events (node_id, edge_type, shape_type, etc.)
  - Component filtering: Ability to filter logs by specific components using RUST_LOG syntax

### Changed
- Enhanced error reporting with structured logging context
- Improved debugging capabilities through detailed trace information

## [0.1.0] - Initial Release

### Added
- Basic flowchart diagram support
- ASCII rendering with multiple character sets
- CLI tool for diagram conversion
- Core plugin architecture
