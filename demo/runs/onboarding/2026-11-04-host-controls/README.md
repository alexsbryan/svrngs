# Host controls — 2026-11-04

Unit: `onboarding-host-controls`. Local Chromium, real native publication
routes and owner-only receipts, fake resolver only. No deployment or people
scene. The inherited V1.6 publish artifacts were inspected and left untouched.

## Reuse and behavior

The existing `LINK` machine and `Links` view own the controls; the existing
`Copy` owns clipboard access. There is no new page, invitation implementation,
browser storage, HTTPS client or resolver browser access. ONBOARDING §4's
upstream inventory has no short-invitation controls to fork. Native status now
includes `address`, using the persisted resolver's origin and phrase, so the
view never guesses a short-address host or changes the distribution origin.

On mount, read local status. Publish/retry and close name the receipt id. A
failed mutation reads local status before offering retry: publication may have
committed even when its answer was lost, and failed close must recover as
closing. A failed status read offers its own retry. Ordinary/device invitation
creation rereads publication status because it supersedes the short invitation.
Invitation-changing buttons wait for the current request to finish. Machine
steps/refusals use the existing debug trace. No automatic mutation retries.

## Red first

Plant: HEAD `72cd9f05`'s home page, with no short-invitation controls. The new
browser test was installed before changing product code. Actual output:

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs-node --test browser publication::onboarding_host_controls_survive_restart -- --exact --nocapture
running 1 test
running 1 test
the browser half failed (exit status: 1):
locator.waitFor: Timeout 30000ms exceeded.
Call log:
  - waiting for locator('.publication').getByText('Invite people here', { exact: true }) to be visible
test publication::onboarding_host_controls_child ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 97 filtered out; finished in 31.01s
host controls phase pending: exit status: 101
test publication::onboarding_host_controls_survive_restart ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 97 filtered out; finished in 31.14s
exit 101
```

## Green

The Rust test starts five separate processes over one native data directory.
Each process opens Chromium through the existing authenticated opener, then
reloads the page. The fake resolver returns 429 on pending-publication and
closing phases; success on their explicit retries. It asserts exactly one
resolver mutation per press and none during closed-status inspection. Native
invitation identity stays unchanged across retry, failure and reads, and changes
only on successful close. The issuing participant still stands. The existing
native-publication tests also prove a previously admitted Sam stays and copied
long links stop admitting after close.

The browser asserts address text and actual clipboard contents, computed font
size of at least 20 px, expiry/reachability guidance, manual retry controls,
no ended/closed wording on refusal, and that **every** browser request stays at
the native home origin. Actual output below omits the request arrays:

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs-node --test browser publication::onboarding_host_controls_survive_restart -- --exact --nocapture
host controls pending:
  reloaded: joining is temporarily unavailable; retry in a minute from this computer. Your invitation is kept for retry.Retry joiningClose joining
test publication::onboarding_host_controls_child ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 2.18s
host controls open:
  reloaded: http://127.0.0.1:63357/acorn-basil-cabin-daisy-eagle Copy the addressEnds 11/5/2026, 1:55:38 AM. This computer must remain reachable. A new invitation or device link ends this one; everyone already in stays.Close joining
test publication::onboarding_host_controls_child ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 2.16s
host controls closing:
  reloaded: http://127.0.0.1:63357/acorn-basil-cabin-daisy-eagle Copy the addressjoining is temporarily unavailable; retry in a minute from this computer. Closing has not been confirmed.Retry closing
test publication::onboarding_host_controls_child ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 2.22s
host controls closed:
  reloaded: Joining is closedInvite people here
test publication::onboarding_host_controls_child ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 2.13s
host controls inspect:
  reloaded: Joining is closedInvite people here
test publication::onboarding_host_controls_child ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 2.11s
test publication::onboarding_host_controls_survive_restart ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 11.43s
exit 0

$ lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs-node --test browser the_home_page_makes_a_link_to_invite_someone -- --nocapture
test the_home_page_makes_a_link_to_invite_someone::chromium ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 7.54s
exit 0

$ node scripts/types.mjs
types: 0 errors in 129 files
$ node scripts/typed-lint.mjs
typed lint, jsconfig.json: 125 files, exit 0
typed lint, jsconfig.workers.json: 3 files, exit 0
$ python3 scripts/lean_check.py
crates/svrngs-core     3486 / 3494 lines
.                     19748 / 19833 lines
js                     7931 / 8090 lines
swift                     0 / 1 lines
$ cargo clippy --quiet -p svrngs-node --test browser -- -D warnings
exit 0 (no output)
```

