#!/usr/bin/env bash
# The definition of done for svrngs (AGENTS.md). Each step reports one of four
# verdicts — passed, failed, could-not-judge, never-ran — and the script exits 1
# if any step failed. A step that never ran is reported, never counted as a pass.
set -u
cd "$(dirname "$0")/.."
failed=0
step() {  # step <name> <command...>
  local name=$1; shift
  if "$@" >/tmp/svrngs-check.$$ 2>&1; then echo "passed     $name"
  else echo "failed     $name"; sed 's/^/           /' /tmp/svrngs-check.$$; failed=1; fi
}
step "principles pinned"  python3 scripts/principles_check.py
step "lean and RUST_ARCH" python3 scripts/lean_check.py
if [ -f Cargo.toml ]; then
  step "rustfmt"  cargo fmt --all --check
  step "clippy"   cargo clippy --quiet --all-targets -- -D warnings
  step "tests"    cargo test --quiet
else
  echo "never-ran  rustfmt, clippy, tests: no Cargo workspace yet"
fi
tests=$(find apps -name '*.test.mjs' 2>/dev/null)
if [ -n "$tests" ]; then step "app laws" node --test $tests
else echo "never-ran  app tests: none yet"; fi
rm -f /tmp/svrngs-check.$$
exit $failed
