# Where a synced folder's time goes, at scale — 2026-10-10

The perf spike (`ralph/next/syp/STATE.md`, perf-1-diagnose-and-propose; the
operator, 2026-10-10: "I'd like to prioritize one spike (just a row or two)
focused on diagnosing and proposing solutions to make performance at scale /
bounds closer to \"blazing fast\""): **where does a group's synced folder
spend its time, change by change and scale by scale, and what would make it
fast?** Part (a) measures and splits each number into the phases of a
change; part (b) proposes, from that data, what makes each target.

This file is part (a)'s first commit: the question, what is known, the
targets, the phases, the paths, the scales, the load and disk rules, how
each number is taken, and a prediction per target — committed before any
trace or run of the spike, at `607add5`. None of it moves after its data;
what the data gives is written below it, in later commits. One host, one run
of each: not a rate (AGENTS.md 7). A run that could not finish is
could-not-judge, never a pass (AGENTS.md 6). A member that did not answer is
never an empty folder, and a refusal is never an empty answer.

## 1. What is known, cited

From sy-1's run (`demo/runs/sy/2026-10-09/README.md` §7, its data in
`run/`) and syp-1's rerun (`demo/runs/syp/2026-10-10/no-stream/`), one host,
through iroh's public relays, each node `target/debug/svrngs`
(`demo/runs/sy/2026-10-09/run.py:14`), the host at load averages of 40-54 on
12 cores (`run/events.jsonl:1` of each):

| What | Number | Where |
|---|---|---|
| a small change at another member's disk | 1.06-1.67 s (sy), 1.13-1.67 s (syp-1) | sy §7 S1; syp-1 README |
| a file whose author is asleep, at a member who starts | 4.47 s (sy), 4.27 s (syp-1), 3 s of it `ANSWER_WITHIN` | sy §7 S2, `run/cy.log:76`; `crates/svrngs-node/src/node.rs:103` |
| 1,000 files of 4 KiB at two members, and at one who joins later | 395.27 s, and 242.24 s; one fetch at a time, 0.38 s a file and 0.25 s | sy §7 M1 |
| one 100 MiB file at two members | 96.23 s | sy §7 M2 |
| a 1 MiB edit inside it | 74.34 s, the whole 104,857,600 bytes moved to each | sy §7 M2 |
| one later change's record | 1,166 bytes of a member's journal, in 7.1 s | sy §7 M1 |
| a conflict kept | 17.0 s (sy), 17.42 s (syp-1) after the stopped member's start | sy §7 S3; syp-1 README |

What the code says of where it goes, read at `607add5`, not yet run:

- **The walk, once a second.** Each synced directory is walked, then
  applied, then the node sleeps a second (`both_ways`,
  `crates/svrngs-node/src/synced.rs:53-65`). A change waits for the next
  walk on its writer's node, and again for the next step on each member's.
  Each step reads and writes the whole kept state, `synced-<folder>.json`
  (`synced.rs:73-77`, `:128`), and stats each file twice (`walk`,
  `crates/svrngs-node/src/folder.rs:122`; `unchanged`, `synced.rs:38-44`).
- **The record read whole, often.** A synced folder's layout is every
  group's admission read from its journal on disk, every line parsed
  (`placed`, `synced.rs:68-70`; `admissions`, `folder.rs:106-111`;
  `RingRail::admission`, `crates/svrngs-runtime/src/lib.rs:181-184`, through
  `Journal::read`, `crates/svrngs-runtime/src/journal.rs:236-241`, and
  `admit`, `:250-259`). It is read once a step by the apply
  (`synced.rs:142-146`) and once for **every get a provider is asked**
  (`shares`, `folder.rs:179-182`), so a member fetching n files makes its
  holder read the journal n times.
