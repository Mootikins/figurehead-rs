# WebAssembly Support

Figurehead fully supports WebAssembly (WASM) compilation for use in web browsers.

## Quick Start

### Building WASM Module

1. Install `wasm-pack`:
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

### Running the Web Example

The web example requires an HTTP server (not `file://`) due to CORS requirements:

```bash
just web-server
# Or: cargo run --manifest-path examples/web-editor/Cargo.toml
# Then visit http://localhost:8000/
```

Or build and run together:
```bash
just web
```

Or use any HTTP server:
```bash
cd examples/web-editor
npx http-server -p 8000
# Then visit http://localhost:8000/
```

## WASM API

The WASM module exposes these functions:

### `render_flowchart(input: string) -> string`

Renders a Mermaid flowchart to ASCII art using the default Unicode style.

```javascript
import { render_flowchart } from './pkg/figurehead.js';

const ascii = render_flowchart("graph LR; A-->B-->C");
console.log(ascii);
```

### `render_flowchart_with_style(input: string, style: string) -> string`

Renders with a specific character set style:
- `"ascii"` - Pure ASCII characters
- `"unicode"` - Unicode box-drawing (default)
- `"unicode-math"` - Unicode with mathematical symbols
- `"compact"` - Single-glyph compact mode

```javascript
import { render_flowchart_with_style } from './pkg/figurehead.js';

const ascii = render_flowchart_with_style("graph TD; A-->B", "ascii");
```

### `parse_flowchart(input: string) -> string`

Parses a diagram and returns JSON with metadata:

```javascript
import { parse_flowchart } from './pkg/figurehead.js';

const result = parse_flowchart("graph TD; A-->B-->C");
const data = JSON.parse(result);
console.log(`Nodes: ${data.node_count}, Edges: ${data.edge_count}`);
```

## Logging in WASM

Logging in WASM builds uses `tracing-wasm`, which outputs to the browser console.
All tracing spans and events from the native build are available in WASM.

To see logs, open your browser's developer console (F12).

## Building for Different Targets

- **Web**: `wasm-pack build --target web` (for browsers with ES modules)
- **Node.js**: `wasm-pack build --target nodejs` (for Node.js environments)
- **Bundler**: `wasm-pack build --target bundler` (for webpack, etc.)

## Integration Testing

Run WASM integration tests:

```bash
cargo test --test wasm_integration -- --ignored
```

Note: These tests require `wasm-pack` to be installed and are ignored by default.

## Browser Compatibility

Works in modern browsers that support:
- WebAssembly
- ES6 Modules
- Fetch API

Tested in:
- Chrome/Edge (Chromium) 90+
- Firefox 88+
- Safari 14+

## Troubleshooting

### "Failed to load WASM module"

- Make sure you're serving via HTTP (not `file://`)
- Check browser console for CORS errors
- Verify `pkg/` directory exists with WASM files

### "Module not found"

- Run `./build.sh` to generate the `pkg/` directory
- Check that `index.html` references the correct path to `pkg/figurehead.js`

### Build Errors

- Ensure `wasm-pack` is up to date: `cargo install --force wasm-pack`
- Check Rust toolchain: `rustup target add wasm32-unknown-unknown`
