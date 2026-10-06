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

## The fix, and what was tried

`web/lend.js` `bounded`: an open-ended range — Chromium's every seek —
is asked of the door as a part of 256 KiB where no part of the file ended
just there, and twice the last each time the reading runs on from where
the last ended, up to 2 MiB. A seek let go of costs the link at most
256 KiB. A closed range, WebKit's `bytes=N-<end>`, stays a part of 2 MiB.

`again`, ms after the twentieth seek + 250 ms, the four phone lends of
each engine (floor and wildcard, Sam's and Bo's), sorted; every run all
six tests at once, the host's 1-minute load 1.7–4.6 throughout (12
cores):

| run | what the worker and door did | Chromium | WebKit as an iPhone |
|---|---|---|---|
| before-1 | as at the row's start | 4,049, 4,322, 4,706, **5,010** | 2,282, 3,190, **5,015**, **5,713** |
| before-2 | as at the row's start | 4,171, 4,357, 4,641, 4,707 | 3,984, 4,658, **5,737**, **6,173** |
| tried-all-small | every range's part starts at 256 KiB | 670, 722, 727, 758 | 2,183, 2,975, **6,480**, **7,008** |
| tried-newest-first | each stream's answer sent before any older's (the door's QUIC stream priority, `iroh.rs`'s acceptor) | 2,577, 2,927, 3,559, 3,665 | 1,344, 2,592, 3,528, 4,536 |
| tried-newest-first-all-small | both | 607, 651, 683, 716 | 3,839, 4,178, 4,727, **24,190** |
| tried-newest-first-open-small | newest first, and the fix | 550, 644, 716, 794 | 1,776, 3,335, **5,161**, **7,631** |
| tried-closed-whole | the fix, and a closed range asked whole | 721, 733, 744, 751 | 2,783, 3,553, 3,623, **13,706** |
| after-1 | the fix | 744, 747, 759, 803 | 2,448, 2,615, 4,607, **6,026** |
| after-2 | the fix | 721, 743, 744, 828 | 240, 3,263, 3,671, 3,984 |

Computers played again in 360–1,018 ms in every run. Bold: missed 5 s
with the 250 ms. Nothing tried is kept but the fix: newest-first moved no
WebKit seek that the runs can tell from noise, and leaves a long-lived
socket behind every newer download.

- **Chromium: fixed.** 1 of 8 missed before, 4,049–5,010; none of 8
  after, 721–828 — faster than Chromium straight at the canary over the
  same link (below), 1,224–1,414, since a part the lend asks is smaller
  than what Chromium asks of a server.
- **WebKit: 4 of 8 missed before, 1 of 8 after; not fixed.** Its seeks
  are its own. Through a lend, a closed range answered in parts makes
  WebKit fetch at its seek in serial 64 KiB reads, a round trip each, and
  fill the gaps behind it from the file's start (`tried-newest-first-
  all-small`, floor Sam: 14 reads of 64 KiB or less at 28.4–29.3 MB,
  asked over 8.1 s). Parts
  starting smaller make it slower (`tried-all-small`); a closed range
  answered whole, it reads through, 18–40 MB at a time for 36–65 s, as
  in u4-8 (`tried-closed-whole`, wildcard Sam; ralph/REVIEW_FINDINGS.md
  #53).

## The control: straight at the canary, the same link

`control.mjs` (`control.json`): the canary's page opened by Playwright
straight at the canary, no svrngs, through a proxy giving the browser the
same link — 150 ms each way, what the canary sends paced to
16,666,666 bit/s, a second's bytes queued, shared by every connection.
Eight tries each, load 1.7–3.3:

- WebKit as an iPhone: 1,944, 2,159, 2,184, 3,792, 3,841, 3,983, 4,151,
  4,224 — none missed.
- Chromium: 1,224–1,414 — none missed.

Straight at the canary WebKit asks one closed range at each place it
seeks, streams it, and closes the connection when it moves on (each run's
`log`): three or four places for twenty seeks, in the two read here.

## Not reproduced here

u5-9's stall was 21 of 22 seeks never playing again within 30 s, in
WebKit, in Jellyfin's converted film (HLS), through n0's relay. The
canary's file, played as sound, never stalled like that here: its worst
was 24,190 ms, under a change not kept. Whether the converted film's
stall is this one is not shown.
