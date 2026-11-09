# The send limit: one large send at a time

The question the 2026-11-08 runs left open (`../2026-11-08/README.md`, "The
stream receive window, measured"): four fetchers of one 40 Mbit/s uplink read a
ping p95 of 430 ms, about four receive windows queued, against a bar of 140.1.
Built by the operator's "Ok agreed" of 2026-11-09 to funding this alone.

One run of each (AGENTS 7), on the Fedora box in `/dev/shm`, with
`../2026-11-08/run.py` (`one --fetchers 4`, `fast`, and `many`, added for this)
and the same shaped path: 40 Mbit/s, 500 ms buffer, 40 ms RTT. The load was
12 at the start of the two shaped runs, which ran beside each other, and 0.3
for the unshaped ones. `data/results.jsonl` holds every summary.

## Why turns and not a queue

A get held whole until its turn shows its fetcher no byte, and a fetcher takes
a holder silent past `ANSWER_WITHIN` (3 s) as asleep for a minute. So a turn
lasts about 3 s (`folder.rs`, `Sends`), and at each look, every second, every
waiting get sends one chunk through iroh-blobs' throttle hook.

## Verdicts

| Run | Number | Bar | Verdict |
|---|---|---|---|
| Four fetchers, 1 GiB, no limit (the plant, `../2026-11-08/`, `w-four`) | ping p95 430 ms | Must miss 140.1 | **missed, as a plant must** |
| The same, turns of 3 s (`s4-t3`) | ping p95 **116 ms**, idle 40.1; slowest **915.3 s**; 336 turns; no holder taken as silent | 140.1 ms; 1,152 s | **passed** both |
| The same, turns of 6 s (`s4-t6`) | ping p95 **114 ms**; slowest **915.1 s**; no holder taken as silent | 140.1 ms; 1,152 s | **passed** both |
| 100 MiB, one fetcher, no shaping, no delay | 0.50 s with the limit (`fast-sends`), 0.75 s without (`fast-window`) | No slower | **passed** |

The 3 s turn landed: the two read the same and a small file waits less behind
it. `tests` beside the code hold the turns (`folder_tests.rs`), watched red with
`ask` handing every get back: 3 of 4 failed.

## What it costs

100 files of 4 MiB, one fetcher, unshaped (`many`):

| Path | Without the limit | With it | |
|---|---|---|---|
| 40 ms RTT | 26.1 s | 36.9 s | **41% slower** |
| No delay | 28.8 s | 26.2 s | no cost |

A computer's large sends together now go at one stream's rate, window ÷ RTT.
At 40 ms that is 13 MB/s where eight files at once had more. That is the
price of queueing one window: a fast uplink on a long path gives up some of its
rate to keep the short queue.

## What this does not cover

- A file under one window (512 KiB) sends without a turn, so a folder of only
  such files, to several fetchers at once, is not held by this. Not run.
- Slices (part B) were not run with this. Each slice would be a get, and so
  take turns the same way; not measured together.
- A lend's stream is not a get and takes no turn.
- One holder, one path, one run of each.
