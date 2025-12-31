#!/bin/bash
set -e

VERSION=${1:-"dev"}
RELEASE_DIR="releases/$VERSION"

echo "Building release $VERSION for all platforms..."

# Clean previous builds
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"/{linux,windows,macos}

# Add targets
rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-gnu x86_64-apple-darwin

# Build Linux
echo "Building for Linux..."
cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/saltglass-steppe "$RELEASE_DIR/linux/"
cp target/x86_64-unknown-linux-gnu/release/mapgen-tool "$RELEASE_DIR/linux/"
cp -r data "$RELEASE_DIR/linux/"

# Build Windows
echo "Building for Windows..."
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/saltglass-steppe.exe "$RELEASE_DIR/windows/"
cp target/x86_64-pc-windows-gnu/release/mapgen-tool.exe "$RELEASE_DIR/windows/"
cp -r data "$RELEASE_DIR/windows/"

# Build macOS
echo "Building for macOS..."
SDKROOT=/opt/MacOSX-SDKs/MacOSX12.3.sdk PATH="/opt/zig:$PATH" cargo zigbuild --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/saltglass-steppe "$RELEASE_DIR/macos/"
cp target/x86_64-apple-darwin/release/mapgen-tool "$RELEASE_DIR/macos/"
cp -r data "$RELEASE_DIR/macos/"

# Create zip files
echo "Creating release packages..."
cd "$RELEASE_DIR"
zip -r "saltglass-steppe-$VERSION-linux-x86_64.zip" linux/
zip -r "saltglass-steppe-$VERSION-windows-x86_64.zip" windows/
zip -r "saltglass-steppe-$VERSION-macos-x86_64.zip" macos/

echo "Release $VERSION built successfully!"
echo "Assets created in $RELEASE_DIR/"
ls -la *.zip
