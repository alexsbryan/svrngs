# Economizing pass: relay-fallback's landing

Read before the queue's cap ask (AGENTS.md "Done": a raise is asked only
after an economizing pass). Baseline is f8a57035; the queue's commits are
b6db94fd (the keeper meets at the fallback), 02ac74b8 (the page takes
every route its policy allows), a44164cd (the published-policy join),
adc262fe (a route that cannot carry does not win) and 38d4bc0c (the tests'
shared setup and driver).

## The counts

`python3 scripts/lean_check.py` at 38d4bc0c, exit 1:

```
crates/svrngs-core     3491 / 3494 lines
.                     19912 / 19833 lines
js                     8187 / 8188 lines
swift                     0 / 1 lines
BROKEN  . has 19912 lines of code, budget 19833: raised only by the operator, with the reason in the commit
```

Counted-line growth over the baseline, by file (nonblank, noncomment, the
lean rule): `crates/svrngs-core/src/relay.rs` +5, `web/src/lib.rs` +75;
every other product file the queue touched is +0 (lines replaced in
place). So the queue's slices hold — core +5 (≤10), browser Rust +75 (≤80),
JS +0 (≤30), native+core +5 (≤40) — and the repository cap is what the
landing does not fit: `.` needs 19913, +80 over 19833.

Test code added over the baseline, counted the same way:

```
crates/svrngs-node/tests/relay_fallback.rs   +124   (fallback-proof's)
crates/svrngs/tests/join_page.rs             +217
crates/svrngs-core/src/relay_tests.rs          +7
crates/svrngs-node/tests/relay_tiers.rs        +3
web/tests/page.mjs                             +2
total                                        +353
```

against the queue's 150-line test allowance.

## The pass, and what it cut

The queue's own folds, each in its behavior or move commit:

- `web/src/lib.rs`, before its commit: `upgrades_insecure`'s guard returns
  folded into one chain (-8), `this_page_takes` and `page_protocol` made
  one function (-4), the relay trace loop one expression (-2), the filter
  chain one line (-5), `holder_endpoint`'s parse folded (-3): +90 became
  +75.
- `crates/svrngs/tests/join_page.rs`, its own commit 38d4bc0c:
  `sam_meeting_at` and `phone_joining` in place of the setup and driver
  written twice, behaviour kept (-28 of test code).

The pass over the touched files found no other supported cut: every item
is live. `relay_of` (the older-link guard) is called at dial.js:121,
join.js:270 and visit.js:202; `relays_of` at hand.js:80 and visit.js:149;
`holder_endpoint` at lend.js:280 and apps/taste/check/other/other.js:85;
`Meeting::{own,build,decide,relays,join,carried}` each have their caller
(tiers.rs `made_at`/`at_fallback`/`check`, core `carried`, the home page's
`Tier`); `tiers::{made_at,home,at_fallback,moved}` keep their callers
(group.rs, surface.rs, the home page). Nothing is said twice across
`relay.rs` and `tiers.rs`: the core decides, the node applies.

No cap was edited; the measured landing and the ready diff are in
`ralph/next/relay-fallback/NEEDS_HUMAN.md`.

## What is not proven here

The focused checks are in the queue's commits; the unchanged full
`scripts/check.sh` is REVIEW-audit-relay-fallback's, not this pass's. The
real phone, the Pi and the public relays were not run by this unit.
