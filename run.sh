#!/usr/bin/env bash
set -e

echo ""
echo "  ████████╗██╗  ██╗██████╗ ███████╗ █████╗ ██████╗  ██████╗██████╗  █████╗ ███████╗████████╗"
echo "     ██╔══╝██║  ██║██╔══██╗██╔════╝██╔══██╗██╔══██╗██╔════╝██╔══██╗██╔══██╗██╔════╝╚══██╔══╝"
echo "     ██║   ███████║██████╔╝█████╗  ███████║██║  ██║██║     ██████╔╝███████║█████╗     ██║   "
echo "     ██║   ██╔══██║██╔══██╗██╔══╝  ██╔══██║██║  ██║██║     ██╔══██╗██╔══██║██╔══╝     ██║   "
echo "     ██║   ██║  ██║██║  ██║███████╗██║  ██║██████╔╝╚██████╗██║  ██║██║  ██║██║        ██║   "
echo "     ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝  ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝        ╚═╝"
echo ""
echo "  Premium Custom Printed Apparel Store"
echo ""

# Check Rust
if ! command -v cargo &> /dev/null; then
  echo "❌  Rust is not installed."
  echo "    Install it from: https://rustup.rs"
  echo "    Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 1
fi

echo "✅  Rust found: $(rustc --version)"
echo ""
echo "🔨  Building ThreadCraft (this may take 2-3 minutes on first run)..."
echo ""

RUST_LOG=info cargo run

