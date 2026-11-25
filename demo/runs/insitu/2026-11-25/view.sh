#!/usr/bin/env bash
# The vendor's agent's view of the clinic: GET a path of the site's view.
# It only reads. In the field this reaches the site through a svrngs lend
# (demo/runs/support-link/2026-11-25); here, the rehearsal's seam, podman.
#   ./view.sh /            lists the routes
exec podman exec onprem-clinic curl -s "http://127.0.0.1:8620${1:-/}"
