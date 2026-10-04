# Architecture

The map of svrngs as it is built today. The design it is built toward is
`docs/DESIGN.md`. This file says what exists, and it changes in the same commit
as what it describes.

## Bird's eye

svrngs owns the group's code, forked from commonwealth-ai's ring slice
(DESIGN, Direction). The record has landed; the node that holds a group's
journal, syncs it and serves its doors comes next, forked the same way. Nothing
runs as a program yet. E0's Discord bot, which did, is at tag `e0-bot`.

## Code map

**`crates/svrngs-core`**: the group's record — signed acts (`Record`,
`Correct`, `Seal`, `Admit`, `Remove`), admission into one total order
(`admit.rs`), membership (`membership.rs`), the per-actor sync digest
(`sync.rs`, `view.rs`), attestation and introductions. Pure: no filesystem,
clock or network; time and keys arrive as arguments. It is commonwealth-ai's
`commonwealth-rail-core` at `e1a43bcd2`, with `oplog-types` (`op.rs`) and
`ContentHash` (`hash.rs`) inlined. Every file carries its `ported from` line;
`scripts/ports.py` lists each beside its original. It keeps upstream's edition,
rustfmt policy and clippy allowances so a diff against upstream shows only
what svrngs changed.

**`crates/svrngs-engine`**: the `Engine` trait and QuickJS behind it — a fold
engine that waits in DESIGN D8 until an app needs its state offline.

**`apps/expenses/`**: the Expenses fold and its laws test.

**`bridge/`**: the Discord door's first code — `setup.mjs` and `discord.mjs`
(the bot's application, invite and channels), `words.json` (the plain-words
list), and `bridge.mjs`, E0's bridge core, kept for its log and tests until the
door replaces it.

**`scripts/`**: `check.sh` is the definition of done; `lean_check.py` keeps the
repository standalone and within budget; `ports.py` lists the fork's pairs.
