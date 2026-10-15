#!/bin/bash
set -euo pipefail

echo "🔨 Building agcu for all platforms..."
echo ""

BIN_DIR="$(pwd)/bin"
mkdir -p "$BIN_DIR"

# macOS (native only — must run on macOS)
if [ "$(uname)" = "Darwin" ]; then
  echo "→ macOS ARM64..."
  cargo build --manifest-path cli/Cargo.toml --locked --release --target aarch64-apple-darwin
  cp cli/target/aarch64-apple-darwin/release/agcu "$BIN_DIR/agcu-macos-arm64"
  echo "✅ agcu-macos-arm64"

  echo "→ macOS x64..."
  cargo build --manifest-path cli/Cargo.toml --locked --release --target x86_64-apple-darwin
  cp cli/target/x86_64-apple-darwin/release/agcu "$BIN_DIR/agcu-macos-x64"
  echo "✅ agcu-macos-x64"
else
  echo "⚠  Skipping macOS builds (not on macOS)"
fi

# Linux + Windows via Docker
if command -v docker &>/dev/null; then
  echo "→ Linux + Windows (via Docker)..."
  docker compose -f docker/docker-compose.yml build
  docker compose -f docker/docker-compose.yml run --rm build-linux
  docker compose -f docker/docker-compose.yml run --rm build-windows
  echo "✅ Linux + Windows builds"
else
  echo "⚠  Skipping Linux/Windows builds (Docker not found)"
fi

echo ""
echo "📦 Built binaries:"
ls -lh "$BIN_DIR"/agcu-* 2>/dev/null || echo "   (none)"
