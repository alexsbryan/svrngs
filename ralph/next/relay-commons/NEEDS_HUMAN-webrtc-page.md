# webrtc-page: the JavaScript is 16 lines past its cap; the page side is otherwise done

> **Director's review, 2026-11-19:** in the patch's `web/shim.js`, `setTimeout(done, 3000)` never ends the wait: `done` resolves only once `iceGatheringState` is `complete`, so a gathering that stalls holds the attempt forever. Make the timeout resolve unconditionally (`setTimeout(gathered, 3000)`), and plant it: a gathering that never completes still sends its offer after 3 s.

> **Set aside 2026-11-19 (director):** the page side is in `ralph/next/relay-commons/ctl/webrtc-page.patch` (a copy in the director's scratchpad), out of the worktree so self-host's commits are not held by its JavaScript. `git apply` it first.

Unit: `webrtc-page` (ralph/next/relay-commons/STATE.md:30). Written
2026-11-19 by the unit's session. One operator decision is open: the
JavaScript cap. The row's browser check is written and not yet run
(§2); every other check this session ran is green.

## 1. The blocker: JavaScript is +56 against 40 free

The page side is implemented, and its focused checks are green — but its
JavaScript lands at **+56** (`web/lend.js` +31, `web/shim.js` +25)
against the cap's **40 free**: `scripts/lean_check.py` prints 8,667 of
the 8,651 the cap allows, and the commit hook's lean step says BROKEN on
every commit while it stands (tried; nothing commits).

**The economizing pass was made, and its cuts do not hold** (why the
rest cannot be found in the code there is):

- `partAsked` and `keepCookies` — each a step of one call site — were
  inlined into it (4 lines), and `web/tests/lend-worker.test.mjs` fails
  without them: it cuts those steps from `lend.js`'s text, and its own
  words are `AssertionError [ERR_ASSERTION]: declares partAsked`
  (session log `ctl/log/sessions/webrtc-page-1.out`).
- Every other single-use step's inlining was tried and
  `.oxlintrc.json`'s complexity law refuses it, in the lint's own words:
  `eslint(complexity): function has a complexity of 11. Maximum allowed
  is 10` (`bounded`, after `partAt`'s inlining) and
  `eslint(complexity): async function `lent` has a complexity of 14.
  Maximum allowed is 10` (after `unheldAt`, `actsPosted` and `opening`
  moved into it) — the same log. The steps are there because the law
  asks for them; the relay-survey's pass found the same (`SURVEY.md`,
  "The economizing pass", the `web/lend.js` entry).
- Nothing else in the counted JavaScript is dead: no unused top-level
  step (checked over every counted `.js`/`.mjs`), no 5-line duplicate
  block (a window hash over them), no unowned area to cut.

**The smallest ready cap diff** — `scripts/lean_check.py`'s `BUDGET`,
which is the operator's, never a worker's to edit:

```diff
-    "js": 8652,
+    "js": 8668,
```

## 2. What is done, and what is owed

Committed (the node side; each green, its plant in its body):

- `536cd2fc` webrtc-page: the offer is a pass's, checked as a lend's
  request is. The row's first step: an offer is answered only to a
  phone currently authorized for a lend this computer lends, which is
  what a pass is. `door::holder_of_offer` (the door's own check,
  `holder_of` reused), `webrtc::router` takes the door, `admit_dialer`
  moves `Alpn::WebRtc` beside `Alpn::Lend` (any dialer may dial; the
  route asks the pass). Planted: the pass check removed, and the
  test's bare offer was answered — "an offer with no pass must be
  refused: {"type":"answer","sdp":"v=0…"}" (`webrtc_path.rs:349`).
- `2664de4b`, `07b52fcd`: the path timing and the offer's route move to
  `webrtc/paths.rs`, the file both ends compile, behaviour kept.

Uncommitted, in the worktree and as
`ralph/next/relay-commons/ctl/webrtc-page.patch` (729 lines; `git apply`
it if the worktree is ever reset):

- `web/src/lib.rs` +214: the wire (the node's file by `#[path]`), the
  free `data_push`, `direct_start`/`Direct` (the fresh data endpoint,
  its two dials, `signal`, `paths`), `Endpoint::dial_webrtc`, and
  `Direct::go` (the attempt's protocol, so the worker's JavaScript
  stays thin);
- `web/lend.js` +31, `web/shim.js` +25: the direct attempt, and the
  window's `RTCPeerConnection` (the spike's Q4 port: the window holds
  the channel and passes each datagram over a `MessageChannel`);
- `web/Cargo.toml`, `web/Cargo.lock`: iroh's
  `unstable-custom-transports`, and the wire's `iroh-base`, `n0-watcher`
  and `noq-udp` at the node's pins;
- `docs/DESIGN.md` §3 in place: the lend's endpoint offers the path,
  and the offer is answered only to a live pass's holder;
- the browser tier, **written and not yet run**:
  `crates/svrngs-node/tests/browser/webrtc_page.rs` (both engines: the
  film whole on `webrtc*` with the relay beside it; the withholding
  plant relayed) and `web/tests/runtime.mjs`'s `direct`/`withhold`
  args. It compiles (`cargo test -p svrngs-node --test browser
  --no-run`).

Green, this session (toolbox, CARGO_INCREMENTAL=0): `scripts/build-web.sh
--lint` clean; `node web/tests/run.mjs unit` 359 passed; `cargo test -p
svrngs-node --lib` 189 passed; `cargo test -p svrngs-node --test
webrtc_path` 1 passed; oxlint and oxfmt clean.

**Owed:** the browser tier's run in `svrngs-gate` (the film on
`webrtc*`, the withholding plant, the channel closed mid-film's 4 s
fallback, the CSP unchanged) and the fixing of whatever it finds. The
feature has not yet run in a browser; the channel-close plant is not
yet written.

## 3. What the next session does

1. When the cap holds (raised, or the row cut to fit by the operator's
   word): commit the page side, its pass's cuts apart if any (AGENTS.md
   2), then run the browser tier in `svrngs-gate` and repair what it
   finds. The row's own checks close it; the audit is after.
2. The offer's gate is this row's design decision (`536cd2fc`): a pass
   holder's offer, because the group page's endpoint is closed when the
   lend opens — the person's key never enters a lend's origin
   (`web/open.js`'s header; DESIGN §3, "Joining" call 3). If the
   operator would rather the offer stay a member's, that is a different
   page side, and it is the operator's call.

## 4. The landing commit's `Size:` paragraph, ready

    Size: +214 browser Rust (web/src/lib.rs; the row's estimate was
    +120) and +56 JavaScript (web/lend.js +31, web/shim.js +25) of the
    cap's 40 free; tests unbudgeted (the browser tier's files). The
    cap's smallest ready diff is `BUDGET`'s js 8652 → 8668
    (NEEDS_HUMAN-webrtc-page.md §1). The economizing pass's cuts do not
    hold: the linter's complexity ≤ 10 and web/tests/lend-worker.test.mjs's
    cutting of the single-use steps' text both refuse the inlinings
    (§1 has the words); what remains traces the row's three parts — the
    window's channel, the worker's fresh endpoint and its two dials, and
    the offer through the lend's own dial.

