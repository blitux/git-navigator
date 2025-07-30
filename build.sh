#!/bin/bash
set -e

# Cross-compilation build script for git-navigator
# Builds binaries for multiple platforms

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu" 
    "x86_64-unknown-linux-musl"
    "x86_64-pc-windows-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
)

OUTPUT_DIR="dist"
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

echo "Building git-navigator for multiple platforms..."

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    
    # Add target if not already installed
    rustup target add "$target" 2>/dev/null || true
    
    # Build for target
    if cargo build --release --target "$target"; then
        # Copy binary with platform-specific naming
        if [[ "$target" == *"windows"* ]]; then
            binary_name="git-navigator-${target}.exe"
            cp "target/$target/release/git-navigator.exe" "$OUTPUT_DIR/$binary_name"
        else
            binary_name="git-navigator-${target}"
            cp "target/$target/release/git-navigator" "$OUTPUT_DIR/$binary_name"
        fi
        echo "✓ Built: $binary_name"
    else
        echo "✗ Failed to build for $target"
    fi
done

echo ""
echo "Build complete. Binaries in $OUTPUT_DIR/:"
ls -la "$OUTPUT_DIR/"