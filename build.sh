#!/bin/bash

set -e

echo "Building PM2 Rust..."
echo ""

echo "==> Cleaning previous build..."
cargo clean > /dev/null 2>&1

echo "==> Building release version..."
cargo build --release > /dev/null 2>&1

echo ""
echo "==> Build completed!"

echo "==> Copying binary to project root..."
rm ./pm2
cp ./target/release/pm2 ./pm2
chmod +x ./pm2

echo ""
echo "==> Showing binary info..."
ls -lh ./pm2

echo ""
echo "==> Done!"
