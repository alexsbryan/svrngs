# streaming-stall (a): the 9 MB range that holds still — predictions, before any data

Row streaming-stall (`ralph/next/stall/STATE.md`), part (a). Written at
54844461, after the instrument (drive.mjs `drain` reports what it drained
when a step times out) and before any run of this battery. Nothing in the
product changes in this part.

## What is known before it runs

- Six of six closing gates of models-fresh (2026-10-22,
  `ralph/next/models/ctl/log/sessions/models-fresh-{1..6}.await.log`): the
  whole browser suite (`scripts/check.sh`'s "browser" step, 121 tests,
  272 s), webkit's range answered 206 0.26-0.63 s in, then no end inside
  60 s; its "the stream is done" only after the next step opened a stream,
  0.9-9 s later; the next step (`past`, a 416 with no body) itself took
  2.1-9.2 s; the 8 MiB upload 36.1-40.5 s. Chromium green in all six.
- Alone, the same test passed: arch-stream's ten runs under burn24 after
  its fix, 10/10 green in both engines, webkit's range 17-31 s and upload
  18-31 s (`demo/runs/stream/2026-10-15/b-summary.txt`); here, at load 9.5,
  webkit's range in 2.4 s.

## The setup

`run.sh`, `battery.sh`, `summary.py` here. Two cells, at HEAD:

- **alone-burn24**: the streaming test alone, both engines at once, under 24
  `yes` of the run's own (twice this host's 12 cores), ten runs.
- **suite**: the whole browser suite as the gate runs it, no load planted
  (other sessions' load recorded at each run's start and end), three runs.

Each run keeps its output, the relay's and doors' debug log and the load.

## What I expect, and what would falsify it

1. **Alone under burn24 does not make it at will**: webkit's range red in at
   most 2 of 10. Falsified by 3 or more, and then that cell is the
   reproduction.
2. **The suite makes it**: webkit's range red in at least 2 of 3. Falsified
   by 1 or none; then neither cell makes it at will and the next battery
   must look for what the suite adds besides load.
3. **Where the body stopped**: in a red run the page had drained part of the
   body, not none and not nearly all — `drained.bytes` between 64 KiB and
   8 MB — and its last chunk came within the first 10 s of the step, the
   rest of the 60 s with nothing. Falsified by a red with nothing drained (a
   head with no body: the stall is before the first byte) or a red that
   drained steadily to near the end (slowness, not a stall).
4. **Which side holds the bytes**: the page's side of the relay. The door's
   line for the stalled range ("cut off" or "to its end", door.rs
   `Carriage`) says it sent more than the page drained — the door handed
   QUIC bytes the page never read. Falsified by a door that sent no more
   than the page drained, plus a stream window: then the door, or the
   service behind it, stopped.
5. **The upload's 36-40 s**: slowness of a loaded host, not a second stall:
   in the suite runs webkit's upload 25-45 s, and alone under burn24 within
   arch-stream's 18-31 s. Falsified by an upload over 45 s in a green run.

## Battery 2 (`battery-2.sh`), written after battery 1 and before this one

Battery 1 made no red in 13 runs (`README.md`). Battery 2 is one cell,
**gate-burn24**: scripts/check.sh:53's own `cargo test --quiet --workspace
--test browser` (with `--nocapture`), under 24 `yes` of its own, five runs.

6. **The gate's command under load makes it**: webkit's range red in at
   least 3 of 5, and the suite at least 250 s. Falsified by 2 or fewer reds;
   then, if the suite ran 250 s or more, load and the workspace build
   together do not make it, and the next battery looks at what the gates'
   trees held besides; if it ran under 250 s, the planted load was not the
   gates' and the next battery plants more.
7. Predictions 3 and 4 stand for any red here.
