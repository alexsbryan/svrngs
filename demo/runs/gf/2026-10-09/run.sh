#!/bin/sh
# gf-1's run, rerunnable (README §2-§4): apps/jellyfin/folders/run.sh, which
# keeps the run's own Jellyfin driving in apps/ (fence 5), its data in run/.
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
cargo build --offline -p svrngs --manifest-path "$HERE/../../../../Cargo.toml"
exec "$HERE/../../../../apps/jellyfin/folders/run.sh" "$HERE/run"
