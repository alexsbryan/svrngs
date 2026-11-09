# The swarm runs: Lessons from BitTorrent, §6 of the brief

`docs/LESSONS_FROM_BITTORRENT.md`, steps 0-5, one run each (AGENTS 7), on the
Fedora box (`ssh alexbryan@fedora`, 32 cores, kernel 7.2.8) in `/dev/shm`.
Load 16-21 on the box throughout from another build (`uptime`, recorded at each
start; the shaped path is the limit in these runs, not CPU). `run.py` is the driver as it ran; `data/`
holds the summaries (`results.txt`) and the raw pings. A verdict below is one of
passed, failed, could-not-judge, never-ran.

## How to run again

```
# On Fedora, each member a network namespace; rootless (`unshare -Urnm`).
# A veth to a bridge in `hub`, tbf rate 40mbit at 500 ms buffer
# (burst 50 KB, limit 2.5 MB), netem 20 ms on the hub side: 40 ms RTT.
cargo test --release -p svrngs-node --test perf --no-run   # bin/base, bin/bd, bin/plant
python3 run.py one  cubic      bin/base          # step 1, 1 GiB, ping beside it
python3 run.py one  bbr3       bin/bbr3
python3 run.py quic q-cubic    bin/diag cubic    # step 1's diagnostic (perf_quic.rs)
python3 run.py quic q-bbr3     bin/diag bbr3
python3 run.py wave wave-today bin/base
python3 run.py wave wave-bd    bin/bd
python3 run.py slow slow-bd    bin/bd
python3 run.py slow slow-plant bin/plant --bound 300   # plant.patch applied
PERF_SIZE=20000000 PERF_DIR=/dev/shm/swarm/s0 target/release/deps/perf-* --ignored --exact perf_partial --nocapture
```

`bin/base` is `847e991b` with `tests/common/member.rs`'s `lan` path. `bin/bbr3`
adds `parts/part-a.patch`. `bin/bd` is parts B and D on HEAD with no part A
(`parts/part-b-on-part-d.patch` on part D). `bin/plant` is `bin/base` with
`plant.patch`: today's `hold`, the throttled holder first in its order.

## Verdicts

| Step | Observation | Number | Bar | Verdict |
|---|---|---|---|---|
| 0 | A member fetches from another that stopped mid-get | 10,190,848 of 20,000,000 bytes, exactly the held range (`perf_partial`: `observed_equals_store`, `arrived_exactly`, `bytes_match`) | Exactly the held ranges, verified | **passed** |
| 0's plant | A flipped byte in the half-holder's data file | The provider resets the stream (`error 3`), the chunk is not held; a clean 4 MiB beside it arrives (`control_unflipped_range`) | Rejected | **passed** |
| 1 | 1 GiB over a 40 Mbit/s uplink, Cubic (the plant) | ping p95 261 ms, idle 40.1, bar 140.1; 230.4 s | Cubic must miss the ping bar | **failed the bar, as a plant must** |
| 1 | The same, BBR3 | ping p95 261 ms, idle 40.1; 236.2 s (+2.5%) | Under 140.1 ms | **failed** |
| 2 | Lend stream and 100 MiB rerun | — | No regression | **never-ran**: part A did not land |
| 3 | 8 computers, 256 MiB, 40 Mbit/s uplinks, `bin/bd` | **201.0 s**; today's `hold` **399.8 s**, ratio 0.50 | Under a third of today's (under 133.3 s) | **failed** |
| 3's plant | Today's `hold` must miss the bar | 399.8 s, arithmetic 376 | | **missed, as a plant must** |
| 4 | Step 3 with one holder (p5) throttled to 400 Kbit/s | **186.2 s**; p5 gave 6 gets, one to each fetcher, `no slice within` 3 | Under the bar (133.3 s); at most one slice per fetcher | **failed** the time, **passed** the one-slice trace |
| 4's plant | Today's order, the throttled holder first (`bin/plant`, `plant.patch`) | Not one of 6 fetchers had the file at 300 s, the bound | Must miss the bar (133.3 s) | **missed, as a plant must** |
| 5 | Writer's bytes unavailable, line appears and goes | `a_file_only_on_a_computer_that_is_off_is_said_and_goes_when_it_arrives` | Line names the writer, goes on arrival | **passed** |
| 5's plant | A disk failure must not produce the line | `a_file_a_disk_would_not_take...` red with a write failure answered `NoHolder` (tests on `bin/bd`'s tree on Fedora) | Red | **watched fail**, then passing |

## What was found

**BBR3 does nothing here.** Under Cubic and BBR3 the ping's p95 is the same
261 ms (`data/results.txt`). `perf_quic.rs` (a bare QUIC stream, `run.py quic`)
names which controller ran and what it did each second: Cubic's cwnd reaches
12 MB, BBR3's grows past 800 MB, neither loses a packet. With a cwnd that size,
what bounds in-flight is iroh's stream receive window
(`noq-proto-1.0.1/src/config/transport.rs:550`, 12.5 MB/s × 100 ms = 1.25 MB),
which on a 5 MB/s uplink is about the 221 ms of queue both runs show. One run
of each, so this reads the path that was run, and the window was at this point an inference
from the arithmetic and the cwnd (measured in its own section below). **The brief's fallback,
LEDBAT, costs about 90 counted lines and slows lends with it, so it is the
operator's.** A smaller `stream_receive_window` is the alternative it would
decide with: it shortens the queue and caps a long, fat path.

