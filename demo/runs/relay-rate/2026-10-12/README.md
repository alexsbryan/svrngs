# A film through a relay at Demo A's bitrate — this host, 2026-10-12

Row relay-verb (b) (`ralph/next/relay/STATE.md`): an `--open` relay holds
each computer or phone to a rate, and the operator's word is that no limit
is a number nobody chose. This measures what one phone's film draws through
a relay, and `OPEN_RATE` (`crates/svrngs/src/relay.rs`) is set from it.

## How it ran

```
RELAY_RATE_OUT=$PWD/demo/runs/relay-rate/2026-10-12/data/take-<n>.json \
  cargo test -p svrngs-node --test browser -- --ignored --exact a_films_peak_through_a_relay --nocapture
```

The test is `a_films_peak_through_a_relay`
(`crates/svrngs-node/tests/browser/main.rs`), three takes, 23:01-23:02
local, 12 cores, macOS 27.0.1, the 1-minute load 32.71, 31.05 and 28.35 as
each began (another project's build beside it). Ada's computer lends a film
of 120 s made by ffmpeg at `-b:v 3730k -maxrate 3730k` — Demo A's bitrate,
3,730,555 bit/s, Jellyfin's conversion for WebKit
(`demo/runs/a/2026-10-11/README.md:38`): 55,979,835, 55,989,301 and
55,954,991 bytes, 3.73 Mbit/s each. A phone's page in Chromium plays it
through iroh's relay on 127.0.0.1 and seeks to 40 s, 20 s and 100 s, the
worker stopped before the last, as Demo A's people do (`web/tests/runtime.mjs`).
The relay's bytes received from all its clients (`bytes_recv`) are sampled
every 50 ms (`samples`: ms, received, sent), and the most received in any
1,000 ms is kept.

## What it measured

| take | the drive | received | most in one second | from | first play, seeks (ms) |
|---|---|---|---|---|---|
| 1 | 3,606 ms | 10,970,230 | 7,259,538 | 2,003 ms | 267; 378, 289, 554 |
| 2 | 3,199 ms | 11,546,019 | **8,799,728** | 1,586 ms | 260; 285, 319, 653 |
| 3 | 3,296 ms | 9,289,481 | 5,848,955 | 1,581 ms | 273; 289, 295, 663 |

- **The most a film drew in a second is 8,799,728 bytes, 70.4 Mbit/s**,
  nineteen times its bitrate: the page fills its buffer as fast as the
  relay passes bytes after a play and each seek, then waits. So
  `OPEN_RATE` is 8,800,000 bytes a second, take 2's most rounded up to
  the next hundred thousand, its burst iroh-relay's default, a tenth of a
  second's (`iroh-relay-1.0.2/src/server/streams.rs:421`).
- **What the rate holds is a computer's sending, not a phone's**: iroh-relay
  limits what it reads from each client (`Limits::client_rx`), and a film's
  bytes are read from the lender's computer. At this rate a computer lends
  a film at Demo A's bitrate to eighteen phones at once before the relay
  slows it, each at its own 3.73 Mbit/s; it slows one fetching its buffer
  as fast as loopback allows, as no phone on a network does — n0's relay
  gave 8.1 Mbit/s (`docs/DESIGN.md`, §3, "The room").
- The relay's count is every client's, so each second's most bounds the
  lender's from above: the page's own sending, requests and
  acknowledgements, is in it.

## Not judged

- A phone on a real network, and a relay that is not on loopback: the
  operator's run through `relay.svrnme.sh` (DESIGN §8, "our own relay").
- Three takes on one host are not a rate (AGENTS.md 7); the number is the
  most seen, said with where it came from.
