# Claude Context

## Architecture

Leptos 0.7 SSR app with Axum backend, deployed to Fly.io.

### Workspace

```
crates/
  app/       Leptos SSR full-stack app (Axum server + WASM hydration)
  scraper/   Scraper library for footballmundial.com
  secrets/   Bitwarden Secrets Manager integration
  generator/ Legacy generator (to be removed)
```

### Data Flow

```
scraper (triggered daily at 1am or manually from admin UI)
  → fetches footballmundial.com pages
  → upserts venues, leagues, divisions, teams, fixtures into Postgres
  → regenerates iCal files to disk (/data/ical/)

Axum server
  → SSR renders pages from Postgres via server functions
  → serves iCal files from /data/ical/ at /ical/{team_id}.ics
  → admin portal at /admin (password-protected)
```

### Secrets (BWS)

Secrets injected from Bitwarden at startup via `footical-secrets` crate.
Token source: `BWS_ACCESS_TOKEN` env var (Fly.io) or `/run/secrets/bws_access_token` (Docker).

Required secrets: `DATABASE_URL`, `ADMIN_PASSWORD`, `COOKIE_SECRET`.

## Deployment

GitHub Actions deploys to Fly.io on push to main.
Required GitHub secret: `FLY_API_TOKEN`.
Required Fly secrets: `BWS_ACCESS_TOKEN`, `TAILSCALE_AUTHKEY`.

Tailscale runs in the Fly.io container for Postgres access to Zeus
(`postgres.darter-bebop.ts.net`).

## Leptos Quirks

- `leptos_router` has no `csr` or `hydrate` features. Only `leptos` itself
  has `ssr`/`hydrate`. `leptos_router` and `leptos_meta` only have `ssr`.
- The `A` component does not accept a `class` prop. Use `use_location()` from
  `leptos_router::hooks` to detect the active route and apply classes to inner
  elements instead.
- `leptos_axum 0.7.x` depends on `axum 0.7.x`. Do not use axum 0.8.
- `cargo-leptos` is the build tool. `Leptos.toml` at workspace root configures
  the build. `Trunk.toml` is no longer used.

## Build Commands

- `cargo leptos serve` for dev (requires cargo-leptos installed)
- `cargo leptos build --release` for production
- `cargo check -p footical-app --features ssr` to check server side
- `cargo check -p footical-app --features hydrate --target wasm32-unknown-unknown`
  to check client side
