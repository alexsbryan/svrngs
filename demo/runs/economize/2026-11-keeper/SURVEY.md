# Economizing pass before svrngs-renew, 2026-11-22

`ralph/next/keeper/STATE.md` row svrngs-economize: the files svrngs-renew
extends, cut with behaviour kept, before its raise (CHARTER "Budget").

## Scope

`door/bound.rs`, `share.rs`, `uid.rs`, `home/page/lent-row.js` and
`home/page/lend-change.js`, all under `crates/svrngs-node/src/`.
`git fetch origin app-state && git diff --name-only main...origin/app-state`
at the start (app-state's tip then) names none of them, so all five are in.
It names `lend.rs`, `door.rs` and `scripts/lean_check.py`; the first two
stay out, and the last takes only the caps (CHARTER "Alongside app-state").

macOS-only code in `uid.rs` (`mac`, the `cfg(target_os = "macos")` readers)
is left alone: this computer has no Apple target (`rustup target list
--installed`: wasm32, x86_64 linux gnu and musl), so a cut there could not
be compiled, let alone tested.

## Baseline

`python3 scripts/lean_check.py` at 3e8d7275:

```
  the core (svrngs-core)     3,465   most   3,539
  all code but JavaScript   24,517   most  24,517
  JavaScript                 8,693   most   8,693
```

Counted lines per file, by lean_check's rule: bound.rs 183, share.rs 480,
uid.rs 948, lent-row.js 167, lend-change.js 66.

## The rule for a cut

Each is a rewrite the formatter lays out shorter, never a join of lines it
would split: a chain or closure named first, a loop as its condition, a
struct's one field set in place of a literal, a type named once. Each
answer, refusal, trace and wait is as before. Counted after `cargo fmt` and
`oxfmt`.

## Candidates

| File | Where | What | Lines |
|---|---|---|---:|
| bound.rs | imports | `LendName` in the one `crate::lend` use | 1 |
| bound.rs | `bound` | the timeout's `Elapsed` read by its own `Into<io::Error>`, `TimedOut` (tokio 1.54.1 time/error.rs:119) | 2 |
| bound.rs | `here` | the stopped read named first | 1 |
| bound.rs | `hold` | the retry as a `while` over busy and tries, 20 at most as before | 3 |
| share.rs | imports | `SendRow` in the one `crate::lend` use | 1 |
| share.rs | `reapproved` | `data_dir` as `dir`, so its signature fits | 4 |
| share.rs | `token` | `held` cleared on a clone | 2 |
| share.rs | `unrecognized` | the slip's words in one `map_or` | 2 |
| share.rs | `sends_of`, `fits` | the rows named first | 2 |
| share.rs | `key_header_fits` | the names never a lend's secret named apart | 1 |
| share.rs | `sent` | the host named first; the body read to bytes at once | 3 |
| share.rs | `lend_of` | the carries word's test named first | 1 |
| uid.rs | `waited` | its `expect` reason short enough for one line (not cut: below) | 3 |
| uid.rs | `read` (Linux) | the end's test named first | 2 |
| uid.rs | `sockets` (Linux) | the bound-listing kernel's test named, still read only when reached | 1 |
| uid.rs | `peer`, `lists_bound` | the line's and the number's reads named first | 2 |
| uid.rs | `linux::read` | its answer's type named; the states by `match` on `Raw`; `procs` by `bool::then` | 7 |
| uid.rs | `linux::procs` | a link's inode read in two steps; a file's link named | 7 |
| uid.rs | `diag::dump` | the kernel's address named | 1 |
| lend-change.js | `CHANGE` `io` | a change's answer, read again, written once for `change` and `approve` | 4 |
| lent-row.js | `Lent` | the machine's first state named | 5 |
| lent-row.js | `Lent` | `saveAudience`, used once, written where it is drawn | 3 |

Left: the struct literals rustfmt lays out field by field (`Flow::open`,
`Pid::parse`, `ReadersSignIn`, `Answering`, `Read`, `Diag`), the long
`tracing` calls, and the signatures that stay past 100 columns
(`decided`, `udp_port`, `lend_of`).

Not cut: `waited`'s reason is pinned word for word in door/tests.rs's
list of named allowances (`every_dial_of_a_share_s_port_goes_through_its_check`,
which failed on the shorter one), and "at the edge" is its justification.

## The cuts

Each its own commit, named by its file (`git log --oneline --
demo/runs/economize/2026-11-keeper`), its caps lowered to one past where
it lands.

| File | Cut | All but JavaScript | JavaScript |
|---|---:|---:|---:|
| door/bound.rs | 6 (`bound` 1 of the 2 thought) | 24,511 | 8,693 |
| share.rs | 16 (`sent` 3, `fits` 1, as thought) | 24,495 | 8,693 |
| uid.rs | 20 (`linux::read` 6, `linux::procs` 7) | 24,475 | 8,693 |
| lend-change.js | 3 (`again`, 1 short of the 4 thought) | 24,475 | 8,690 |
| lent-row.js | 8 (as thought) | 24,475 | 8,682 |