- **One fetch at a time, a piece at a time.** The apply fetches each file
  after the last and each file's pieces of 1 MiB (`PIECE`, `folder.rs:44`)
  one after another (`synced.rs:173-204`); each piece is one get on a
  stream of the holder's connection (`hold`, `fetch`, `folder.rs:192-270`),
  each get asked of the record by the holder's provider before a byte is
  sent (`provider`, `folder.rs:293-299`).
- **A version by its own hash.** A file's new version is fetched by its
  hash, whose tree the member's store never held, so nothing of the old
  version counts (sy §8, M2).
- **The writer first, then each member, 3 s each.** A fetch asks the
  entry's writer first and waits up to `ANSWER_WITHIN`, 3 s, on a holder
  that does not answer, then the next (`hold`, `folder.rs:206-236`); one
  that did not answer within 60 s is asked last (`:207-212`).
- **The record carried by the round.** A write nudges the round
  (`folder.rs:174`), which dials every member at once
  (`run_one_round`, `crates/svrngs-node/src/ring_sync.rs:283-398`); a body
  past 8 MiB is refused at the receiver (`MAX_REQUEST_BODY_BYTES`,
  `crates/svrngs-node/src/peer_wire.rs:33`), and one act is sent whole even
  when it alone is past the 4 MiB budget (`RING_SYNC_OPS_BUDGET_BYTES`,
  `peer_wire.rs:51`; `ops_missing_from_within`,
  `crates/svrngs-core/src/sync.rs:408-413`).
- **The build.** sy-1 and syp-1 ran debug builds (`run.py:14`); this run
  builds release, so its numbers and theirs differ by the build too, said
  wherever one is set beside the other.

## 2. The targets

What "blazing fast" means here: each a number set before the data, the
seat's from the operator's words and the operator's to change. Each is
judged **on a direct path** (§4):

| # | Target |
|---|---|
| T1 | a small change (4 KiB) at another member's disk within **300 ms** |
| T2 | 1,000 files of 4 KiB at a member within **10 s** |
| T3 | one 100 MiB file at **100 MB/s** or more |
| T4 | a 1 MiB edit inside 100 MiB moving **at most 2 MiB** to a member |
| T5 | a folder of 100,000 small files (1 KiB): a change **found within 1 s**, and an idle node **under 2% of a core** |
| T6 | ten members, each holding one change within **1 s** |

Through a relay each is reported, not judged: n0's relays are rate-limited.
Measured with no target: **the record's bytes per change**, and **a member
joining the 100,000-file folder**.

## 3. The phases of one change

Each a debug trace of svrngs's own (AGENTS.md 1), added after this commit
with behaviour unchanged (AGENTS.md 2), so each number splits into them:

| Phase | From | To | Where |
|---|---|---|---|
| P1. found on disk | the file's write | the writer's walk that finds it | the step's walk |
| P2. hashed and stored | the walk | the file in the writer's store, by hash | `take` |
| P3. the act written | stored | the act committed to the record | `write` |
| P4. the record carried | committed | the act ingested by the member | the round; the member's `ring sync: exchange` (`ring_routes.rs:146-156`) |
| P5. the member's apply | ingested | the member's step reading the layout and choosing the fetch | the apply |
| P6. the fetch | chosen | verified in the member's store: its **dial** (a new connection), **first byte** and **last byte** | `hold`, `fetch` |
| P7. the write | verified | the file in the member's directory | the export |

