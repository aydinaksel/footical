#!/bin/sh
set -e

tailscaled --state=/var/lib/tailscale/tailscaled.state --socket=/var/run/tailscale/tailscaled.sock &
tailscale up --auth-key="${TAILSCALE_AUTHKEY}" --hostname=footical-fly

until tailscale status --json | grep -q '"Online": true'; do sleep 0.5; done

for attempt in 1 2 3 4 5 6 7 8 9 10; do
  if getent hosts demeter.darter-bebop.ts.net > /dev/null 2>&1; then
    break
  fi
  echo "waiting for tailnet DNS (attempt $attempt/10)..."
  sleep 2
done

exec footical-website
