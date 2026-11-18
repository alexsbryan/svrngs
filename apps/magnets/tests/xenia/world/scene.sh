#!/bin/sh
# Every Magnets scene's command: each scene here is a link to this file, and
# its name, $0's, names its scene in scene.mjs.
exec node "$(dirname "$0")/world/scene.mjs" "$(basename "$0")"
