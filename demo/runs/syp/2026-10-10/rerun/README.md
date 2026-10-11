# The spike run again on production — 2026-10-10

syp-5-rerun-and-merge, part (a) (`ralph/next/syp/STATE.md`): sy-1's run
(`demo/runs/sy/2026-10-09/README.md` §2-§5) again, on the branch `folders`
at `af109e9`, with every row of sy §9 landed (syp-4) and perf-1's
candidates 1-7 (perf-2). The question is sy-1's: **does a group's folder, a
real directory on each member's disk, sync both ways, stay available while
its author sleeps, keep both sides of a conflict, and give no byte to
anyone outside the group** — now as production carries it — and what M1
and M2 measure now.

This file is the part's first commit: the run, the bars, what changed in
the driver, and a prediction each, committed before any data, as sy-1's
were. None of it moves after its data; what the data gives is written below
it, in later commits. One host, one run of each: not a rate (AGENTS.md 7).
A run that could not finish is could-not-judge, never a pass (AGENTS.md
6). A member that did not answer is never an empty folder, and a refusal
is never an empty answer.

## 1. The run

`run.py` here, derived from sy-1's `run.py` (loaded as a module, its setup,
`s1`-`s4`, `m1` and `m2` as they ran), with these changes, each said in its
docstring:

- `share --folder <dir> --group Shared`, the group named (syp-4 a), as
  `../production/run.py` asks;
- the readiness probe asks `GET /v1/groups`, the endpoint sy-1 named gone
  (syp-1), as `../no-stream/run.py` asks;
