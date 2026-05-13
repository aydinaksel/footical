#!/bin/bash
set -e

mkdir -p /data/ical

tailscaled --tun=userspace-networking --state=/data/tailscale-state &
tailscale up --authkey="${TAILSCALE_AUTHKEY}" --hostname=footical-fly
until tailscale status --json | grep -q '"Online": true'; do sleep 0.5; done

exec footical-app
