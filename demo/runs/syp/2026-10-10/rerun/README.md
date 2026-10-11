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
