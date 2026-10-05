# U4's dry run — 2026-10-05

Not a run of U4's bar: there was no phone and no https origin, and the browsers
were Playwright's on a Mac. It leaves no `runs/u4/<date>.json`. It measures what
the row u4-8-phone-dry asks: what a phone will pay to open a group's link and play
a lent film. It is the evidence for the operator's package
(`ralph/next/u4/ctl/NEEDS_HUMAN.md`).

**How it ran.** `dry.sh`, at svrngs `182e44e` (the page and node as at
`15a9de7`), one pass per invocation (both passes at once took 9 min 48 s,
too close to a session's ten minutes):

```
PASSES=local  demo/runs/u4/2026-10-05/dry.sh /tmp/u4-8-run demo/runs/u4/2026-10-05/data   # 3 min 46 s
PASSES=public demo/runs/u4/2026-10-05/dry.sh /tmp/u4-8-run demo/runs/u4/2026-10-05/data   # 5 min 0 s
```

The host was an Apple M2 Max on macOS 26.5.2, with Playwright 1.59.1: Chromium
147.0.7727.15 headless, and WebKit 26.4 as an iPhone 15 (`data/host.txt`). The
page was built by `scripts/build-web.sh` without binaryen, so its wasm is not
`wasm-opt`ed, and served by `drive.mjs serve` on 127.0.0.1. That server sends
no compression and no cache headers. The film is 120 s of 1280x720 H.264 near
3 Mb/s, 45,245,598 bytes, with a keyframe every 2 s and its index first.
`lend.mjs` lends it with byte ranges, and Sam's group `House` lends it as
`film`. Each run is cold: a fresh browser, a new person (Ana, Ben, …), the
link opened, the name typed and the list awaited. Then the link is opened
again, `Sam’s film` opened from the list, its `<video>` played and sought to
60, 20, 100, 5 and 110 s. `local` is Sam's computer through iroh's relay on
127.0.0.1 (`crates/svrngs/tests/phone_dry.rs`). `public` is the `svrngs`
command itself (`new House --as Sam --origin …`, `share <port> --kind film
--limit 16`), through n0's relays and address lookup, from this host's
connection. Every run is a line in `data/runs-<pass>.jsonl`, and every range the
film's server answered is a line in `data/lend-<pass>.log`. `data/summary.txt`
is `summary.py` over them.

## What it measured

Medians of three cold runs, the range in brackets. "Plays on" is u4-6's
seek bar: not seeking, not paused, and a quarter second past the target.

| | local, Chromium | local, WebKit | public, Chromium | public, WebKit |
|---|---|---|---|---|
| link opened → name asked | 479 ms (454-544) | 326 (323-383) | 2,556 (2,431-2,764) | never: 0 of 3 |
| name typed → list | 869 (824-878) | 845 (832-846) | 2,830 (2,830-2,859) | — |
| link → list, untyped | 1,348 | 1,171 | 5,386 (5,261-5,623) | — |
| link again → list | 365 | 419 | 1,349 | — |
| lend opened → first frame | 404 (386-550) | 505 (500-516) | 2,092 (2,086-2,376) | — |
| seek → plays on, 15 seeks | 433 (304-826) | 351 (262-29,032), 2 over 5 s | 1,631 (423-1,999) | — |
| the origin's bytes, a run | 15,350,659 | 15,350,659 | 15,415,045 | 10,259,034 |

## What it found

1. **The page weighs 5.1 MB, and a cold join fetched its wasm three times.**
   The page fetches 5,103,571 bytes raw: 1,732,396 gzip -9, 1,209,407 brotli
   11. The wasm is 5,027,239 of those bytes; upstream's, `wasm-opt`ed, was
   4,295,444. A cold Chromium run took 15.35 MB from the origin. The origin's
   log shows the wasm fetched three times: by the page, as
   `wasm/svrngs_web_bg.wasm` (wasm-bindgen's glue resolves it from its own URL
   without the `?v=`); by the worker's install, as `…wasm?v=<version>`
   (`web/sw.js` `SHELL`); and by the worker's own `init()`, without the `?v=`
   again. A worker's fetches never pass through its own cache. WebKit's public
   runs, whose worker never reached a lend, took two copies: 10.26 MB. An
   origin that marks the wasm immutable should let the browser's HTTP cache
   answer the third fetch, but nothing here measured that.

2. **WebKit cannot reach n0's relays: every public WebKit run failed.** Each
   ended "The computer that sent this link did not answer" after the page's
   60 s, and WebKit logged `https://use1-1.relay.n0.iroh.link./ping: The
   server did not accept the certificate.` for each of n0's four relays. iroh
   spells its default relays with a trailing dot
   (`iroh-1.0.2/src/defaults.rs:27-33`, made into URLs at `:48`). WebKit
   refuses TLS to any host so spelled: `https://example.com./` fails too ("A
   TLS error caused the secure connection to fail."), while the same hosts
   without the dot load. Chromium loads both. Every browser on iOS is WebKit,
   so an iPhone joining a group that uses n0's relays should fail the same
   way. Upstream's one browser dial ran in Chromium (DESIGN §8, U4's
   inventory), and the browser tests run a relay on 127.0.0.1, so neither
   could have shown this. `ralph/REVIEW_FINDINGS.md` #52 holds the probe.

3. **In WebKit, an early seek far ahead waited for the open range.** Each
   WebKit run first asked `bytes=66394-` or `bytes=65642-`, to the end of the
   film. Its seek to 60 s (byte 22.6 M), 1.1 s after play, sent no new range
   while that one was still being read. In runs 1 and 3 the range ran to the
   end of the file (45,179,204 bytes in 26.2 s and 23.3 s, about 1.8 MB/s
   through the relay, the worker and the wasm). The seek played on only after
   that: past the 30 s wait in run 1, and in 29.0 s in run 3. In run 2 the
   range was cut at 23.9 MB, 3.9 s in, and the seek played on in 3.4 s.
   Chromium cut each open range within a second and sent a new one for every
   seek: its 30 seeks all played on within 2.0 s. WebKit's later seeks, after
   the first range had ended, played on within 3.2 s. A real film is
   gigabytes, so a seek like this one on a phone would wait for the bytes
   between. u4-7's 60 s film passed its three WebKit seeks in under 300 ms.
   `lend-local.log` has every range, with its timing.

4. **Through n0, Chromium did everything U4's bar asks.** It joined in 5.4 s
   from opening the link, not counting typing, and showed the film's first
   frame 2.1 s after opening it. Its 15 seeks all played on within 2.0 s,
   well inside Demo A's 5 s.
