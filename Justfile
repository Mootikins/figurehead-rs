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
