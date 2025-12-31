# Release Build Script

## Usage

```bash
# Build with version number
./build-release.sh v0.2.5

# Build with default "dev" version
./build-release.sh
```

## Requirements

- Rust with cross-compilation targets
- Zig toolchain for macOS builds
- macOS SDK at `/opt/MacOSX-SDKs/MacOSX12.3.sdk`
- cargo-zigbuild installed

## Output

Creates `releases/{version}/` directory with:
- Platform-specific folders (linux/, windows/, macos/)
- Zip files for each platform
- Complete binaries and data files

## Setup Commands

```bash
# Install targets
rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-gnu x86_64-apple-darwin

# Install cross-compilation tools
sudo apt-get install mingw-w64
cargo install cargo-zigbuild

# Download macOS SDK (if needed)
curl -L https://github.com/joseluisq/macosx-sdks/releases/download/12.3/MacOSX12.3.sdk.tar.xz -o /tmp/MacOSX12.3.sdk.tar.xz
sudo mkdir -p /opt/MacOSX-SDKs && cd /opt/MacOSX-SDKs && sudo tar -xf /tmp/MacOSX12.3.sdk.tar.xz

# Install Zig
curl -L https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz -o /tmp/zig.tar.xz
cd /tmp && tar -xf zig.tar.xz && sudo mv zig-linux-x86_64-0.11.0 /opt/zig
```
