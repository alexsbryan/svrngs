# arch-command: the command asking the node costs lines; it saves none

The row has to close below the count it started at, 16,160, and it may sit at
most 60 over it between its commits (CHARTER "The budget"; fence 7). Built and
measured, part (b) alone puts the repository at **16,366: 206 over the start
count**. That leaves (c)'s form on the page, (d) and an economizing pass still
to do. The audit expected the row to *save* 60–100 lines (AUDIT §2.3). It does
not, so the row stops here for you (CHARTER "Leave for the operator: the
budget").

## What was built, and what it proved

| | Code lines (`scripts/lean_check.py`) | Over 16,160 |
|---|---:|---:|
| Row's start, 4a741668 | 16,160 | 0 |
| (a) `join_group` moved into the node, behaviour kept: 0e4684ca | 16,195 | +35 |
| (b), first shape: the command calls the node in-process when it started it, and over HTTP when svrngs already runs | 16,425 | +265 |
| (b), second shape (`b-wip.patch` here): one HTTP path; the home page's own handlers also serve `/v1/group`, `/v1/link` and `/v1/join` | **16,366** | **+206** |

- **(a) worked.** `cargo test --quiet` passed across the workspace with no
  test file changed (6 min 43 s; every `test result: ok`). It is reverted in
  0c5df4aa so that main's caps hold while the row waits. Its
  `scripts/lean_check.py` said "BROKEN . has 16195 lines of code, budget
  16161". `git cherry-pick 0e4684ca` brings it back.
- **(b)'s red run** was taken before the command changed (the new
  `crates/svrngs/tests/asks.rs`, in the patch). The plant is the command
  claiming the data dir, as at 0e4684ca:

  ```
  panicked at crates/svrngs/tests/asks.rs:145:5:
  svrngs: svrngs is already running on this computer (…/.svrngs/lock is held); stop it, then run this again
  test result: FAILED. 0 passed; 1 failed
  ```

  In that same run, the home page's `POST /group` already kept the relay
  that the computer reached: the assertion just before the red one passed.
- **(b)'s second shape, green:** `cargo test --quiet --workspace --test asks`
  passed (`1 passed; finished in 16.39s`).
  - With svrngs running, `svrngs new Trip`, `svrngs link Lab` and
    `svrngs join <link>` each did their verb and exited, and none of them
    started svrngs.
  - `new` kept the same relay as a group made on the home page.
  - With nothing running, each verb started svrngs, did its verb, and said
    "Leave this running".
- **Not run on the second shape:**
  - The rest of the workspace's tests. About 55 calls in tests/ still use
    `cli::new_group`, `cli::link_to` and `cli::join_group`, which the second
    shape removes. They would move to a helper in tests/, which the budget
    does not count.
  - `svrngs room` over the API (the node's `POST /v1/room`; the answer's body
    holds the room open).
  - `scripts/check.sh`.

## Where the lines go (second shape, against 4a741668)

| File | Δ | What |
|---|---:|---|
| crates/svrngs-node/src/join.rs | +123 | the join, moved from cli.rs, and its refusal answered as itself (serde, the name's adapter) and in the page's words |
| crates/svrngs-node/src/home.rs | +70 | `POST /join` for (c) and the command; `/group` taking a page and a relay and answering seed, id and folder; `/link` taking a page; the three `/v1/` doors |
| crates/svrngs-node/src/room.rs | +30 | the room opened through the API, held open for as long as the command holds the answer |
| crates/svrngs-node/src/invite.rs | +17 | `Linked`, the answer the page and the command both read; the page `link --origin` keeps |
| crates/svrngs-node/src/{group,node,lib}.rs | +15 | `Host::found` taking the page and relay asked; the routes registered |
| crates/svrngs/src/cli.rs, issuer.rs | −49 | the command's makers go, and the asks come in: reading answers, a refusal for an answer it cannot read, the room's held answer |

**Why the audit's estimate missed.** It counted the command's makers as the
saving: about 75 code lines at 5bf9941 (`new_group`, `link_to`, `mint`; the
join is a move and saves nothing). Before the audit, start-home and
start-group had already moved what those makers decided into the node
(invite.rs `Host::mint`, group.rs `Asking::found`), so what was left in the
command was thin. Asking a node over HTTP costs lines on both sides: a route
and an answer in the node, then sending, reading and refusing in the command.
The audit counted neither.

## What I could not find

- An economizing pass on this shape could take perhaps 25–40 lines, chiefly
  where rustfmt lays the command's `match` out long. That is my estimate; I
  did not measure it.
- Each of (c)'s form and (d) is a few lines either way. (d) removes the stop
  and the start around install.sh's join, and maybe `join --no-run`.
- Either way the row closes about 150–180 over 16,160, not below it. These
  ~ figures are estimates.

## Choices, as I see them

1. **A raise for this row**, closing where its economizing pass lands, about
   16,310–16,340 (~ estimated). Everything the row's WHAT lists ships: every
   verb asks svrngs running here, one relay rule, a join on the home page,
   and install.sh with no stop and start.
2. **Narrow it.**
   - Keep `svrngs new`, `link`, `join` and `room` claiming the data dir as
     today.
   - Ship (a); one relay rule (`svrngs new` keeps what `Host::found` keeps);
     and (c), the page's join, through the node's one `Host::join`.
   - That is about +40–70 (~). It still needs a raise, or cuts from
     elsewhere, to close below 16,160.
   - install.sh keeps its stop and start.
3. **Drop `room` from (b).** It saves about 45 (~): room.rs's route and the
   command's held answer. `svrngs room` would keep refusing while svrngs
   runs, as it does today. Combine with 1.
4. **Park the row.** groups-leave does not need it. Its row writes
   `svrngs leave` "asking the running node as `svrngs remove` does"
   (ralph/next/groups/STATE.md:15), and `remove` already asks that way
   (cli.rs `removal`, `ask_running`). The page's join would wait.

## The question

Which of these do you want? With 1, at what cap? With 2, where should the
lines come from? Until you say, main stands at 4a741668's code, at 16,160 /
16,161 (0c5df4aa).

To resume:

```
git cherry-pick 0e4684ca
git apply ralph/next/arch/arch-command/b-wip.patch
```

The patch is the second shape, on top of 0e4684ca, `asks.rs` included.
