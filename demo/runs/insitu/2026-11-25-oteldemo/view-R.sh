#!/usr/bin/env bash
# Arm R's tool: GET a route of the site's view (site-raw), through svrngs.
exec env HOME=/home/alexbryan/dev/svrngs/demo/runs/insitu/2026-11-25-oteldemo/out/vendor/home SVRNGS_DIR=/home/alexbryan/dev/svrngs/demo/runs/insitu/2026-11-25-oteldemo/out/vendor/data node /home/alexbryan/dev/svrngs/demo/runs/insitu/2026-11-25-oteldemo/view.mjs site-raw "${1:-/}"
