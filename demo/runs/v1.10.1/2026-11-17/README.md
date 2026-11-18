# V1.10.1 release, 17 November 2026

Released at the operator's word, 2026-11-17: "Commit, push, release,
deploy please". A security release: the page and downloads at
`group.svrnme.sh`, from the commit tagged `v1.10.1`. The resolver is not
deployed: `git diff v1.10..HEAD -- resolver` is empty, so
`join.svrnme.sh` keeps V1.10's.

## Scope and source evidence

The release before this one is V1.10 at 18d3b04e; its readback is
`../../v1.10/2026-11-16/README.md`. Before this run, `group.svrnme.sh` was
serving V1.10's page: at 2026-11-18 00:17 UTC its `sw.js` named version
`660baf7db628`, V1.10's own, and its `lend.js` came with the whole site's
`frame-ancestors 'none'` and no other policy (`page-before.txt`).

Since `v1.10`, the shipped files that changed are, on a computer,
`svrngs-node/src/home.rs` and `session.rs` and the session scripts
`home/session.js` (new), `home/session-worker.js` and
`home/session-client.js`, which `session.rs` serves from the binary
(`include_str!`, `session.rs:423`); and on the phone's page `web/lend.js`,
`web/shim.js`, `web/store.js` and `web/vercel.json`, with
`scripts/build-web.sh` now copying `home/session.js` into the site beside
`lend.js`. No crate is added: `git diff v1.10..HEAD -- Cargo.lock
web/Cargo.lock` is empty. All other changes are tests, docs, hooks, the
trust check and records.

`git diff v1.10..HEAD -- scripts/trust_floor.txt` is not empty. Four
commits change the floor, each with its `Trust floor:` line:

- 9f237d3c: "pins what d539989d already sends, the strictest opener
  policy, so no app that opens the home page, or that it opens, holds a
  handle on it; no widening."
- 5078e98e: "pins the two new worker policies (session-scripts,
  lend-worker); the lend worker's `wasm-unsafe-eval` and `wss: https:` are
  the group page's own, for svrngs's iroh wasm and the relays a pass
  names. Nothing widens: both narrow what ran with no policy before. The
  head now says what the floor promises of workers."
- 2fae8287: "the head now says an app's worker runs in a realm of its own
  under the policy its pages get, never beside svrngs's; no policy or
  listener changes, and nothing widens."
- 3b836125: "the head now says an app's worker answers only its own
  page's subresources and every document and worker comes from the lend
  under svrngs's policy; no policy, listener or gate changes, and this
  narrows what 2fae8287 allowed."

5ea79897 changes how a floor change is committed (`.githooks/commit-msg`,
`scripts/trust_check.py`), not the floor.

