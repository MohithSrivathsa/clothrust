#!/usr/bin/env bash
set -a
[ -f .env ] && source .env
set +a

echo ""
echo "  ThreadCraft Store"
echo ""

if ! command -v cargo &> /dev/null; then
  echo "❌ Rust not installed. Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 1
fi

echo "✅  Rust found: $(rustc --version)"
echo "🔨  Building..."
RUST_LOG=info cargo run
