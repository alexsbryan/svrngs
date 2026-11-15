# Linux's WebKit stalls a film it loaded whole, 2026-11-15

Why `playback::a_lent_film_plays_and_seeks_under_its_worker::webkit`
failed on Linux in every run: Fedora round 1 (`demo/runs/flakes/2026-11-12/`,
C1, "never moved from its last seek"), the gate image at 953f5f42, and 3
runs of 3 alone in the gate image at bb46c17c. The film played in its first
browser and, once the browser was opened again, stopped at 0.167 s, not
paused, not seeking; the seek after it stopped at 52 s.

## What was measured

`web/tests/page.mjs` `play` now says, of a play that missed, the
`<video>`'s readyState, networkState and buffered ranges. The test's miss
was `{"at":0.167,"buffered":[[0,60]],"network":2,"ready":2}`: the whole
film held, the decoder at its fourth frame (24 fps), and no frame after.

`probe.mjs`, in the gate image (Playwright 1.59.1, WPE MiniBrowser): no
svrngs, no worker, a plain origin on 127.0.0.1 answering the same film
(the test's ffmpeg line) by range, in `<video preload="auto">`; iPhone 15
as the test, a persistent profile, play, seek 40, close, open again, play,
seek 52. `runs.txt` has every run.

- It stalls as the test does, at 0.167 s, the first play included, in most
  launches and not all: the first launch in one container played and every
  later one stalled (out3); in another the first stalled too (out4). The
  profile, a fresh profile, an ephemeral context, a new page or the first,
  desktop or iPhone, a click before: each stalled. Removing GStreamer's
  registry changed nothing.
- `stalled-trace.txt` (GST_DEBUG `webkitmediaplayer:4`): the download
  finished at 0.158 s ("Stream is fully downloaded, stopping monitoring
  downloading progress"), playback began at 0.195 s, and at 0.362 s WebKit
  paused "for buffering", after which no buffering message came, so it
  never restarted. `buffering-trace.txt`, a launch that played: the same
  pause, then a buffering message at 100%, "Restarting playback". Which
  comes first is a race inside WebKit's on-disk buffering (its
  `downloadbuffer`, "Enabling on-disk buffering").
- `preload="metadata"`, which takes no on-disk buffering: 2 runs of 2,
  every play (out6).
- `preload="auto"` with `WPE_SHELL_DISABLE_MEDIA_DISK_CACHE=1`, WebKit's
  own switch ("Should disable media disk cache: true", "Disabling on-disk
  buffering"): 5 runs of 5, every play and seek in 269-324 ms (out7).

## What it means

The stall is WebKit's on Linux, with nothing of svrngs in it. The tests'
WebKit stands in for a phone's Safari, which plays through AVFoundation and
has no such cache. So every WebKit the tests start on Linux starts with the
switch (`page.mjs` `starting`), and the `<video>` keeps `preload="auto"`,
whose ranges to the end the test asserts (#53). In the gate image, with
it, the playback step passed both engines 5 runs of 5.
