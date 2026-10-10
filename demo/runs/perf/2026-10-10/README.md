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
