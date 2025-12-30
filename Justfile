# Basic project automation for Figurehead

default: test

# Run full test suite with nextest
test:
	cargo nextest run

# Run tests with standard cargo (fallback)
test-cargo:
	cargo test

# Run tests showing only summary
test-quiet:
	@cargo nextest run 2>&1 | grep -E "Summary|FAIL|error\[" | grep -v "^$" || echo "All tests passed"

# Run CI checks locally (fmt, clippy, test)
ci: fmt-check clippy test
	@echo "CI checks passed!"

# Check formatting
fmt-check:
	cargo fmt --all -- --check

# Fix formatting
fmt:
	cargo fmt --all

# Run clippy lints
clippy:
	cargo clippy --all-targets

# Update snapshot fixtures and run snapshot tests
snapshots-update:
	UPDATE_FIXTURES=1 cargo test --test snapshots

# Fast snapshot check without updates
snapshots:
	cargo test --test snapshots

# Release dry-run (patch/minor/major)
release-dry level="patch":
	cargo release {{level}}

# Release for real (patch/minor/major)
release level="patch":
	cargo release {{level}} --execute

# Build WASM module for web examples
wasm-build:
	@echo "Building WASM module..."
	wasm-pack build crates/figurehead --target web --out-dir ../../examples/web-editor/pkg

# Run web server for examples
web-server:
	cargo run --manifest-path examples/web-editor/Cargo.toml

# Build WASM and run web server
web: wasm-build web-server

# Generate current Unicode outputs for all ideal samples
ideal-current:
	mkdir -p docs/ideal-output/current
	for f in docs/ideal-output/*.mmd; do \
	  base=$(basename "$f" .mmd); \
	  FIGUREHEAD_LOG_LEVEL=off cargo run -q -p figurehead-cli -- convert -i "$f" --style unicode > "docs/ideal-output/current/${base}.unicode.current.txt"; \
	done

# Diff ideal targets vs current outputs (requires ideal-current first)
ideal-diff: ideal-current
	for f in docs/ideal-output/*.mmd; do \
	  base=$(basename "$f" .mmd); \
	  diff -u "docs/ideal-output/${base}.unicode.ideal.txt" "docs/ideal-output/current/${base}.unicode.current.txt" || true; \
	done
