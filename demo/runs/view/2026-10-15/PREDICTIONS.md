# arch-view (a): what the view costs a journal and a commit — predictions, before any data

Row arch-view (`ralph/next/arch/STATE.md`), part (a). Written at 1d6bf599,
the instrument committed and no point run. Nothing in the product changes in
this part.

## What is measured

- **The instrument**: `crates/svrngs-node/tests/scale.rs`, `#[ignore]`d, in
  release: a converged group of real nodes on loopback, every member writing
  an equal share of record acts of about two hundred bytes, so every act's
  view names every member. It prints `view_bytes` (each line's `,"view":`
  and its value as written, read from the journal file), `oplog_bytes` (the
  file's length), `view_share`, `journal_acts`, and `append_s`: the median
  of five commits of one act on node 0, the whole of `Rail::commit`.
- **The points**: 2, 5 and 25 members at 1,000 acts, and 25 at 10,000, where
  A_RECORD_FOR_YEARS §5 bar 3 read 0.74 s for an append (`scale.rs`'s
  ceiling-3 notes, `append_s` medians 0.740 and 0.734 s). One run a point,
  `run.sh before` at the row's start; `run.sh after` at its close. One run is
  not a rate (AGENTS.md 7): the bytes are structural and need one, a time is
  a reading of one host on one day, its load in `load.txt`.
- **The survey**: `survey.py` over every journal left under `/private/tmp`
  (116 files, `find /private/tmp -maxdepth 6 -name ring_oplog.jsonl`), the
  same count per line, beside the audit's 25–39% (AUDIT §2.9).

## How a line divides, by hand

A view entry is `"<key, 64 hex>":{"from":0,"mark":<d digits>,"head":"<64
hex>"}` and a comma, 160 + d bytes; a view is 29 bytes (`,"view":` and
`{"v":2,"entries":{}}`) and an entry per member. The instrument's first read
(2 members, 100 acts, 1d6bf599's body) gave 343 view bytes a line out of 881,
so a line without its view is about 540.

## What I expect, and what would falsify it

1. **The view's share at the row's start**, `view_share`:
   - 2 members, 1,000 acts: 0.37 to 0.42 (29 + 2 × 162 ≈ 353 of ≈ 893);
   - 5 members: 0.58 to 0.64 (≈ 840 of ≈ 1,380);
   - 25 members, 1,000 and 10,000 acts: 0.86 to 0.90 (≈ 4,080 of ≈ 4,620).
   Falsified by any point outside its range.
2. **The survey**: every journal's views name at most six keys (this host's
   runs are of two to five people); the share over all their bytes, and the
   median journal's, inside 0.20 to 0.45, the audit's 25–39% among them.
   Falsified by either outside it, or by a view naming seven keys or more.
3. **At the row's close**: `view_bytes` 0 at every point (every act there is
   new) and a line 520 to 560 bytes (`oplog_bytes / journal_acts`) whatever
   the members, so the journal at 25 members about a ninth of the start's.
   Falsified by any view byte written, or a line outside that range.
4. **A commit's time**. What a commit pays for the view today is
   `sync::digest` over everything held (`command.rs:234-235`): for each act
   its id derived again, which serialises its view, and its signature
   checked, about 30–45 µs, then the fold. The rest of a record's commit
   reads only the register (`Reads::Membership`, `command.rs:198-200`), but
   still copies what is held: at `:199` before and after, and at `:176`
   (`:234` is the view's own copy). So:
   - start, 25 × 10,000: `append_s` 0.55 to 0.95 s;
   - close, 25 × 10,000: 0.02 to 0.20 s — at least four times faster;
   - close, at 1,000 acts, every size: at least twice as fast as the start's
     same point.
   Falsified by the close at 25 × 10,000 above 0.20 s, or any 1,000-act
   point less than twice as fast. A close slower than the start at any point
   would mean the digest was not the commit's cost, and is named so.
5. **The setup's own time** (`setup_s`, which writes every act through the
   same commit, in batches of 500, and moves them): lower at the close at
   every point. Falsified by any point slower.

What would surprise me most: a share at 25 members below 0.80. That would
mean something in a line other than the view grows with the members.
