.PHONY: build test clean doc release clippy fmt

# Default target
all: clean build test

# Build the project in debug mode
build:
	cargo build

# Build the project in release mode
release:
	cargo build --release

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Generate documentation
doc:
	cargo doc --no-deps

# Run clippy for linting
clippy:
	cargo clippy -- -D warnings

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt -- --check

# Run in development mode
dev:
	cargo run --example simple
