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

## Relay seek: credit and order

The first replay (`avail-robust-e2e-1.await.log`, `db159de5`) had 10/10
Chromium closes, WebKit closes, WebKit navigations and page-dial seeks,
but 9/10 data-endpoint seeks. Its last data-endpoint run read 787,827 B
before the seek against a 786,432 B limit. The two lookahead readers were
still granting fresh connection credit, so that byte count wasn't a bound
on the bytes that were queued before the seek.

The replacement stops returning any credit while a controlled QUIC sender
fills exactly `QUEUED` across two streams. It checks whether the next
stream can write before readers return credit. The HTTP half goes through
the real door and relay: it observes both partial responses, reads the
seek's 206 and body, then releases both old tails and reads them to EOF.

Focused command in the gate image:

```sh
TOKIO_WORKER_THREADS=4 cargo test --locked --offline -p svrngs-node \
  --test relay_tiers a_seek_over_the_share -- --nocapture
```

Red, remove the connection receive window from the ordinary page dial and
leave the data endpoint as a control:

```text
seek write exceeded QUEUED: Ready(Ok(1))
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out; finished in 27.63s
```

Red, restore the cap but make the fixture hold the seek behind the old
tails. Run only `a_seek_over_the_share_on_a_pages_dial`:

```text
the seek over QUIC: 262144 B filled the connection; the next stream waited for credit
the seek and lookahead prefixes must arrive before the lookahead tails are released: Elapsed(())
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 15 filtered out; finished in 67.76s
```

Green, no plants, both dials:

```text
the seek over QUIC: 262144 B filled the connection; the next stream waited for credit
the seek over QUIC: 262144 B filled the connection; the next stream waited for credit
the seek over a page's dial: both lookaheads started; seek arrived in 1299 ms before releasing their tails
the seek over a data endpoint: both lookaheads started; seek arrived in 1751 ms before releasing their tails
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 27.63s
```

The repetition driver is `robust-e2e.py` in this directory. Run it inside the
normal gate image under its lock. It uses the ordinary browser step's
selection and parallelism, with the ordinary relay-tiers binary alongside.
