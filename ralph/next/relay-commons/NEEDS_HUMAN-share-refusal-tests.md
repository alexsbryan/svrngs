# share-refusal: the row's test allowance

**Question:** share-refusal is done and green. Its product is within
allowance: +68 Rust of +70, +38 JS of +40. Its tests are **+325 code lines
against the row's 150**. Raise the row to 325, or name a check to drop?

Your rule (`2dc7ebe4`, PROMPT.addendum.md) makes this yours: "Each row's
own allowance is its ceiling … Past it, after an economizing pass, stop
with the counts and a ready diff."

## The director's pass

The worker asked at +452. The director found duplication the worker's pass
missed and cut it in `a17b5aa2`, test-only, every check kept:

- `tests/relay_share.rs` (181 code lines) copied relay_tiers.rs's `relay`,
  `at`, `homing_by` and `homed`, and the door-over-relay setup, line for
  line. It now lives in relay_tiers.rs, at +64.
- node/tests.rs: two copies of a 25-line wire setup became one `sharing`.

The re-run, in the toolbox:
- `--test relay_tiers`: 11 passed.
- `--lib`: 182 passed.
- clippy and rustfmt clean.

The throttle scene's plant, `Net::ratelimited` returning 0, was watched
red again after the move (`Elapsed`, 33.33 s).

## Counts

Code lines, meaning non-blank and not starting with `//`, measured from
`59c1b5b5`:

    node/tests.rs +78   tests_support.rs +20   door/tests.rs +42
    relay_tiers.rs +64  surface_list_tests.rs +1
    shim.test.mjs +68   lend-worker.test.mjs +35   home.test.mjs +17
    total +325

The queue's tests, by the same measure: 266 before this row, 591 now, of
the 1,200 envelope. The rows still to come allow webrtc-node 200,
webrtc-page 250 and self-host 120, which comes to 1,161. The raise fits the
envelope.

## Options

- **A. Raise to 325 (recommended).** What remains is the row's own named
  checks, one planted law each:
  - the door's predicate and its words;
  - the watcher, both ways;
  - the worker's page;
  - the shim's banner, by fetch and by XHR;
  - the home page's line;
  - the real throttled relay.

  The 150 estimate counted the checks but not their words in three places.
  Cost: 175 test lines over the estimate, within the envelope.
- **B. Hold 150.** That needs about 175 lines cut, which means dropping
  named checks: at least the XHR banner, the home page's line, and one
  direction of the watcher. That weakens the row's pass bar, so the
  director won't choose it.

## Ready diff, for A (STATE.md, the share-refusal row)

```diff
- At most +70 Rust, +40 JS, 150 test lines.
+ At most +70 Rust, +40 JS, 325 test lines.
```

After A, the row closes and the queue moves to `webrtc-node`.