Which releases ran an app's worker inside svrngs's: on a phone, lend.js
`appWorker` came with e502de99. `git tag --contains e502de99` names
`v1.1.1` first, but V1 and V1.1 have no tags; the commits they were built
from, de8923f7 and a555a225 (`RELEASES.md`), each contain it (`git
merge-base --is-ancestor`) and each one's `web/lend.js` holds `function
appWorker`. So phones from V1. On a computer, the session worker's
`new Function` host came with dbf83f5a; `git tag --contains dbf83f5a`
names `v1.4` first, and `v1.3.1` does not contain it. So computers from
V1.4. Both are gone at `v1.10.1`: `appWorker` is in no line of
`web/lend.js`, and `new Function` in none of `home/session-worker.js`
(5078e98e).

| Release note | Source or run |
|---|---|
| Before, svrngs ran an app's service worker inside its own, with no rules; there it could use what svrngs holds for that app's address, and load and run code from anywhere | 5078e98e's body ("an app's code reached `eval`, any wasm and any address, and the builtins svrngs's own code calls"); 2fae8287's plant, the app's script run in svrngs's realm again: `eval` and wasm ran, signatures and proved requests seen, the open's key read (svrn-docs app-worker-realm, `data/after-option-b-plant.json`) |
| Phones from V1, computers from V1.4 | the paragraph above |
| The app's worker runs in a worker of its own, which its page starts, under the same rules as the app's pages | 2fae8287, `home/session.js` `registered` (a dedicated `Worker` at the app's script), `realmScript` and `preluded` (the script as the lend answered it, its headers and so its policy kept); conformance canary "its own service worker", computer and wildcard, Chromium and WebKit (`gate-9297a8e9.txt`) |
| It answers only what that page loads: scripts, styles, images, media and data | 3b836125, `home/session.js` `SUBRESOURCES` and `realmAnswer` (the realm of the asking page's client id alone) |
| Every page, frame and worker the app opens comes from the computer that lends it, as svrngs serves it, never from the app's worker | 3b836125; conformance canary "its worker answers no page or worker" (a page, a frame and a nested worker its worker would answer under no policy each came from the lend, `eval` refused), computer and wildcard, both engines (`gate-9297a8e9.txt`) |
| svrngs's own worker runs only svrngs's code, under rules of its own | 5078e98e, `session.rs` `POLICY` (`default-src 'none'; script-src 'self'; connect-src 'self'`) and `web/vercel.json`'s `/lend.js` rule, both pinned in `scripts/trust_floor.txt`; conformance canary "svrngs's worker refuses eval" (`gate-9297a8e9.txt`); this readback's live lend.js policy |
| `register` works; the app's worker runs while its page is open | `home/session.js` `registered`, called by `session-client.js:108` and `web/shim.js:234`; the dedicated worker is its page's, and svrngs's worker keeps its realm by that page's client id (`realms`; "A realm lasts while its page is open", `home/session.js:123`) |
| Opening any of the app's pages is answered by the computer that lends it, never by its worker | `SUBRESOURCES` holds no `document`, `frame`, `iframe` or `worker` destination; conformance canary "a cold navigation, the lend's" and "its worker answers no page or worker" (`gate-9297a8e9.txt`) |
| In its worker, `clients` and `skipWaiting` do nothing | `home/session.js` `prelude`: `clients.claim` and `skipWaiting` resolve doing nothing, `clients.matchAll` finds no page |
| On a computer, a page's request for data that a lent service redirects to another address fails | 5078e98e: the session worker's `connect-src 'self'` (`session.rs` `POLICY`), which a fetch's every redirect is checked against. `session-worker.js` `authenticated` sends a no-cors request as same-origin, so an image, style or classic script redirected away already failed in V1.10 (`git show v1.10:crates/svrngs-node/src/home/session-worker.js`, line 45); a request for data is the case newly refused. Not run by a test of its own |
| Following it could send svrngs's proof for that app on to that address | `session.rs:33`: a page's fetch "follows a redirect itself, sending the same headers on", and the proof is the `x-svrngs-browser` header (`session.rs:21`) |
| A page that redirects still opens | a navigation's redirect is the browser's to follow, not the worker's fetch's, so the worker's policy does not reach it; no test redirects a page to another address |
| An app that your home page opens, or that opens it, gets no handle on it, and in Safari's engine doesn't share its process | d539989d, `home.rs` `HEADS` (`Cross-Origin-Opener-Policy: same-origin` on every answer); `tests/home.rs` `the_home_page_answers_only_itself_and_takes_only_the_person_s_choices`, pinned as a gate by 9f237d3c. The process was measured in WebKit 26.4, the engine Safari is built on, not in Safari (svrn-docs `wasm-guard-spike/data/step6-proc-webkit.json`) |
| Groups, lends and the record work as in V1.10 | `git diff v1.10..HEAD --stat -- '*.rs'`: `home.rs` (the opener policy, and 3a8f7d35's two branches, behavior kept) and `session.rs` (the scripts' policy, and 7a5c676d's three shorter forms, behavior kept), with tests. No route, act kind or record format changes |
| Press **Update and restart** on your home page, or run `svrngs update` and then `svrngs restart` | `home/page/update-line.js:50` (the button's words); `crates/svrngs/src/cli.rs:757` ("Updated. `svrngs restart` uses it.") |
| Your phone's page updates itself | `web/sw.js` sends a navigation to the network first (verify.py `network_first`, read on the live `sw.js`) |
| Nothing else changes | the other commits since `v1.10` change tests, docs, hooks and records: e9572248 (an advisory recorded), 5ea79897 (a floor change judged by its commit's message), 8285d6fd (a comment), 35b6a19b and 31093b29 (the realm's page side in one home, and handed again when svrngs's worker starts afresh), 9297a8e9 (a test in a binary of its own), 72334265 (a test's count), ff8fbf29 (`docs/PLAN.md`), d29dc872 (V1.10's record) |

## Validation before packaging

`scripts/check.sh` ran whole on the Mac at 9297a8e9 with the tree clean,
under the gate lock: exit 1 in 1,423 s, 44 of its 45 steps passed, none
could-not-judge or never-ran (`gate-9297a8e9.txt`). The browser step failed one test,
`keep_going::a_new_release_replaces_a_lend_s_saved_pages` (chromium,
`keep_going.rs:206`, left 10, right 9): 35b6a19b added `session.js` to
`web/store.js` `SAVED`, and the test still named nine saved pieces.
72334265 fixed the test, `web/tests/reopen.mjs` `PIECES` naming
`session.js`. Then the browser step was run again by the session that ran
the gate, `cargo test --quiet --workspace --test browser -- --skip
playback::`: 36 passed, 0 failed, 3 ignored. That run's output was not
kept here. ff8fbf29 changes only `docs/PLAN.md`. The whole check was not
run again.

In the conformance canary, "its worker, after svrngs's restarts" was
could-not-judge in WebKit, on a computer and with the wildcard: the
harness can't stop WebKit's worker. It passed in Chromium.

The release commit, tagged `v1.10.1`, changes no runtime file: only
`RELEASES.md` and this record.

Not run on Linux: V1.8.2's whole Linux run
(`../../v1.8.2/2026-11-15/gate-3da19985-linux.txt`) is still the last one.

## Publication

PUBLICATION

## Readback

READBACK

## Not verified, not done

NOT_VERIFIED