Beside them, inside P5 and P6, **the record read**: the time each
admission takes to read (the apply's, and the holder's provider's per get).

## 4. The paths

Each number names the path iroh reports for its connection — the selected
path of `Connection::paths()` (iroh-1.0.2 `src/endpoint/connection.rs:1144`,
`is_relay`/`is_ip`), traced at the fetch — never assumed:

- **direct on loopback**: no relay, no n0 lookup, each node told the
  others' loopback addresses (as `crates/svrngs-node/tests/folders.rs`).
- **a relay on this host**: iroh's relay on 127.0.0.1, as the conformance
  canary's (`crates/svrngs-node/tests/conformance.rs:62-80`), no n0, no
  address told; iroh may still move the connection direct, and says so.
- **n0's public relays**: iroh's defaults with n0's lookup, as `svrngs`
  runs (`crates/svrngs/src/main.rs:46`), no address told.

## 5. The scales

Files 1, 1,000, 10,000 and 100,000 (4 KiB each to 1,000, 1 KiB each past
it); sizes 4 KiB, 1 MiB and 100 MiB; members 3 and 10; two writers at once.

## 6. How each number is taken

**The instrument.** Each member is a node of its own, a process on this
host with its own data dir and directory, started by the run's driver
(`demo/runs/perf/2026-10-10/run.py`, derived from sy-1's `run.py`) from an
ignored test of svrngs-node's that runs one node on the path asked
(`crates/svrngs-node/tests/perf.rs`), built in release. The group is set up
as `tests/folders.rs` sets it (one seed, the founder's admits); each
directory is shared as `share --folder` asks the node (`POST /v1/folders`).

- **A directory's state**, by the driver polling each directory: each
  expected file's size and BLAKE3 against what the step wrote, every 20 ms
  (every 250 ms past 10,000 files, said with the number). A time is from the
  write returning to the first poll that matches in every directory named,
  or the bound passing (then the number is the bound, failed).
- **The phases**, from each node's log at debug: each phase's line and its
  timestamp, joined by path and hash to the change it carries; a number's
  phases are the median of its changes' where it has many.
- **The path**, from the fetch's trace; a number whose fetches took more
  than one path names each, with its share.
- **Bytes moved**, from each fetch's payload and other bytes (`hold`'s
  line, `folder.rs:221`).
- **The record's bytes per change**: a member's journal's size before and
  after one change.
- **An idle node's processor**: its process's CPU time (`ps -o cputime`)
  over 60 s with nothing changed, after it holds the folder, divided by 60.
- **T3's rate**: the file's bytes over P6 (first dial to last byte), and the
  end-to-end time beside it.

**The load.** Each number with the host's load average, its cores
(`sysctl -n hw.ncpu`, 12) and its swap in use (`sysctl vm.swapusage`),
taken at its start and its end. A number taken with the 1-minute load above
1.5x the cores, 18, at its start or end, or while swap in use grew during
it, is **could-not-judge** against its target (AGENTS.md 6) and kept as its
phases' shares. This host's swap was 23.0 of 24.6 GB in use at 19:48Z,
when browser tests timed out (the row).

**The disk.** The run's scratch is at most 6 GB, removed after; no run
starts under 20 GB free; `df` before and after each pass, in its data.

**Two passes.** A breakdown pass at small scales first, whatever the load,
for the phases' shares (each number in it could-not-judge if its load says
so); then the targets' pass, started through `ralph-result await` behind a
wait for the 1-minute load under 18 and 20 GB free, up to 12 h, its start's
load and free disk in its data — never a wait in a session.

## 7. Predictions, one per target

Release build, on the direct path, the host under 18:

- **T1: missed, about 1.0 s** (0.3-2.0 s). P1 and P5 are each a wait for
  the next second's step, half a second each on average; P2-P4, P6 and P7
  together under 100 ms on loopback for 4 KiB. The two waits hold about
  90%.
- **T2: missed, about 30 s** (10-90 s). The writer takes the 1,000 in one
  or two acts in a few seconds; the member fetches them one after another,
  each a get its holder asks of the record first. P6 holds most of it, and
  in it the holder's record read; under sy-1's 0.25-0.38 s a file by the
  build and the path.
- **T3: met or near, 50-150 MB/s over P6**; end to end about 3 s with P1,
  P2 (hashing and copying 100 MiB into the store) and P7 (exporting it).
  One get per 1 MiB piece, in turn, is the risk: a round trip and a record
  read each.
- **T4: missed, 100 MiB moved** — 104,857,600 bytes, as sy-1's M2: nothing
  of the old version counts.
- **T5: both missed.** The walk stats 100,000 files twice, the step reads
  and writes a kept state of 100,000 entries and reads the record whole
  (§1), each second: a step of 2-5 s, so a change found in about 3-8 s, and
  an idle node at 60-90% of a core, the steps back to back but for the
  second's sleep.
- **T6: missed, about 2 s** (1-4 s): each member's change waits for its
  writer's walk, each member's apply for its own step, and each member
  fetches nine files one after another from nine holders, a dial each.
- **Through a relay** (reported): T1 and T2 slower by the relay's round
  trips, T3 by its rate; on this host iroh may move each connection direct.
- **The record's bytes per change**: about 130 bytes in an act of many
  (sy-1's 133,738 for 1,000), about 1,200 for an act of one (sy-1's 1,166).
- **A member joining the 100,000-file folder: not within 2 h.** The act of
  100,000 changes is about 13 MB, past the 8 MiB a receiver takes, so the
  writer's push is refused and the member holds the act only if its own
  round pulls it (§1); then 100,000 fetches, each a read of a 13 MB journal
  by the holder.

## 8. The lines

The traces, about 50 lines counted by `scripts/lean_check.py`'s rule
(`python3 scripts/lean_check.py` at `607add5`: 13,383 of 15,750). The
instrument (`crates/svrngs-node/tests/perf.rs`) is a test and the driver is
under `demo/runs/`, neither counted.

## 9. What the run gave

Two passes, each `run.py`'s, its data under `run/<pass>/`: `events.jsonl`
each event as it happened, `summary.json` each number with its load at
its start and end, its path and its phases, and each node's log at debug
(gzipped past 5 MiB, `run.py:423`).

- **The breakdown pass** at `9ea632a` (`run/breakdown/events.jsonl:1`):
  load 21.17 at its start, 12.92 at its end, 161.9 GB free; `df -k`
  158,078,580 KiB available before and 157,838,216 after.
- **The targets' pass** at `6a1d3ac`, through `ralph-result await`, its
  wait past at once (`run/targets/events.jsonl:1`): load 7.75 on 12 cores,
  swap 5,920.56 MB in use, 160.4 GB free, `df -k` 156,663,644 KiB
  available; at its end, 2 h 44 min later (the last line): load 12.89,
  swap 6,279.56 MB, `df -k` 156,143,668 KiB; the scratch, `/tmp/perf-run`,
  removed. Its largest group's scratch was 1,075,620 KiB (`t-relay`'s
  `scratch` event), under the 6 GB.

