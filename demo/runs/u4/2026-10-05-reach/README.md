# U4's dry run again, after u4-9 — 2026-10-05

Not a run of U4's bar: no phone, no https origin, and Playwright's browsers on
a Mac, as u4-8's was (`../2026-10-05/README.md`). It reruns that dry run after
u4-9's three fixes, to show what each changed before the phones run:

1. Every relay URL without the DNS root's trailing dot, which WebKit refuses
   (`ralph/REVIEW_FINDINGS.md` #52; svrngs `ea651e0`).
2. A range that runs on is asked of the lender's door as a 2 MiB part
   (`web/sw.js` `PART`). This covers Chromium's open `bytes=N-` and WebKit's
   closed `bytes=N-<end>` (#53; `b6ea0ab`, `3c4873b`).
3. The wasm fetched once on a cold join (#51; `0cb236c`).

**How it ran.** The same `dry.sh`, `drive.mjs`, `lend.mjs` and `summary.py` as
u4-8 (`../2026-10-05/`). At `7239474` they also count the wasm each run
fetches, the time from play to the first seek, and each 2 MiB part. The page
and the node were at `0cb236c` (`data/host.txt`). One pass per invocation:

```
PASSES=local  demo/runs/u4/2026-10-05/dry.sh /tmp/u4-9-run demo/runs/u4/2026-10-05-reach/data   # 1 min 37 s
PASSES=public demo/runs/u4/2026-10-05/dry.sh /tmp/u4-9-run demo/runs/u4/2026-10-05-reach/data   # 2 min 38 s
python3 demo/runs/u4/2026-10-05/summary.py demo/runs/u4/2026-10-05-reach/data/bytes.json \
  demo/runs/u4/2026-10-05-reach/data/runs-*.jsonl > demo/runs/u4/2026-10-05-reach/data/summary.txt
```

The summary was read again after the run by `summary.py` as committed. Its
parts measure came after the run, and it reads only the lend logs.

Same host, browsers and film as u4-8: an Apple M2 Max on macOS 26.5.2,
Playwright 1.59.1, Chromium 147.0.7727.15 headless, and WebKit 26.4 as an
iPhone 15. The film is 120 s of 1280x720 H.264 near 3 Mb/s, about 45 MB,
lent as `film`. The page was served from 127.0.0.1 with no cache headers, and
its wasm was not `wasm-opt`ed. `local` reaches Sam's computer through a relay
on this computer. `public` uses the `svrngs` command through n0's relays and
address lookup, from this host's connection.

## What it measured

Medians of three cold runs, the range in brackets, beside u4-8's
(`../2026-10-05/data/summary.txt`). "Plays on" is u4-6's seek bar.

| | local, Chromium | local, WebKit | public, Chromium | public, WebKit |
|---|---|---|---|---|
| link opened → name asked | 577 ms (570-582) | 333 (325-344) | 2,332 (2,260-2,528) | 2,412 (2,314-2,735); u4-8: never, 0 of 3 |
| link → list, untyped | 1,410 | 1,183 | 4,875 (4,592-5,187) | 5,598 (5,276-5,692) |
| link again → list | 387 | 463 | 1,384 | 1,454 |
| lend opened → first frame | 366 (324-449) | 527 (514-535) | 2,098 (1,630-2,368) | 2,164 (2,101-2,235) |
| play → first seek asked | 268 | 871 (852-875) | 349 | 848 (839-858) |
| seek → plays on, 15 seeks | 368 (322-432) | 310 (258-806); u4-8: up to 29,032 | 1,672 (462-1,997) | 1,876 (264-3,358) |
| seeks over 5 s | 0 | 0; u4-8: 2 | 0 | 0 |
| the wasm, fetches a run | 1; u4-8: 3 | 1; u4-8: 3 | 1 | 1 |
| the origin's bytes, a run | 5,335,290; u4-8: 15,350,659 | 5,335,290 | 5,403,119 | 5,403,119 |

What a 2 MiB part took, from asking for it to asking for the part after
it: 132 ms median locally (106-300, 48 pairs), and 633 ms through n0 (585-645,
3 pairs; the seeks came too soon for more). The service itself answered
every range within 125 ms. u4-8's longest answer was 26,190 ms.

## What it shows

1. **WebKit joins through n0's relays now, and plays and seeks.** All three
   public WebKit runs asked a name, listed `Sam’s film`, played it and
   played on after each of 15 seeks, the slowest in 3,358 ms. In u4-8 all
   three stopped at "The computer that sent this link did not answer". The
   page's log names n0's four relays without the dot
   (`https://use1-1.relay.n0.iroh.link/`, `data/runs-public.jsonl`).
   WebKit still logged a failed WebSocket to one or two non-home relays in
   each run (`wss://usw1-1.relay.n0.iroh.link/relay … Socket is not
   connected`), and every run joined and played anyway.

2. **No seek is held behind a range that runs on.** In u4-8, WebKit read
   its first range on to the end, `bytes=66394-45245597`, closed and not
   open, and its seek to 60 s, asked 1.1 s after play, waited up to 29 s.
   Now every range of the film reaches the door closed and at most 2 MiB:
   all 96 locally and all 48 through n0 (`data/lend-*.log`), 124 of them
   whole parts. Each run's first seek was asked within 0.9 s of
   play, inside the first two seconds, and WebKit's played on within
   1,255 ms through n0 (543, 1,255 and 741 ms) and 806 ms locally.

3. **A cold join fetches the wasm once.** It was fetched three times in
   u4-8. The origin's bytes for a run fell from 15.35 MB to 5.34 MB, just
   above the page's 5.14 MB raw. The origin's log has one
   `/wasm/svrngs_web_bg.wasm?v=eb2cb14954ee` line per run.

U4's bar is still unmet: it needs a stock phone browser, and no phone ran
here. Playwright's WebKit on a Mac is not iOS Safari on cellular.
