#!/bin/bash
set -e

# Cross-compilation build script for git-navigator
# Builds binaries for multiple platforms

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu" 
    "x86_64-unknown-linux-musl"
    "x86_64-pc-windows-gnu"
)

USE_CROSS_TARGETS=(
    "aarch64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
)

OUTPUT_DIR="dist"
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

echo "Building git-navigator for multiple platforms..."

for target in "${TARGETS[@]}"; do
    echo ""
    echo "🔧 Building for $target..."

    # Add target
    rustup target add "$target" 2>/dev/null || true

    # Decide whether to use cross or cargo
    if [[ " ${USE_CROSS_TARGETS[*]} " == *" $target "* ]]; then
        build_cmd="cross build --release --target $target"
    else
        build_cmd="cargo build --release --target $target"
    fi

    echo "→ Running: $build_cmd"
    if eval $build_cmd; then
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
echo "✅ Build complete. Binaries in $OUTPUT_DIR/:"
ls -la "$OUTPUT_DIR/"
