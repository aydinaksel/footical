# Footical

## Deploy

Runs on `apollo` as a flake input of `~/Projects/chichek-infrastructure`.
Pushing to `main` is the deploy: `.github/workflows/deploy.yml` re-locks the
pin in that repo and activates apollo from a self-hosted runner.

Roll back a bad commit by pinning a good SHA instead of tracking `main`:

```sh
cd ~/Projects/chichek-infrastructure
nix flake lock --override-input footical "git+https://github.com/aydinaksel/footical?rev=<good-sha>"
nix run . -- .#apollo
```

The access token lives at `/var/lib/bws/footical-access-token` on the host and
reaches the service as a systemd credential. A new host needs that file placed
by hand before the unit will start.
