# Keeper: browser and relay regression checks

The flaky close assertion compared elapsed time to a 2,500–4,000 ms window;
the audit found correct bytes after 4,006–4,018 ms. The seek's 12 s sleep
did not show that both lookaheads were flowing, and its 6 s assertion measured
both queued bytes and scheduling. The navigation abort is Playwright's own
[provisional-target crash](../../../flakes/2026-11-19/crash.txt).

## Red / green in `svrngs-gate`

`TOKIO_WORKER_THREADS=4` throughout. Focused browser command:

```sh
cargo test --locked --offline --quiet -p svrngs-node --test browser -- \
  the_channel_closed_mid_film a_navigation_from_another_lend_s_page_is_judged_as_its_fetch
```

Red: suppress the shim's `RTCPeerConnection.close` at the browser boundary,
and make the WebKit navigation driver exit 1 with an unrelated error.
Both engines still read the full 6,291,456 bytes with the original SHA, but
failed `the WebRTC channel must actually close before it can carry the whole range`.
Navigation failed once, without a retry:

```text
the browser half failed (exit status: 1):
plant: unrelated browser-driver error
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 48 filtered out; finished in 30.84s
```

Positive control: plant `crash.txt`'s exact stack on the first attempt. The
WebKit driver logged `retrying the WebKit browser half once after Playwright #43067:`
and its restarted browser passed the Rust origin assertions.

Green: remove the plants. Both navigation cases and both fallback cases passed.
Chromium and WebKit observed `closed`, with 2,791,144 and 2,790,873 bytes on
the channel at close, respectively. The lender traced `network_path=Custom`
abandoned and `network_path=Relay` selected for the same data peer. The exact
range and SHA matched. `fallbackMs` was 3,755 and 3,856 ms (diagnostic).

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 27.32s
```

Seek: `cargo test --locked --offline --quiet -p svrngs-node --test relay_tiers a_seek_over_the_share -- --nocapture`.
Red: remove the connection receive window from the ordinary page dial, leaving
the data endpoint as a control. Green: restore the window.

```text
the seek over a page's dial did not overtake the lookahead: 787510 B read ahead of it
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out; finished in 33.52s
the seek over a data endpoint: [136291, 138596] B ahead of its answer, arrived in 2399 ms
the seek over a page's dial: [254491, 144805] B ahead of its answer, arrived in 3499 ms
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 33.63s
```

The repetition driver is `robust-e2e.py` in this directory. Run it inside the
normal gate image under its lock. It uses the ordinary browser step's
selection and parallelism, with the ordinary relay-tiers binary alongside.
