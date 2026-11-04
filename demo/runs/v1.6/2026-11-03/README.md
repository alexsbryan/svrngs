# V1.6 release, 3 November 2026

Published from source **SOURCE**, tagged **v1.6** locally. The page and all
eight downloads are live at https://group.svrnme.sh/.

## Scope and source evidence

The preceding release is V1.5 at 5a4cb567. This release includes:

| Release note | Source or run |
|---|---|
| Join another group while svrngs runs | 2020f28e, 2636b5d7; `crates/svrngs-node/src/join.rs` (`JOINS_ROUTE`), `crates/svrngs/src/cli.rs` (`join_running`) |
| Every group on the home page; a joined one appears when shown again | `crates/svrngs-node/src/home/page/home-page.js`, `home.js` `visible`; 2636b5d7 |
| svrngs updates itself from the page it came from | 467d5d16; `crates/svrngs-node/src/update.rs`, `crates/svrngs-node/tests/update.rs` |
| A stale page worker never answers a newer build; reload after an update | d6d2a120; `web/tests/worker.test.mjs` |
| `install.rs` reads compile (reqwest's `json` feature was off) | abf6fe4d |

The update path's trust is the page's HTTPS and the same `get/SHA256SUMS`
`install.sh` checks; nothing is signed and no release key exists. Give
still follows the install line; this path replaces svrngs alone. The join
route is on the record API, loopback alone and behind its token, and joins
through the one process that writes this computer's acts.

## Validation before packaging

`scripts/check.sh` exited 0 at c9771c1a in 1,334 s. The release commit
changes no runtime file: `git diff --stat c9771c1a..SOURCE` is this record
and `RELEASES.md` alone. The gate ran with the merged tree (join-while-running
and the update work), 37 steps passed and none failed; the lean caps are the
operator's, approved 2026-11-03 (`.` 19,442, `js` 7,500).

## Publication

To be filled by the publish run.

## Readback

To be filled by the verify run.

The Git tag is local. No Git push or new GitHub release is performed, as in
V1.5.
