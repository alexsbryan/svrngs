# Making a synced folder fast — 2026-10-10

perf-2-make-it-fast (`ralph/next/syp/STATE.md`): perf-1's candidates 1-7
(`demo/runs/perf/2026-10-10/README.md` §10), each a part with its
falsifier, then the targets' pass again. The operator chose 1-6 as one row,
7 with `notify`, and 8 — a version as its blocks — to wait for this rerun
(DESIGN §10). The targets, the phases, the load rule and the instrument are
perf-1's (that README §2, §3, §6), the driver copied unchanged at `e5a9847`
and changed after only as each commit says (`run.py`). One host, one run of
each: not a rate (AGENTS.md 7).

Each part's data is under `run/part-<case>-<label>/`, the targets' passes
under `run/targets/` and `run/targets-at-fix/`, each `events.jsonl` and
`summary.json` with the 1-minute load at each number's start and end. No
number below was taken with the load at or past 18 or swap in use growing:
each is judged.

## 1. The parts, each against its falsifier

| Part | Candidate | Falsified if (perf-1 §10) | Measured | Data |
|---|---|---|---|---|
| (a) | one get a file, `ANSWER_WITHIN` bounding the next byte | P6 for 100 MiB on loopback not under 5 s | 1,302.5 ms, load 9.95 (perf-1: 8.85 s); the test `a_holder_slower_than_the_bound_to_its_last_byte_still_answers` red first (`d1d24f5`) | `part-p6`, `7603cf0` |
| (b) | the apply's match by path | the 100,000-file take's first step over 1 s in its apply | 264 ms (perf-1: 95.3 s) | `part-take-b`, `2651140` |
| (c) | the record read once a change | the holder's `record_ms` at 1,000 files not under 0.5 ms, or T2 not under 10 s | 0.1 ms a get, T2 2,087 ms (perf-1: 6.5 ms, 24.3 s); the test `a_hash_the_record_stopped_placing_is_refused_at_the_next_get` red with the digest's check out (`556bd0d`) | `part-t2-c`, `4ea2ec9` |
| (d) | the apply woken by the record | T1's P5 not under 50 ms | 3.8 ms, T1 290 ms (perf-1: 958 ms, 1,182) | `part-t1-d`, `3946793` |
| (e) | files taken at once | 100,000 taken 8 at once not 4x faster at the same load | 1.28x and 1.38x back to back; 32 at once no faster: **falsified**, the code kept, faster at each load; held by iroh-blobs' one database actor (`4ff2452`) | `part-take-e-*` |
| (f) | the step on the OS's file events | at 100,000 files idle over 2% of a core or a change found past 1 s | idle 0.33%, found 193.2, 72.8, 73.3 ms (perf-1: 58.2%, 993-2,950 ms); the tests of a save by rename and of a change found at the start's sweep red first (`14aeeb4`) | `part-idle-f`, `08fb51f` |
| (g) | fetches at once, 8 | the 10,000-file join within 60 s; if one at a time meets it too, the bar a quarter of one at a time's | 8 at once 1,853-1,916 ms, one at a time 5,093-5,494 (perf-1: 688.7 s): both met 60 s, so the bar became 1,328 ms, and 8 at once **missed it**, 2.75x faster, not 4x (`df8f62c`) | `part-join-g-*` |

Two parts missed their bars, (e) and (g), each at what lies past svrngs's
step: 8 or 32 imports at once, and 4 to 32 fetches, each end at about
120-160 us a file. An import ends through iroh-blobs' one database actor
(0.103 `store/fs.rs:542-544`); for a fetch that actor on the member's side
is the likeliest, unmeasured (`df8f62c`). Lines: 13,730 of 15,750 at syp-4's close (`c4aeda6`'s
body), 13,982 now (`python3 scripts/lean_check.py` at `41cee42`), +252
against perf-1's about +160.

## 2. The targets' pass again, and what it found first

The first pass at `2d6035b` (`run/targets/`, `5030440`) met T1, T2, T5 and
T6 on loopback, but through n0 the 1 MiB edit of T4 **never reached a
member in 900 s**. Not slowness: the writer's step took its own file again
2,087 times, each time the old hash (`targets/t-n0-p1.log`). Traced
(`77be89f`) and reproduced (`run/part-p6-h-probe/`): the step stat'ed a file
mid-rewrite at 0 bytes, then took it whole at 100 MiB, so the act said a
100 MiB hash was 0 bytes; no later stat ever matched what it held, and the
edit after it was never written. iroh-blobs' own import has the same race
for a file at or under its inline bound, and panics where debug asserts
(`fs.rs:1025`). The fix (`de20272`): `take` stats again after the take and
leaves a file that moved for its write's own event; a small file is read
here, at most a byte past its stat. Its test,
`a_file_written_as_it_is_taken_is_sent_as_taken_and_the_next_edit_arrives`,
red at `2d6035b` (3 of 3 debug panics, 2 of 2 release misses), green after.

The pass again at `3a5a64b` (`run/targets-at-fix/`, `51438f0`): load 3.76 at
its start, 14.63 at its end, 109.4 GB free; 20 min 44 s. The re-take is
gone: `grep -c 'changed since it was last held'` is 0 in its
`t-n0-p1.log` and `t-n0-p2.log`.

## 3. Against the targets

Each on loopback, direct (`targets-at-fix/summary.json`, keyed by name),
against perf-1's measured (its §9) and the prediction with 1-7 (its §10,
"Predicted, with 1-7"):

