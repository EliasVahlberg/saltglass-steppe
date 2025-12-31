#!/bin/bash
set -e

cargo build --release --target x86_64-pc-windows-gnu

rm -rf dist saltglass-steppe-windows.zip
mkdir dist
cp target/x86_64-pc-windows-gnu/release/saltglass-steppe.exe dist/
cp -r data dist/
cd dist && zip -r ../saltglass-steppe-windows.zip . && cd ..
rm -rf dist

echo "Created saltglass-steppe-windows.zip"
