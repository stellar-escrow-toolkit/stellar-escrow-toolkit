.PHONY: all build test clean format contracts-build contracts-test

all: build test

build:
	@echo "Building contracts..."
	@cd contracts && cargo build --release
	@echo "Building packages..."
	@npm run build

test: contracts-test
	@echo "Running package tests..."
	@npm run test

contracts-build:
	@cd contracts && cargo build --target wasm32-unknown-unknown --release

contracts-test:
	@cd contracts && cargo test

format:
	@cd contracts && cargo fmt --all
	@npx prettier --write "**/*.{ts,tsx,json,md}"

clean:
	@cd contracts && cargo clean
	@rm -rf packages/*/dist packages/*/node_modules node_modules
