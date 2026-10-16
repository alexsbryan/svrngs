# arch-view: what the view cost a journal and a commit, before and after

Row arch-view (`ralph/next/arch/STATE.md`). Predictions in `PREDICTIONS.md`
(daea976d), committed before any data. The instrument is
`crates/svrngs-node/tests/scale.rs` (1d6bf599), run by `run.sh`, one run a
point, in release, on this host (12 cores). One run is not a rate
(AGENTS.md 7): the bytes are structural; each time is one reading of one
host, its load in `data/<label>/load.txt` (3 to 5 before, 3 to 9 after).

- `data/before/` — daea976d's tree: the product as at c276c34f, the row's
  start, which stamps every act with its writer's view.
- `data/after/` — 53a35336's tree: 8ab83940's change, a commit signs no
  view.
- `data/survey.json` — `survey.py /private/tmp`, the journals left there.
- `b/` — the change's red and green runs; `run-maker-*/` — the two-build run
  (`mixed.py`).

## The view's share of a journal, and a commit's time

| point | share before | bytes a line, before → after | `append_s` before → after | `setup_s` before → after | `push_round_s` before → after | `max_rss` before → after |
|---|---|---|---|---|---|---|
| 2 × 1,000 | 0.392 | 890 → 541 | 0.050 → 0.010 s | 0.20 → 0.22 s | 0.032 → 0.025 s | 50 → 43 MB |
| 5 × 1,000 | 0.604 | 1,366 → 540 | 0.054 → 0.010 s | 0.36 → 0.35 s | 0.081 → 0.039 s | 84 → 58 MB |
| 25 × 1,000 | 0.878 | 4,406 → 537 | 0.085 → 0.012 s | 1.81 → 1.67 s | 1.02 → 0.17 s | 549 → 170 MB |
| 25 × 10,000 | 0.882 | 4,575 → 541 | 0.791 → 0.064 s | 5.20 → 3.08 s | 10.06 → 1.40 s | 5,230 → 1,066 MB |

Bytes a line is `oplog_bytes / journal_acts`; after, `view_bytes` is 0 at
every point. The journal at 25 members and 10,000 acts: 45,973,506 bytes
before, 5,438,109 after.

Against the predictions:

1. **The share at the start**: 0.392, 0.604, 0.878, 0.882, each inside its
   range (0.37–0.42, 0.58–0.64, 0.86–0.90). Held.
2. **The survey**: 131 journals under `/private/tmp` (the glob reaches past
   the six levels the prediction's `find` counted 116 at), 541,279 of
   1,297,584 bytes views: 0.417 of all, 0.353 the median journal's (0.159 to
   0.559), no view naming more than six keys. Held: the audit's 25–39% is
   inside this spread, from groups of two to six on this host's runs.
3. **At the close**: no view byte written, a line 537 to 541 bytes whatever
   the members. Held; and A_RECORD_FOR_YEARS §3's bar 1, a record of about
   two hundred bytes of payload under 1 KB at 25 members, read here at 537.
4. **A commit's time**: 0.791 s at the start at 25 × 10,000 (0.55–0.95
   predicted), 0.064 s at the close (0.02–0.20 predicted), twelve times
   faster; at 1,000 acts five, five and seven times faster. Held. What a
   commit still does over everything held, read in the code and not timed
   apart, is copy it (`command.rs:176`, `:199`).
5. **The setup's time lower at every point**: falsified at 2 × 1,000, 0.20 s
   before and 0.22 s after. At 2 members the setup is two nodes starting and
   converging, where the commit is a few milliseconds of it; one reading
   each, under a load of 2.8–4.0 after against 3.2 before, does not read a
   difference of 8%. Lower at the other three, 41% at 25 × 10,000.

Not predicted: the round that moves five new acts to 24 members took 10.06 s
before and 1.40 s after at 25 × 10,000, and the process holding all 25
nodes peaked at 5.2 GB before and 1.1 GB after. Both move bytes and hold
acts, and each act is now about a ninth of what it was at 25 members.
A_RECORD_FOR_YEARS §3 said memory and a newcomer's first sync "are
measured again afterwards": the round and the peak are that, for one run.

## The two builds in one group

`mixed.py`, once each way (`run-maker-close/`, `run-maker-start/`): a
computer on the start's build and one on the close's, either making the
group, each wrote three records; both held the same ten lines, byte for
byte, and counted all six records, complete, with no gaps, in step 0.67 s
after the last write. The start's records carry a view; the close's carry
none; each build admits the other's.

## The lines the row cut

Two, both in `svrngs-core`'s `command.rs` at c276c34f: `:234`, the copy of
what is held made for the view, and `:235`, `sync::digest` over it.
`scripts/lean_check.py`: 16,162 → 16,160, svrngs-core 2,954 → 2,952; the caps
16,161 and 2,953 (cf08b1bf, whose body names the second line as the
`digest` import's name: the import stays a line, `digest` only left it).
