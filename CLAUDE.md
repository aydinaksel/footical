# Footical

## Deploy

Runs on `apollo` as a flake input of `~/Projects/infrastructure-chichek`
(tracks `main`). Push, then re-lock the pin and activate:

```sh
cd ~/Projects/infrastructure-chichek
nix flake update footical    # re-lock to main HEAD
deploy .#apollo
```

Roll back a bad commit by pinning a good SHA instead of tracking `main`:

```sh
nix flake lock --override-input footical "git+ssh://git@github.com/aydinaksel/footical?rev=<good-sha>"
deploy .#apollo
```
