# give-redial (b): A lands, svrngs closes every connection to the key

The director, 2026-10-27, on `NEEDS_HUMAN.md` beside this file.

## The decision

**A lands:** when a request on the connection kept to a computer fails,
`fetch` closes every connection to that computer, the ones its acceptor
admitted included, before its one fresh dial. The clean version is the one
`NEEDS_HUMAN.md` describes under "A", with a debug trace saying why it
closes them and a comment citing iroh 1.0.2's rule (`remote_state.rs:486-488`).

**B does not land now.** The fix inside iroh stays the better remedy at
its cause. It is the operator's to send upstream to n0 if they choose
(the diff is in `NEEDS_HUMAN.md`). When an iroh release carries it, A can
come out.

## Why

- **The question's premise was half false.** It says A "needs a raise,
  which is the operator's". The operator already approved that raise for
  this row: "30 lines approved", up to +30 library and 0 core, to one
  past where it lands (`ralph/next/give/STATE.md`, give-redial, "THE
  OPERATOR DECIDED, 2026-10-27"). A's ~15 lines in `.` and none in
  svrngs-core fit inside it.
- **B needs a publish, which the charter leaves to the operator.** A git
  `[patch.crates-io]` needs a fork pushed somewhere public ("push and
  publish", CHARTER "Leave for the operator"). Vendoring iroh here instead
  would put its whole crate under `lean_check`'s count, far past the
  ceiling. A needs neither.
- **Both were measured the same.** Each was 3 of 3 on this Mac and on
  Fedora's pasta at about 4 s (`README.md` here, f4514ae7). So the choice
  turns on what each costs, not on whether it works.
- **A is the smaller reversible step** (CHARTER "Decide"). It is svrngs
  code under svrngs's gate, with no fork to keep and nothing to re-make at
  each iroh upgrade.
- **A has a cost, and it is accepted.** A lend the restarted computer
  opened in the few seconds before the request failed is cut once, and its
  page opens it again. That happens only after a restart, inside that
  window, and the debug trace names it (AGENTS.md 1).

## Reproduced here

- iroh 1.0.2 is the one in `Cargo.lock` (`name = "iroh"`, `version = "1.0.2"`, line 1886).
- `handle_connection_close` clears `selected_path` only when
  `self.connections.is_empty()`, and otherwise selects nothing
  (`~/.cargo/registry/src/*/iroh-1.0.2/src/socket/remote_map/remote_state.rs:473-490`).
- A new connection's datagrams go only to `selected_path` when one is set
  (`:799-809`).
- An abandoned path calls `self.select_path()` (`:626-627`), which is what
  B copies.
- `python3 scripts/lean_check.py`: `. 19430 / 19431 lines`,
  `crates/svrngs-core 3241 / 3242 lines`.

## What the worker does now

This is the row's (b), unchanged:

- Put `mod redial;` back in `crates/svrngs/tests/scenes/main.rs`, and see
  it red at HEAD.
- Land A clean.
- Plant the cause by skipping the closes, and see the instrument and the
  grapevine scene turn red.
- The grapevine scene green in ten runs in a row here.
- The Fedora image green on pasta.
- The raise to one past where `.` lands. If the `BUDGET` edit is refused
  as a CI bypass, write the ready diff in `NEEDS_HUMAN.md` and stop.
- `scripts/check.sh` once, at the close.
