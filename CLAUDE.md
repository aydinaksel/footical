# Footical

## Architecture

Every page gets its data from a `#[server]` function in `crates/website/src/server`,
read through a `Resource` owned by the component that displays it. There is no
global prefetch and no shared data signals.

The content routes (`/`, `/fixtures`, `/today`, `/team/*`) declare
`ssr=SsrMode::Async` and `await` their resources inside `Suspense`, so their HTML
leaves the server complete. Reading a resource with `.get()` inside `Suspense`
instead renders the fallback into the HTML and swaps the content in from a
template after hydration, which is what those pages used to do.

Filtering belongs in SQL. A server function returns the rows a page shows, not a
table for the browser to sift through.

Mutations are `ServerAction`s. Keying a `Resource` on an action's `.version()`
refetches it when the action completes, which is what the removed version
counters did by hand.

The followed team lives in the `footical_team` cookie rather than local storage,
because the server can only render a visitor's team if the request carries it.

`require_admin_session` in `main.rs` redirects unauthenticated admin requests
before Leptos renders. A `Redirect` component cannot do this on a streamed
response: the 200 status is already committed by the time it sets the location
header. The `AdminOnly` component still guards client-side navigation, and each
admin server function checks the session itself.

## Develop

`direnv allow` puts the toolchain, `cargo-leptos`, `leptosfmt`, `just` and
`sqlite` on the path. Without direnv, use `nix develop`.

```sh
just watch   # or: nix run .#watch
just check   # fmt, leptosfmt, clippy (ssr and hydrate), tests
```

`footical-watch` serves on `http://localhost:3003` against its own SQLite
file at `~/.local/share/footical/footical.db`, created on first run. It sets
`DATABASE_URL`, `ADMIN_PASSWORD` (`development`) and `COOKIE_SECRET`, and
because all three are present the service never reaches for Bitwarden, so no
access token is needed locally. Export any of them yourself to override.

The admin pages can trigger a scrape on demand to fill an empty database.

## Deploy

Runs on `apollo` as a flake input of `~/Projects/chichek-infrastructure`.
This repo is on a personal account with no self-hosted runner, so the deploy
is manual: push, then re-lock the pin and activate.

```sh
cd ~/Projects/chichek-infrastructure
nix flake update footical    # re-lock to main HEAD
nix run . -- .#apollo
```

Pushing to `chichek-infrastructure` main also deploys apollo on its own
runner, so committing the re-locked `flake.lock` is enough on its own.

Roll back a bad commit by pinning a good SHA instead of tracking `main`:

```sh
nix flake lock --override-input footical "git+https://github.com/aydinaksel/footical?rev=<good-sha>"
nix run . -- .#apollo
```

The access token lives at `/var/lib/bws/footical-access-token` on the host and
reaches the service as a systemd credential. A new host needs that file placed
by hand before the unit will start.
