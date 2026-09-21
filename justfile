default: check

fmt:
    cargo fmt
    leptosfmt --quiet crates

test:
    cargo test --workspace

watch:
    footical-watch

check:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo fmt --check
    leptosfmt --check crates
    cargo clippy --workspace --all-targets -- --deny warnings
    cargo test --workspace
    cargo clippy -p footical-website --no-default-features --features ssr \
        --all-targets -- --deny warnings
    cargo clippy -p footical-website --lib --no-default-features --features hydrate \
        --target wasm32-unknown-unknown --target-dir target/check \
        -- --deny warnings
