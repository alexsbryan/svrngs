# The browser suite under load — this host, 2026-10-12 (bash-film-load (a))

Three tests of `crates/svrngs-node/tests/browser/main.rs` failed in three of
the bash queue's `scripts/check.sh` runs, each passing alone after
(ralph/next/bash/STATE.md, bash-film-load):

| check | test | where | what the output shows |
|---|---|---|---|
| bash-first-minute-1 | the film, Chromium | `:711` | the third seek, after the worker's stop, played again at 11,424 ms; the worker's re-dial 5.8 s, a 256 KiB part read in 9.5 s |
| bash-devices-1 | the film, WebKit | `:704` | "the film must play": the first play and the two seeks after it never played in 15 s each; the page logged "WebSocket connection to 'ws://127.0.0.1:52328/relay' failed … Socket is not connected" before it opened the lend; after the browser was opened again the film played in 582 ms and the last seek in 531 ms |
| bash-same-name-1 | the ask, Chromium | `:1134` | after Stop the group's page said "What apps use of cy’s shelf (k) cannot be asked now.", its trace "GET /choices/… — the body stopped short: error reading a body from connection" |

The logs are `ralph/next/bash/ctl/log/sessions/<check>.await.log`. In those
checks the suite took 157-282 s; here, at like loads, 33-41 s.

## How it ran

```
demo/runs/film-load/2026-10-12/measure.sh <label> <runs> <burners> [tests…]
python3 demo/runs/film-load/2026-10-12/seeks.py demo/runs/film-load/2026-10-12/data
```

`measure.sh` runs the suite, or the tests it names, `<runs>` times, with
`<burners>` `yes` processes beside each run (started 20 s before it), and
keeps each run's output, its load at the start and end, and
`target/traces/browser.log` — the relay's, doors' and nodes' debug lines,
kept since 0485576 — under `data/<label>/<run>/`; `data/runs.tsv` has a
line per test per run (3 of 397 lines were lost where a test's output ran
into its result line; each run's `test result` line, in its `test.out`, is
whole). `seeks.py` prints each film run's play and seek times. 12 cores,
macOS 27.0.1, base 0485576, 16:58-17:44 local. The host was not idle: another
project's build kept the load at 29-69 with nothing of ours added. N was
set by an hour of the host: a film pair is about 30 s, the suite 35-200 s.

## What it measured

| label | runs | added | load at start (end, most) | passed | slowest seek, Chromium / WebKit |
|---|---|---|---|---|---|
| film-idle | 10 | none | 29.8-39.4 (39.4) | 20 of 20 | 784 / 1,046 ms |
| film-load36 | 8 | 36 `yes` | 43.2-81.0 (105.3) | 16 of 16 | 2,017 / 1,070 ms |
| suite-idle | 6 | none | 51.2-68.6 (68.6) | 6 × 36 of 36, 32.5-41.1 s | 2,050 / 956 ms |
| suite-load36 | 3 | 36 `yes` | 61.6-85.5 (121.6) | 3 × 36 of 36, 87.3-112.0 s | 1,704 / 951 ms |
| suite-load96 | 2 | 96 `yes` | 91.2-139.7 (230.1) | 2 × 36 of 36, 157.3-198.3 s | 2,948 / 2,135 ms |
| ask-load96 | 4 | 96 `yes` | 94.4-119.3 (164.2) | 8 of 8 | — |

Nothing failed: the film test 29 times in each engine, the ask 12, the
suite 11 times, at loads to 230 on 12 cores (19 a core). One host, these
runs: not a rate (AGENTS.md 7).

- **The film's seeks slow with load and stayed under the bar.** Chromium's
  slowest seek grew from 0.8 s unloaded to 2.9 s at 16-19 a core; none came
  near bash-first-minute-1's 11.4 s. What made that check's suite take
  157-282 s, where `yes` at the same load gave 35-112 s, is not measured:
  busy processes take processor, not memory, and the host's swap stood at
  12.7 of 14 GB used at 16:51. The film's seek bar is now judged at the
  load as conformance's is (bash-film-load (b)): a miss past 3 a core
  is could-not-judge.
- **The relay drops packets in every run, and nothing fails of it.** The
  relay's `failed to forward packet: Full` — a node's packets to a browser
  dropped at the relay's queue — is in every film run's trace, 14 to 1,016
  a film pair and 388 to 2,709 a suite run, the most at the most load;
  QUIC sends them again.
- **The ask's miss is the product, not the load.** Its trace read with the
  code: the group's page closed its dial before reading the answer's body
  (`web/app.js` `uses()`), so a body that came after its head — as a dropped
  packet sent again comes — was cut off. Repeating it under load did not
  show it (ask-load96, 8 of 8); a plant does on every run, a body past the
  page's first read (c6b387b, red), and the page now reads before it closes
  (b7126e2).
- **WebKit's film that never played is not judged.** The socket that failed
  was the group's page's own, logged before it sent the browser to the
  lend; the film plays in the lend's worker, on an endpoint of its own,
  whose console Playwright cannot read, and it played at once in the
  browser opened again. Not seen here in 29 runs, so whether it was the
  harness or a page that stops for good when its relay socket breaks is
  could-not-judge; since 0485576 a check that fails it keeps the relay's
  and the door's lines in `target/traces/browser.log`. Written as a §9 row
  (DESIGN §9) with its trigger.

## What it leaves out

The memory a full check's builds take; two checks at once on one host;
a phone, a real relay and a network; any person.