## Full closing gate

The loop ran the single closing gate against implementation `96f40310` and
resumed this unit with exit **0**, after **1,290 seconds**. The complete output
is `ralph/next/onboarding/ctl/log/sessions/onboarding-host-controls-1.await.log`.
Pasted run:

```text
$ lockf -k /tmp/svrngs-gate.lock scripts/check.sh
note       open files: 256, raised to 10240
note       kept awake while this runs (caffeinate, pid 96876)
passed     principles pinned (0 s)
passed     lean and RUST_ARCH (0 s)
passed     trust floor (1 s)
passed     advisories (1 s)
passed     app nouns gate (0 s)
passed     app nouns (0 s)
passed     lint (0 s)
passed     format (0 s)
passed     page laws (17 s)
passed     rustfmt (0 s)
passed     clippy (11 s)
passed     tests (550 s)
passed     canaries (8 s)
passed     ceiling canary (34 s)
           CEILING idle round at 25 x 10,000: 0.0267 s of processor, the median of [0.1866140000000005, 0.026660999999998936, 0.022576999999998293], against 0.1 s; the last round's account {"acts_moved":0,"asleep":0,"cpu_s":0.022576999999998293,"dialled":24,"folds":{"digest":{"kept":96,"rebuilt":0},"offer":{"kept":48,"rebuilt":0},"register":{"kept":27,"rebuilt":0},"write_index":{"kept":126,"rebuilt":0}},"reached":24,"refused":0,"wall_s":0.003233208}
passed     web rustfmt and clippy (8 s)
passed     browser (159 s)
passed     types (1 s)
passed     typed lint (1 s)
passed     joining in a browser (44 s)
passed     pages (62 s)
passed     scenes (241 s)
passed     scene doc (chromium)
passed     scene events (chromium)
passed     scene give (chromium)
passed     scene films (chromium)
passed     scene give_move (chromium)
passed     scene give_gone (chromium)
passed     scene grapevine (chromium)
passed     scene house_rules (chromium)
passed     scene money (chromium)
passed     scene life (chromium)
passed     scene pantry (chromium)
passed     scene ports (chromium)
passed     scene redial (alone)
passed     scene night (chromium)
passed     conformance canary (111 s)
           computer  /-absolute paths                   passed
           computer  64 MiB read by Range               passed
           computer  64 MiB uploaded, its length told   passed
           computer  Host check, absolute redirect      passed
           computer  SharedArrayBuffer under COOP/COEP  passed
           computer  WebSocket echo, OPEN read          passed
           computer  cookie sign-in, read unreloaded    passed
           computer  its own service worker             passed
           computer  local storage                      passed
           computer  post, Origin checked               passed
           computer  seeks back to back                 passed
           computer  seeks back to back                 passed, a rate
           wildcard  /-absolute paths                   passed
           wildcard  64 MiB read by Range               passed
           wildcard  64 MiB uploaded, its length told   passed
           wildcard  Host check, absolute redirect      passed
           wildcard  SharedArrayBuffer under COOP/COEP  passed
           wildcard  WebSocket echo, OPEN read          passed
           wildcard  cookie sign-in, read unreloaded    passed
           wildcard  its own service worker             passed
           wildcard  local storage                      passed
           wildcard  post, Origin checked               passed
           wildcard  seeks back to back                 passed
           wildcard  seeks back to back                 passed, a rate
rate       WebKit seeks back to back (u5-11): way, lend; through the lend and straight at the canary, the ms the last took to play again and its verdict
           computer	sam	392	passed	3600	passed
           computer	bo	410	passed	4152	passed
           wildcard	sam	384	passed	3719	passed
           wildcard	bo	385	passed	20380	failed
           of 492 here since 2026-10-22T17:12:29Z: 2 missed 5 s through a lend, 80 straight; 326 ended at a load above 12 cores
passed     app laws (41 s)
note       took 1290 s in all
exit 0
```

The rate observation includes one straight-canary miss (20,380 ms); every
through-lend observation passed. The gate reports that rate separately from
its passing conformance checks. This local gate does not judge the real-laptop
welcome scene in ONBOARDING §6.
