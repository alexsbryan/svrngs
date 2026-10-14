#!/bin/bash
# Build the product; re-pub each item a downstream crate is refused; repeat.
# start-economize's loop.sh, its paths v1ui-economize's.
S=/tmp/v1ui-econ-scratch
HERE=$(cd "$(dirname "$0")" && pwd)
cd $S/${DIR:-.}
for i in $(seq 1 25); do
  CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=${TD:-/tmp/v1ui-econ-target} cargo build ${ARGS:---workspace} --offline --message-format=json 2>/dev/null > /tmp/v1ui-econ-b.json
  out=$(DIR=${DIR:-.} python3 "$HERE/repub.py" $S /tmp/v1ui-econ-b.json)
  echo "round $i: $(echo "$out" | tail -1)"
  echo "$out" | grep "UNHANDLED" | head -3
  echo "$out" | grep -q "fixed 0" && break
done
