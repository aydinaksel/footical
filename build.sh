#!/usr/bin/env bash
set -euo pipefail

# Install wasm32-unknown-unknown target for WebAssembly compilation
rustup target add wasm32-unknown-unknown

# Install Trunk from binary release (faster than compiling from source)
TRUNK_VERSION="v0.21.9"
curl -sL \
  "https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz" \
  | tar xz -C "$HOME/.cargo/bin"

# Build the release WASM bundle (trunk downloads tailwindcss v4 via Trunk.toml)
trunk build --release
