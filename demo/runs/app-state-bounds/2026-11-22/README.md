# Magnets' read and write on svrngs-db, at its pack and at W9's rows (2026-11-22)

One run on this Mac (Darwin arm64), of svrngs `9da65e7b` in a debug build,
its dependencies at opt-level 2. The page's own `BOARD` query
(`apps/magnets/board.js:8-9`) runs through `Db::read`, as the route runs it.
A tap's write is one op of the write form. One run is a result, not a rate.

- `measure.rs` is the driver, kept byte for byte. It ran as a test of
  svrngs-db, copied to `crates/svrngs-db/tests/` for the run and taken out
  after.
- `run.txt` is its output.

| Rows | A tap's write, µs, best / median / slowest | `BOARD` read, µs, best / median / slowest | Answer |
|---|---|---|---|
| 16, Magnets' pack | 308 / 378 / 653 | 21 / 24 / 65 | 493 B |
| 10,000, W9's rows | 302 / 10,000 / 815,694 | 1,685 / 1,730 / 5,445 | 30,912 B, cut at `ROWS_MOST` |

What it says of the bounds:
- **`QUERY_MOST`, 250 ms.** Magnets' read is 0.024 ms at its pack, and
  1.7 ms at 10,000 rows. So the bound sits more than 140 times over the
  slowest read here. It stays the spike's number.
- **`ANSWER_MOST`, 4 MiB, and `ROWS_MOST`, 1,000.** Magnets' answer is
  493 B. At 10,000 rows the row bound cuts it first, at 30,912 B, so the
  bytes bound never decides for Magnets. Both stay provisional.
- **Writes slow as rows grow.** A write's median rises from 0.38 ms to
  10 ms, and the slowest from 0.65 ms to 816 ms. The 10,000 writes took
  most of the run's 127 s. Each write prunes, and prune reads every kept
  set against the clocks (svrngs-db `sync.rs`, `prune`), so its cost grows
  with what the copy keeps. This is W9's to measure at 25 devices, and to
  fix before the release. It isn't a bound's number.
