#!/bin/bash
set -e

mkdir -p /data/ical

tailscaled --tun=userspace-networking --state=/data/tailscale-state &
sleep 2
tailscale up --authkey="${TAILSCALE_AUTHKEY}" --hostname=footical-fly

exec footical-app
