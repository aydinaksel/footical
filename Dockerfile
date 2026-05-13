FROM rust:1.82-bookworm AS builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

RUN cargo leptos build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://tailscale.com/install.sh | sh

COPY --from=builder /app/target/release/footical-app /usr/local/bin/
COPY --from=builder /app/target/site /app/site

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENV LEPTOS_SITE_ROOT=/app/site
ENV LEPTOS_OUTPUT_NAME=footical-app
ENV ICAL_OUTPUT_DIR=/data/ical

EXPOSE 3000

CMD ["/entrypoint.sh"]
