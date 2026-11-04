# Native attendee arrival — 2026-11-04

Assigned row: `onboarding-native-arrival`. Base `4e8caa6f`. Local tests only;
no publication, deployment, push or provider request. The preexisting V1.6
`publish.json` and `publish.log` were inspected and remain untouched/untracked.
Their `state: running` is an old artifact, not a run started by this session.

## Reuse

The current inventory is `web/page/{enter,join,name-form,computer,computer-line}.js`,
the resolver's lookup/JSON and error policy, `web/install.sh` and its existing
`crates/svrngs/tests/install.rs` service-manager stubs, `/v1/open {at}` and
`home/page/{home-page,group,group-folder}.js`. `redeem.rs::serve_ask` already
owns the invitation/seed query and `Host::own_person` owns the issuer's name.
Native metadata wraps those, without changing the ordinary `/ask` response,
adding a membership rule, or adding a folder API. The upstream survey remains
ONBOARDING §4: this extends the repository's existing surfaces.

## Red before the change

Pasted output excerpts (the ordinary fixture tokens/temporary directories are
not live invitations):

```text
$ lockf -k /tmp/svrngs-gate.lock node --test web/tests/pages/native-arrival.test.mjs
not ok 1 - a typed short address redirects only to the canonical page in native mode
    200 !== 303
not ok 2 - a resolver limit renders temporary unavailability and manual retry, never ended
    The input did not match the regular expression /text\/html/. Input:
    'application/json'
not ok 3 - native arrival asks the name once and offers a literal shell command without browser admission
    + 'failed'
    - 'join'
# tests 3
# pass 0
# fail 3
exit 1

$ cargo test -p svrngs --test install a_refused_join_restarts_the_existing_installation -- --nocapture
Stopped svrngs, to update it.
svrngs: `http://127.0.0.1:52620/#not-a-link` is not a whole link; copy all of it, to the end
assertion `left == right` failed: restarted after refusal: launchctl bootstrap gui/502 .../sh.svrnme.svrngs.plist
launchctl bootout gui/502/sh.svrnme.svrngs
  left: 1
 right: 2
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 6.14s
exit 101

$ cargo test -p svrngs --test install native_arrival_retries_one_identity_and_opens_the_exact_group -- --nocapture
svrngs: unknown argument `--at`
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 5.84s
exit 101
```

The literal-shell law passes the actual generated command through `/bin/sh`
with inert curl/sh functions. Its chosen name is
`Sam O'Brien; $(touch NEVER) ` plus literal backticks around `uname` and
literal `"friend"`. It asserts the exact argv, including that whole name as
one argument, a canonical download/link and `/#group-abcd` destination.

## Browser instrument: watched admitting the unwanted second identity

The optional `NATIVE_ARRIVAL_PLANT=browser-identity` changes only the test's
served canonical-page `join.js`: the native name handoff falls through to
ordinary browser redemption. The test checks IndexedDB and the actual host
journal, rather than trusting a page-authored status.

```text
$ NATIVE_ARRIVAL_PLANT=browser-identity lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs-node --test browser native_arrival_names_once_and_shows_the_actual_folder -- --nocapture
AssertionError [ERR_ASSERTION]: no browser membership persisted
+ [ { ... person: "Bo O'Brien", ... } ]
- []
PAGE: named: .../naming/... -> .../redeeming/..., running admit
HTTP: POST /redeem answered 200
STORE: the sealing key absent: making one
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 98 filtered out; finished in 14.24s
exit 101
```

An earlier plant was invalid: it replaced the wrong conditional and also
intercepted the authenticated home module, producing a local 403. That run
is not the red witness. The corrected plant is restricted to the canonical
page's origin and changes the name handoff itself, as the trace above proves.

## Green

```text
$ lockf -k /tmp/svrngs-gate.lock node --test web/tests/pages/native-arrival.test.mjs
# tests 3
# pass 3
# fail 0
exit 0

$ cargo test -p svrngs --test install a_refused_join_restarts_the_existing_installation -- --nocapture
Stopped svrngs, to update it.
Restarting svrngs after the interrupted installation. You can retry the same installation line.
test a_refused_join_restarts_the_existing_installation ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 9.92s
exit 0
```

That test exercises both the macOS launchctl and Linux systemctl branches
(system managers are stubs; the installed native binary is real). It checks
a nonzero refusal, a second service start and unchanged identity bytes.

```text
$ cargo test -p svrngs --test install native_arrival_retries_one_identity_and_opens_the_exact_group -- --nocapture
You're in house, as Bo O'Brien.
house's folder is .../svrngs/house: what anyone in house puts there reaches this computer, and what you put there reaches theirs.
You're in house already, on this computer.
test native_arrival_retries_one_identity_and_opens_the_exact_group ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 15.40s
exit 0
```

