#!/bin/bash
# Purge all demo content by recreating the demo container.
# Storage is container-local, so a recreate = guaranteed wipe.
# Installed as hourly root cron on CI; safe to run manually anytime.
set -e
cd "$(dirname "$0")/.."   # repo root (script lives in demo/)
docker compose -f demo/docker-compose.yml up -d --force-recreate demo
docker image prune -f --filter "label=com.docker.compose.project=demo" >/dev/null 2>&1 || true
echo "$(date -Is) demo purged (container recreated)"
