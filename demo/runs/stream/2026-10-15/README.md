# arch-stream (a): the browser suite's streaming test under load (2026-10-15, this host)

Row arch-stream (`ralph/next/arch/STATE.md`), part (a), measured against
`PREDICTIONS.md` (879de7b2, before any run). One host, ten runs a cell: a
spread, not a rate (AGENTS.md 7).

## How

`run.sh <tree> <cell> <burn> <first> <last>` runs
`cargo test -p svrngs-node --test browser -- streaming_both_ways --nocapture`
in a tree, both engines at once as the suite runs them, and keeps per run:
the output (`<n>.out`, each engine's answers and the page's lines with their
times, printed by the instrument fb5023bf), the relay's debug log
(`<n>.relay.log`, the harness's `target/traces/browser.log`) and the host's
load at start and end (`<n>.load`). `summary.py` reads them:
`summary.txt` (one line per run and engine), `retakes.txt` (`--retakes`).

- **Trees**: HEAD (fb5023bf, the driver's `endpoint(..).dial_link()`), and
  1acbba2 in `/tmp/stream-old` (the driver's own `connect`), fb5023bf's
  instrument applied there uncommitted.
- **Loads**: *idle*, nothing started on purpose; **burn24**, 24 of
  `yes > /dev/null` started by `run.sh` a minute before the cell's first run
  and killed at its end. Other sessions on this host loaded it too, and
  `uptime`'s 1-minute average is in each line: the "idle" cells ran at
  3.8–24.6, the burn24 cells at 21–108. "Idle" here means not loaded on
  purpose, never unloaded.
- **Interleaved** (`x-*`): after the cells, three runs each of 1acbba2 with
  its `connect`, 1acbba2 with the driver switched to `endpoint().dial_link()`
  (drive.mjs only), and HEAD, one after another, so the three meet one load.

## What it found

| cell | load (1-min) | runs red, either engine | chromium red | webkit red | bo's key retaken after the driver's bind |
|---|---|---|---|---|---|
| old-idle | 3.8–12.7 | 0 / 10 | 0 | 0 | in 0 runs |
| head-idle | 11.0–24.6 | 10 / 10 | 5 | 9 | in 10 runs |
| old-burn24 | 21.0–46.4 | 10 / 10 | 9 | 10 | in 10 runs |
| head-burn24 | 47.7–108.2 | 10 / 10 | 7 | 10 | in 10 runs |
| x-old-connect | 18.4–26.6 | 0 / 3 | 0 | 0 | in 0 runs |
| x-old-dial-link | 19.2–31.8 | 0 / 3 | 0 | 0 | in 0 runs |
| x-head | 18.1–23.8 | 0 / 3 | 0 | 0 | in 0 runs |

1. **The reds are one thing: the page's own endpoint takes bo's key back
   from the driver's.** The page lists the lends through an endpoint it
   binds as bo and never closes (`web/app.js` `enter`, `dialler`, `bound`).
   The driver then binds a second endpoint as bo in the same page
   (`web/tests/drive.mjs` `httpChecks`). The relay keeps one active client
   per key, and it sets the older aside when another binds
   (`iroh-relay-1.0.2/src/server/clients.rs:80-92`). In every red run the
   relay logs it again 16.0–23.9 s after the driver's bind
   (`retakes.txt`). Just before, the page's endpoint ends its socket to the
   relay ("actor errored Stream terminated"), and its new socket takes the
   key. From then on the driver's connection hears nothing. It fails at its
   idle timeout, 30 s later (iroh's, for the issuer's dial, `web/src/lib.rs`
   `reach`): "no stream could be opened to the computer: timed out", "error
   reading a body from connection", or "asleep: the computer's answer is not
   HTTP". Whatever step is running at that moment fails, and every step
   after it. In the 19 runs whose checks all ended within 15 s (old-idle,
   x-*), the relay logs only the two drivers' binds.
2. **Load matters only as time.** The checks take 7–15 s when the host is
   quiet. Once they run past the retake, a run is red, whatever the build:
   at burn24 both builds are red in 10 of 10, and interleaved at one load
   (18–32) all three drivers are green with the range in 3.6–8.8 s.
   **#172 did not cause it**: 1acbba2's own `connect` bound a second
   endpoint as bo just as `endpoint()` does. The head-idle cell's slower
   ranges (8.7–16.0 s at load 11–25, against old-idle's 2.5–7.8 s at 3.8–13)
   are not the build: interleaved, HEAD's range took 4.0–8.8 s against
   1acbba2's 3.6–7.7 s. Other sessions' load changed between the cells, and
   uptime's average does not say what kind of load it was.
3. **Where each run stopped**, by its first failed assert (main.rs at
   fb5023bf), of 50 engines red: :512 (range not 206) 24, :533 (past not
   416) 8, :541 (chunked not 200) 6, :551 (post not 200) 12. It depends only on which step
   was running when the key moved: the range in a run loaded heavily, the
   upload in a run loaded lightly.
4. **The driver's stall message covers the real error.** The upload's
   30 s stall timer (`drive.mjs`, `stalled`) is started when the page holds
   1 MiB, and it is never cleared once the computer has the first byte. It
   fires 30 s later whatever happened, and the POST's failure is then
   reported in its words: "the computer had no byte of the upload while the
   page held 7143424 bytes of it" (head-idle 4, chromium). The page's own
   line says otherwise: "the stream ended in error: error writing a body to
   connection" at +48,703 ms, and the page had made 7 MiB of the upload by
   then. All 12 :551 reds read so.
5. **In a person's page, one endpoint holds a key.** `app.js` binds one
   endpoint per key (`dialler`). A lend's worker binds one as a pass's
   holder, whose key is its own (`web/lend.js:137`). The two endpoints of
   one key are the driver's alone. Two tabs of one group's page, in one
   browser, would each bind one as the same key. That case was not run
   here; it is ralph/REVIEW_FINDINGS.md #176.

### Against the predictions

1. **Idle green — falsified for HEAD, held for 1acbba2.** head-idle was red
   in 10 of 10. Its "idle" ran at 11–25 from other sessions, and its checks
   ran past the retake (2 above). old-idle, at 3.8–13, was green in 10 of 10.
2. **burn24 HEAD red in at least 3 of 10 — held** (10 of 10).
3. **The builds alike under burn24 — held** (10 and 10).
4. **The reds mostly :509 (now :512), the connection timed out — held in
   part.** :512 is the most common (24 of 50 engine-reds), with the timeout's
   words. But a step's `ms` is not near 30 s by itself: the connection dies
   30 s after the retake, not after the step began.
5. **Two endpoints of bo's key — held, with two runs I cannot judge.** Every
   red run has the key retaken after the driver's bind. Every run whose
   checks all ended within 15 s has none. The relay log does not say which
   engine's relay a line is from, since both engines' relays write to one
   file. So in head-idle 6, 8 and 9, a green chromium ending at +17.2–17.3 s
   beside retakes at +16.8–17.6 cannot be told apart from webkit's, and in
   head-idle 1–3 and head-burn24 6, 8 and 10 one engine went green past 20 s
   with one retake in the run, its own or the other's. These are
   could-not-judge, not passes. The retake is not at a fixed 16 s: across
   runs it fell at 16.0–23.9 s.

## What (b) changes, by these

The driver, not the page and not the test's wait. The reds come from a
second endpoint of the page's own key, which only the test has (5). Nothing
in the runs shows a step that worked yet came near its 60 s: the slowest
working range was 25,042 ms (head-burn24 5, chromium). So the wait stays as
it is, and widening it would not have turned one red green, since the
connection was dead. The driver speaks as a second person of the group,
under a key of its own, so the relay holds one client per key. Its stall
timer is cleared once the computer has the first byte (4).
