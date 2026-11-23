# Magnets' write on svrngs-db, once prune and the seal follow what a write changes (2026-11-23)

One run on this Mac (Darwin arm64), in a debug build with its dependencies at
opt-level 2, as the run before it (`../2026-11-22/`). The tree was `c7c630c2`
with the prune fix in place, the source its commit holds. The driver is
`../2026-11-22/measure.rs`, byte for byte. It ran as a test of svrngs-db,
copied to `crates/svrngs-db/tests/` for the run and taken out after. One run
is a result, not a rate.

- `run.txt` is its output.
- `run-held.txt` is the second run's, the same way, on `054f16d6` with
  the totals of what each device holds in place, the source their commit
  holds.

A tap's write, µs, best / median / slowest, of every write as the board grew:

| Rows | Before (`../2026-11-22/`, `9da65e7b`) | Prune (`run.txt`) | Prune and totals (`run-held.txt`) |
|---|---|---|---|
| 16 | 308 / 378 / 653 | 358 / 428 / 724 | 426 / 491 / 856 |
| 10,000 | 302 / 10,000 / 815,694 | 344 / 7,709 / 27,560 | 290 / 340 / 1,899 |

What it says:
- **With prune, the slowest write fell from 816 ms to 28 ms.** Prune visits only the
  sets at the places a write outranked and the set it keeps (svrngs-db
  `sync.rs`, `Db::prune`).
- **With prune alone, the median still grew with the rows,** from 0.43 ms to
  7.7 ms. The rest was the seal's count of what this computer holds of its
  share (svrngs-db `sync.rs`, `Db::seal`, which asked `Db::held`), measured
  apart in svrn-docs `app-state-diligence/prune-cost-2026-11-23/`.
- **With the totals, a write no longer grows with the rows:** the median is
  0.34 ms at 10,000 and the slowest 1.9 ms. The seal reads its computer's
  one row of the totals SQLite keeps as each set is kept or dropped
  (`sync.rs`, `recorded`). The whole run took 5.9 s, where it took 127 s
  before. At 16 rows the median is 0.06 ms above prune alone's, one run
  each.
- The `BOARD` read is unchanged, 24 µs and 1.7 ms.
