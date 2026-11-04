# give-play — the cap's raise to 19,095, for the operator's session to land

**The question.** Land `scripts/lean_check.py`'s raise of the repository cap
from 18,868 to 19,095 (core unchanged at 3,242)? The operator approved this
row's raise in advance on 2026-10-26 ("Go for both", `ralph/next/give/STATE.md`
give-play, "THE OPERATOR DECIDED"), to one past where it lands, inside the
row's range and the ceiling. A worker's edit of `BUDGET` is refused as a CI
bypass (give-tcp-1.out), so this row commits its work and stops here.

## Where the row stands

The work is done and committed:

- 1304624e: the red tests (`crates/svrngs/tests/play.rs`,
  `crates/svrngs-node/tests/browser/play.rs`).
- 64b96771: the room's store (`room/kept.rs`) keeps any list, behaviour kept.
- 86dd073d: Play (`crates/svrngs-node/src/play.rs`), `Offered.raw`, the home
  page's row and its line under "What acts as you", DESIGN §3, and the trust
  floor's two Play lines with their `Trust floor:` line.

Every step of `scripts/check.sh` but the size budget was run at 86dd073d, one
at a time, nothing else running:

    cargo test --quiet                                        no test failed
    cargo test --quiet --workspace --test browser             78 passed, 2 ignored
    cargo test -p svrngs --test scenes -- --test-threads=2    9 passed
    cargo test --quiet --workspace --test canaries            2 passed
    cargo test --quiet --workspace --test join_page           2 passed
    cargo test --quiet --workspace --test conformance         7 passed, 1 ignored
    cargo clippy ... -D warnings (both, as check.sh runs them) exit 0
    cargo fmt --all --check; web/node_modules/.bin/oxlint      exit 0
    python3 scripts/trust_check.py                            11 listeners, none broken
    python3 scripts/vocab_check.py                            78 app nouns, 0 in infra code

The row's own: `cargo test -p svrngs --test play` 4 passed;
`cargo test -p svrngs-node --test browser a_member_presses_play` 1 passed;
the room's, unchanged by name, `--test room` 4 passed and `--lib room` 3
passed. Each plant turned its test red; 86dd073d's body lists them.

## Why it stops here

`python3 scripts/lean_check.py` at 86dd073d:

    crates/svrngs-core     3241 / 3242 lines
    .                     19094 / 18868 lines
    BROKEN  . has 19094 lines of code, budget 18868: raised only by the operator, with the reason in the commit

The raise is +226 library and 0 core. That is inside the row's range of +215
to +350 (STATE.md's opening, the estimate's table) and under the ceiling of
19,233 for give-token through give-play.

## The edit, ready to land (scripts/lean_check.py:276-281)

    -# door (door/tcp.rs), sharing the UDP door's admission and opening
    -# (083cc6b6): +126 and 0, at 18,867.
    -BUDGET = {"crates/svrngs-core": 3242, ".": 18868}
    +# door (door/tcp.rs), sharing the UDP door's admission and opening
    +# (083cc6b6): +126 and 0, at 18,867. give-play, under the same ceiling,
    +# its range +215 to +350 and 0 core: Play's owner on the room's model
    +# (play.rs), sharing the room's store (64b96771) and port, `Offered.raw`,
    +# and the home page's row and line, after its own cut of 10: +226 and 0,
    +# at 19,094.
    +BUDGET = {"crates/svrngs-core": 3242, ".": 19095}

## The row's own economizing pass

- `play.rs` is 204 counted lines, `home.js` +9 and `question.js` +0 net; the rest is the wiring in `node.rs`, `lend.rs`, `door.rs`, `home.rs`, `lib.rs` and `room/kept.rs`.
- It shares the room's store (`room/kept.rs`, made generic in 64b96771, +2)
  and does not copy it (about 20 lines), and the room's port (`port_of`)
  and look interval (`LOOK_EVERY`).
- Cut in 86dd073d: an answer field the page never read (`raw` on `Playing`)
  and four chains rustfmt split over lines, -10.
- Kept: the lender's last words of a program it refused (`Open::At`'s
  words, about 6 lines). Without them a person past a lend's limit sees
  "Connect to …" while their program is answered by nothing.
- Nothing else in what this row touches could be cut with its behaviour kept.

## The four ledgers

- **Core:** 0.
- **Library:** +226.
- **App (`apps/`):** 0.
- **Surface:** +2. A lend in `/v1/lends` says `"raw": "udp"` or `"tcp"` when
  it is a port. `/v1/play`, the home page's, also answers a program holding
  the token, as `/v1/room` does.

## Said apart, not a verdict

- The restart test starts Alex's node again from a copy of its data dir: a
  node dropped in-process leaves a blocking task that `Node::start` on the
  same dir waits on (sampled at `BlockingPool::shutdown` under
  `block_in_place`). The command's own restarts are processes
  (`crates/svrngs/tests/share.rs`). A process restart of Play is give-play-scene's
  to play if the operator wants it judged so.
- The browser test runs in Chromium alone: Play is a computer's page (the
  queue's fence 13). The row's letter said both engines.

## What remains once the raise lands

1. The edit above, as its own commit, with a `python3 scripts/lean_check.py`
   plant (one counted line too many reads BROKEN).
2. PLAN §1's ledger line for give-play: +226 library, 0 core, 0 `apps/`, +2
   surface, the cap 19,095.
3. One run of `scripts/check.sh`, at the close.
