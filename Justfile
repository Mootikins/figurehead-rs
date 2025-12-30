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

# Prepare a release: ensure clean tree, run tests, and dry-run publish
release-check:
	git diff --quiet --exit-code
	cargo test
	cargo publish -p figurehead --locked --dry-run
	python -c "from pathlib import Path; import re, subprocess; text=Path('Cargo.toml').read_text(); m=re.search(r'version\\s*=\\s*\"([^\"]+)\"', text); version=m.group(1) if m else 'unknown'; res=subprocess.run(['git','tag',f'v{version}'], capture_output=True, text=True); print('Tag v{version} already exists.' if res.returncode==0 and res.stdout.strip()==f'v{version}' else f'Ready to tag: v{version}')"

# Publish the library crate (runs release-check first)
publish-lib: release-check
	cargo publish -p figurehead --locked

# Publish the CLI after the library is on crates.io
publish-cli:
	cargo publish -p figurehead-cli --locked

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
