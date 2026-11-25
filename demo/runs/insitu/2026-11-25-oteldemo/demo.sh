#!/usr/bin/env bash
# Runs OTel Demo 3.1.0 (vendor-cache/src, sha256 8c461c53…4777) with the
# run's override, from its own folder so its relative paths hold.
#   ./demo.sh up | down | ps | logs <service>
set -euo pipefail
cd "$(dirname "$0")/vendor-cache/src"
# DEMO_VERSION: 3.1.0's .env runs `latest` images, whatever was built that
# day (2026-11-25's recommendation image lacks the nc its health check
# calls), so the run pins the images to the release it runs. IMAGE_VERSION,
# the version services report, says the same.
export DEMO_VERSION=3.1.0 IMAGE_VERSION=3.1.0 HOST_FILESYSTEM=$(realpath ../../out/empty-hostfs) DOCKER_SOCK=$(realpath ../../out/no-docker.sock)
f=(-f compose.yaml -f ../../compose.site.yaml)
case "$1" in
  up) podman-compose "${f[@]}" up -d ;;
  down) podman-compose "${f[@]}" down ;;
  ps) podman ps --format '{{.Names}} {{.Status}}' | sort ;;
  logs) podman logs --tail 40 "$2" ;;
esac
