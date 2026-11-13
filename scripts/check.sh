#!/usr/bin/env bash
# The definition of done for svrngs (AGENTS.md). Each step reports passed,
# failed or never-ran, and the script exits 1 if any step failed or never ran:
# a step that never ran is owed, never counted as a pass.
set -u
cd "$(dirname "$0")/.."
git config core.hooksPath .githooks # every clone that runs this gets the hook
# The conformance canary holds about 250 files open at once (six tests, three
# nodes each, in one process), so under launchd's soft limit of 256
# (`launchctl limit maxfiles`) it fails by the launcher, not the code: raised
# here, said, so a run reads the same from any shell (sup-1-app-named).
soft=$(ulimit -n)
ulimit -Sn 10240 2>/dev/null || ulimit -Sn "$(ulimit -Hn)"
echo "note       open files: $soft, raised to $(ulimit -n)"
# Awake while this runs, on a Mac: an idle Mac slept 3.16 h of a run on
# 2026-10-30 (`pmset -g log`), its steps going on only in the dark wakes, and
# `tests` said "passed (10396 s)" against a limit of 1535 s, since a step's
# limit is a `sleep`, which stops while the Mac sleeps, and its time is
# $SECONDS, which does not. Idle sleep only: a closed lid on battery still sleeps.
command -v caffeinate >/dev/null && { caffeinate -i -w $$ & echo "note       kept awake while this runs (caffeinate, pid $!)"; }
failed=0
unset SVRNGS_STEPS SVRNGS_TESTS  # set only by --touched, below
# Each step's time limit, so a hung step fails in minutes, not when the
# loop's budget ends (capability-step-limit: `app laws` held 2h35m, its
# runner kept open by a page test's server). The limit is 5 times the step's
# slowest passing run in the capability queue's 14 gates (2026-10-23 to
# 2026-10-24; demo/runs/step-limit/2026-10-24/README.md): those runs spread
# up to 2.7 times (scenes, 204 to 544 s) with the host's load, and 5 leaves
# room past the slowest seen while ending a hang in about 45 min at most. Every
# step not named passed in at most 59 s there (app laws), so has 300 s.
# `playback`, new on 2026-11-12, has no queue's gates behind it: with the
# night scene's playing in it, it took 225 s on the Mac that day
# (demo/runs/flakes/2026-11-12/mac/night-playback-step.log), so 225.
slowest() { case $1 in tests) echo 307 ;; browser) echo 246 ;; scenes) echo 544 ;; "conformance canary") echo 178 ;; playback) echo 225 ;; *) echo 60 ;; esac; }
# Each step runs in a process group of its own (set -m), ended whole at its
# limit and when it ends, so no server or browser it started outlives it
# (Playwright's browsers close as their driver is told TERM); the loop's
# TERM, or a Ctrl-C, is passed on to the step running.
set -m
trap 'kill -TERM -${running:-} 2>/dev/null; exit 1' INT TERM HUP
# A step the touched tier leaves out (`--touched`, below) is said untouched.
reached() { [ -z "${SVRNGS_STEPS+x}" ] || printf '%s\n' "$SVRNGS_STEPS" | grep -qxF "$1"; }
step() {  # step <name> <command...>
  reached "$1" || { echo "untouched  $1: nothing changed reaches it (scripts/touched.py)"; return; }
  local name=$1 began=$SECONDS limit=$((5 * $(slowest "$1"))) status watch; shift
  "$@" >/tmp/svrngs-check.$$ 2>&1 & running=$!
  (sleep "$limit"; kill -TERM -$running) 2>/dev/null & watch=$!
  wait $running; status=$?; { kill -TERM -$watch; kill -TERM -$running && sleep 2; kill -KILL -$running; } 2>/dev/null
  if [ $status = 0 ]; then echo "passed     $name ($((SECONDS - began)) s)"
  elif ((SECONDS - began >= limit)); then echo "failed     $name: still running at its limit of $limit s, so it and all it started were ended; its last output:"; tail -n 20 /tmp/svrngs-check.$$ | sed 's/^/           /'; failed=1
  else echo "failed     $name ($((SECONDS - began)) s)"; sed 's/^/           /' /tmp/svrngs-check.$$; failed=1; fi
}
step "principles pinned"  python3 scripts/principles_check.py
step "lean and RUST_ARCH" python3 scripts/lean_check.py
# What a page or a socket may do, held to its one home, scripts/trust_floor.txt
# (svrn-docs rings/current/WHAT_RUNS_AS_YOU.md §4; canon-floor).
step "trust floor"        python3 scripts/trust_check.py
step "advisories"         python3 scripts/advisories.py
step "app nouns gate"   python3 scripts/vocab_check.py --self-test
step "app nouns"        python3 scripts/vocab_check.py
# Every JavaScript file but what is vendored or recorded, as .oxlintrc.json
# says, by oxlint as web/package.json pins it: a page that does not parse
# fails here in a second, not minutes on as a browser's timeout (keys-names:
# Money's page from bb8d4a3e to 7dfc59c6 declared `said` twice). Without it installed
# the step fails naming the install, never skipping.
step "lint" sh -c 'test -x web/node_modules/.bin/oxlint || { echo "oxlint is not installed here: npm ci --prefix web"; exit 1; }; web/node_modules/.bin/oxlint'
# The same files, as oxfmt lays them out (.oxfmtrc.json), so how a line looks
# is the formatter's and never a way to fit a budget (ralph/next/js/CHARTER.md,
# "Why"): each file it would change is named. Not installed fails, as lint.
step "format" sh -c 'test -x web/node_modules/.bin/oxfmt || { echo "oxfmt is not installed here: npm ci --prefix web"; exit 1; }; web/node_modules/.bin/oxfmt --check "**/*.{js,mjs,cjs}"'
# Isolated Node contracts: no browser, native build or service. The catalog
# refuses an empty tier; *.integration.test.mjs has its own step below.
step "page laws" node web/tests/run.mjs unit
step "rustfmt"  cargo fmt --all --check
# The scripts' own laws (scripts/tests: advisories, the touched tier's plans).
step "script laws" python3 -m unittest scripts.tests.test_advisories scripts.tests.test_touched
# All of the above takes under a minute. A failure there is one the rest cannot
# change, so it ends the run; `--fast` stops here always (.githooks/pre-commit).
if [ "$failed" != 0 ] || [ "${1:-}" = --fast ]; then exit "$failed"; fi
# `--touched [base]`: only the steps below a change since `base` can reach,
# by scripts/touched.py, which says why each runs. Never the gate before a push.
if [ "${1:-}" = --touched ]; then plan=$(python3 scripts/touched.py "${@:2}") || exit 1; eval "$plan"; fi
# Actual Redis/Lua lifecycle and WebKit/CSS evidence, never replaced by fixtures.
step "page integration" node web/tests/run.mjs integration
# Then the test targets `cargo test` skips (`test = false`), which
# --all-targets leaves out and a glob of names reaches.
step "clippy"   sh -c "cargo clippy --quiet --all-targets -- -D warnings && cargo clippy --quiet --workspace --test '*' -- -D warnings"
# What the browser tests could not judge, printed after the browser's step,
# and the conformance canary's verdicts: kept from this check alone.
rm -rf target/could-not-judge target/conformance target/playback target/scenes/night_played-*
# The canaries, browser, joining and conformance targets are `test = false`
# (crates/svrngs-node/Cargo.toml, crates/svrngs/Cargo.toml), so each runs
# once, in its own step below, and not again here.
step "tests"    cargo test --quiet --no-fail-fast ${SVRNGS_TESTS:-}
# The canaries (DESIGN §5), by name: the apps that keep the infra general.
step "canaries" cargo test --quiet --workspace --test canaries
# The ceiling's canary (DESIGN §5, "The canaries"; PLAN §4, "The ceiling";
# ceiling-4), the one scale number held here: at 25 members and 10,000 acts
# an idle round under 0.1 s of processor, in release, by name from the
# ignored instrument. Its reading is printed, passed or failed.
step "ceiling canary" cargo test --release --quiet -p svrngs-node --test scale -- --ignored --exact an_idle_round_is_under_the_ceiling --nocapture
! reached "ceiling canary" || grep -h '^CEILING' /tmp/svrngs-check.$$ | sed 's/^/           /' | grep . || { echo "never-ran  ceiling canary: no CEILING reading, so no test of that name ran"; failed=1; }
# The page (PLAN §4, U4): its crate is a workspace of its own, for wasm.
step "web rustfmt and clippy" scripts/build-web.sh --lint
# The page in a browser, by name: it builds the page, runs a relay
# and a node on 127.0.0.1 and drives Playwright, each test in the engine
# its page stands for, a computer's page in Chromium and a phone's in
# WebKit as an iPhone, and in both only where they differ (harness.rs
# `in_chromium!`, `in_webkit!`, `in_both_engines!`); one browser of each
# for the step, each test a context of its own (harness.rs `browsers`).
# Without Playwright or its browsers it fails naming the install command;
# it never skips. A lent film here is its bytes through the lend's worker
# (main.rs `film_answered`); its playing is the `playback` step's, below.
step "browser" cargo test --quiet --workspace --test browser -- --skip playback::
# The pages' types (jsconfig.json, and the service workers' in
# jsconfig.workers.json, by scripts/types.mjs), checked against the wasm
# glue's declaration built here, target/web/site/wasm/: the browser step
# builds its own page per test process (harness.rs `site`), so read alone
# that directory was whatever an earlier build left (release 2: a glue of
# 10 November, with no `Group.older`). Not installed fails, as lint.
step "types" sh -c 'test -f web/node_modules/typescript/package.json || { echo "typescript is not installed here: npm ci --prefix web"; exit 1; }; scripts/build-web.sh >/dev/null && node scripts/types.mjs'
# The rules of .oxlintrc.json that read those types, which `lint` skips
# without a word, over the same two programs (scripts/typed-lint.mjs): after
# `types`, for the same wasm declaration. Not installed fails, as lint.
step "typed lint" node scripts/typed-lint.mjs
# A phone joining by the link `svrngs new --origin` prints (u4-5), in
# WebKit as an iPhone.
step "joining in a browser" cargo test --quiet --workspace --test join_page
# What the page queue made the pages do, each feature's test
# (crates/svrngs/tests/pages): read again, answered beside the button,
# refused by name, with no reload.
step "pages" cargo test --quiet --workspace --test pages
# The demo scenes, played (capability-scenes; crates/svrngs/tests/
# scenes): each scene end to end, every person this build's svrngs with
# a HOME of their own, met at relays on 127.0.0.1 and never n0's (the
# scenes' binary as `main`, people.rs `program`),
# played once, each person's browser the engine
# their device is (scenes/main.rs `scene!`); each
# scene's verdict on its own line, in the order they ended (libtest's
# line for `svrngs_here`, the ignored test each person's svrngs runs, is
# not a scene's). A scene whose
# third-party app is not installed here writes never-ran and why in its
# place, target/scenes/<scene>-<engine>/never-ran.txt (each place is
# emptied as its scene starts), and the gate fails on it. A scene that
# passed all it could judge but one timing bar on a host past load.rs
# `BUSY` a core (people.rs `late`) writes could-not-judge.txt there: said
# so, never a pass, and the gate does not fail on it, as the playback
# step's film. No scene here waits on a film playing: night's TV playing,
# paused and sought from a phone is the playback step's (night.rs
# `night_played`; the operator, 2026-11-12, demo/runs/flakes/2026-11-12).
# Two scenes at a time, the operator's choice (2026-10-23, "2 at a time")
# from what capability-scenes-found (f) measured on this host (12 cores,
# about 10 of load from elsewhere; load.rs's bar, 3 a core; sampled each
# 5 s): 12 threads 162 s, peak 140, 29 of 32 samples past; 4, 201 s, peak
# 66, 28 of 40; 2, 319 s, peak 44, 6 of 63; 1, 560 s, peak 21, none. At 2
# a timing is now and then could-not-judge, never a false failure.
step "scenes" cargo test -p svrngs --test scenes -- --test-threads=2 --skip night_played::
! reached scenes || awk '/^test .* \.\.\. / && $4 !~ /^ignored/ {split($2, n, "::"); k = n[1]; if (!(k in s)) o[++m] = k; e[k] = e[k] (e[k] ? ", " : "") n[3]; if (!(k in s) || $NF != "ok") s[k] = $NF == "ok" ? "passed" : "failed"} FILENAME ~ /never-ran/ {k = FILENAME; sub(/.*scenes\//, "", k); sub(/-[a-z]+\/never-ran.txt$/, "", k); if (!(k in s)) o[++m] = k; s[k] = "never-ran"; w[k] = ": " $0; x = 1} FILENAME ~ /could-not-judge/ {k = FILENAME; sub(/.*scenes\//, "", k); sub(/-[a-z]+\/could-not-judge.txt$/, "", k); if (s[k] == "passed") {s[k] = "could-not-judge"; w[k] = ": " $0}} END {for (i = 1; i <= m; i++) printf "%-10s scene %s (%s)%s\n", s[o[i]], o[i], e[o[i]], w[o[i]]; exit x}' /tmp/svrngs-check.$$ $(ls target/scenes/*/never-ran.txt target/scenes/*/could-not-judge.txt 2>/dev/null) || failed=1
# The conformance canary (DESIGN §5; u5-2): apps/web-conformance, lent by
# two computers, opened from a computer and from a phone, each app at an
# address of its own (DESIGN §7), in both engines; and, with accounts of its
# own, two people signed in through a lend's sign-in (mwp-4). Each route's
# verdict per way is printed, once for every engine and lender that agree.
# Its seeks back to back, the one route that plays, are the `playback`
# step's (conformance.rs `seeks_back_to_back`).
step "conformance canary" cargo test --quiet --workspace --test conformance -- --skip seeks_back_to_back::
! reached "conformance canary" || cat target/conformance/*.txt 2>/dev/null | cut -c1-10,25- | sort -u | sed 's/^/           /'
# The one place a browser is asked to decode media (demo/runs/flakes/
# 2026-11-12): a lent film played, sought three times and played again
# once its worker stopped, in each engine (browser/playback.rs), the
# conformance canary's audio sought back to back (conformance.rs
# `seeks_back_to_back`), and movie night's TV playing Ana's film, paused
# and sought from her phone (night.rs `night_played`, a scene: its
# never-ran said here). Every other test asserts a film's bytes instead.
# Decoding is what varies with the host's load and media stack, so it is
# judged here alone: a miss on a host past load.rs `BUSY` a core is
# could-not-judge, said apart and never a pass; WebKit's seeks back to
# back are a rate. Each half must run a test: a filter that matched none
# is never-ran.
step "playback" sh -c 'cargo test --workspace --test browser playback:: && cargo test --workspace --test conformance seeks_back_to_back:: && cargo test -p svrngs --test scenes night_played::'
! reached playback || { [ "$(grep -cE '^test result: ok\. [1-9]' /tmp/svrngs-check.$$)" = 3 ] && ! cat target/scenes/night_played-*/never-ran.txt 2>/dev/null; } || grep -q '^test result: FAILED' /tmp/svrngs-check.$$ || { echo "never-ran  playback: a part ran no test (browser playback::, conformance seeks_back_to_back::, scenes night_played::), or the scene never ran, as above"; failed=1; }
# What the browser and playback steps could not judge (a miss past the load
# bar; acts.rs's read rate, read slower than the rate): said apart, never a
# pass, each test's line in target/could-not-judge/<step>.txt.
! reached browser && ! reached playback || cat target/could-not-judge/*.txt target/scenes/night_played-*/could-not-judge.txt 2>/dev/null | sed 's/^/could-not-judge /'
# WebKit's seeks back to back, a rate and not a gate (u5-11): this run's,
# beside the same seeks straight at the canary, then every run's here.
if reached playback && [ -s target/seeks-webkit.log ] && ls target/playback/seeks-*.tsv >/dev/null 2>&1; then
  echo "rate       WebKit seeks back to back (u5-11): way, lend; through the lend and straight at the canary, the ms the last took to play again and its verdict"
  cut -f1,2,4,5,7,8 target/playback/seeks-*.tsv | sed 's/^/           /'
  awk -F'\t' -v cores="$(getconf _NPROCESSORS_ONLN)" '{n++; l+=$5=="failed"; s+=$8=="failed"; split($10, a, "averages?: "); hot+=a[2]+0>cores} END {printf "           of %d here since %s: %d missed 5 s through a lend, %d straight; %d ended at a load above %d cores\n", n, f, l, s, hot, cores} NR==1 {f=substr($9,1,20)}' target/seeks-webkit.log
elif reached playback; then echo "never-ran  WebKit's seeks back to back, a rate: nothing kept"; failed=1; fi
step "app laws" node web/tests/run.mjs apps
rm -f /tmp/svrngs-check.$$; echo "note       took $SECONDS s in all"
exit $failed
