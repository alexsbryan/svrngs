# V1.8 release, 13 November 2026

Published from the commit tagged **v1.8**, on the operator's word of
2026-11-13 ("So let's release then unless there's a strong objection").

## Scope and source evidence

The preceding release is V1.7 at 23ed4ef7, whose readback is
`../../v1.7/2026-11-07/README.md`. Before this run `group.svrnme.sh` still
served V1.7's page: its `sw.js` named version `f21ccefa713c`, V1.7's own,
read at 2026-11-14 02:14 UTC. The notes cover `git log v1.7..13182400`, 327
commits; the test-seam moves among them change no behavior and are left out.

| Release note | Source or run |
|---|---|
| One set of rules for every group; a group made before V1.8 changes nothing, stays readable, and says to make it again | e237ba76, d6a3ff81, dd8580b0, abf5b0de, 0d203da7; `crates/svrngs-core/src/lib.rs` `made_older`; `docs/DESIGN.md` §6 "A group an older svrngs made changes nothing"; `docs/guide/people.md` "A group made by an older svrngs" |
| A computer on V1.7 can't join a group made on V1.8 | `crates/svrngs-core/src/seed.rs` (a new group's id is hashed under `SOURCE_GROUP_ID_DOMAIN`, which V1.7 does not know); `crates/svrngs-core/src/sig.rs` `source_redemption_message`. `crates/svrngs-node/src/version.rs` `STEPS` is unchanged since v1.7 (6 entries, version 6), so no lend-listing step is added |
| Handing on waits for an accept; take it back; a keeper may leave | e237ba76; `svrngs accept`; `crates/svrngs-node/src/home/page/leave-group.js`; `docs/guide/people.md` |
| Guests per computer, 2 by default, at most 8, One fewer / One more | d779b190, 41da2e1c; `crates/svrngs-node/src/home/page/visits.js` |
| A guest's page confirms its host | ad25eab2 |
| Movie Night; apps read friends' lends; the readers' account | cc15b40f, 998a3047, 4d1400c8, 52276a97, 75ee2ebe, b66d5aaa, 56ae435e, fe89850d; `docs/guide/apps.md`, `docs/guide/lend.md` |
| `--program` for `--kind`; Immich and Audiobookshelf by port; "Did you mean" | 0e0bab39, 58b40ec3, 3cf0568b, be2ff6c7, 10cee7ba |
| For people who build apps: help and guide, `svrngs check`, own change at once, limits, `reads.txt`, `sends.txt` | afb8138e, 470ff938, 791d31c9, 18d4e616, b8bed43f, d8b64304, d05674c0; `docs/guide/build.md` |
| Fewer relay lines in the record; the record read after a change in about half the time | 8e71db1e; f1b0cbcc (refold after a synced line at 10,006 lines 102 to 50 ms, at 1,006 lines 8.3 to 3.9 ms, in its body) |
| Files still only on a computer that's off | bdda040d |
| Large sends take turns | 39805b40, 81630374 (`demo/runs/swarm/2026-11-09/README.md`) |
| Open first; Shared apps and Shared files apart | 64cad1ad |
| Give's Stop running and keep a copy | d0abaed0, 217e3491; `apps/give/README.md` |

## Validation before packaging

`scripts/check.sh` ran whole at e09e8462, exit 0 in 1,708 s, in a clean
detached worktree with Eudoxia from git e995acd4
(`../../flakes/2026-11-13/gate-e09e8462.txt`). 13182400 adds only that
record and rows of `docs/seams.tsv` (`git diff --stat e09e8462 13182400`:
four files, 120 insertions), and the release commit changes only
`RELEASES.md` and this directory, so the gate was not run again.

The toolchain pin (`rust-toolchain.toml`, Rust 1.95.0) came after v1.7, and
on this Mac it lacked the `aarch64-unknown-linux-musl` target that
`scripts/build-release.sh` builds linux-aarch64 for. It was added with
`rustup target add aarch64-unknown-linux-musl --toolchain 1.95.0` before
the build, so no platform is missing.
