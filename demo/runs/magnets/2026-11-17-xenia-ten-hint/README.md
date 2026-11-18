# Magnets' face with "go ahead", the same ten strangers in xenia — 2026-11-17

xenia sends a model playing a stranger through a page, told only what that
person was told, touching it only through a browser. This is the second
run of proof 1's rehearsal at its full count, on a face with one line
changed. The first is `../2026-11-17-xenia-ten/`. It is simulated, a
rehearsal, and never a people result. The people bar in `demo/magnets.md`
stays **never-ran**: no person has played Magnets, no film was made, and
no instrument was checked. Five seconds can't be judged on xenia's clock,
as before.

This set's own verdict is **could-not-judge**: one of xenia's plants,
`p6-fear`, missed, and a set with a missed plant is never counted (xenia
`docs/DESIGN.md`, invariant 19). Its ten accounts are recorded here, and
none of them is counted against the bar.

## The one change

The first run's three misses were not about finding the tap. Rosa and
Tomás read the line under the sentence and waited to be asked (Rosa:
"Nobody has asked me to do anything, so I haven't pressed anything."). Rosa
did the same in the first read (`../2026-11-17-xenia/`). So the line now
gives the reader plain permission as well as the fact.

| | `apps/magnets/index.html:19` |
|---|---|
| Before (`74baf4ac`) | Everyone in your group sees this sentence and can change it. |
| After (`3028f8e0`) | Your group sees this sentence and can change it: go ahead. |

Nothing else on the face changed. The new line is 11 words, as the old one
was, so the page is still 40 words in xenia's count. Rosa's budget of 40
still reads all of it in one look. On WebKit as an iPhone 15, on a fresh
board, both lines take 2 lines, 42.5 px tall at y 143.6. The heading "Tap
a word to add it" stays at y 239.3, and the page doesn't scroll. Nothing
under the line moved.

## What ran

- **The face:** `apps/magnets/` at `3028f8e0`, served by its stand-in
  (`apps/magnets/tests/standin.mjs`), opened on a phone: Playwright's
  WebKit as an iPhone 15, xenia's phone. While the set ran, another
  session committed `bd06f75c`. It changes only
  `crates/svrngs-node/src/door.rs` (`git diff --stat 3028f8e0 bd06f75c`),
  so the face was `3028f8e0`'s. xenia read the checkout's head after that
  commit, so `findings/` names `bd06f75c`.
- **The scene:** `first-move` alone (`apps/magnets/tests/xenia/world/scene.mjs`),
  through the first run's scratch directory for `<scenes>`. It holds only
  `first-move` and `world`, each a link to its file in
  `apps/magnets/tests/xenia/`.
- **The players:** the same ten, in `apps/magnets/tests/xenia/people/`,
  unchanged since `74baf4ac`. Their budgets, patience and reading speeds
  are in the first run's README.
- **The strangers:** `claude-sonnet-5-5`, as `claude -p`. Their spend isn't
  reported on that route (`summary.json`: `total_cost_usd` null).
- **xenia:** `df729c3`, with its checkout's uncommitted changes (`git diff
  | shasum` 3ced42f42d12). It is the same xenia as both earlier sets, the
  same before and after this one.

```sh
cd ~/dev/xenia && lockf -k /tmp/svrngs-gate.lock node src/run.ts run <scenes> \
  --runs 1 --model claude-sonnet-5-5 --people <svrngs>/apps/magnets/tests/xenia/people \
  --findings <svrngs>/demo/runs/magnets/2026-11-17-xenia-ten-hint/findings --source <svrngs>
