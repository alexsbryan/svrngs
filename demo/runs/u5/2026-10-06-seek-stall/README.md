# Seeks back to back, through a lend — u5-11, this host, 2026-10-06

u5-11 (`ralph/next/u5/STATE.md`): the stall u5-9 found
(`demo/runs/a/2026-10-05/README.md`, "What it found", 1), first on the
conformance canary's 64 MiB file. The canary's route "seeks back to back"
(`apps/web-conformance/checks.js`, `seeks`) plays the file as sound
(`/big.wav`, the 64 MiB under a WAV head, 384,000 bytes a second), seeks
twenty times 250 ms apart, forward and back in turn, and passes if the last
seek plays again — its time past where it was sent — within 5 s of being
sent. `again` below is the ms from 250 ms after the twentieth seek, so the
bar is `again + 250 <= 5,000`.

```
cargo test -p svrngs-node --test conformance        # all six: three ways, two engines, in parallel
demo/runs/u5/2026-10-06-seek-stall/keep.sh <dir>     # what each <dir> holds, from target/conformance/
```

Each `<way>-<engine>.json` holds, per lend (Sam's and Bo's canary), the
check's verdict and what it saw, and the worker's `RANGE` lines (phones
only: a computer has no worker). `carriage.log.gz` is every door's
`carriage` lines for the run's six tests at once. `verdicts.txt` is every
route's verdict. `uptime.txt` is the host's load at the run's start and
end; this host has 12 cores, and no run here came near them.

## The phone's link

Through n0's relay, Demo A's phones got bytes from Sam's door over this
host's uplink. The test gives the phone that link
(`crates/svrngs-node/tests/conformance.rs`, `uplink`): every byte each way
held 150 ms; what the lender sends the phone paced to a third of Demo A's
50 Mbit/s, 16,666,666 bit/s, the share each of the three was told, with
up to one second's bytes queued, as this host's uplink answered in 959 ms
under load (`demo/runs/a/2026-10-05/data/upload.txt`). The calibration
runs found that link: with less queued the stall did not show every run.

| run | link | phone seeks that missed 5 s |
|---|---|---|
| calibrate-a | 3 MB/s, 1.5 MB queued | 1 of 8: WebKit, floor, Sam's, 6,767 + 250 |
| calibrate-b | 3 MB/s, 1.5 MB queued | 0 of 8 |
| calibrate-c | 2.08 MB/s, 1 MB queued | 0 of 8 |
| before-1 | 2.08 MB/s, a second queued | 3 of 8: Chromium floor Bo's 5,010; WebKit floor Sam's 5,713; WebKit wildcard Sam's 5,015 (+ 250 each) |
| before-2 | 2.08 MB/s, a second queued | 2 of 8: WebKit floor Sam's 6,173; WebKit wildcard Bo's 5,737 (+ 250 each) |

Computers (no relay, no worker) played again in 359–938 ms in every run.
Before that, a run with both directions paced at 3 MB/s and 1.5 MB queued
missed twice in WebKit (5,231 and 5,928 + 250; not kept, its directory
overwritten by the next), and its 64 MiB uploads failed: the phone's own
sending crosses the lender's downlink, so the link paces only what the
lender sends.

## What the trace shows

In `before-1` and `before-2`:

- **The door sends what a seek asked long after the browser has moved
  on.** Each run's doors cut off 142 and 141 answers because their
  askers stopped reading them, having sent a median of 1,767,184 and
  1,814,211 bytes of each, 685 MB across each run's six tests
  (`carriage.log.gz`, "the answer cut off"). The door's service answers
  a range in 0–4 ms, nearly every time. The worker asks every seek's open range as a part of 2 MiB
  (`web/lend.js` `PART`, since u4-9), and the door sends it into the link
  as fast as the link takes it.
- **Neither engine tells the worker it let a seek's request go.** No
  `the browser let go of it` line, in either engine, in any run: a media
  element's abandoned request never aborts its fetch event's signal. The
  worker learns only when it hands back the answer, which the browser then
  cancels: Chromium after reading 1,154–1,167 bytes of it, WebKit after
  2,596 bytes to 434 KB in `before-1`'s floor Sam (`cancelled by the
  browser after`). By then the part is in the link.
- **So each seek let go costs the link its part, ahead of the last seek's
  bytes.** In Chromium every seek asks a new open range: in `before-1`'s
  wildcard Sam, 20 answers were cancelled, and the time the worker waited
  for each answer rose from 312 ms to about 2.2 s as their parts filled the
  link's queue; the last two took 3,655 and 3,832 ms. In WebKit the seeks are
  taken one at a time, with a 64 KiB probe a round trip at each, and its
  2 MiB parts for each place it looked queue ahead of the next probe.
