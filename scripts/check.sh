#!/usr/bin/env bash
# The definition of done for svrngs (AGENTS.md). Each step reports passed,
# failed or never-ran, and the script exits 1 if any step failed or never ran:
# a step that never ran is owed, never counted as a pass.
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
step "app nouns gate"   python3 scripts/vocab_check.py --self-test
step "app nouns"        python3 scripts/vocab_check.py
if [ -f Cargo.toml ]; then
  step "rustfmt"  cargo fmt --all --check
  step "clippy"   cargo clippy --quiet --all-targets -- -D warnings
  step "tests"    cargo test --quiet
  # The canaries (DESIGN §8) again, by name: the apps that keep the infra general.
  step "canaries" cargo test --quiet -p svrngs-node --test canaries
  # The page (DESIGN §8, U4): its crate is a workspace of its own, for wasm.
  step "web rustfmt and clippy" scripts/build-web.sh --lint
  # The page in a browser, again by name: it builds the page, runs a relay
  # and a node on 127.0.0.1 and drives Playwright's Chromium, and its
  # WebKit as an iPhone, each test in both (u4-7). Without Playwright or its
  # browsers it fails naming the install command; it never skips.
  step "browser" cargo test --quiet -p svrngs-node --test browser
  # A phone joining by the link `svrngs new --origin` prints (u4-5), in
  # both engines.
  step "joining in a browser" cargo test --quiet -p svrngs --test join_page
  # The conformance canary (DESIGN §8; u5-2): apps/web-conformance, lent by
  # two computers, opened from a computer and from a phone both ways a group
  # opens its apps (DESIGN §7), in both engines. Each route's verdict per
  # way is printed, once for every engine and lender that agree.
  rm -rf target/conformance
  step "conformance canary" cargo test --quiet -p svrngs-node --test conformance
  cat target/conformance/*.txt 2>/dev/null | cut -c1-10,25- | sort -u | sed 's/^/           /'
else
  echo "never-ran  rustfmt, clippy, tests: no Cargo workspace yet"; failed=1
fi
tests=$(find apps -name '*.test.mjs' 2>/dev/null)
if [ -n "$tests" ]; then step "app laws" node --test $tests
else echo "never-ran  app tests: none yet"; failed=1; fi
rm -f /tmp/svrngs-check.$$
exit $failed
