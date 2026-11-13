# check.sh's flakes, diagnosed (2026-11-12)

The operator: "once and for all diagnose the check.sh flakes, and to test as
much as we possibly can without the flakey bits. And where we need to test
exactly that playback started we do it in containment and not in 10
different tests."

What follows: the recorded failures, the measured runs and their load, each
cause with its evidence, the playback inventory, what changed, and what is
left open with its cause. Raw files are beside this one.

## 1. Recorded failures

`recorded-failures.tsv`: 307 rows over 156 test names, swept from
`demo/runs/**`, `ralph/next/*/ctl/log/sessions/*`, `/tmp/svrngs-check.*` and
commit bodies. Of those, 157 were real breaks fixed by a later commit, 44 a
flake (the same commit passed elsewhere or on a rerun), 34 the environment
(podman machine down, Playwright not installed, a cold release build), 5
load, 60 unknown. No failure of `page integration` or `pages` is recorded.
The recurring flakes, by distinct runs:

| Test | Runs | Mechanism the record names |
|---|---|---|
| `a_page_speaks_http_through_the_door_streaming_both_ways::webkit` | 14 | "the range must come back 206"; never reproduced in 18 planted-load runs (demo/runs/streaming-stall/2026-10-23) |
| `on_a_phone_with_the_wildcard`, `on_a_computer` (conformance) | 12, 6 | mostly cascades (open files, b482e083; a second tracing subscriber, ec6fb890); the rest **seeks** played again in 14,843 ms at load 35 (f435980d), a 60 s timeout on 11-08 |
| films scene | 3 | **Alex's film stalled** 51 s at load 128 and 132; passed at ~60 (0ce95b69 made it could-not-judge past BUSY) |
| `a_lent_film_plays_and_seeks_…::webkit` | 2 | "each **seek** must play again within five seconds" |
| `carried::an_app_one_computer_brought…` | 5 | a 15,000 ms bar missed by 9–705 ms under load |
| `the_home_page_lends_what_runs_here::chromium` | 5 | named known flake, passed alone (f86a155a) |
| `guests::sam_opens_the_link_kim_sent…::webkit` | 2 | Sam's first request 403, 2 in 5 alone at e5cd75fc |
| `leave_race`, grapevine step 12, the redial instrument | 3, 1, 1 in 10 | named races (5b46435e, 2da49698) |
| conformance (Mac, 2026-10-08) | 1 | **the binary aborted**: `noq-1.0.1` endpoint.rs:648 "attempt to subtract with overflow" (ralph/next/compose/ctl/log/sessions/mn-2-verdict-1.out:1041) |

Media playback is in four of the eight recurring rows.

## 2. Measured

The Fedora box (32 cores, `podman` gate image, `localhost/svrngs-gate`),
from a clean clone at fc357324 with Eudoxia 506388e, each round the slow
steps of check.sh run through check.sh's own `step` (`round.sh`, `launch.sh`;
per-test verdicts by `tally.py`), load and free memory sampled every 5 s.

Round 0 (32 cpus, the image as built):

| Step | Verdict | Why |
|---|---|---|
| page integration | failed 12 of 13 | the image has no redis-server: environment |
| tests | failed | 14 `give` tests, podman absent in the image (never-ran in all but name); `publication_refuses_redirects_oversized_bodies_and_timeouts` |
| browser | **aborted** at 218 s | noq-1.0.1 endpoint.rs:648 overflow, then a panic in a destructor: every test in flight failed with it (11) |
| joining, pages, scenes, conformance | passed | give and holyarrs scenes never-ran (podman) |

Load peaked at 413 (1-min) with 1.1 GB free: the browser step at 32 test
threads drove the host into swap. Rounds from 1 on ran with `--cpus=12`,
the Mac's cores.

Round 1 (12 cpus): peak load 28.5, 1.2 GB free at least.

