# share-refusal — the row's test allowance, past it

**The question, one line:** share-refusal's checks are all done and green,
its product is within its allowance (+68 Rust of +70, +38 JS of +40), and
its tests stand at **+452 code lines of the row's 150** — raise the row's
test allowance to 452, or name what to cut?

The operator's decision (2026-11-18; `2dc7ebe4`, the queue's
`PROMPT.addendum.md`): "Each row's own allowance is its ceiling, for
product and tests alike. Past it, after an economizing pass, stop with the
counts and a ready diff (`ralph-result needs-human`). Do not carry on
because the queue's total holds." The CHARTER says the same. This is that
stop.

## State: the unit is complete

`share-refusal` (ralph/next/relay-commons/STATE.md:25), all its checks
green, each planted:

| the row's check | where | plant, watched red |
| --- | --- | --- |
| a relay with a tiny rate throttles a lender | `crates/svrngs-node/tests/relay_share.rs` (1 passed, 7.2 s) | `Net::ratelimited` answering 0: the range after the pull served again, poll timed out, 33.4 s (`8fe3c5b5`) |
| a film's range gets 429 share, and the page shows the words | `door/tests.rs` (429, `x-svrngs-refused: share`, the words), `web/tests/lend-worker.test.mjs`, `web/tests/shim.test.mjs` | a range taken as small; `shareSpent` returning at once; `spent_words` back to the old string (`1b08385c`, `0089b3b1`, `a5850dae`) |
| a small request still passes | `relay_share.rs` (200, `ok`) | as above |
| a direct or own-relay path is never refused | `door/tests.rs` `over_share(false, …)` | `over_share` false for a range (`1b08385c`) |
| a spent share stays spent while the connection holds, clears when it ends | `node/tests.rs` | a minute clearing it; `at_build` dropped from the watcher (`f80942db`) |
| the lender's home page says when | `web/tests/pages/home.test.mjs` | `shareSaid` answering the words whatever `share` says (`b77b6233`) |

Commits since the row's start (`59c1b5b5`): `1b08385c`, `0089b3b1`,
`f80942db`, `1038879a`, `b77b6233`, `8fe3c5b5`, `a5850dae`. The operator's
`be0a6c1c` (a person's words never say "relay") is in `a5850dae`.

Green at `a5850dae`: `cargo test -p svrngs-node --lib` 182 passed; `--test
door` 7; `--test relay_tiers` 10; `--test relay_share` 1; `node
web/tests/run.mjs unit` 358; clippy, rustfmt, oxfmt, oxlint clean.

## The counts, measured

lean_check.py at `a5850dae` (caps in brackets):

    the core (svrngs-core)     3,510   most   3,539
    all code but JavaScript   21,600   most  23,101
    JavaScript                 8,594   most   8,651

- the row's product Rust: **+68 of +70** (start 21,532; the operator's
  removal of the timer and `Net::remade` cut 27, `f80942db`);
- the row's JavaScript: **+38 of +40** (start 8,556; the economizing pass
  cut 10, `1038879a`; the home page's line added 3, `b77b6233`);
- the row's tests: **+452 of 150**, code lines by the rule the queue's
  envelope counts (non-blank, non-comment), each file's growth:

      crates/svrngs-node/src/door/tests.rs              +41
      crates/svrngs-node/src/node/tests.rs              +93
      crates/svrngs-node/src/node/tests_support.rs      +18
      crates/svrngs-node/src/surface_list_tests.rs       +1
      web/tests/lend-worker.test.mjs                    +35
      web/tests/shim.test.mjs                           +68
      web/tests/pages/home.test.mjs                     +17
      crates/svrngs-node/tests/relay_share.rs          +179

  (The same measure without `tests/` directories, the caps' own rule,
  gives +273; raw added lines give about +600. 452 is the lean-style
  count including `tests/`, the measure the queue's envelope uses —
  n0-emergency's "240 against 120".)

The queue's tests: **712 of 1,200** (`66,910` at `d59c8246`, `67,622` at
`a5850dae`; n0-emergency's and the earlier rows' 260, share-refusal's 452).
The queue's total has room; the row's own ceiling is what this package
asks about, and the operator's decision says the total is not the reason
to carry on.

## The economizing pass, and why nothing more

- `f80942db` cut the product: the minute's timer, `SHARE_WAITED`,
  `Net::remade` and its impl, `Wire::remade` and its map — 27 product
  lines, and 21 test lines with them (`tests_support.rs`'s `remade`
  plumbing, the unit test's remake half).
- `1038879a` cut JavaScript, behavior kept: 10 lines (shim.js's `wordsIn`
  fold, the banner's `style.cssText`, the fetch chain with a named `noop`,
  lend.js's one `asIs` for two Response constructions).
- The remaining tests are the row's checks themselves, one law per
  planted defect above; nothing there is redundant with another: the
  door's predicate, the door's words, the worker's page, the shim's
  banner, the home page's line, the watcher's two directions, and the
  real-relay scene, each fails alone on its plant. No further cut was
  found that keeps a check.

## The ready diff

The smallest one, in `ralph/next/relay-commons/STATE.md`'s share-refusal
row (the allowance decision is the operator's; not applied):

```diff
- At most +70 Rust, +40 JS, 150 test lines.
+ At most +70 Rust, +40 JS, 452 test lines.
```

(452 is the measured growth; a round 500 would leave 48. The queue's
envelope already takes it in: 712 + 48 of 1,200.)

## What the next session does after the answer

- Raised: the row closes; the loop marks it; the queue moves to
  `webrtc-node`.
- Cut: the named cut lands as its own behavior-preserving commit, the
  row's counts re-run, then the row closes.
