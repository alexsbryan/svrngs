# S3's two holes closed — 2026-10-11

syp-6-restart-and-retry (`ralph/next/syp/STATE.md`): the two holes syp-5's
rerun found under S3, conflicts kept (`../../2026-10-10/rerun/README.md`
§6, items 3 and 4; its §8, first item), closed before the operator's merge
into main. One host, one run of each: not a rate (AGENTS.md 7).

## 1. A file no member gave, asked again (a)

**Before:** a step whose fetch got nothing from any member waited for its
next wake: an OS event under the directory, an act in the group's record,
or the sweep each 600 s (`SWEEP_EVERY`, `crates/svrngs-node/src/synced.rs:59`
at `d4fb5bf`).

**Now** (`4a79316`): the step counts the files the layout places here that
it could not write. While that count is above 0 it is woken again after
1 s, the wait doubling at each step that leaves one unwritten, up to 15 s.
15 s is iroh's path idle timeout (iroh-1.0.2 `src/socket.rs:117`), and
syp-5's dead paths dropped at 15.6-17.6 s (`run-s3-2/ana.log:443`, `:448`
there). A step that writes every file arms nothing. Each arming and each
wake is traced at debug.

- Test: `tests/folders.rs`
  `a_file_no_member_gave_is_asked_again_with_nothing_to_wake_it`. A holder
  leaves the first get unanswered and answers the next. Red at `d4fb5bf`:
  "asked 1 times in 15.073960209s". Green: held in 4.36 s.
- perf-2's idle bar, rerun (`../../../perf/2026-10-10-make-it-fast/run/part-idle-syp6/`):
  100,000 files idle at **0.4%** of a core, under 2%, passed (perf-2's
  was 0.33%). Load was 13.92 at the start and 7.51 at the end. The node's
  log has no retry line.

## 2. A member restarted, reached (b)

**Instrumented first** (`31750ee`): a fetch's dial now traces at debug the
addresses iroh holds for the key (`Endpoint::remote_info`), and which are
in use. The cause, from iroh 1.0.2's source and syp-5's `run-s3-2`:

- iroh keeps one state per remote key. While it has a path in use, it
  sends each new connection's packets to that path alone
  (`src/socket/remote_map/remote_state.rs:799-809`). It also starts no
  address lookup then (`:867`).
- That path stays in use until it idles out at 15 s, or until every
  connection to the key closes (`:486-488`).
- svrngs waited on Ctrl-C alone (`crates/svrngs/src/cli.rs:1050` at
  `d4fb5bf`), so a SIGTERM ended the process with its endpoint open.
  syp-5's driver stops nodes with SIGTERM (`../../../sy/2026-10-09/run.py:38-39`),
  as a service manager does.
- So Ben's dead process's connections into Ana stayed open on her side.
  His dead port stayed in use, and her dial of his next process went
  there (`run-s3-2/ana.log:361` there).

No change inside iroh was needed.

**Now** (`c50d7f2`): `svrngs_node::node::asked_to_stop` waits on Ctrl-C or
SIGTERM, and the command then closes its endpoint, as it did after Ctrl-C.
Each peer is told that each connection is closed, and its iroh forgets the
dead path.

- Test: `tests/kept.rs` `a_member_restarted_is_answered_at_once`. Ana's
  process is stopped by SIGTERM and started again, and Cy asks her at once,
  within 3 s.
- Red at `d4fb5bf` in three runs of three, with Cy's iroh holding Ana's
  dead port as Active. In one run it held that port alone: no lookup runs
  while a path is in use.
- Red again with the plant, `asked_to_stop`'s SIGTERM branch taken out.
  Green: 1.72 s, and the kept suite passed 8 runs of 8.

## 3. S3 rerun, by syp-5's driver

`run.py <n>` here loads `../../2026-10-10/rerun/run.py` unchanged and
runs its `s3` mode, with `folder` at debug. The data is in `run-s3-<n>/`.
The runs were at `c50d7f2`, 12 cores:

| Run | Load start / end | S3, both edits in all three directories | Judged |
|---|---|---|---|
| 1 | 12.62 / 14.17 | **0.34 s**, passed | yes |
| 2 | 14.90 / 15.31 | **0.37 s**, passed | yes |
| 3 | 16.13 / 17.80 | **0.65 s**, passed | yes |

Swap stayed at 5,644.12 MB at every start and end. syp-5 had failed 3 of 3,
not within 30 s.

- In each run, Ana and Cy fetched Ben's edit `eb066e59…` from his
  restarted process at the first ask. It took 5 to 72 ms
  (`run-s3-1/ana.log:98`, `cy.log:71`).
- "did not answer" appears 0 times in any log.
- Ben's first process said "asked to stop by SIGTERM" before its restart
  (`run-s3-1/ben.log:49`).
- The dial's trace still shows Ana's iroh holding Ben's dead port beside
  his live one, both Active (`run-s3-1/ana.log`, "dialled by key"). With
  no path in use after his connections closed, iroh sends to each, and the
  live one answers.

## 4. What waits

- **A process killed outright** (`kill -9`, a crash, a computer losing
  power) still leaves the dead path in use at its peers, for up to 15 s.
  (a)'s retry, capped at 15 s, asks again after that. Its time to a fetch
  is unmeasured on production. It waits for a member restarted that way
  who waits on a file past S3's bound.
- **Off one host**: DESIGN §9's row, unchanged.