Both opens are redeemed by their actual same-origin POST, asserting the
returned `at`, not a nonexistent URL query field (an initial instrument
assumption corrected before this green run). The test compares identity
bytes, the saved canonical origin and exactly one `Admit` in the issuer's
journal after both pastes.

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs-node --test browser native_arrival_names_once_and_shows_the_actual_folder -- --nocapture
native arrival: {
  "folder": ".../already-used-name-2",
  "hash": "#group-bdb1e28bf4a78cb7ead9605e3856126d4716c37a9cec3027db7ab6fd2d65a68d",
  "kept": [],
  "line": "curl -fsSL https://group.svrnme.test:53532/install.sh | sh -s -- 'https://group.svrnme.test:53532/#<fixture-token>' --at '/#group-bdb1e28bf4a78cb7ead9605e3856126d4716c37a9cec3027db7ab6fd2d65a68d' --as 'Bo O'\\''Brien'"
}
test native_arrival::native_arrival_names_once_and_shows_the_actual_folder::chromium ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 13.97s
exit 0

$ lockf -k /tmp/svrngs-gate.lock node --test web/tests/pages/*.test.mjs
# tests 45
# pass 45
# fail 0
exit 0

$ node scripts/types.mjs
types: 0 errors in 129 files
$ node scripts/typed-lint.mjs
typed lint, jsconfig.json: 125 files, exit 0
typed lint, jsconfig.workers.json: 3 files, exit 0
```

The browser types a synthetic short address answered by the real resolver
handler with a fixture transport, follows its canonical native-mode redirect,
uses a real local relay/issuer and its `/ask?native=true`, sees
`ada invited you to house`, names once, gets the command, and keeps no browser
group. The issuer's journal remains byte-identical. It then follows a real
authenticated local open to the group section, reads its nondefault folder
from the native answer, and finds **Open this folder** there. The existing
folder-open browser test continues to prove the actual guarded OS opener.
The resolver law injects provider HTTP 429 and reads its HTML 503/retry answer.
This is a simulated technical run, not the real-laptop people scene.

The final browser driver also includes the typed short-address navigation
through the real handler with a fixture transport:

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs-node --test browser native_arrival_names_once_and_shows_the_actual_folder -- --nocapture
  "folder": ".../already-used-name-2",
  "hash": "#group-62ddae42031de15d0f75201d61bb7abb48d88dc93345291d2bd8f917f3347927",
  "kept": [],
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 13.61s
exit 0

$ cargo test -p svrngs --test install
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.86s
exit 0
$ lockf -k /tmp/svrngs-gate.lock node --test web/tests/resolver.test.mjs web/tests/pages/native-arrival.test.mjs
# tests 20
# pass 20
# fail 0
exit 0
$ cargo clippy --quiet --all-targets -- -D warnings && cargo clippy --quiet --workspace --test '*' -- -D warnings
exit 0
$ python3 scripts/lean_check.py && python3 scripts/trust_check.py
crates/svrngs-core     3486 / 3494 lines
.                     19779 / 19833 lines
js                     8028 / 8090 lines
swift                     0 / 1 lines
trust floor: 10 policies, 11 listeners, 9 gates
exit 0
```

## Closing gate

The full gate ran after feature commit `3f29ba16`. The loop resumed this unit
with exit **0**; its log is
`ralph/next/onboarding/ctl/log/sessions/onboarding-native-arrival-1.await.log`.
Pasted closing output (the detailed conformance rows remain in that log):

```text
$ lockf -k /tmp/svrngs-gate.lock scripts/check.sh
passed     principles pinned (0 s)
passed     lean and RUST_ARCH (0 s)
passed     trust floor (0 s)
passed     advisories (2 s)
passed     app nouns gate (0 s)
passed     app nouns (0 s)
passed     lint (1 s)
passed     format (0 s)
passed     page laws (16 s)
passed     rustfmt (1 s)
passed     clippy (0 s)
passed     tests (575 s)
passed     canaries (13 s)
passed     ceiling canary (34 s)
passed     web rustfmt and clippy (9 s)
passed     browser (161 s)
passed     types (1 s)
passed     typed lint (1 s)
passed     joining in a browser (45 s)
passed     pages (65 s)
passed     scenes (236 s)
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
passed     conformance canary (105 s)
passed     app laws (41 s)
note       took 1306 s in all
exit 0
```

The conformance log also reports its existing comparative WebKit rate sample:
both wildcard straight-to-canary samples missed 5 seconds, while all four
through-lend samples passed. The gate's verdict is passed; these rate samples
are not a native-arrival people scene. The row's marker is left to the loop.
