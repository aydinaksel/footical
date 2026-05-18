#!/bin/bash
set -e

tailscaled --tun=userspace-networking --state=/data/tailscale-state &
tailscale up --authkey="${TAILSCALE_AUTHKEY}" --hostname=footical-fly
until tailscale status --json | grep -q '"Online": true'; do sleep 0.5; done

for attempt in 1 2 3 4 5; do
  if getent hosts demeter.darter-bebop.ts.net > /dev/null 2>&1; then
    break
  fi
  echo "waiting for tailnet DNS (attempt $attempt/5)..."
  sleep 2
done

exec footical-website
