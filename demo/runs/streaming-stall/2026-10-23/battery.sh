#!/bin/bash
# streaming-stall (a)'s first battery, one cell after another; then the
# summary of every run.
here=$(cd "$(dirname "$0")" && pwd)
"$here/run.sh" alone-burn24 alone 24 1 10
"$here/run.sh" suite suite 0 1 3
python3 -I "$here/summary.py" "$here"/data/*
