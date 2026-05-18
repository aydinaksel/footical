FROM rust:1.95-bookworm AS builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos --locked

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

RUN cargo leptos build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates iptables && rm -rf /var/lib/apt/lists/*

COPY --from=docker.io/tailscale/tailscale:stable /usr/local/bin/tailscaled /usr/local/bin/tailscaled
COPY --from=docker.io/tailscale/tailscale:stable /usr/local/bin/tailscale /usr/local/bin/tailscale
RUN mkdir -p /var/run/tailscale /var/cache/tailscale /var/lib/tailscale

COPY --from=builder /app/target/release/footical-website /usr/local/bin/
COPY --from=builder /app/target/site /app/site

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENV LEPTOS_SITE_ROOT=/app/site
ENV LEPTOS_OUTPUT_NAME=footical-website
EXPOSE 3000

CMD ["/entrypoint.sh"]
