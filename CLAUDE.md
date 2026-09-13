# Footical

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
