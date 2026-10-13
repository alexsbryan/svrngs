#!/bin/bash
# Build the product; re-pub each item a downstream crate is refused; repeat.
cd /tmp/econ-scratch/${DIR:-.}
for i in $(seq 1 25); do
  CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=${TD:-/Users/alexsbryan/dev/svrngs/target} cargo build ${ARGS:---workspace} --offline --message-format=json 2>/dev/null > /tmp/econ_b.json
  out=$(python3 /tmp/econ_repub2.py /tmp/econ-scratch /tmp/econ_b.json)
  echo "round $i: $(echo "$out" | tail -1)"
  echo "$out" | grep -q "fixed 0" && break
done