- **no plant phase**: since syp-2 a plant is a test's red run, never a
  switch in product code (the queue's fence 3). Each bar's plant is run
  here as its guard's test, the guard taken out by a temporary edit at
  this commit's tree, the test watched red, the edit restored, then green
  (§2), the output kept under `plants/`;
- each phase's host load, cores, swap in use and free disk at its start
  and end, in `events.jsonl`, by `../production/run.py`'s `host`;
- M2 also measures each member's store, in bytes, for each version
  (`../production/run.py`'s `stored`).

The same as sy-1: Ana, Ben and Cy, three svrngs data dirs on this host,
`target/debug/svrngs` (the workspace's dev profile, dependencies at
opt-level 2), each node through iroh's public relays and n0's discovery,
in one group `Shared`; the planted file beside Ana's `Shared` and a link to
it inside; Dee's probe (`crates/svrngs-node/tests/sy_dee.rs`); Eve joining
for M1. A difference production makes, not the driver's: a node resumes
the directories it syncs at its start (syp-4 e), so each `share` after a
restart asks again what the node already resumed.

## 2. The bars, and their plants as tests' red runs

The bars are sy-1's §3, unchanged, read for their spirit (sy-2's §8):

| Bar | sy-1's plant | Its guard's test (syp-2, `4867dcf`) | The guard taken out here |
|---|---|---|---|
| S1 both ways, each change within 10 s | `deaf` | `a_change_by_any_member_reaches_another_s_directory` | the step's changes cleared before its write (`synced.rs` `step`) |
| S2 its author asleep, Cy within 10 s from Ben | `own-only` | `a_synced_file_is_served_by_a_member_who_did_not_write_it_and_to_no_one_outside` | `serves` placing only what this node wrote (`folder.rs`) |
| S3 conflicts kept, within 30 s of Ben's start | `later-wins` | `both_sides_of_a_conflict_are_kept` | the layout read without the heads placed beside (`folder.rs` `layout`) |
| S4 members only | `open-gate` | the same test as S2's | `admit_dialer` admitting before it asks who stands (`node.rs`) |

S4's spirit's first gap in sy-2 — a member of another group of Ana's — is
not asked by this run's one group; it is held by
`a_synced_folder_is_one_group_s` (syp-4 a), run with the tests.

## 3. Predictions

From sy-1's numbers (its §7), syp-4's reruns (`../production/README.md`)
and perf-2's (`demo/runs/perf/2026-10-10-make-it-fast/README.md` §3), on a
debug build where perf-2's was release:

- **S1: pass, each change under 1 s** (sy-1: 1.06, 1.63, 1.36 s): the step
  woken by the OS's event and the apply by the record, not each a second's
  walk; perf-2's T1 164 ms on loopback, through n0's 159-163 ms, iroh
  having moved each connection direct.
- **S2: pass, about 4 s** (sy-1: 4.47 s): Cy's fetch still asks Ana first,
  the writer, and passes her over at `ANSWER_WITHIN`, 3 s, a holder that
  did not answer lately being asked last only once one has not
  (`ASLEEP_FOR`); then Ben, under 0.1 s. The risk named: syp-4's restarted
  member not dialled back in 30 s (`../production/README.md`, "Seen, not
  asked"), here Cy restarted, her round's first exchange with Ben the one
  that brings Ana's act.
- **S3: pass, under 5 s of Ben's start** (sy-1: 17.0 s): Ben's start
  resumes his directory and its sweep finds his edit, sent with its base
  before anything is applied; his first round at start brings Ana's act,
  and the apply, woken by it, places both. Which is under the file's name
  is the key order's, said. The risk the same as S2's: a round that misses
  its first exchange waits the next, 60 s, past the bound.
- **S4: pass**: Dee refused on both protocols, 0 hits in her data dir, 0
  of the planted file anywhere.
- **Each plant's test red with its guard out, green with it in**: four of
  four, at this tree as at `4867dcf`.
- **M1** (sy-1: Ben and Cy 395.27 s, Eve 242.24 s; one later change 7.1 s
  and 1,166 bytes of Eve's journal): **Ben and Cy within 5 s** of Ana's
  writes, the 1,000 taken in one or a few acts and fetched 8 at once
  (perf-2's T2 373 ms, release, loopback). **Eve within about 70 s** of her
  command, most of it waiting a round to stand (perf-2 §5.2, 54-58 s), the
  1,000 then within 5 s. Her journal about 120 KB once she holds the
  folder (syp-4's M1, 118,793); one later change at her within 1 s,
  800-1,200 bytes of her journal.
- **M2** (sy-1: first version 96.23 s, the edit 74.34 s, 104,857,600 bytes
  each): **each version at Ben and Cy within 10 s**, about 83 MB/s each from
  Ana's one upload (perf-2 §4) plus Ana's take of 100 MiB; **104,857,600
  bytes received by each** for the edit, a version as its blocks waiting
  (DESIGN §9); each store holding the edit's version, 104,857,600 bytes,
  and none of the first (syp-4 b).
- **The load** past 1.5x cores at a phase's start or end, or swap in use
  growing through it, makes that phase's numbers could-not-judge against
  these predictions (perf-1's rule); the bars, as sy-1's at load 40, are
  judged on what reached each disk.

## 4. The lines

`run.py` is not counted: `demo/runs/` is set apart by
`scripts/lean_check.py` (:52, main's lean-1, merged at syp-3). At `af109e9`: 13,982 of 15,750,
svrngs-core 2,776 of 3,000. Nothing in product code changes in this part.

## 5. The run of 23:24, against the predictions

`python3 run.py` at `6a73e3c`, exit 0 after 1m36s; its data in `run/`
(`events.jsonl`, `summary.json`, `dee.out`, each node's log). Load 5.35 on
12 cores at its start and 5.2 at its end, swap in use 7,980.19 MB at the
start and 7,972.19 at the end (not grown), 105.2 GB free: every phase
judged by perf-1's rule (`summary.json` `load`; each phase's `load`
lines in `events.jsonl`). Then S3 alone, twice, at `6a73e3c`'s tree with
`run.py`'s `s3` mode written and not yet committed (committed in
`7410f98`): `run-s3-1/` at sy's log levels, load 4.31; `run-s3-2/` with
iroh and the transport at debug (`SYP5_LOG`), load 3.66.

| Bar | Predicted | Run | Number (`run/summary.json`) |
|---|---|---|---|
| S1 both ways | pass, each under 1 s | **passed** | Ana's add at Ben and Cy 0.32 s; Ben's edit at Ana and Cy 0.16 s; Cy's delete gone at Ana and Ben 0.17 s (bound 10 s) |
| S2 its author asleep | pass, about 4 s | **passed** | Cy held `s2.bin` 3.56 s after her command, from Ben: Ana passed over at 3 s, "did not answer within 3s" (`run/cy.log:27`), then Ben's in 14 ms (`cy.log:28`) |
| S3 conflicts kept | pass, under 5 s | **failed**, 3 of 3 | 30 s after Ben's start, Ana and Cy each held Ana's edit `36f7c2fd…` alone; Ben held both, `s3.txt` Ana's and `s3 (conflict Ben).txt` his `eb066e59…` (`S3` `found`); the same in `run-s3-1/` and `run-s3-2/` |
| S4 members only | pass | **passed** | Dee's three asks refused (`run/dee.out`): the member connection "closed by peer: not admitted (code 0)", the record's ask under it `IncompleteMessage`, the bytes "closed by peer: not admitted"; 0 hits in Dee's data dir, 0 of the planted file anywhere (`dee_hits`, `beside_hits`) |

**The plants** (`plants.py`, its output in `plants/`): predicted four of
four red. At `6a73e3c` three were: `open-gate`'s test stayed green, exit 0
(`plants-6a73e3c/open-gate.out`) — since syp-4 (a) the provider asks who
stands for each get (`serves`, `crates/svrngs-node/src/folder.rs:442-458`),
so Dee was refused by it alone and the test's `is_err()` held with the gate
out. The gate had no test watching it; `57f6e82` makes the test ask the
gate's own refusal, the close at the accept ("not admitted",
`crates/svrngs-node/src/iroh.rs:453`), red with the gate out
(`tests/folders.rs:327`, "stream reset by peer: error 1", the provider's)
and green with it in. At `57f6e82` all four red, exit 101, and every
guard's test green after (`plants/green.out`):

| Plant | Guard out | Its test, red (`plants/<plant>.out`) |
|---|---|---|
| `deaf` | the step's changes cleared before its write | `a_change_by_any_member_reaches_another_s_directory`, `tests/folders.rs:254`, "Ana's file reaches Ben" |
| `own-only` | `serves` placing only what this node wrote | `a_synced_file_is_served_by_a_member_who_did_not_write_it_and_to_no_one_outside`, `:311`, "Ben serves Cy what Ana wrote" |
| `later-wins` | the layout read without the heads placed beside | `both_sides_of_a_conflict_are_kept`, `:689`, "both edits in both directories: Ana's [\"c.bin\"], Ben's [\"c.bin\"]" |
| `open-gate` | `admit_dialer` admitting before it asks who stands | the same test as `own-only`'s, `:327`, "Dee, in no group, is closed at the accept" |

**The measurements, no bar:**

| | Predicted | Run (`run/summary.json`) | sy-1 |
|---|---|---|---|
| M1, Ben and Cy hold the 1,000 | within 5 s | **0.92 s**, Ana's take in two acts of 471 and 529 (`run/ana.log:61`, `:534`) | 395.27 s |
| M1, Eve from her command | about 70 s, most a round to stand | **0.53 s** | 242.24 s |
| M1, Eve's journal once she holds the folder | about 120 KB | **128,209 bytes**, its longest line 61,783 | 133,738, 80,300 |
| M1, one later change at Eve | within 1 s, 800-1,200 bytes | **0.43 s, 1,165 bytes** | 7.1 s, 1,166 |
| M2, the first version at Ben and Cy | within 10 s | **6.0 s**, each a get of 5,132 and 5,128 ms (`run/ben.log:2108`, `run/cy.log:2105`) | 96.23 s |
| M2, the 1 MiB edit at Ben and Cy | within 10 s | **7.04 s**, gets of 6,019 and 6,011 ms (`ben.log:2111`, `cy.log:2108`) | 74.34 s |
| M2, bytes each received for the edit | 104,857,600 | **104,857,600** each | 104,857,600 |
| M2, each store after the edit | the edit's alone | **104,857,600 of the edit's, 0 of the first**, in all three, 0.63 s after (`stores`, `dropped_s`) | both kept |

Eve's prediction missed by a hundred times: perf-2's 54-58 s to stand was
its harness's, members started apart with a round each 60 s; here Eve's
`join` stands her before her `share` (`events.jsonl`: her `up` at
…982.367, her `share` answered at …982.946). M2's gets moved about 20 MB/s
to each member at once, about 40 MB/s from Ana, where perf-2's release
build on loopback moved 83 each (its §4): this run's path is not traced at
sy's log levels (`folder=info`), so whether it was loopback, the LAN
address or a relay is not in its data.

## 6. S3: what holds it

From `run/` and `run-s3-2/`, each line cited:

1. **Ben's edit reached every record.** Ben's start sent his edit with its
   base, the entry both edited (`run/ben.log:39`, "a change sent … base=
   Some(Entry { key: \"3046b7c7…\", seq: 5 …"); Ana's and Cy's layouts
   placed it beside as a conflict within 0.6 s (`run/ana.log:42`,
   `run/cy.log:41`, `conflicts=1`), and Ben wrote Ana's edit under the name
   and his beside it (`ben.log:42-44`). The record kept both: the layout's
   rule held.
2. **Its bytes did not.** Ana and Cy each asked Ben, the writer, first:
   "did not answer within 3s" (`ana.log:45`, `cy.log:42`); then each other,
   who lacked it ("stream reset by peer", `ana.log:46`, `cy.log:43`);
   "not written — no member gave it" (`ana.log:49`, `cy.log:46`).
3. **A dial to a member restarted reaches his last process.** In
   `run-s3-2/`, Ana's dial named Ben's key and no address ("connecting
   relay_url=None ip_addresses=[]", `ana.log:346`), and her iroh tried
   Ben's previous process's port, 49907 (`ana.log:361`, `remote_candidates`;
   bound at `ben.log:83`), where his running process was bound at 59318
   (`ben.log:225`) — while Ben's own connections into Ana ran direct on
   59318 (`ana.log:320`, `Ip(192.168.1.4->192.168.1.4:59318)`). Ben's
   process took no connection from anyone after its start (`ben.log`, no
   `side=Server` past 06:29:57.832, `:201`). Ana's iroh abandoned the
   paths to Ben's dead processes at 15.6 and 17.6 s (`ana.log:443`,
   `:448`, `TimedOut`). This is syp-4's "Seen, not asked"
   (`../production/README.md`), off its harness: a restarted member not
   dialled back.
4. **Nothing asked again.** A file no member gave waits for the step's next
   wake — an OS event under the directory, an act in the group's record,
   or the sweep, due each 600 s (`SWEEP_EVERY`,
   `crates/svrngs-node/src/synced.rs:59`; the loop's `select!`, `:256-286`).
   In `run/` none came within the bound: Ana fetched Ben's edit in 473 ms
   only at her own restart in S4, 30.6 s after Ben's start (`ana.log:50`,
   `:53`); Cy never did in S3.

**Why sy-1's S3 passed, at 17.0 s.** The same miss, asked again each
second: in sy-1's run Cy asked Ben for `eb066e59…` and was given nothing
at 03:35:09.9, :13.9, :17.9 and :22.0 (`demo/runs/sy/2026-10-09/run/cy.log:140-178`),
then fetched it from him at 03:35:23.1 (`:183`), each step then a walk
each second (sy-1's §2). perf-2 (d) and (f) woke the step on the record's
acts and the OS's events in place of the second (its README §1); with
them the only retry is the sweep. So S3 is two holes, each needed for the
miss: the dial to a restarted member (3), present since sy-1 and hidden by
the walk each second; and no retry of a file not written (4), from
perf-2. Either closed passes S3 on this run's evidence: retried, Ana's
iroh dropped the dead paths within 17.6 s (3), inside the bound; dialled
right, the first ask is answered.

## 7. What changed since sy-1

| | sy-1 (`ea0f3a5`, load 40.38) | syp-5 (`6a73e3c`, load 5.35) |
|---|---|---|
| S1 add, edit, delete | 1.06, 1.63, 1.36 s | 0.32, 0.16, 0.17 s |
| S2 Cy from Ben | 4.47 s | 3.56 s |
| S3 both kept | 17.0 s, passed | not within 30 s, failed (§6) |
| S4 | passed | passed |
| plants | four switches, each failing its bar | four tests, each red with its guard out; one re-aimed (`57f6e82`) |
| M1 Ben and Cy / Eve | 395.27 / 242.24 s | 0.92 / 0.53 s |
| M2 first / edit | 96.23 / 74.34 s | 6.0 / 7.04 s |
| M2 bytes for the edit | 104,857,600 each | 104,857,600 each |
| M2 the first version kept | yes, every store | no store |

Between them: syp-1 dropped streaming, syp-2 the plants, syp-4 the rows
sy §9 named (the folder one group's, the store keeping what the layout
places, the path parsed, the store kept across a stop, a node's syncing
kept), perf-2 perf-1's candidates 1-7. The host's load differs by eight
times between the two runs; the times are not compared as a rate.

## 8. What waits, with its trigger

- **S3's two holes** (§6) — a fix before the operator's merge into main,
  the operator's to queue, as one row: (i) a file no member gave asked
  again, bounded, with no event to wake it; (ii) a fetch's dial reaching a
  member's running process when its previous one is dead — whether by the
  address its own connection in names, or iroh's to change, the row's to
  find, instrumented first. Its check: S3 rerun by `run.py s3`, passed;
  and a test of svrngs's, a member restarted and asked at once, red first.
  Its trigger is this run: a bar of sy-1's failed on production.
- **A version as its blocks**: M2 moved 104,857,600 bytes to each member
  for a 1 MiB edit, as before; DESIGN §9, its trigger unchanged.
- **The record's refusal named on its answer**: Dee's ask for the record
  still breaks as `IncompleteMessage` under the gate's close (`run/dee.out`),
  as sy-2's §8, fifth part; it waits for a person refused who is told
  nothing.
- **Off one host**: every number here is three nodes on one computer;
  the operator's run with a group in the wild, DESIGN §9's row, unchanged.
