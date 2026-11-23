# Magnets' write on svrngs-db, once prune visits what a change outranks (2026-11-23)

One run on this Mac (Darwin arm64), in a debug build with its dependencies at
opt-level 2, as the run before it (`../2026-11-22/`). The tree was `c7c630c2`
with the prune fix in place, the source its commit holds. The driver is
`../2026-11-22/measure.rs`, byte for byte. It ran as a test of svrngs-db,
copied to `crates/svrngs-db/tests/` for the run and taken out after. One run
is a result, not a rate.

- `run.txt` is its output.

A tap's write, µs, best / median / slowest, of every write as the board grew:

| Rows | Before (`../2026-11-22/`, `9da65e7b`) | After |
|---|---|---|
| 16 | 308 / 378 / 653 | 358 / 428 / 724 |
| 10,000 | 302 / 10,000 / 815,694 | 344 / 7,709 / 27,560 |

What it says:
- **The slowest write fell from 816 ms to 28 ms.** Prune now visits only the
  sets at the places a write outranked and the set it keeps (svrngs-db
  `sync.rs`, `Db::prune`).
- **The median still grows with the rows,** from 0.43 ms to 7.7 ms: prune was
  not the whole of a write's growth. The rest is the seal's count of what
  this computer holds of its share (svrngs-db `sync.rs`, `Db::seal`, which
  asks `Db::held`), measured apart in svrn-docs
  `app-state-diligence/prune-cost-2026-11-23/`.
- The `BOARD` read is unchanged, 23 µs and 1.7 ms.
