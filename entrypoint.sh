#!/bin/sh

tailscaled --state=/var/lib/tailscale/tailscaled.state --socket=/var/run/tailscale/tailscaled.sock &
tailscale up --auth-key="${TAILSCALE_AUTHKEY}" --hostname=footical-fly

exec footical-website
