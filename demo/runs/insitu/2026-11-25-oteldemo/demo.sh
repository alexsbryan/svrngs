#!/usr/bin/env bash
# Runs OTel Demo 3.1.0 (vendor-cache/src, sha256 8c461c53…4777) with the
# run's override, from its own folder so its relative paths hold.
#   ./demo.sh up | down | ps | logs <service>
set -euo pipefail
cd "$(dirname "$0")/vendor-cache/src"
export HOST_FILESYSTEM=$(realpath ../../out/empty-hostfs) DOCKER_SOCK=$(realpath ../../out/no-docker.sock) ENVOY_PORT=8090
f=(-f compose.yaml -f ../../compose.site.yaml)
case "$1" in
  up) podman-compose "${f[@]}" up -d ;;
  down) podman-compose "${f[@]}" down ;;
  ps) podman ps --format '{{.Names}} {{.Status}}' | sort ;;
  logs) podman logs --tail 40 "$2" ;;
esac
