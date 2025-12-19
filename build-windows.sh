#!/bin/bash
set -e

cargo build --release --target x86_64-pc-windows-gnu

rm -rf dist tui-rpg-windows.zip
mkdir dist
cp target/x86_64-pc-windows-gnu/release/tui-rpg.exe dist/
cp -r data dist/
cd dist && zip -r ../tui-rpg-windows.zip . && cd ..
rm -rf dist

echo "Created tui-rpg-windows.zip"