**Slices halved it and did not reach a third.** Writer-first `hold` 399.8 s
against the sliced `hold_sized` 201.0 s, the bar 133 s. The writer still sent
184 of 448 slices (41%; the brief's arithmetic wants about one in seven) and
one get at a time per file caps a member's download at one holder's upload
(`data/results.txt`, per-holder counts from `fetched, verified`).
Part B's selection is not the brief's: a holder by the slice's place among
those covering it, not fastest measured, and a holder that fails is passed
over for the fetch; a 400 Kbit/s holder cost 15 s and one slice per fetcher
(step 4's trace), not the 80 s the sketch feared. It did not land: 93 counted
lines over the 35 the operator gave the package, and short of its own bar.
`parts/part-b-on-part-d.patch` lands it on top of part D if the operator
wants it anyway. Its plant was watched fail: the provider's observe arm
answering without `serves` hands Cy, who stands in no group placing the hash,
the bitfield (`a_bitfield_is_told_only_as_a_get_would_be_served` red on the
Cy assertion, then green with the check back).

**Step 0 passed**: iroh-blobs 0.103 serves exactly what a stopped fetch holds,
verified, from a store it was never finished in. Slicing has what it needs,
and the fallback of serving only from whole holders was not needed.

Part D is the one that lands: its plants are above, its counted cost 27 `.`
lines and 5 JS, and its line, "3 files are still only on Ana's computer.
They'll arrive when it's back on.", is on the home page beside the folders.
The 512 KiB window measured below landed too (+6), in part A's place.

## Not done, and why

- Step 2's lend stream and perf-2's 100 MiB rerun: never-ran, since part A did
  not land.
- A lend's stream was never measured under BBR3 (no browser film test was run
  here); moot with part A not landed.

## The folder suite at 4 test threads

`a_hash_the_record_stopped_placing_is_refused_at_the_next_get` failed at
`RUST_TEST_THREADS=4`, 3 of 3, passing at 2 and 12 and passing alone. Neither
Part D's code nor its two new tests is the cause; they only shifted the
schedule. Instrumented on a copy of the tree (an `eprintln` in the provider's
refused branch, the test's subscriber set printed) the failing runs read:

```
SUBSCRIBER-SET thread="a_hash_the_record_stopped_…"   tv_nsec: 360350193
REFUSED-BRANCH thread="a_hash_no_synced_folder_…"     tv_nsec: 398511401
REFUSED-BRANCH thread="a_hash_the_record_stopped_…"   tv_nsec: 533873464
```

The neighbour test, which sets no subscriber, reaches the refused trace's
callsite first. With one live dispatcher tracing-core asks only the calling
thread's (`tracing-core-0.1.36/src/callsite.rs:545,564`, `has_just_one`,
`Rebuilder::JustOne`), so that callsite is cached as never wanted and the
test's own later refusal is not captured, though its branch ran. In the one
instrumented run where the test's own refusal came first, it passed. So the
capture lost the line; the refusal was never missing. The cause was already
known and fixed elsewhere (`tests/common/mod.rs` `heard`, a second dispatcher
kept alive; `tests/removal.rs:337`); the two hand-rolled captures in
`tests/folders.rs` predate it. Both now use `heard("folder=debug")`. After the
fix: 27 passed at 4, 2 and 12 threads, once each. No retry, sleep or pinned
thread count.

## The stream receive window, measured

The inference above, that iroh's stream receive window holds the queue, as a
measurement. `parts/blobs-receive-window.patch` sets
`stream_receive_window` to 512 KiB on the fetcher's Blobs dial only
(`iroh.rs` `dialled`, a per-dial config; lends keep theirs), 6 counted lines,
not applied to the tree. Same shaped uplink as step 1, Cubic, once each:

| Run | Ping p95 | Bar (idle+100) | Fetch | Bar (25% slower) | Verdict |
|---|---|---|---|---|---|
| (a) one fetcher, 1 GiB (`w-one`) | **110 ms** (idle 40.1) | 140.1 | **230.7 s** (step 1's Cubic 230.4) | 288 s | **passed** both |
| (b) four fetchers of that uplink at once (`w-four`) | **430 ms** | 140.1 | **914.1 s** slowest (step 1 scaled ×4 is 921.6) | 1,152 s | ping **failed**, fetch passed |

The inference held: at one fetcher the queue is about window minus BDP
(524,288 − 200,000 B over 5 MB/s = 65 ms, plus 40 ms, predicted 105, measured
110), and at four about four windows (predicted 420, measured 430), where the
default 1.25 MB window gave 261. So the bound is fetchers × window: a smaller
per-dial window fixes one fetcher and does not fix several at once. Bounding
the sender's total needs its own decision.

What 512 KiB costs on a path, by arithmetic, per stream: window ÷ RTT is
26.2 MB/s (210 Mbit/s) at 20 ms and 6.55 MB/s (52.4 Mbit/s) at 80 ms.

Connection-level `receive_window`, by reading: it defaults to `VarInt::MAX`
(`noq-proto-1.0.1/src/config/transport.rs:558`), so it is no limit. Eight files
at once (`FETCH_AT_ONCE`, `synced.rs:53`) are eight streams on the one kept
connection to a holder (`iroh.rs` `kept`), each granted its own window: 8 ×
512 KiB = 4 MiB, bounded on the sender only by `send_window`, 10 MiB
(`transport.rs:559`). So eight files from one holder queue about eight
windows too. Setting `receive_window` on that dial as well would cap the
connection's total, one more line, and was not run.