The load rose past 18 within the pass — the pass's own nodes and this
host's other work, not told apart — so most targets are **could-not-judge**
by §6's rule, kept as their phases' shares. Swap in use did not grow
during any judged number.

**Two things of the instrument, before the numbers.**

- **The nodes tick together.** Each group's nodes start within half a
  second of each other (`up` events), and each step is a few
  milliseconds then a second's sleep (`synced.rs:57-61`), so every node's
  step lands at the same point of the second. Around three of T1's
  changes the member's step ended 13, 24 and 29 ms before the writer's act
  was written, and the writer's next step 1,006-1,008 ms after
  (`run/targets/t-loopback-p0.log`, `-p1.log`, the steps around the
  second to fourth `written to the record`). So P5 here is nearly a whole
  second, where between computers started apart it would fall anywhere in
  the second, half of one on average. P1 is the driver's cadence against
  the writer's tick, a median of 65 ms. T1's sum is near its ceiling, not
  its mean: the shares below hold, the totals are the worst of the second.
- **Every path was direct.** Every fetch on every path names a direct
  address as iroh selected it: `127.0.0.1` on loopback and on this host's
  relay, `192.168.1.4`, this host's LAN address, through n0's
  (`summary.json`, each number's `paths`). iroh moved each relayed
  connection direct before the fetch, so no number here measures a byte
  through a relay; the relay and n0 numbers are direct paths reached
  another way, each reported, not judged (§2), and each above 18's load
  besides.

**Against the targets** (`run/targets/summary.json`, keyed by the name
given; each on loopback, direct, unless said):

| # | Measured | Load, start-end | Verdict | Predicted (§7) |
|---|---|---|---|---|
| T1 | **1,182 ms**, the median of 10, 1,087-1,468 (`T1-loopback-median`) | 9.38-14.2, swap flat | **failed** | missed, about 1.0 s (0.3-2.0): held |
| T2 | 24.3 s (`T2-loopback`); 20.1 s on the relay's (`T2-relay`), 16.4 s through n0's (`T2-n0`) | 14.2-26.29 | could-not-judge | missed, about 30 s (10-90): within it |
| T3 | 11.8 MB/s over P6, 11.07 s end to end (`T3T4-loopback-rate`); 13.4 and 11.1 on the others; 15.2 MB/s for 10 MiB in the breakdown pass at load 15.17 (`loopback-10m-rate`) | 25.53-25.93 | could-not-judge | met or near, 50-150 MB/s: **wrong**, a tenth of it |
| T4 | **104,857,600 bytes** moved to each member for a 1 MiB edit, on all three paths (`T3T4-*-edit`) | 25.93-22.84 | could-not-judge | 100 MiB moved: held |
| T5 | a change found in 993, 2,950 and 1,821 ms (`f100k-found#0-2`); idle **58.2%** of a core (`f100k-idle`) | 24.33-27.17; 39.94-26.68 | could-not-judge | found in 3-8 s: lower, 1-3 s; idle 60-90%: just under |
| T6 | 1,640, 1,500 and 2,372 ms, ten members (`T6#0-2`) | 19.63-20.46 | could-not-judge | missed, about 2 s (1-4): held |

Measured with no target: two writers at once, each change at the other
members in 1,811, 1,410 and 1,457 ms (`two-writers#0-2`). **The record's
bytes per change**: 739-741 bytes of a member's journal for an act of one
(`T1-loopback-median`'s `record_bytes`; predicted about 1,200, sy-1's
1,166, so lower), and 119 bytes a change in an act of many, 1,193,305
bytes for 10,003 changes and 11,903,305 for 100,003 (the `journal`
events of `f10k` and `f100k`; predicted about 130). **A member joining**:
10,003 files in 688.7 s (`f10k-join`, load 25.37 rising to 63.58); the
100,000-file folder **did not finish within its 2 h bound**, 16,238 of
100,003 held at it (`f100k-join`, its value null) — predicted not within
2 h: held.

**Where each goes** — the phases' medians over each number's changes
(`summary.json`, each number's `phases`), and what the traces split
beyond them:

- **T1: the two waits for a step.** P5 958 ms (91.7%), P1 65 ms (6.3%);
  P2-P4, P6 and P7 together 21 ms — the act 7.7, the record carried 11.2,
  the fetch 2.0 (its first byte 2.4 ms after the ask). The same on the
  relay's path (1,216 ms, P5 86.6%) and n0's (1,224 ms, P5 86.0%).
- **T2: the holder's record read, once a get, one get at a time.** Each
  member took the 1,000 files one after another, 23.3 s from the first
  `to fetch` to the last `written` at each, a file each 13.3 ms at the
  median, 23.4 on average; of the 23.3 s, 19.0 s inside `hold`
  (`fetch_ms` summed, `t-loopback-p1.log`; the same at p2). Each 4 KiB
  get's first byte came 11.8 ms after the ask, its last 12.1: what holds
  a get is before its first byte. There the holder reads the record whole
  for each get (`shares`, `folder.rs:181-183`): 6.5 ms at the median,
  2,000 gets, 13.5 s summed (`T2-loopback`'s `holder`), in one task that
  takes one get after another (`provider`, `folder.rs:313-327`), so the
  two members' gets wait on each other's. P5 is 87.3% of the median
  change: a file's wait behind those fetched before it. P1 is 1.24 s: the
  writer taking the 1,000 before the act.
- **The record read grows with the record.** The holder's read per get:
  0.9 ms at 100 files (`loopback-100-files`), 6.5 at 1,000, 50.6 at
  10,000 (a journal of 1.19 MB), 399.6 at 100,000 (11.9 MB) — linear in
  the journal, so a member fetching every file of a folder costs its
  holder the files times the journal: quadratic. It held 532.5 s of the
  10,000-file join's 688.7 (77%), and 7,067 s of the 100,000-file join's
  7,200 (98%) (`f10k-join`'s and `f100k-join`'s `holder`).
- **T3: a get a MiB, a store asked between each.** P6 81.7% of the
  change, 8.85 s for 100 MiB at the member p1: 100 gets of 1 MiB one
  after another (`PIECE`, `folder.rs:46`; the loop, `synced.rs:210-215`),
  5.14 s inside them — 48.5 ms each at the median, its first byte at 9.6,
  of which the holder's record read was 6.7 (200 gets, both members, 1.43
  s summed) — and **3.69 s between them**, 36 ms at the median from one
  get's end to the next's start: `hold`'s ask of the store what of the
  blob it lacks (`local_for_request`, `folder.rs:202`) and the loop,
  split no further by a trace. Inside a get, 1 MiB came in about 39 ms
  after its first byte, about 27 MB/s at that load (about 32 ms, 33 MB/s,
  at load 15 in the breakdown's 10 MiB): the transport and the two
  stores, not split by these traces. P1 829 ms, P5 866, P2 (hashing and
  copying 100 MiB in) 219.
- **T4: a version by its own hash.** 104,857,600 bytes of payload to each
  member, P6 81.7% of the edit's 10.5 s (`T3T4-loopback-edit`): nothing
  of the old version counted, as §1 read.
- **T5: the step, linear in files, once a second.** An idle step of the
  100,000-file folder took 1,952 ms at the median: the kept state read
  195, the walk 576, the files stat'ed again 540 (`unchanged`,
  `synced.rs:93`), the apply 524 (the record read whole and its layout,
  `synced.rs:166-170`), the kept state written 116 (`f100k-idle`'s
  `steps`). At 10,000 files, 204 ms (`f10k-idle`). An idle node's share
  of a core is about the step over the step and its second's sleep:
  1.95/2.95 is 66%, measured 58.2; 0.204/1.204 is 17%, measured 16.1.
  A change is found at the next step, so within a step and a second.
- **Taking 100,000 files: the apply's match, quadratic.** The share's
  first step took 196.6 s (`f100k-taken`): 98.4 s taking the files one
  after another, about 1 ms each, and **95.3 s in the apply** — each file
  taken matched against every placed entry (`placed.iter().find(mine)`,
  `synced.rs:190`), 100,000 by 100,000. At 10,000 files the apply took
  589 ms (`f100k-p0.log.gz`, `f10k-p0.log.gz`, the first `a step`): ten
  times the files, 162 times the time.
- **The 100,000-file act, refused and pulled.** The writer's push of the
  11.9 MB act was refused, 413 (`f100k-p0.log.gz`, 21:35:32); the
  member's own round pulled it 13 s later (`f100k-p1.log.gz`, 21:35:45,
  `ops_pulled=1`) — as §7 predicted. A joining member's first fetch came
  58.3 s (10,000) and 63.8 s (100,000) after its start: the round's
  minute (`DEFAULT_RING_SYNC_INTERVAL`, `ring_sync.rs:104`).
- **T6: the round to nine.** P5 58.6-68.8%, P1 12.6-40.9%, and P4 —
  the act carried to nine members — 233 and 201 ms in two of the three
  (15.6%), against 11 ms to two.
