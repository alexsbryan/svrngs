# give-app-scenes: the people runs' bars, for the operator to set, then run

**The question.** What are the bars for the two people runs in
`demo/give.md`: a Factorio world kept through a move and a takeover
(GIVE.md §3 scenes 3 to 5), and a newcomer's fluency in starting
Minecraft for a group? Once you set them, will you run both? The charter
leaves both to you ("Leave for the operator": "the people runs … The
operator sets their bars before they run").

## Where the row stands

(a) and (b) are done and committed. The gate's "scenes" step now plays
Give's scenes 1, 3, 4 and 5 with the stand-in, every person on this
build's svrngs, in rootless podman. Each scene is never-ran where podman
does not answer (`crates/svrngs/tests/scenes/give.rs` `no_podman`).

- **Scene 3**, `give_move.rs`: Cy moves the stand-in to Bo's computer on
  Bo's yes, and Cy opens it lent by Bo's computer alone, with Cy's start
  and then Bo's.
- **Scenes 4 and 5**, `give_gone.rs`: Bo's computer loses power. Dee's
  page, read fifteen minutes on, asks "Keep it here?", and Dee's computer
  starts from Bo's copy. When Bo's computer comes back, it lends and runs
  nothing, and every row lists "as Bo’s computer had it". Dee presses
  **Start instead**, and Dee's computer starts from that copy.
- **The fifteen minutes** are the page's alone. The fold decides "gone"
  at the time the page asks (`apps/give/fold/src/lib.rs` `GONE`), so the
  scenes' driver gained a `clock` step (352d84f7). Dee's page reads its
  clock fifteen minutes ahead, while svrngs, Give's program and the
  record keep this host's time.

Each scene, red on its plant, then green
(`cargo test -q -p svrngs --test scenes <scene>::`):

| Scene | Plant | Red | Green |
|---|---|---|---|
| 3 | `keep.rs` `start` restores nothing (`run.remove()`) | `give_move.rs:183` "Bo's computer starts from exactly Cy's last copy: before [02:08:44], after [02:08:52]" | ok, 44.07 s |
| 4 | `page/index.js` `takeover` never says gone | `give_gone.rs:236` "Dee's page, fifteen minutes on, asks Dee to keep it: Give …" | ok, 88.20 s |
| 5 | `keep.rs` `had` never unlends | `give_move.rs:69` "Dee lists ["Bo", "Dee"], not ["Dee"]" | ok, 88.20 s |

Scene 1 stayed green through the move of its helpers (1bfff296, 27.10 s).
No `svrngs-test-*` pod or volume was left after any run.

**What the scenes found.**

1. **Fixed in this row (4fa9003a).** A computer back after losing power
   lent the app again, at a port nothing listened on. svrngs keeps a lend
   across its restart (`~/.svrngs/lends/<name>.json`), and Give's `had`
   never unlent it. Give's own test could not see this, because its
   stand-in svrngs keeps no lend across a restart. scene 5's plant is
   that fix taken back.
2. **Not fixed, and for a row of its own.** Give's page, left open while
   its computer's svrngs is down, says nothing about it. It throws "Failed
   to fetch" each round: four in the first run's driver errors. A
   computer that loses power closes its browser with it, so the scene now
   does too. A person whose svrngs restarts, during an update for
   example, reads a page that has silently gone stale. That breaks
   AGENTS.md 6 ("an absence is reported, never defaulted") in Give's page.
   The fix is in `apps/give/page/index.js` `show`: say the record can't
   be read just now, as the page already does for an answer that is not
   JSON.

**The four ledgers** (GIVE.md §9): core 0, library 0, `apps/` +7
(`apps/give/src/keep.rs`, 611 to 618 by lean_check's rule), and surface
0. `python3 scripts/lean_check.py`: core 3,241 of 3,242, repository
19,107 of 19,109. `python3 scripts/vocab_check.py`: 89 app nouns, 0 in
infra code. `scripts/check.sh` ran once at the close. Its result is in
this unit's await log (fence 12).

## The bars to set, one answer each

Candidates, from GIVE.md §11 and `demo/give.md`. Change any of them.

1. **Run 1, the kept world:** both markers there after the move and after
   the takeover, no more than five minutes of play lost, the later world
   listed when Bo's computer comes back, and no terminal opened past
   "Before the run". Or name another.
2. **Run 2, fluency:** the minutes, decisions and questions are counted
   and written down, with no pass mark. Or give marks, for example at most
   5 minutes with Podman and 20 without, and at most 2 questions.
3. **Who:** you as Cy, with two others in their own homes, for run 1. Two
   newcomers and a stranger to judge, for run 2. Or name the people.
4. **The order:** run 1 can run now. You build Give's program for each
   computer before the run, and that step is not counted. Run 2's half
   "without Podman" needs Give's install line (give-install, the next
   row), because the program's link leads to build steps until then. Run
   it after give-install, or run its "with Podman" half now?
5. **Finding 2:** a row of its own after give-install, or one folded into
   give-install?

## Then run

`demo/give.md` gives each run's steps, what each person is told, what is
watched or counted, and the records: `demo/runs/give/<date>-keeps.json`,
`<date>-fluency-with.json` and `<date>-fluency-without.json`. A run not
played is recorded as never-ran, with its reason.

Nothing here could run on this host. Factorio is not installed: `ls -d
/Applications/Factorio.app` finds nothing. Run 1 needs three computers
with a person at each, and run 2 needs a newcomer, a friend and a judge.
The command each run starts is the person's own press on Give's page,
in step 1 of its run.

## The answer

The operator, 2026-10-26: "Go with those". Run 1 takes the candidate bars
above. Run 2 counts its minutes, decisions and questions, with no pass
mark. The operator names who plays when the runs are played. Both runs
wait for another time, never-ran until then. Finding 2 is a row of its
own, give-page-down, before give-install (`ralph/next/give/STATE.md`).
Factorio 2.1.23 is installed on this Mac through Steam, outside
`/Applications`, so a one-host rehearsal of run 1 can be played here.