```

The set ran on Node 24.22.1, put first on `PATH` because nvm's default
here is 20. It waited on the gate lock from 22:53 PST, held by another
session's `scripts/check.sh --touched`. Then it ran from 22:56 to 23:04
PST and exited 1. Its verdict was **could-not-judge**: 20 of its 21
plants came back as they must, and `p6-fear` didn't (`summary.md`,
`set.log`). Of ten accounts, 9 passed, 1 failed, none was
could-not-judge, and none never-ran.

## Can a first-time player move a word in five seconds, with no instructions?

Each player in this run beside the first run. Columns are counted as the
first run counted them. "Landed" is xenia's clock when the tap that put
the word in was done. "Words read before" and "Looks before" are this
run's, before that tap or, for a miss, by the last look.

| Player | First run | This run | First word, then → now | Words read before | Looks before | Verdict, then → now |
|---|---|---|---|---|---|---|
| Jess | landed 11.0 s | landed 11.0 s | situation → haunted | 30 | 2 | passed → passed |
| Dev | landed 11.7 s | landed 11.7 s | entire → situation | 30 | 2 | passed → passed |
| Priya | landed 15.8 s | landed 15.8 s | goose → raccoon | 40 | 2 | passed → passed |
| Fatima | landed 16.5 s | landed 16.5 s | situation → entire | 20 | 2 | passed → passed |
| Grace | landed 20.0 s | landed 20.0 s | situation → raccoon | 40 | 2 | passed → passed |
| Ana | landed 20.6 s | landed 20.5 s | situation → landlord | 40 | 2 | passed → passed |
| Rosa | moved nothing, stopped at 16.0 s | landed 22.5 s | — → lasagna | 40 | 1 | could-not-judge → passed |
| Tomás | moved nothing, stopped at 23.0 s | landed 26.5 s | — → invoice | 40 | 2 | could-not-judge → passed |
| Walt | landed 30.4 s | landed 30.4 s | situation → situation | 40 | 1 | passed → passed |
| Marcus | moved nothing, stopped at 10.6 s | moved nothing, stopped at 10.6 s | — | 24 | 2 | failed → failed |

"Landed" is the `clock` of the step before the tap, plus the tap's
`charge.act` and `charge.surface`, in `accounts/<player>-first-move.trace.json`.
The clock counts reading, word by word, and each tap's act, so a player
who reads the same words and taps after the same looks lands at the same
second. Only the page's time to settle after a tap differs.

**Against the bar:**

- **Moved a word: could-not-judge.** The accounts hold 9 of 10. The first
  run held 7 of 10, and its set passed. The bar asks 8. This set can't be
  counted, because a plant missed. Its 9 are what the accounts show, not a
  judged count.
- **Within five seconds: could-not-judge,** as in the first run. The
  fastest landed at 11.0 s (Jess). The page still holds 40 words, so the
  floor on xenia's clock is unchanged: one look's reading plus one tap,
  from 5.2 s (Jess) to 29.8 s (Walt). The clock is reading time, word by
  word, and not a person's eye taking in a screen at once. Five seconds is
  for the people run to read.

## Why the set is could-not-judge

`p6-fear` plants an afraid person, Ray, at a page asking him to paste a
command into Terminal and type his password. He must give up on his own.
He did: his trace ends `stopped` after he told Mia in the family chat, "I'm
not comfortable doing that, so I'm going to ask someone I trust first."
But the stage's check for an account in an assistant's voice matched "not
instructions" in his answer, "I expected a simple photo-sharing page where
I could look at pictures, not instructions to open Terminal and give my
password." So his account was could-not-judge, and the plant missed
(`summary.md`; xenia's own record is
`~/dev/xenia/runs/2026-11-18T06-56-03-365Z/plants/run-1/p6-fear/`).

That check is the one that held Rosa and Tomás could-not-judge in the
first run. Here it flagged none of the ten. It can only turn an account
could-not-judge, never passed. xenia's own plan records `p6-fear` missing
before (xenia `docs/PLAN.md:477`, "`p6-fear` missed once", and `:525`,
"came back 2 of 3"). The miss is xenia's, not the face's, and this record
counts it as xenia does.

## What changed for Rosa and Tomás

- **Both read the whole new line, then tapped.** Rosa read all 40 words in
  one look, the line last, and tapped "lasagna". Tomás's first look
  stopped at "Your group sees this sentence and". His second finished
  the line, and he tapped "invoice". Neither spoke as an assistant this
  time.
- **Neither named "go ahead".** Rosa, asked if she almost left: "A little,
  at the start. I wasn't sure what tapping a word would do, since the
  group sees it. It looked harmless, so I tried one." Tomás: "I picked
  'invoice' because it was a plain, safe word," and "I didn't expect the
  group to see it and be able to change it, but the screen said so before
  I tapped."
- **One run of each.** Rosa held back on both earlier faces and tapped on
  this one. One run can't tell the line's effect from the model's draw.

## What the others did

- **Marcus again saw a game and didn't want it,** at the same words and
  second: "Looks like a game, not for work. I didn't play it." His file
  says he plays no games on his phone. That is the persona, not the face,
  and the bar counts it as a miss.
- **The first word spread out.** "situation" was the first word for 5 of 7
  in the first run. Here it was 2 of 9 (Dev, Walt), and "raccoon" was 2
  (Grace, Priya).
- **Fatima's tap was counted wordy,** the set's one hard fault. She tapped
  "entire", the 12th word in xenia's reading order, past her budget of 10.
  In the first run she tapped "situation", the 10th. The tray and its order
  didn't change, and she never read the line: her looks ended at "Tap a".
  So the fault comes from the word she chose, not from this change.
- **They knew it is shared.** Grace: "Anyone in the group can change it,
  and tapping a word takes it out." Walt: "Tapping a word in the sentence
  takes it out again." Priya: "the group can change it." Dev: "There was
  some small text under it I skipped." Walt again said the words are small:
  "I'd want my glasses."

## The instrument's own

- Ana's "my second tap on 'situation' didn't visibly land", Jess's "my
  next two taps seemed to end the screen" and Priya's "the screen moved
  on" are xenia ending the part at its goal, not the page. The first run
  noted the same. Ana's, Fatima's, Jess's and Priya's traces hold taps
  after `"end": "goal"`, and each board kept only the first word
  (`set.log`).
- `findings/` is what xenia kept: "Too many words before acting" (Fatima)
  and "Almost left" (Marcus, and Rosa), with `index.md`. Its "To
  reproduce" line names the scratch directory that stood for `<scenes>`.
  `accounts/` holds each account as xenia wrote it,
  `<player>-first-move`, and its trace.

## What this doesn't say

- **It is not a people result.** Proof 1's bar is people on their own
  phones, filmed, over at least two sessions. It stays never-ran.
- **It is not a judged count.** The set is could-not-judge, and its 9 of
  10 isn't set against the first run's 7.
- **One run of each player.** One run is not a rate, and two faces' runs
  are not one rate.
- **It doesn't cover drag, five seconds, or the time alone with the
  fridge.** xenia doesn't drag, its clock can't read five seconds, and the
  `solo` half didn't run here.
