# Architecture

The map of svrngs as it is built today. What it is built toward is
`docs/DESIGN.md`. This file says what exists, and it changes in the same commit
as what it describes.

## Bird's eye

svrngs owns the group's code, forked from commonwealth-ai's ring slice (DESIGN
§4). The record and the journal on disk have landed; sync, the routes and the
member door come next, forked the same way (DESIGN §8, U1-U3). Nothing runs as
a program yet.

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

**`crates/svrngs-node`**: a group's journal on disk — one append-only JSONL
log per group under `<root>/rings/<ns>/` (`journal.rs`), the single-writer
door that signs and the peer-ingest path that does not, roster sources, and
the checkpoint (`ring_checkpoint.rs`). The writer underneath is `oplog.rs`. It
is commonwealth-ai's `commonwealth-rail` and `oplog` at `e1a43bcd2`, forked the
same way as the core; its one edit is gating `oplog.rs`'s test-only serde
import. Upstream's shape stays: `RingRail` keeps its open journals in a map
behind a `Mutex` (`lib.rs`), which an actor per journal replaces when the node
becomes ours (DESIGN §5).

**`crates/svrngs-engine`**: the `Engine` trait and QuickJS behind it — a fold
engine that waits in DESIGN §9 until an app needs its state offline on a device
that is not its host.

**`apps/expenses/`**: the Expenses fold (`expenses.js`) and its tests, from E0,
the bot at tag `e0-bot`. Its reducer (`init`, `reduce`) is what the localhost
Expenses app of DESIGN U3 builds on; `propose`, `say` and `due` answered chat
commands and go with the bot.

**`bridge/`**: the Discord door's first code — `setup.mjs` and `discord.mjs`
(the bot's application, invite and channels), `words.json` (the plain-words
list), and `bridge.mjs`, E0's bridge core, kept for its log and tests until the
door (DESIGN U4) replaces it.

**`demo/`**: how a demo is recorded. **`ralph/`**: the work-loop prompt; a
queue goes in `ralph/next/<queue>/` and runs under commonwealth-ai's driver,
`python3 ~/dev/commonwealth-ai/scripts/ralph.py run --workdir ~/dev/svrngs
--queue <queue>` — process tooling, not code svrngs links. There is no queue
yet; U1's is next.

**`scripts/`**: `check.sh` is the definition of done; `lean_check.py` keeps the
repository standalone and within budget; `ports.py` lists the fork's pairs;
`principles_check.py` keeps `docs/principles/` pinned; `ralph-mark.sh` marks a
queue row done.
