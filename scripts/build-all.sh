#!/bin/bash
# Build uhash-prover for all supported platforms
# Requires: rustup, cross (for cross-compilation)

set -e

VERSION="${1:-dev}"
DIST_DIR="dist"

echo "=== Building uhash-prover v${VERSION} ==="
echo ""

# Create dist directory
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Detect host platform
HOST_TARGET=$(rustc -vV | grep "host:" | cut -d' ' -f2)
echo "Host: $HOST_TARGET"
echo ""

# Build function
build_target() {
    local target=$1
    local artifact_name=$2
    local use_cross=${3:-false}

    echo "Building for $target..."

    if [ "$use_cross" = "true" ]; then
        if ! command -v cross &> /dev/null; then
            echo "  Installing cross..."
            cargo install cross --git https://github.com/cross-rs/cross
        fi
        cross build --release --target "$target"
    else
        cargo build --release --target "$target"
    fi

    # Package
    local ext=""
    [[ "$target" == *"windows"* ]] && ext=".exe"

    local binary="target/${target}/release/uhash${ext}"
    if [ -f "$binary" ]; then
        if [[ "$target" == *"windows"* ]]; then
            # Windows: create zip
            mkdir -p "$DIST_DIR/tmp"
            cp "$binary" "$DIST_DIR/tmp/"
            (cd "$DIST_DIR/tmp" && zip -q "../${artifact_name}-${VERSION}.zip" *)
            rm -rf "$DIST_DIR/tmp"
        else
            # Unix: create tar.gz
            mkdir -p "$DIST_DIR/tmp"
            cp "$binary" "$DIST_DIR/tmp/"
            chmod +x "$DIST_DIR/tmp/uhash"
            (cd "$DIST_DIR/tmp" && tar -czvf "../${artifact_name}-${VERSION}.tar.gz" *)
            rm -rf "$DIST_DIR/tmp"
        fi
        echo "  ✓ Created $DIST_DIR/${artifact_name}-${VERSION}.*"
    else
        echo "  ✗ Build failed for $target"
        return 1
    fi
}

# Install targets if needed
install_target() {
    local target=$1
    if ! rustup target list --installed | grep -q "$target"; then
        echo "Installing target $target..."
        rustup target add "$target"
    fi
}

# Build native (current platform) - always works
echo "=== Native Build ==="
case "$HOST_TARGET" in
    *"darwin"*)
        if [[ "$HOST_TARGET" == *"aarch64"* ]]; then
            build_target "aarch64-apple-darwin" "uhash-macos-arm64"
            # Also build for Intel Mac if possible
            install_target "x86_64-apple-darwin"
            build_target "x86_64-apple-darwin" "uhash-macos-x64" || true
        else
            build_target "x86_64-apple-darwin" "uhash-macos-x64"
            # Also build for ARM Mac if possible
            install_target "aarch64-apple-darwin"
            build_target "aarch64-apple-darwin" "uhash-macos-arm64" || true
        fi
        ;;
    *"linux"*)
        if [[ "$HOST_TARGET" == *"aarch64"* ]]; then
            build_target "aarch64-unknown-linux-gnu" "uhash-linux-arm64"
        else
            build_target "x86_64-unknown-linux-gnu" "uhash-linux-x64"
        fi
        ;;
    *"windows"*)
        build_target "x86_64-pc-windows-msvc" "uhash-windows-x64"
        ;;
esac

# Cross-compilation (Linux only, requires Docker)
if [[ "$HOST_TARGET" == *"linux"* ]] && command -v docker &> /dev/null; then
    echo ""
    echo "=== Cross-Compilation ==="

    # Linux ARM64 (if on x64)
    if [[ "$HOST_TARGET" == *"x86_64"* ]]; then
        build_target "aarch64-unknown-linux-gnu" "uhash-linux-arm64" true || true
    fi

    # Windows (cross-compile from Linux)
    install_target "x86_64-pc-windows-gnu"
    build_target "x86_64-pc-windows-gnu" "uhash-windows-x64" true || true
fi

echo ""
echo "=== Build Complete ==="
echo "Artifacts in $DIST_DIR/:"
ls -la "$DIST_DIR/"
