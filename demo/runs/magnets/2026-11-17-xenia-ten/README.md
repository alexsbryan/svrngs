# Magnets' face, ten strangers in xenia — 2026-11-17

xenia sends a model playing a stranger through a page, told only what that
person was told, touching it only through a browser. This is proof 1's
rehearsal at its full count: ten first-time players, where the first read
(`../2026-11-17-xenia/`) had three. It is simulated, a rehearsal, and never
a people result. The people bar in `demo/magnets.md` stays **never-ran**:
no person has played Magnets, no film was made, and no instrument was
checked.

## What ran

- **The face:** `apps/magnets/` at `c0859400`, served by its stand-in
  (`apps/magnets/tests/standin.mjs`), opened on a phone: Playwright's WebKit
  as an iPhone 15, xenia's phone. The face is the one the first read changed
  it to (`7a1ac173`). `index.html`, `style.css` and `pack.json` haven't
  changed since. Taps send `{write: [op]}` since `73cd91d0`.
- **The scene:** `first-move` alone (`apps/magnets/tests/xenia/world/scene.mjs`).
  The fridge is already open, and nobody has told them anything. The goal
  is a word in the sentence. xenia runs every scene in the directory it is
  given, so `<scenes>` was a directory holding only `first-move` and
  `world`, each a link to its file in `apps/magnets/tests/xenia/`.
- **The players:** ten, in `apps/magnets/tests/xenia/people/`: the first
  read's Ana, Dev and Rosa, and seven new ones. Their files were in the
  checkout, uncommitted, when the set ran, and they land with this record.

  | Player | Age | Phones | Reading | budget | patience | wait_s | reads_wpm | act_s |
  |---|---|---|---|---|---|---|---|---|
  | Ana | 24 | does most things on hers | asks rather than guesses | 20 | 80 | 10 | 200 | 4 |
  | Dev | 19 | always in hand | taps first, reads later | 15 | 80 | 8 | 250 | 2 |
  | Rosa | 61 | messages, photos, weather | reads first; leaves what she can't tell | 40 | 200 | 30 | 150 | 6 |
  | Walt | 79 | set up by his granddaughter | slow, with glasses; presses to see | 45 | 220 | 40 | 110 | 8 |
  | Marcus | 46 | work only; no games | a glance to see what it's for | 12 | 50 | 10 | 190 | 3 |
  | Priya | 34 | quick; on night-shift breaks | skims, tired | 20 | 90 | 12 | 260 | 3 |
  | Tomás | 41 | WhatsApp all day | English as a second language, word by word | 35 | 160 | 20 | 120 | 3 |
  | Fatima | 37 | one-handed, a child on her arm | glances | 10 | 50 | 8 | 200 | 5 |
  | Jess | 28 | daily word games | plays first, reads the rules later | 15 | 150 | 15 | 280 | 2 |
  | Grace | 54 | comfortable | every word, top to bottom | 60 | 240 | 25 | 230 | 4 |

- **The strangers:** `claude-sonnet-5-5`, as `claude -p`. Their spend isn't
  reported on that route (`summary.json`: `total_cost_usd` null).
- **xenia:** `df729c3`, with its checkout's uncommitted changes (16 files;
  `git diff | shasum` 3ced42f42d12). It's the same xenia as the first read.

```sh
cd ~/dev/xenia && lockf -k /tmp/svrngs-gate.lock node src/run.ts run <scenes> \
  --runs 1 --model claude-sonnet-5-5 --people <svrngs>/apps/magnets/tests/xenia/people \
  --findings <svrngs>/demo/runs/magnets/2026-11-17-xenia-ten/findings --source <svrngs>
```

The set ran on Node 24.22.1, put first on `PATH` because nvm's default
here is 20. It ran from 22:31 to 22:39 PST and exited 0. Its verdict was
**passed**: all 21 plants came back as they must, so its accounts can be
judged (`summary.md`, `set.log`). Of ten accounts, 7 passed, 1 failed, 2
were could-not-judge, and none never-ran.

## Can a first-time player move a word in five seconds, with no instructions?

| Player | Moved a word | First word | Landed, xenia's clock | Words read before | Looks before | Verdict |
|---|---|---|---|---|---|---|
| Jess | yes | situation | 11.0 s | 30 | 2 | passed |
| Dev | yes | entire | 11.7 s | 30 | 2 | passed |
| Priya | yes | goose | 15.8 s | 40 | 2 | passed |
| Fatima | yes | situation | 16.5 s | 20 | 2 | passed |
| Grace | yes | situation | 20.0 s | 40 | 2 | passed |
| Ana | yes | situation | 20.6 s | 40 | 2 | passed |
| Walt | yes | situation | 30.4 s | 40 | 1 | passed |
| Marcus | no | — | stopped at 10.6 s | 24 | 2 | failed |
| Rosa | no | — | stopped at 16.0 s | 40 | 1 | could-not-judge |
| Tomás | no | — | stopped at 23.0 s | 40 | 2 | could-not-judge |

**"Landed"** is xenia's clock when the tap that put the word in was done:
the `clock` of the step before it, plus the tap's `charge.act` and
`charge.surface`, in `accounts/<player>-first-move.trace.json`. It leaves
out the look that followed. The account's own "Clock" is later, since it
also counts their answers afterward. "Words read before" is their patience
less the `left` of that step before. "Stopped at" is the clock at their
last look.

**Against the bar:**

- **Moved a word: 7 of 10.** The bar asks 8. One miss was judged
  (Marcus). Two were could-not-judge (Rosa, Tomás), and neither moved a
  word: the board was left `[]` (`set.log`). So this set can't reach 8
  however those two are judged.
