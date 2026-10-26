# give-play-scene — the game sessions' bars, for the operator to set, then play

**The question.** What are the bars for the Factorio session and the
Minecraft Java session (`demo/factorio.md`, `demo/minecraft.md`)? Once
they are set, the operator plays both, on a direct path and with the
relay forced. The charter leaves both to the operator ("Leave for the
operator": "the people runs: the Factorio and Minecraft sessions … The
operator sets their bars before they run").

## Where the row stands

(a) and (b) are done and committed:

- b9fbd681: the scene `crates/svrngs/tests/scenes/ports.rs`, in the gate's
  "scenes" step. Ana lends a UDP echo program and Bo lends a TCP one. Each
  presses Play on the other's row on their own home page, in Chromium. A
  program on each computer is answered through the address the row copied.
  Then Bo's Stop under "What acts as you" closes his port.
- 3611ef12: the two run-of-shows, written before either game is played
  (`demo/README.md`).

The scene's plant is a hand edit to `crates/svrngs-node/src/play.rs`
`opened`, `Ok(carried) => drop(carried)`, which unbinds the listener while
the row still says where to connect. Reverted after.

    red:   test ports::ports::chromium ... FAILED
           Bo's program at 127.0.0.1:43664 is not answered by Ana's walkie within 20 s
    green: test ports::ports::chromium ... ok   (9.23 s)

`scripts/check.sh` ran once at the close. Its result is in this unit's
await log (fence 12).

**The four ledgers** (GIVE.md §9): core 0, library 0, `apps/` 0, surface
0. The row adds a test and two documents, and tests are not counted
(`scripts/lean_check.py:306`). `python3 scripts/lean_check.py`: core
3,241 of 3,242, repository 19,094 of 19,095.

## The bars to set, one answer for each

Candidates, from GIVE.md §11 ("B, play"). Each is the operator's to
change.

1. **Factorio, each path:** 30 minutes, **zero desyncs**, no drop. The
   ping is recorded but has no bar. Or give a ping bar for the relayed
   path, for example under 150 ms.
2. **Minecraft Java, each path:** 30 minutes, **no drop**. Rubber-banding
   is counted but has no bar. Or give one, for example at most 3 in 30
   minutes.
3. **Who plays:** the operator hosts, and one friend in another home
   joins. Or name the people.
4. **A failed relayed path:** if the direct path passes and the relayed one
   fails its bar, is that a finding recorded for a later row (GIVE.md §10's
   waits), or does it stop Give's half (HUMAN-give-half)?

## Then play

Each run-of-show gives its steps, its two paths, how the relay is forced
(a pf rule on the friend's Mac that drops outgoing UDP but DNS), how the
path is read (`tcpdump` for the host's address in UDP packets), and its
records: `demo/runs/factorio/<date>-<path>.json` and
`demo/runs/minecraft/<date>-<path>.json`. A path not played is recorded
never-ran, with its reason.

Nothing here could run on this host. Neither game is installed: `ls -d
/Applications/Factorio.app /Applications/Minecraft.app` finds neither,
and `which factorio` finds nothing. Both runs also need two homes on two
networks, with a person at each. The command
each would start is the game's own, in step 1 of its run-of-show.
