# Footical

## Deploy

Runs on `apollo` as a flake input of `~/Projects/chichek-infrastructure`
(tracks `main`). Push, then re-lock the pin and activate:

```sh
cd ~/Projects/chichek-infrastructure
nix flake update footical    # re-lock to main HEAD
nix run . -- .#apollo
```

Roll back a bad commit by pinning a good SHA instead of tracking `main`:

```sh
nix flake lock --override-input footical "git+https://github.com/aydinaksel/footical?rev=<good-sha>"
nix run . -- .#apollo
```
