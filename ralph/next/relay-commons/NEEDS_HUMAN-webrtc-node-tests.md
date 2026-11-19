# webrtc-node: the row's test allowance

**Question:** webrtc-node is done and green. Its product is +346 counted
lines, within the operator's +400 (`7b0b36cd`), and the trust floor's line
for the WebRTC peer's socket is approved there too. Its tests are **+310
code lines against the row's 200**: `tests/webrtc_path.rs` +284 and
`src/webrtc/tests.rs` +26. Raise the row to 310, or name a check to drop?

Your rule (`2dc7ebe4`, PROMPT.addendum.md) makes this yours: "Each row's
own allowance is its ceiling … Past it, after an economizing pass, stop
with the counts and a ready diff."

## The economizing pass

Two passes, test-only, every check kept (`69dd09e0`, `85cbdf2f`):

- the wait for the offerer's channel-open went: the data endpoint's dials
  retransmit until the channel is open, as QUIC does (−6);
- the single-use `relay()` and `homed()` helpers folded into the test
  (−15);
- the timeout's diagnostics say each path's address alone (−3);
- rustfmt's wraps taken where they fit.

What is left is not foldable: the offerer's str0m loop (`pump`, the
node's `run` mirrored, ~60 lines — it cannot borrow the product's `run`,
which owns the channel's close differently), the node's setup, the offer
over `Alpn::WebRtc`, the two dials of the spike's Q1, the two asks, and
the two plants. The row's check alone needs ~230: the 200 estimate
counted the check but not its offerer.

## Counts

Code lines, meaning non-blank and not starting with `//`:

    tests/webrtc_path.rs 284   src/webrtc/tests.rs 26   total +310

The queue's tests, by the same measure: 591 before this row, 901 now, of
the 1,200 envelope. `webrtc-page` allows 250 and `self-host` 120, which
comes to 1,271: 71 over the envelope unless the row is cut to ~230 (the
check's floor) or the envelope moves.

## Options

- **A. Raise to 310 (recommended).** Every check kept: the offerer's
  loop and the two dials (the row's check), its two plants watched red on
  the final revision, the offer-refusal unit check, and the route-table
  unit check. Cost: 110 over the estimate, 71 over the envelope if
  `webrtc-page` and `self-host` hold their estimates.
- **B. Hold 200.** Needs about 110 cut, which means dropping checks: the
  route-table unit check (−16), the offer-refusal unit check (−10), the
  second ask over the relay (−6) — still ~250, so B cannot reach 200
  without dropping the row's own check, which the row names.

## Ready diff, for A (STATE.md, the webrtc-node row)

```diff
- At most +400 product lines (operator, 2026-11-18: "Approved both", raised from 250 at ~+389 measured, the port of the spike's node half and wire), 200 test lines.
+ At most +400 product lines (operator, 2026-11-18: "Approved both", raised from 250 at ~+389 measured, the port of the spike's node half and wire), 310 test lines (operator, 2026-11-19: raised from 200 after the passes, 69dd09e0 and 85cbdf2f; its check's offerer was not in the estimate).
```

After A, the row closes and the queue moves to `webrtc-page`.