- **Within five seconds: could-not-judge.** The fastest landed at 11.0 s.
  On xenia's clock, none of these ten can land in five seconds even if
  they tap from their first look. That floor is one look's reading (their
  budget, or the page's 40 words if that is fewer) plus one tap. It runs
  from 5.2 s (Jess: 15 words at 280 a minute is 3.2 s, and a tap is 2 s)
  to 29.8 s (Walt). The clock is reading time, word by word. It is not a
  person's eye taking in a screen at once. Five seconds is for the people
  run to read.

## What failed

Three players moved nothing. None of them missed because they couldn't
find the move. Each named the tap ("words to tap", "about 15 tappable
words"). Their misses are of two kinds.

- **Marcus saw a game and didn't want it.** He looked twice, 24 words:
  the tray, the heading and "Tap a word to add it". Then he stopped:
  "It's a game, not for work, so I'm leaving it alone." "I couldn't tell
  what it was for at a glance, and once I saw it was a game, I stopped."
  His file says he plays no games on his phone. It is a judged miss, and
  the bar counts it as one.
- **Rosa and Tomás read that the sentence is shared, and waited to be
  asked.** Each took in the whole page, its 40 words, with "Everyone in
  your group sees this sentence and can change it." among them. Rosa:
  "Nobody has asked me to do anything, so I haven't pressed anything."
  Tomás: "I didn't tap anything, because nobody asked me to make a
  sentence and it would show to the whole group." Both looked for a task:
  "I expected a screen telling me what to do" and "I expected a task or
  instructions, like a message to send or a place to go". Then each spoke
  as an assistant ("Do you want me to pick a word?", "Tell me which words
  you want, if any."), so xenia can't judge them. The holding back may be
  the model's, not the person's. Rosa did the same in the first read,
  before the shared line moved under the sentence. The line now reaches
  her first look, and she still didn't tap.

`demo/magnets.md` names two causes of a miss: starting with a drag, or a
first look that holds too many words. These misses fit neither. xenia only
taps. Rosa and Tomás read the whole page within their budgets, and they
stopped at what it said, not at how much there was. The part of the face
they stopped at is the one line about sharing, on a page with no other
invitation than "Tap a word to add it". Nothing in the app changed for
this record. Whether it changes is the operator's.

## What the seven who moved did

- **Six of the seven looked twice before tapping.** xenia reads things to
  tap first. With a budget of 20 or less, the first look took in the tray's
  words, and "Magnets" and "Tap a" at most. All five of those players
  looked again before tapping. Four of them reached the sentence's words
  in that second look. Fatima's ended at "Tap a". Walt, whose budget of 45
  covers the page's 40 words, tapped from his first look. Grace read the
  whole page in one look, looked again, found nothing new, and tapped. The page doesn't
  scroll on this phone (`../2026-11-17-xenia/first-look.json`, "after").
  xenia's scroll here is a second look.
- **"I can explain the situation."** It was the first word for 5 of 7:
  Ana, Fatima, Grace, Jess and Walt. Dev put in "entire" and Priya "goose".
- **The tray held still.** No one said it moved. In the first read, two
  of three did ("The word list rearranged after the first tap"). No one
  called the list too much, and no `wordy` fault was counted. Dev had one
  in the first read's `solo`, when the tray held 32 words.
- **They knew it is shared, and how to take a word out.** Ana: "The whole
  group can see and change it." Priya: "The house can see and change it."
  Grace: "Tapping a word in the sentence removes it." Walt: "Tap it in the
  sentence to take it out." Grace also: "I didn't know it was shared with a
  group until I read the line at the bottom." The line sits under the
  sentence. It is last in xenia's reading order, which is things to tap,
  then headings, large text, and the rest.
- **Walt: "Small print was hard to read."** That echoes his file ("I need
  my glasses for the small print"). The two lines under the sentence are
  `.hint`, 0.95rem in dimgray (`apps/magnets/style.css`). A people run can
  see whether that is real.

## The instrument's own

- Dev's "the screen went blank after my taps", Priya's "my second tap,
  'situation', didn't seem to register" and Jess's "it ended after one tap"
  are xenia ending the part at its goal, not the page. Dev's and Priya's
  traces hold taps after `"end": "goal"`, and Jess's a look. This is the
  same thing the first read noted for Dev.
- Each look gives the stranger a screenshot beside the words its budget
  reads. In Marcus's session, each look's result is its text and an image.
  Marcus quoted "I can explain the ____", which the words of his two looks
  never held: he read it off the image. "Words read" is what the stage
  charged, not all a stranger took in.
- The could-not-judge is the stage's check for an account in an
  assistant's voice. It held a careful player in the first read too
  (Rosa). Whether a careful person on a phone waits to be asked is for the
  people run to see.
- `findings/` is what xenia kept: one finding, "Almost left" (Dev, and
  Marcus), with `index.md`. Its "To reproduce" line names the scratch
  directory that stood for `<scenes>`. `accounts/` holds each account as
  xenia wrote it, `<player>-first-move`, and its trace.

## What this doesn't say

- **It is not a people result.** Proof 1's bar is people on their own
  phones, filmed, over at least two sessions. It stays never-ran.
- **One run of each player.** One run is not a rate.
- **It doesn't cover drag, five seconds, or the time alone with the
  fridge.** xenia doesn't drag, its clock can't read five seconds, and the
  `solo` half, whether they send it, didn't run here.