| # | Target | Measured | Load at start | Verdict | perf-1 | Predicted |
|---|---|---|---|---|---|---|
| T1 | 300 ms | **164 ms**, median of 10, 115-196 (`T1-loopback-median`) | 2.73 | **passed** | 1,182 ms | about 100 ms, met: met, slower — P1 65.8 ms, 66.7%, of it 50 the step's `GATHER_FOR` after the OS's first event (`synced.rs:63`) |
| T2 | 10 s | **373 ms** (`T2-loopback`) | 2.61 | **passed** | 24.3 s | about 2 s, met: met, five times under |
| T3 | 100 MB/s | **83.4 MB/s** per member, two at once (`T3T4-loopback-rate`); one alone 139.3 (§4) | 3.97 | **failed** | 11.8 MB/s | unknown, the get's own rate |
| T4 | 2 MiB moved | **104,857,600 bytes** to each member (`T3T4-loopback-edit`) | 4.53 | **failed** | the same | missed until 8: held |
| T5 | found in 1 s, idle under 2% | found in **431.6, 73.9, 73.6 ms**; idle **0.33%** (`f100k-found#0-2`, `f100k-idle`) | 2.63-3.06; 3.12 | **passed** | 1.0-2.9 s, 58.2% | met: held |
| T6 | 1 s | **515.2, 512.0, 504.1 ms**, ten members (`T6#0-2`) | 4.81-4.82 | **passed** | 1.5-2.4 s | 0.3-0.5 s, met: met, just past the range |

Through this host's relay and n0's, reported (§2 of perf-1): T1 159 and 163
ms, T2 463 and 448 ms, T3 83.0 and 34.8 MB/s, the edit 100 MiB moved on
each, through n0's in 3,790 ms. As in perf-1, iroh moved every connection
direct before its first fetch (`127.0.0.1` on the relay's, `192.168.1.4`
through n0's; each number's `paths`): none of these measures a byte through
a relay.

Measured with no target: two writers at once, each change at every member in
136.5, 140.1 and 261.4 ms (`two-writers#0-2`; perf-1 1,410-1,811). The
record's bytes per change: 739-741 for an act of one, as perf-1. Taking
100,000 files: 19,854.5 ms (`f100k-taken`; perf-1 196.6 s), 18,563 of it the
take, 274 the apply. A member joining, from its share accepted to its disk
holding every file: 10,003 in **1,713 ms** (`f10k-join`; perf-1 688.7 s),
and 100,003 in **61,530 ms** (`f100k-join`; perf-1 did not finish within
2 h, 16,238 held).

## 4. T3: iroh's or svrngs's

perf-1 named the measurement: one get of 100 MiB between two iroh-blobs
stores on loopback; under 100 MB/s there, T3 is iroh's. Two bare `FsStore`s
moved it at 137.5, 136.7 and 136.0 MB/s (`P6-bare-loopback#0-2`, load
3.69). One svrngs member alone, fetching from one holder, moved it at
**139.3 MB/s** over P6, 753 ms (`run/part-p6-h-one-fetcher/`, `41cee42`,
load 2.89): a get through svrngs is a bare get.

T3 as the pass measures it is two members fetching the one holder's file at
once: each asked within 6 ms of the other and each wrote it within 2 ms
(`targets-at-fix/t-loopback-p1.log:4561`, `-p2.log:4562`, `:4566` in both),
83.8 and 83.4 MB/s each, about 167 MB/s from the holder. What holds T3 is
neither svrngs's code nor one get's rate: it is the holder's upload, split
between the members asking it, on one host whose twelve cores carry all
three. A member could fetch from another member that already holds part of
the file, not only from its author; nothing here measures what that would
gain, and on one host it would share the same cores.

## 5. What no prediction named

1. **A file taken as it was written** (§2): a race between the step's stat
   and its take, present since the spike, found only when a 100 MiB rewrite
   landed between them through n0's path. Fixed and tested in this row.
2. **A joining member waits a round to stand.** Its share is refused until
   the round admits it: 57.5 s at 10,000 files and 54.2 s at 100,000
   (`f10k-join`'s and `f100k-join`'s `stand_ms`), about the round's 60 s
   (`DEFAULT_RING_SYNC_INTERVAL`, `ring_sync.rs:104`) — now the join's
   largest wait by far, outside the numbers above, which start at the
   accepted share.
3. **The 100,000 join is 36x the 10,000's for 10x the files** (61,530 ms
   against 1,713). The holder's record checks summed 29,697 ms over 100,003
   gets, 0.2 ms at the median (`f100k-join`'s `holder`), against 2,175 over
   10,003; not split further here.
4. **T6 is now the round to nine**: P4, the act carried, 191-266 ms, 47-63%
   of each change (`T6#0-2`'s `phases`), where perf-1's was P5.

## 6. What waits

- **A version as its blocks** (candidate 8): T4 still moves 104,857,600 bytes
  on every path. It waits in DESIGN §9 with its trigger.
- **T3 past one holder's upload**: no candidate; DESIGN §9, with its trigger.
- **Imports and fetches past 4 at once**: the store's one actor, iroh-blobs'
  (§1); a store of svrngs's own is not proposed.

## 7. The row's close

`scripts/check.sh` at `6428a1a` exited 1 on one gate: the app nouns,
`SETTLE` (`synced.rs:63`, from (f)) an app's noun; renamed `GATHER_FOR`
(`4fe1e39`). At `4fe1e39` it exited 0, every check passed: principles
pinned, lean and RUST_ARCH, app nouns gate, app nouns, rustfmt, clippy,
tests, canaries, ceiling canary, web rustfmt and clippy, browser, joining
in a browser, conformance canary, app laws. `python3 scripts/lean_check.py`:
13,982 of 15,750, svrngs-core 2,776 of 3,000.