| Test | r00 | r01 |
|---|---|---|
| browser `a_lent_film_plays_and_seeks_…::webkit` | (aborted) | **failed**: the last seek, after the browser reopened, never played again (`{"at":52,"ms":null,"paused":false,"seeking":false}`) at load 23.8 |
| browser `acts::a_phone_s_page_reading_past_the_rate_…::webkit` | failed | **failed**: 1000 of 1000 reads taken |
| browser `carried::an_app_s_entry_is_approved_…::webkit` | (aborted) | **failed**: `opened.cookies` 1 though the signer logged it deleted |
| browser `opens::a_carried_app_reads_the_lends_it_declares_…::chromium` | ok | **failed**: three heads where two were sent, the third empty |
| tests `publication_refuses_redirects_…` | failed | ok |
| tests `machine::a_running_small_machine_is_left_running_…` | ok | **failed**: its podman stub's log never written |

Rounds 2 to 5 never finished: during round 2, with a reproduction
container beside it, the box stopped answering (tailscale "offline" from
about 01:30 UTC 13 November), came back once after a reboot and dropped
again. Its free memory had been 1.1 to 1.2 GB at the least in each round
(an earlier load's GPU-held memory left about 6 GB to all of it), so this
measuring is the likeliest cause. The rounds' raw files are on the box,
in ~/flake/r00 and r01, not copied here before it went down; the tables
above are what `tally.py` printed from them. So there are no rates from
repeated runs here, only two rounds and the reproductions in §3: each
cause below is shown by a plant or a reproduction, not by a rate.

## 3. Causes

Each cause named with where it is, and the evidence that shows it rather
than guesses it.

**C1. Media decoding is judged in many tests, and it is what varies.**
`web/tests/page.mjs` `play` waits for `currentTime` to move; four browser
tests, the conformance canary and three scenes waited on it or its kin (§4).
A decoder's progress depends on the host's load and on the media stack:
Fedora round 1, WebKit (GStreamer on Linux) played the film in 639 ms and
then, after the browser reopened, never moved from its last seek
(`{"at":52,"ms":null,"paused":false,"seeking":false}`) at load 23.8 of 36;
the Mac's records show the same seeks miss under load (f435980d, 0ce95b69).
The bytes were answered either way. Shown by plant
(`mac/plant-undecodable.log`): the film's first 64 KiB zeroed, so its bytes
are answered as kept and nothing can decode them. The four byte-level tests
passed; both engines' playback tests failed, Chromium with
"DEMUXER_ERROR_COULD_NOT_OPEN", at load 8.4 under the bar of 36.

**C2. A port probe of one test reaches another test's stub (Linux only).**
`share::listening` (crates/svrngs-node/src/share.rs:498) connects to every
port from 1024 to 49151, from `GET /running` (home.rs:649), which tests in
the same binary call: home.rs `looking_for_what_runs_here_…`, and the
browser binary's `the_home_page_lends_what_runs_here`. On Linux the
ephemeral ports, 32768-60999, sit inside that range; on a Mac they are
49152-65535, outside it. A stub that counts every connection as a client
then sees a stranger: the publication test's `elsewhere` listener took a
probe for a followed redirect (publication.rs:565, "invitation must never
follow a resolver redirect"), and opens.rs's films origin kept an empty
head as a third request (opens.rs:715, 3 where 2). Reproduced on the
Fedora image (`repro-publication.sh`): 5 of 5 passed alone, 5 of 5 failed
beside a prober over 32768-60999. On the Mac, a prober over its own
ephemeral range (49152-65535), where its stubs listen, failed it 3 of 3
(`pub-mac.sh`, `mac/publication.out`): the same mechanism, which a Mac's
gate never meets only because nothing probes that range there.

**C3. The QUIC library aborts the whole test binary (both platforms, rare).**
`noq-1.0.1` src/endpoint.rs:648, `active_connections -= 1` on a
connection's `Draining` event, underflowed: in a debug build that panics
while holding the endpoint's lock, and `EndpointDriver::drop` (:495)
unwraps the poisoned lock, a panic in a destructor, so the process
aborts and every test in flight fails with it. Fedora round 0, browser step
at load ~400 (11 tests failed with it); the Mac's conformance step on
2026-10-08 at load 25-33 (ralph/next/compose/ctl/log/sessions/mn-2-verdict-1.out:1041).
`EndpointDriver::drop` zeroes the counter (:501) and noq-proto 1.0.1
emits `Draining` from seven places (connection/mod.rs:2409, 4396, 4408,
4528, 4858, 5546, 6970); one path that would emit a second for one connection,
read and not run: `kill()` (:6970) on a connection already drained calls
`move_to_drained(Some(_))`, which reports `was_draining` false from
`Drained` (state.rs:76). noq 1.4.0 moves
the emission into the state transitions but keeps the same unguarded
decrement (endpoint.rs:651). In a release build the counter wraps instead,
and `wait_all_draining` (endpoint.rs:410), which iroh's close awaits
(iroh-1.0.2 socket.rs:1164), would never return. Not reproduced at will:
two occurrences, both under load. Open (§6).

**C4. A rate test assumed the reads were fast.** acts.rs asserted 600 to
999 reads before a 429; the door lets in 600 at once and ten a second
(door/limit.rs:159, GCRA), so a page under 25 reads a second takes all
1,000. Fedora rounds 0 and 1, WebKit: 1000 of 1000, both rounds.

**C5. A cookie was read before the browser's store had the deletion.**
web/sign.js:89 deletes the handoff cookie by `document.cookie` in the
signer's frame; approve.mjs asked Playwright's cookie store at once, which
in WebKit still held it: `opened.cookies` 1 beside the signer's own
"deleted its cookie" (Fedora round 1).

**C6. A stub program written and run at once (Linux, candidate).**
apps/give/tests/machine.rs `recording_with` writes a shell script and has
the code under test run it; round 1's `a_running_small_machine_…` found
its log never written (machine.rs:184, NotFound), so the script never ran.
The candidate: Linux refuses to run a file another thread's forked child
still holds open for writing (ETXTBSY), as cargo's parallel tests fork; a
Mac does not. Not reproduced before the box went down: could-not-judge.

**C7. The gate image could not run three steps.** No redis-server, so
`page integration` failed 12 of 13 ("Redis did not start"); Rust 1.94.1
while rust-toolchain.toml pins 1.95.0, so rustup fetched 1.95.0 each run
without the wasm target and every page build failed ("the wasm target is
not installed"); and podman is not in the image, so the give and holyarrs
scenes and the give tests never-ran there. And `--network=host` shares the
host's ports: the Fedora host's own Jellyfin listens on 8096, the browser
tests' Jellyfin port. Not flakes, but each read as red there.

**What the records name and these runs did not see.** The streaming-both-ways
206 (14 recorded runs, not reproduced in 18 planted-load runs, nor here),
`leave_race`, grapevine step 12, guests' first-request 403: none failed in
the rounds this box gave before it went down. They stay as the records say.


## 4. Playback inventory

`playback-inventory.md` has every wait on a decoder with its file:line, its
timeout, its verdict on a miss and what its test owns apart from it. In the
gate there were 12 test functions waiting on playback through 7 instruments
at about 17 call sites:

- browser step, WebKit: the runtime test (first play, 3 seeks, play again
  after the browser reopened), both hand-over tests (play, play again),
  guests (play);
- conformance canary: the `seeks` route (audio), in both ways and engines;
- scenes: films (Jellyfin play, 6 seeks, a watch of the other's film), night
  (the TV's play, pause, seek, play on), holyarrs (play and `seek(50)`, both
  engines).

The same claim was made four times ("a lent film plays at its origin through
the lend's worker"), three times ("it plays again after the worker
restarts"), seven ("a seek plays again within 5 s") and four ("Jellyfin's
player starts").

## 5. What changed

Every commit below, and every Mac run of them, built with Eudoxia 83d482a
at `../eudoxia` (the path dependency); the Fedora rounds, fc357324 with
Eudoxia 506388e.

| Commit | What | Shown by |
|---|---|---|
| 8153e41d | install.rs: the native arrival test's imports on every platform; on Linux `tests` did not compile | the gate image's build, exit 0 after |
| 76e29f22 | acts.rs's read rate bounded by the run's length, could-not-judge when the page never read past the rate (C4); approve.mjs waits for the signer's cookie deletion to land, 10 s at most (C5) | `mac/plant-rate-cookie.log`: no reads counted, 2,000 taken, acts.rs:275 red; cookie left, carried.rs:927 red in both engines |
| 8e9d6260 | publication.rs: a probe that sends nothing is not a followed redirect (C2) | Fedora 5/5 red beside a prober, 5/5 green alone; Mac 3/3 red, then 3/3 green with the fix; the client following redirects, 2/2 red (`mac/publication.out`) |
| 34cfa969 | gate image: redis-server, Rust 1.95.0, eudoxia mounted, its own network with the Mac's ephemeral range (C2, C7) | the sysctl read 49152-65535 inside; the image itself not rebuilt yet |
| 99473e23 | the `playback` step: browser/playback.rs (both engines) and conformance.rs `seeks_back_to_back`; `film_answered` in the runtime, hand-over and guests tests; films and holyarrs assert their streams (person.mjs `streamed`) (C1) | `mac/plant-undecodable.log`: an undecodable film failed only the playback tests; `mac/plant-range.log`: a range one byte off failed only the byte tests; `mac/plant-seeks.log`: no audio failed Chromium's seeks, WebKit's a rate; `mac/plant-films.log` |
| c7b14e09 | give machine.rs says its refusal when its stub never ran (C6, an instrument) | |
| 3bfbb4ef | holyarrs H8's pattern as WebKit writes the cancelled report's URL, "https: /…" | the full gate below; the pattern against the recorded message true, against another path false |

What the playback step gives where: on the Mac both engines judge it. On
Linux, Fedora round 1 played WebKit's film through the worker and missed
the seek after the reopen at a load under the bar: there it fails, as on
a Mac at the same load. It is not marked could-not-judge for Linux, since
nothing here shows the miss is the media stack's rather than ours; the
`playback` step is where that would now be seen, and alone.

### The full gate, once, on the Mac

`mac/gate-c7b14e09.txt`: scripts/check.sh under the gate lock in a clean
detached worktree at c7b14e09 (eudoxia 83d482a), 1,922 s, **exit 1**.
`playback` passed in 110 s, nothing could-not-judge; the conformance canary
passed in 56 s; the WebKit seeks rate: 4 of 4 through a lend within 5 s
this run, one of four straight at the canary not (19,177 ms), at loads of
12 to 15 on 12 cores. What failed, each with its cause:

- **browser, 6 tests**, every one a stub that a lend is read from seeing
  more than was asked (empty heads; "cy's shelf asked 633 times, 622
  taken"; "cy's door must name bo"). Not flakes and not these commits:
  all six fail alone at 8153e41d (Eudoxia 83d482a), before any of them
  (`mac/six-at-8153e41d.log`), and at c7b14e09 (Eudoxia 83d482a,
  `mac/six-at-c7b14e09.log`); at fc357324 (Eudoxia 506388e) on the Fedora
  image three of them passed both rounds. That comparison changed both
  repositories, so it does not isolate a cause: svrngs-core takes Eudoxia
  by relative path (crates/svrngs-core/Cargo.toml, `../../../eudoxia`),
  and between the two runs are both svrngs's abf5b0de ("R1 through
  Eudoxia's rules", another session's) and Eudoxia's own drift from
  506388e to 83d482a. **Cause, neither: this run's own prober.**
  `fl/pub-mac.sh`'s `probe()` (the C2 reproduction, started 17:51:13)
  outlived its `kill $pid` and connected to every port in 49152-65535,
  the Mac's ephemeral range where every browser test's stub listens, until
  it was found as an orphan (pid 5974, parent 1, ~330% CPU) and killed at
  19:46. A connection that sends nothing is the empty head the six count.
  Every failing run began after 17:51 (this gate 18:17-18:49, both six-at
  runs ~18:53); the R1 session's browser step before 17:51 passed; the
  Fedora rounds had no prober; abf5b0de's diff adds no connections.
  **Confirmed**: the next full gate, without the prober, at 263d9753 plus
  the R3/R4 change (landed as 0d203da7; its commit body), exited 0 under the
  lock with the six passing, as did films, playback (197 s), the give tests
  and `tests` (so meet.rs:240 did not recur). The films 401 and
  meet.rs:240 below ran beside the prober too.
- **scenes**: holyarrs::phone::webkit at H8, this run's own pattern bug,
  fixed in 3bfbb4ef; films, Bo's `/svrngs-open` answered 401 at once
  ("open_ms": 9) and his browser never let in, which the films run alone
  on the same tree passed (`mac/scenes.log`): not explained, seen once;
  give_gone and give_move never-ran, podman's socket refusing its ssh
  handshake for those minutes (`podman info` answered again after).
- **tests**: the give tests, the same podman moment; and
  meet.rs:240 `every_relay_down_a_join_ends_bounded_and_leaves_nothing_running`,
  "something still dials after the close", 161 connections where 113 were
  counted at the close: seen once, no record of it before, cause not
  found (the stalled relays count every accept, so a dial in flight at the
  close, or a relay actor iroh keeps, would both read so).

## 6. Open

- **night: decided.** The operator chose to move the TV's playing, and
  Ana's pause and seek of it, into the playback step (2026-11-12): night.rs
  `night_played`, run by `playback` alone, its bars `late` as before; the
  scenes step's `night` asserts the TV received Ana's film from the room's
  address and asked nothing else off it but jellyfin-web's Chromecast
  sender script (`night-held.patch` was its start). Verified on the Mac,
  under the gate lock, in a clean worktree: `night` alone passed in 140 s
  (`mac/night-alone.log`); planted with One's id for Two's film, so the
  TV's stream never came, it failed at night.rs:227, "the TV asked no
  stream of it within 60 s of Play" (`mac/night-plant.log`); check.sh's
  `playback` step alone, `night_played` in it, passed in 225 s
  (`mac/night-playback-step.log`). Its never-ran line, run on planted
  outputs, said never-ran for a filter that matched no scene and for a
  scene that never ran, and nothing for three halves run.
  The step's limit is now 5 times 225 s.
- **noq's abort (C3).** An upstream bug in noq 1.0.1 (and its decrement
  unchanged in 1.4.0): to be reported to n0-computer/noq with the two
  traces above. A version of noq that guards the counter, or a
  `[patch.crates-io]` with a saturating decrement, is the fix; neither
  is made here.
- **The give machine test (C6)**: instrumented; ETXTBSY unconfirmed.
- **holyarrs H8 on WebKit** (after 99473e23, twice at load ~4): Bo leaving Jellyfin while its player's chunk still loads gives a `ChunkLoadError`; expected at that step by 16c7d8d9, not yet seen green under load.
- **The six browser failures** (above): this run's orphaned prober, confirmed by 0d203da7's gate (exit 0, the six passing).
- **films' 401 and meet.rs:240**, each seen once in the full gate.
- **Rates.** No before/after rate: the box went down in round 2 (§2).
  Measured on the Mac instead, once each: the changed tests unplanted,
  the plants, and the full gate (§5).
- The image rebuild, with Redis and Rust 1.95.0, waits on the box.

