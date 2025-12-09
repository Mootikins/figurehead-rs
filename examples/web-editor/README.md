# Figurehead Web Example

This example demonstrates how to use Figurehead in a web browser using WebAssembly.

## Building

1. Install `wasm-pack` if you haven't already:
   ```bash
   cargo install wasm-pack
   ```

2. Build the WASM module:
   ```bash
   just wasm-build
   ```

   Or manually:
   ```bash
   wasm-pack build --target web --out-dir examples/web-editor/pkg
   ```

## Running

Since WASM modules require CORS headers, you need to serve the files via HTTP (not `file://`).

### Option 1: Rust HTTP Server (Recommended)
```bash
just web-server
# Or: cargo run --manifest-path examples/web-editor/Cargo.toml
# Then visit http://localhost:8000/ (shows editor.html by default)
# Or visit http://localhost:8000/index.html for the full editor
```

### Build and Run Together
```bash
just web
# Builds WASM and starts server in one command
```

### Option 2: Node.js HTTP Server
```bash
cd examples/web-editor
npx http-server -p 8000
# Then visit http://localhost:8000/
```

### Option 3: Any HTTP Server
You can use any HTTP server that supports CORS. The server must:
- Serve files with proper CORS headers
- Support `application/wasm` MIME type for `.wasm` files

## Examples

### Live Editor (`editor.html`) - Default
Minimal live editor with real-time updates as you type. Split-pane interface with input on the left and output on the right. Dark theme optimized for coding.

### Interactive Editor (`index.html`)
Full-featured editor with examples, controls, and parse-only mode.

## Features

- **Render Diagrams**: Convert Mermaid.js syntax to ASCII art
- **Multiple Styles**: Choose from Unicode, ASCII, Unicode Math, or Compact styles
- **Live Updates**: Real-time rendering as you type (editor.html)
- **Parse Only**: Parse diagrams and see node/edge counts without rendering (index.html)
- **Example Diagrams**: Quick-load example diagrams to try (index.html)

## API

The WASM module exposes these functions:

- `render_flowchart(input: string) -> string`: Render with default Unicode style
- `render_flowchart_with_style(input: string, style: string) -> string`: Render with specific style
- `parse_flowchart(input: string) -> string`: Parse and return JSON with metadata

## Browser Compatibility

Works in modern browsers that support:
- WebAssembly
- ES6 Modules
- Fetch API (for loading WASM)

Tested in:
- Chrome/Edge (Chromium)
- Firefox
- Safari
