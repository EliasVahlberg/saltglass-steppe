# tui-rpg
A simple TUI RPG.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

## Setup

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build
```

## Run

```bash
cargo run
```

## Cross-compile for Windows (from Linux)

```bash
# Install Windows target and cross-compiler
rustup target add x86_64-pc-windows-gnu
sudo apt-get install mingw-w64

# Build Windows executable
cargo build --release --target x86_64-pc-windows-gnu

# Package for distribution
mkdir dist
cp target/x86_64-pc-windows-gnu/release/tui-rpg.exe dist/
cp -r data dist/
zip -r tui-rpg-windows.zip dist
```

The tester should extract the zip and run `tui-rpg.exe` from Command Prompt (not by double-clicking).
