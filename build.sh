#!/usr/bin/env bash
set -euo pipefail

# Install wasm32-unknown-unknown target for WebAssembly compilation
rustup target add wasm32-unknown-unknown

# Install Trunk from binary release (faster than compiling from source)
TRUNK_VERSION="v0.21.9"
curl -sL \
  "https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" \
  | tar xz -C "$HOME/.cargo/bin"

# Install Tailwind CSS standalone CLI
TAILWIND_CSS_VERSION="v3.4.17"
curl -sLo "$HOME/.cargo/bin/tailwindcss" \
  "https://github.com/tailwindlabs/tailwindcss/releases/download/${TAILWIND_CSS_VERSION}/tailwindcss-linux-x64"
chmod +x "$HOME/.cargo/bin/tailwindcss"

# Build the release WASM bundle
trunk build --release
