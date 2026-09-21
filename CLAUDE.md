# Footical

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
