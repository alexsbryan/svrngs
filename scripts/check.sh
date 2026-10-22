#!/usr/bin/env bash
# The definition of done for svrngs (AGENTS.md). Each step reports passed,
# failed or never-ran, and the script exits 1 if any step failed or never ran:
# a step that never ran is owed, never counted as a pass.
set -u
cd "$(dirname "$0")/.."
# The conformance canary holds about 250 files open at once (six tests, three
# nodes each, in one process), so under launchd's soft limit of 256
# (`launchctl limit maxfiles`) it fails by the launcher, not the code: raised
# here, said, so a run reads the same from any shell (sup-1-app-named).
soft=$(ulimit -n)
ulimit -Sn 10240 2>/dev/null || ulimit -Sn "$(ulimit -Hn)"
echo "note       open files: $soft, raised to $(ulimit -n)"
failed=0
step() {  # step <name> <command...>
  local name=$1 began=$SECONDS; shift
  if "$@" >/tmp/svrngs-check.$$ 2>&1; then echo "passed     $name ($((SECONDS - began)) s)"
  else echo "failed     $name ($((SECONDS - began)) s)"; sed 's/^/           /' /tmp/svrngs-check.$$; failed=1; fi
}
step "principles pinned"  python3 scripts/principles_check.py
step "lean and RUST_ARCH" python3 scripts/lean_check.py
# What a page or a socket may do, held to its one home, scripts/trust_floor.txt
# (svrn-docs rings/current/WHAT_RUNS_AS_YOU.md §4; canon-floor).
step "trust floor"        python3 scripts/trust_check.py
step "app nouns gate"   python3 scripts/vocab_check.py --self-test
step "app nouns"        python3 scripts/vocab_check.py
if [ -f Cargo.toml ]; then
  step "rustfmt"  cargo fmt --all --check
  step "clippy"   cargo clippy --quiet --all-targets -- -D warnings
  # What the browser tests could not judge, either run's (the tests' and the
  # browser's below), printed after the browser's: kept from this check alone.
  rm -rf target/could-not-judge
  step "tests"    cargo test --quiet
  # The canaries (DESIGN §5) again, by name: the apps that keep the infra general.
  step "canaries" cargo test --quiet -p svrngs-node --test canaries
  # The ceiling's canary (DESIGN §5, "The canaries"; PLAN §4, "The ceiling";
  # ceiling-4), the one scale number held here: at 25 members and 10,000 acts
  # an idle round under 0.1 s of processor, in release, by name from the
  # ignored instrument. Its reading is printed, passed or failed.
  step "ceiling canary" cargo test --release --quiet -p svrngs-node --test scale -- --ignored --exact an_idle_round_is_under_the_ceiling --nocapture
  grep -h '^CEILING' /tmp/svrngs-check.$$ | sed 's/^/           /'
  # The page (PLAN §4, U4): its crate is a workspace of its own, for wasm.
  step "web rustfmt and clippy" scripts/build-web.sh --lint
  # The page in a browser, again by name: it builds the page, runs a relay
  # and a node on 127.0.0.1 and drives Playwright's Chromium, and its
  # WebKit as an iPhone, each test in both (u4-7). Without Playwright or its
  # browsers it fails naming the install command; it never skips.
  step "browser" cargo test --quiet -p svrngs-node --test browser
  # The film's seeks on a host past the seek bar's load bar (bash-film-load;
  # crates/svrngs-node/tests/browser/load.rs, conformance's too): said apart,
  # never a pass.
  sed 's/^/could-not-judge /' target/could-not-judge/browser.txt 2>/dev/null
  # A phone joining by the link `svrngs new --origin` prints (u4-5), in
  # both engines.
  step "joining in a browser" cargo test --quiet -p svrngs --test join_page
  # The conformance canary (DESIGN §5; u5-2): apps/web-conformance, lent by
  # two computers, opened from a computer and from a phone, each app at an
  # address of its own (DESIGN §7), in both engines; and, with accounts of its
  # own, two people signed in through a lend's sign-in (mwp-4). Each route's
  # verdict per way is printed, once for every engine and lender that agree.
  rm -rf target/conformance
  step "conformance canary" cargo test --quiet -p svrngs-node --test conformance
  cat target/conformance/*.txt 2>/dev/null | grep -v could-not-judge | cut -c1-10,25- | sort -u | sed 's/^/           /'
  # Chromium's seeks on a host past its load bar (chk-1; tests/browser/load.rs): said apart, never a pass.
  grep -h could-not-judge target/conformance/*.txt 2>/dev/null | sed 's/^/could-not-judge /'
  # WebKit's seeks back to back, a rate and not a gate (u5-11): this run's,
  # beside the same seeks straight at the canary, then every run's here.
  if [ -s target/seeks-webkit.log ] && ls target/conformance/seeks-*.tsv >/dev/null 2>&1; then
    echo "rate       WebKit seeks back to back (u5-11): way, lend; through the lend and straight at the canary, the ms the last took to play again and its verdict"
    cut -f1,2,4,5,7,8 target/conformance/seeks-*.tsv | sed 's/^/           /'
    awk -F'\t' -v cores="$(getconf _NPROCESSORS_ONLN)" '{n++; l+=$5=="failed"; s+=$8=="failed"; split($10, a, "averages?: "); hot+=a[2]+0>cores} END {printf "           of %d here since %s: %d missed 5 s through a lend, %d straight; %d ended at a load above %d cores\n", n, f, l, s, hot, cores} NR==1 {f=substr($9,1,20)}' target/seeks-webkit.log
  else echo "never-ran  WebKit's seeks back to back, a rate: nothing kept"; failed=1; fi
else
  echo "never-ran  rustfmt, clippy, tests: no Cargo workspace yet"; failed=1
fi
tests=$(find apps -name '*.test.mjs' 2>/dev/null)
if [ -n "$tests" ]; then step "app laws" node --test $tests
else echo "never-ran  app tests: none yet"; failed=1; fi
rm -f /tmp/svrngs-check.$$; echo "note       took $SECONDS s in all"
exit $failed
