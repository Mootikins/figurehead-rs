# Basic project automation for Figurehead

default: test

# Run full test suite
test:
	cargo test

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
	python - <<'PY'
import subprocess, tomllib, pathlib
root = pathlib.Path(__file__).resolve().parent
data = tomllib.loads((root / "Cargo.toml").read_text())
version = data["workspace"]["package"]["version"]
res = subprocess.run(["git", "tag", f"v{version}"], capture_output=True, text=True)
if res.returncode == 0 and res.stdout.strip() == f"v{version}":
    print(f"Tag v{version} already exists.")
else:
    print(f"Ready to tag: v{version}")
PY

# Publish the library crate (runs release-check first)
publish-lib: release-check
	cargo publish -p figurehead --locked

# Publish the CLI after the library is on crates.io
publish-cli:
	cargo publish -p figurehead-cli --locked
