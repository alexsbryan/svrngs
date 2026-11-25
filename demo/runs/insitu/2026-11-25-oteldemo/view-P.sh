#!/usr/bin/env bash
# Arm P's tool: GET a route of the site's view (site-view), through svrngs.
exec env HOME=/home/alexbryan/dev/svrngs/demo/runs/insitu/2026-11-25-oteldemo/out/vendor/home SVRNGS_DIR=/home/alexbryan/dev/svrngs/demo/runs/insitu/2026-11-25-oteldemo/out/vendor/data node /home/alexbryan/dev/svrngs/demo/runs/insitu/2026-11-25-oteldemo/view.mjs site-view "${1:-/}"
