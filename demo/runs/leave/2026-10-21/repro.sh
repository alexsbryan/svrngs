#!/usr/bin/env bash
# #211's reproduction (SURVEY.md). Copies each test file into its crate's
# tests/, runs it, and removes the copy on any exit, so the tree is left as
# it was. `core` runs the fold's three cases (they pass: they assert HEAD's
# divergence); `node` runs the three-computer cases (the first is red at
# HEAD: it asserts what a fix must make true). No argument runs both.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../../.." && pwd)"
placed=() made=()
cleanup() {
  for f in "${placed[@]}"; do rm -f "$f"; done
  for d in "${made[@]}"; do rmdir "$d" 2>/dev/null; done
}
trap cleanup EXIT
export CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
status=0
run() { # crate file extra-args...
  local crate="$1" file="$2"; shift 2
  local dir="$root/crates/$crate/tests" to="$root/crates/$crate/tests/$file"
  [[ -d "$dir" ]] || { mkdir "$dir"; made+=("$dir"); }
  cp "$here/$file" "$to"; placed+=("$to")
  (cd "$root" && cargo test -q -p "$crate" "$@" --test "${file%.rs}" -- --nocapture --test-threads 1) || status=$?
}
want="${1:-all}"
[[ "$want" == core || "$want" == all ]] && run svrngs-core leave_race_core.rs --features test-support
[[ "$want" == node || "$want" == all ]] && run svrngs-node leave_race_node.rs
exit $status
