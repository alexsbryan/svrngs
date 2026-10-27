# Give — a group keeps a game, and keeps it when a computer goes away

Gate: **the operator's, set before the first run** (svrn-docs
`rings/current/GIVE.md` §11; `runs/give/2026-10-26-scenes/NEEDS_HUMAN.md`).
Until the operator sets it, neither run below has a gate, and neither is
run. Each run's candidate bar is under its name.

Give keeps an app on one member's computer at a time, in rootless Podman,
and copies its state to the group's folder every five minutes and as it
stops (`apps/give/README.md`). The gate's "scenes" step plays the same
paths with a stand-in page in place of a game: a start
(`crates/svrngs/tests/scenes/give.rs`), a move (`give_move.rs`), and a
computer gone, then back (`give_gone.rs`). These runs are the games
themselves, played by people.

## Run 1: the group keeps a Factorio world (GIVE.md §3 scenes 3 to 5)

Candidate bar: the world is the same after the move. After the takeover,
it is the same as at the last copy, so the group loses at most five
minutes of play. When the old computer comes back, its later world is
listed. Nobody opens a terminal for any of it.

**Before the run.** Three computers, **Cy's**, **Bo's** and **Dee's**,
each with svrngs, Podman and Give's program for computers, and each in
one group, joined by its link. Until Give's install line lands
(`ralph/next/give/STATE.md`, give-install), the operator builds the
program from this commit (`cargo build --release -p give`) and puts it
beside svrngs on each computer. That step is the operator's, and it is
not counted. On each computer, `give keep` runs in a terminal of its own,
left open. Cy brings Give to the group once: `give page <group>`. Each
player has Factorio 2.1.23. All three screens are recorded.

**The steps, as the page has them now.** Nobody is told them.

1. Cy opens Give from the home page and presses **Start it for the
   group** on Factorio's row. The row reads "Factorio · kept by your
   computer".
2. Bo and Dee each press **Play** on Factorio's row on their home pages.
   In Factorio they pick Multiplayer, then Connect to address, paste and
   connect.
3. The three build for 15 minutes. Then Cy places a chest with exactly 17
   iron plates in it, beside the spawn. That chest is **the marker**.
4. **Moving it.** On Factorio's row, Cy presses **Move it to…**, then
   **Bo’s computer**. Bo's row asks: "Cy wants to move Factorio to your
   computer." Bo presses **Keep it here**. Every row then reads "kept by
   Bo’s computer". Bo and Dee connect again, by the same address.
   Somebody checks that the marker is still there.
5. They play 10 minutes more. Dee places a second chest with exactly 23
   copper plates in it, **the second marker**. Then they wait for one more
   copy: the next time the record says Bo's computer copied it, which
   comes within five minutes.
6. **The keeper is gone.** Bo's computer loses power: unplug it, or hold
   its power button. Do not shut it down.
7. Fifteen minutes later, Dee's row says "Factorio’s computer hasn’t
   answered since <time>. You have a copy from <time>. Keep it here?"
   Dee presses **Keep it here**. Cy connects through Play, by the same
   address. Somebody checks that both markers are there.
8. **The old keeper comes back.** Bo's computer is turned on. svrngs and
   `give keep` are started again as they were. Every row lists "Factorio as
   Bo’s computer had it at <time>", with **Start instead**. Nobody presses
   it. Somebody checks that Dee's world is unchanged.

**What Cy is told, and nothing else:** "Start Factorio for your group from
Give, play, and later move it to Bo's computer."

**What Dee is told, and nothing else:** "If the game stops, see what your
home page says."

**What is watched.**

- **The markers.** Whether each one is there after the move (step 4) and
  after the takeover (step 7).
- **Play lost.** The minutes between Bo's last copy and the moment it
  lost power, read from the copy's time on Dee's row and the recording.
- **Words and questions.** Each word a person reads that they did not
  understand, and each question they ask, written down as asked.
- **A terminal.** Any terminal a person opens past "Before the run" ends
  the run as failed.

## Run 2: fluency — a newcomer starts Minecraft for a group (GIVE.md §11 B)

Candidate bar: judged by a stranger, as people bars are (DESIGN §2). The
newcomer starts Minecraft from the shelf, and a friend joins and plays.
The minutes, the decisions and the questions are counted. Each is written
down, and none is a pass mark until the operator sets one. Any help from
the operator makes the run's value zero.

**Who.** **The newcomer** is someone who says "I'm not good with
computers". They have svrngs on their computer and are in a group with
**the friend**, joined by its link. The friend has svrngs and Minecraft:
Java Edition 26.3, and plays from another computer. **The judge** is a
stranger to svrngs, who watches the recording and counts.

**The run is played twice, with two newcomers.**

- **With Podman.** Podman and Give's program are on the newcomer's
  computer already, and `give keep` runs.
- **Without.** Neither is installed. The row asks for them once ("To keep
  Minecraft on this computer, svrngs needs Podman (free, one install) and
  Give’s program for computers"), and the newcomer follows its links. This
  half waits for Give's install line (give-install), since until then the
  program's link leads to a page of build steps.

**The steps, as the page has them now.** Nobody is told them.

1. The newcomer opens Give from their home page.
2. On Minecraft's row they press **Start it for the group**. They read
   Minecraft's makers' terms, and press **Agree and start**.
3. The row reads "Minecraft · kept by your computer".
4. The friend presses **Play** on Minecraft's row on their own home page.
   In Minecraft they pick Multiplayer, then Add Server, paste and join.

**What the newcomer is told, and nothing else:** "Start Minecraft for your
group from your home page, so your friend can play."

**What is counted.** The clock starts when the newcomer is told, and stops
when the friend is in the world.

- **Minutes**, to the friend in the world.
- **Decisions:** each choice the newcomer makes that the steps above do not
  name, such as a dialog of Podman's installer or a button not listed.
- **Questions:** each one asked aloud, written down as asked.

## After

One record per run in `runs/give/<date>-<run>.json`, with README's fields
and `"run": "keeps"`, `"fluency-with"` or `"fluency-without"`. A keeps
record adds `"markers"` (each marker as found after the move and after
the takeover), `"minutes_lost"` and `"terminal_opened"`. A fluency record
adds `"minutes"`, `"decisions"` and `"questions"` (each as written down).
The svrngs commit is the one every computer ran. Each person's
`~/.svrngs/svrngs.log`, and each keeping computer's `give keep` output,
is kept beside it. A run not played is a record with `"never_ran"` and
why, never a run left out.

**What this does not test.** A copy that doesn't start: the gate plays it
against a record of the test's own (`apps/give/tests/keep.rs`
`the_keeper_gone_then_back`). Also untested: more than one keeper at
once, a person picking who may keep, and the game's own LAN list. Each
waits for a group that asks.
