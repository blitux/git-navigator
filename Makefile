# Git Navigator Makefile
# Convenient commands for development and testing

.PHONY: help build test run clean install install-local format lint check all

# Default target
help:
	@echo "Git Navigator - Available commands:"
	@echo ""
	@echo "  make build       - Build release binary"
	@echo "  make build-dev   - Build debug binary"
	@echo "  make test        - Run all tests"
	@echo "  make test-unit   - Run unit tests only"
	@echo "  make test-int    - Run integration tests only"
	@echo "  make run         - Run git-navigator status"
	@echo "  make install     - Install binary and aliases (via install.sh)"
	@echo "  make install-local - Install local binary to ~/.local/bin"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make format      - Format code with rustfmt"
	@echo "  make lint        - Run clippy linter"
	@echo "  make check       - Check code without building"
	@echo "  make all         - Format, lint, test, and build"
	@echo ""

# Build commands
build:
	@echo "🔨 Building release binary..."
	cargo build --release

build-dev:
	@echo "🔨 Building debug binary..."
	cargo build

# Test commands
test:
	@echo "🧪 Running all tests..."
	cargo test

test-unit:
	@echo "🧪 Running unit tests..."
	cargo test --lib

test-int:
	@echo "🧪 Running integration tests..."
	cargo test --test status_command_tests

# Run commands
run: build
	@echo "🚀 Running git-navigator status..."
	./target/release/git-navigator status

# Development commands
format:
	@echo "🎨 Formatting code..."
	cargo fmt

lint:
	@echo "🔍 Running clippy..."
	cargo clippy -- -D warnings

check:
	@echo "✅ Checking code..."
	cargo check

# Installation
install: build
	@echo "📦 Installing git-navigator..."
	./install.sh

install-local: build
	@echo "📦 Installing local binary to ~/.local/bin..."
	@mkdir -p ~/.local/bin
	@cp target/release/git-navigator ~/.local/bin/
	@chmod +x ~/.local/bin/git-navigator
	@echo "✓ Binary installed to ~/.local/bin/git-navigator"
	@if [[ ":$$PATH:" != *":$$HOME/.local/bin:"* ]]; then \
		echo "⚠️  ~/.local/bin is not in PATH. Add it to your shell config:"; \
		echo "  export PATH=\"$$HOME/.local/bin:\$$PATH\""; \
	else \
		echo "✓ ~/.local/bin is already in PATH"; \
	fi

# Cleanup
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Combined commands
all: format lint test build
	@echo "✨ All checks passed! Ready to ship 🚀"

# Development workflow
dev: format check test-unit
	@echo "✅ Development checks complete"

# CI-like workflow
ci: format lint test build
	@echo "🎯 CI pipeline complete"