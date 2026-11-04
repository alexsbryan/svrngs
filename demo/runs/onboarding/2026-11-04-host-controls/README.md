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

The full closing gate is owed. The session hands its single run to the loop
with `ralph-result await`, under `lockf -k /tmp/svrngs-gate.lock`, because the
previous row's recorded full gate took 19m22s. Its resumed session must record
the actual result here, handle any failure and commit before reporting done.
