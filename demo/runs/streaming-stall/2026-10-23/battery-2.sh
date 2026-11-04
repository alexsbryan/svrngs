#!/bin/bash
# streaming-stall (a)'s second battery: the gate's own command
# (scripts/check.sh:53) under 24 `yes` of the run's own; then its summary.
here=$(cd "$(dirname "$0")" && pwd)
"$here/run.sh" gate-burn24 gate 24 1 5
python3 -I "$here/summary.py" "$here"/data/gate-burn24
