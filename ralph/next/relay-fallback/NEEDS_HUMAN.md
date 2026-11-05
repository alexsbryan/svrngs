# fallback-connect: the operator's cap decision

One line: the landing is complete and over the repository cap `.` by 80
lines (19,912 against 19,833) and the queue's test allowance by 203 lines
(353 against 150); the smallest ready diff raises `.` to 19,913 (one past
where it lands) and the test allowance to 353, or the operator decides
otherwise.

CHARTER.md: "If measured necessary growth does not fit the current caps,
stop with the post-pass counts and the smallest ready cap diff for the
operator." This is that stop. No cap was edited by this session.

## What landed, and what proves it

Commits f8a57035..38d4bc0c (each body carries its own red/green run):

- b6db94fd — `Meeting::decide` inverted (crates/svrngs-core/src/relay.rs:150):
  the keeper meets at the build's fallback while it answers, its groups'
  own relays when it does not; `Host::check`
  (crates/svrngs-node/src/tiers.rs:127) asks the fallback. Red: the two
  `decide` arms swapped (core test failed; `relay_tiers` failed "not homed
  at ... within 56 s"). Green: 7 core relay tests; both relay_tiers tests
  (24.1 s); relay_fallback (20.5 s); `node --test web/tests/fallback.test.mjs`
  1 pass.
- 02ac74b8 — `bind_for` (web/src/lib.rs:131) binds every relay the token
  carries that the page's policy takes, `reach` (:241) races them and
  returns only a connection to the key dialled, `relays_of` (:449) hands a
  pass's holder the candidates (`holder_endpoint` :434, `hand.js`/
  `authority.js` `relays`, `lend.js`). Green: join_page 2 passed (55.3 s);
  `scripts/build-web.sh --lint`; oxlint; oxfmt --check; `scripts/types.mjs`
  0 errors in 130 files; `scripts/typed-lint.mjs` clean; `cargo check
  --target wasm32-unknown-unknown`.
- a44164cd — the published-policy browser scene
  (crates/svrngs/tests/join_page.rs): the page served with the built meta
  alone (web/tests/page.mjs `serve` `policy: 'production'`), an HTTP
  preferred route skipped, the public fallback joins once and opens the
  `shelf` lend, no line names the HTTP relay and no console line is a
  policy refusal. Red first on the named plant (`this_page_takes` always
  true: the lend did not open). Green: 29.6 s.
- adc262fe — a live relay the keeper is not at does not win; the browser
  tries it and is let in through the keeper's relay; the reopened page and
  the page alone are still in. Red first on the named plant (`reach`
  dialling `routes.first()` alone: the browser half failed). Green: all
  four join_page tests, 34.7 s.
- 38d4bc0c — the tests' shared setup and driver, behaviour kept; all four
  pass (28.3 s).

## The measured counts

`python3 scripts/lean_check.py`, exit 1, at 38d4bc0c:

```
crates/svrngs-core     3491 / 3494 lines
.                     19912 / 19833 lines
js                     8187 / 8188 lines
swift                     0 / 1 lines
BROKEN  . has 19912 lines of code, budget 19833: raised only by the operator, with the reason in the commit
```

Growth over the baseline f8a57035, counted lines (nonblank, noncomment):
core +5 (`crates/svrngs-core/src/relay.rs`), browser Rust +75
(`web/src/lib.rs`), JS +0, every other product file +0. The queue's slices
hold (core ≤10; browser Rust ≤80; JS ≤30; native+core ≤40; total +80 ≤150).
Only the repository cap `.` does not fit the landing.

Test code added over the baseline: 353 lines (relay_fallback.rs +124,
join_page.rs +217, core relay_tests.rs +7, relay_tiers.rs +3,
page.mjs +2) against the queue's 150.

## The ready diff

`scripts/lean_check.py`, the one home of the cap (#197). Nothing else
changes; the added comment records the operator's raise and the pass:

```diff
@@ -452,6 +452,10 @@
 # Separate rungs let the parallel backend/UI lanes adjust only their own cap.
+# The relay-fallback queue's landing, the operator's decision (2026-11-05):
+# the keeper meets at the advertised public fallback while it answers, the
+# page takes every route its policy allows and races them; +80 over the
+# cap, raised to one past where it lands: 19,913.
 BUDGET = {
     "crates/svrngs-core": 3494,
 
-    ".": 19833,
+    ".": 19913,
 
     "js": 8188,
```

The queue's test allowance is prose in CHARTER.md/STATE.md, not a machine
cap: its raise (150 → 353) is the operator's word, and this file records
the measured number for it.

## What is not done, and is not this stop's to do

- The full `scripts/check.sh` is REVIEW-audit-relay-fallback's; this unit
  ran the focused checks named in its row (pasted in the commit bodies).
- The real phone, the Pi, and the public relays were not run; the phone
  row is the operator's.
- The row's remaining checks not yet landed as tests: a *stalled* (not
  incompatible) eligible primary's no-delay bound, all-routes-down cleanup
  with the bounded failure said, and native local sync with public reach
  cut (relay_tiers measures the move; a sync scene is not written). They
  are the next session's, after the operator's decision, if the allowance
  is raised.
