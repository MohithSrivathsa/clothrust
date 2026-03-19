#!/usr/bin/env bash
set -e

echo "  ThreadCraft Store"
echo ""

if ! command -v cargo &> /dev/null; then
  echo "❌ Rust not installed."
  exit 1
fi

# Load environment variables from .env
if [ -f .env ]; then
  set -a
  source .env
  set +a
  echo "✅  Environment loaded from .env"
fi

echo "✅  Rust found: $(rustc --version)"
echo "🔨  Building..."
RUST_LOG=info cargo run
