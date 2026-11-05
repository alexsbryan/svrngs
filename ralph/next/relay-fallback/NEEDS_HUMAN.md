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

## Director's verification (2026-11-05)

Re-measured in the main tree at 7efc3e55, tree clean:

- `python3 scripts/lean_check.py` exit 1: `.` 19,912/19,833, core
  3,491/3,494, js 8,187/8,188, swift 0/1. The baseline f8a57035 re-run in
  a worktree: `.` 19,832/19,833, core 3,486/3,494, js 8,187/8,188,
  swift 0/1, exit 0 — so the repository cap had one line of headroom
  before this queue and the landing's +80 is what breaks it.
- Counted deltas f8a57035..HEAD, by the lean rule: product +80 exactly —
  `crates/svrngs-core/src/relay.rs` +5, `web/src/lib.rs` +75, every other
  product file +0; test files +353 (relay_fallback.rs +124, join_page.rs
  +217, relay_tests.rs +7, relay_tiers.rs +3, page.mjs +2).
- The ready diff applied to a scratch copy of `scripts/lean_check.py`
  (not the repo file) makes the check exit 0: `.` 19,912/19,913, core
  3,491/3,494, js 8,187/8,188, swift 0/1.
- Focused checks re-run green: core relay 7 passed; relay_tiers 2
  (24.1 s); relay_fallback 1 (20.4 s); join_page 4 (31.5 s);
  fallback.test.mjs 1 pass; oxlint, oxfmt --check, scripts/types.mjs
  (0 errors in 130 files), scripts/typed-lint.mjs clean.
- The survey's caller lists spot-check: `relay_of` at dial.js:121,
  join.js:270, visit.js:202; `relays_of` at hand.js:80, visit.js:149;
  `holder_endpoint` at lend.js:280 and
  apps/taste/check/other/other.js:85.

## The decision, for the operator

The charter reserves the cap to the operator — "Do not edit BUDGET or the
trust floor without the required operator decision"; the test allowance
is the operator's word in CHARTER.md §Budget and no script enforces it.
Neither director nor worker may move either, and no worker can finish the
unit inside the current caps: the test overage (353 against 150) stands
whatever product lines are found, and the row's remaining checks would
add more tests. The fork goes to the operator with this package.

Options and their costs:

- A (recommended): approve as asked — `.` 19,833 → 19,913 (one past the
  landing) and the queue's test allowance 150 → 353. Cost: the repository
  cap rises +80, all of it the authorized feature, whose focused checks
  are green and whose landing is verified above.
- B: order a further economizing pass over the whole repository before
  any raise. Cost: another worker session; the onboarding queue's
  instruments (public/dead-code, test-cfg, line-window, page-structure)
  found no supported cut days ago, and no pass can cut the test overage
  the row's own checks need. Likely re-lands at the same ask.
- C: hold the cap and drop or reshape the landing. Cost: the authorized
  demo — a fresh browser joining through the public fallback under the
  production CSP — does not ship.

Recommendation: A. After the word: apply the cap diff (and record the
test allowance), then the row's remaining checks (a stalled eligible
primary's no-delay bound; all-routes-down cleanup and bounded failure;
native local sync with public reach cut), then REVIEW runs the unchanged
full `scripts/check.sh`.

## Operator decision (2026-11-05): A

Approved as asked. `.` 19,833 -> 19,913 applied in `scripts/lean_check.py`
(lean_check exit 0: 19,912/19,913). The queue's test allowance is 450
(CHARTER.md §Budget, STATE.md), covering the measured 353 and the row's
remaining checks: a stalled eligible primary's no-delay bound; all-routes-down
cleanup with the bounded failure said; native local sync with public reach cut.
No further cap raise is authorized; a landing past 19,913 or 450 stops again.

## Operator decision (2026-11-05): repair the review findings with a strong model

An independent review of the product delta f8a57035..HEAD found defects the
final audit passed; the seat confirmed three by reading (tiers.rs:242-244 with
relay.rs:155-158; tiers.rs:139-140 with relay.rs:169-171; lib.rs:206-209 with
dial.js:110 and lend.js:248,290). The operator chose: a strong-model worker
repairs them in this worktree, one finding per commit with red/green tests, the
seat reviews each diff, then the unchanged full scripts/check.sh runs. The Flash
loop stays parked on stock-relay until then. Test allowance for the repair:
+150 lines beyond the audited 450. Product lines come from the queue's
remaining 71, but the repository cap `.` has 2 lines of headroom (19,911 of
19,913): any raise returns to the operator as a measured one-line diff.

## Operator decision (2026-11-05): raise to fit, add the backoff

Repair commits 7a35fe1a..6d8cfd50 (strong-model worker, seat-reviewed). Caps:
`.` 19,913 -> 19,946 and js 8,188 -> 8,212, each one past where the repair
lands (lean_check exit 0). Repair test lines: +181 net (+200/-19) since
6359387a, the measured total, against the +150 first allowed.
