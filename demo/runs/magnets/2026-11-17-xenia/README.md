# Magnets' face, a xenia read — 2026-11-17

xenia sends a model playing a stranger through a page, told only what that
person was told, touching it only through a browser. This is the cheap read
before people (`demo/magnets.md`, proof 1). Simulated: a rehearsal, never a
people result.

## What ran

- **The face:** `apps/magnets/` at `ba3e9cff`, served by its stand-in
  (`apps/magnets/tests/standin.mjs`), opened on a phone: Playwright's WebKit
  as an iPhone 15, xenia's phone.
- **The scenes** (`apps/magnets/tests/xenia/world/scene.mjs`):
  - `first-move`: the fridge already open, nobody told anything. The goal is
    a word in the sentence.
  - `solo`: an exploration of the same fridge, with a house chat holding
    one message from Sam ("Dinner’s at 7 tomorrow, I’m cooking."). What it
    watches is whether they send the fridge there unprompted.
- **The players:** Ana (`demo/xenia/people/ana.md`, budget 20 words), and
  two new ones in `apps/magnets/tests/xenia/people/`: Dev, impatient
  (budget 15), and Rosa, cautious (budget 40).
- **The strangers:** `claude-sonnet-5-5`, as `claude -p`. Their spend is
  not reported on that route.
- **xenia:** `df729c3`, with its checkout's uncommitted changes (16 files;
  `git diff | shasum` 3ced42f42d12).

```sh
cd ~/dev/xenia && lockf -k /tmp/svrngs-gate.lock node src/run.ts run <svrngs>/apps/magnets/tests/xenia \
  --runs 1 --model claude-sonnet-5-5 --people <svrngs>/apps/magnets/tests/xenia/people \
  --findings <svrngs>/demo/runs/magnets/2026-11-17-xenia/findings --source <svrngs>
```

The set ran on Node 24.22.1, from 16:56 to 17:04 local, and exited 0. Its
verdict was **passed**: all 21 plants came back as they must, so its
accounts can be judged (`summary.md`, `set.log`). Of six accounts, 5 passed
and 1 was could-not-judge: Rosa in `first-move` spoke as an assistant ("Tell
me if you'd like a particular word put in.").

## Question 1: can a first-time player move a word in five seconds, with no instructions?

**Moved a word:** 4 of 6 accounts. Ana and Dev each moved one in
`first-move` and played on in `solo` ("I can explain the entire lasagna
situation", "I can explain the situation somehow"). Rosa moved none in
either scene.

- **Ana and Dev read only word buttons first.** xenia reads things to tap
  first. With 32 of them, the 15- and 20-word budgets never reached the
  prompt or the heading. Ana scrolled before her first tap. In `solo`, Dev
  read 24 words before acting, against a budget of 15, a `wordy` hard fault.
- **Rosa read it, understood it, and left it alone.** "Tapping a word would
  change what everyone in the group sees, and nobody asked me to." "The
  sentence is shared, so a tap would change it for everyone, and nothing
  said so until I scrolled."
- **The tray moved under them.** Dev: "The word list rearranged after the
  first tap." Ana: "The word tiles kept reordering after each tap … I
  expected tapped words to stay put in the list, but they moved up into the
  sentence and the list shifted."
- **The five seconds: could-not-judge.** xenia's clock is reading time at
  each person's words a minute plus their seconds an act, from their file.
  It is not a person's eye. First moves came at 28.9 to 53.3 simulated
  seconds, after 24 to 60 words read. The bar is the people run's.

## Question 2: does a solo player want to send the link?

**Sent it:** 0 of 3. In `solo`, Ana and Dev answered Sam about dinner and
said nothing of the fridge, and Rosa said nothing. All three took it for
the group's game: "a fridge-magnet word game for the house", "a silly game
the group plays together". Two looked for a way to send it. Ana, in
`first-move`: "No send button that I saw." Dev, in `solo`: "I couldn't see a
send or done button."

## What changed in the face, and why

| Change | For | Check |
|---|---|---|
| A word in the sentence leaves its place in the tray as an empty dashed outline its size, so nothing moves under a thumb. The blank and the take-out hint keep their room too, so the first word moves nothing below. | Dev's and Ana's tray that "rearranged" and "shifted" | `apps/magnets/tests/page.test.mjs`, "invoice stays put". Its plant, the tray drawn without gaps, moved invoice from x 380 to 288 and failed there (:76). |
| 32 words become 16, which fit on one iPhone screen. | Dev's "the word list was long and a bit much to scroll through", his `wordy` fault, Ana's scroll before her first tap | `first-look.json`, computed with no model run: before, no budget reached the prompt or the heading, and the page scrolled. After, Ana's 20 words reach "Magnets Tap a", Rosa's 40 reach the sentence and the line below, and the page doesn't scroll. |
| The page's last line moved to just under the sentence: "Everyone in your group sees this sentence and can change it." | Rosa: shared, "and nothing said so until I scrolled", "nobody asked me to" | It is in Rosa's first look now (`first-look.json`). Whether it makes a cautious player tap is the people run's to see. |

`before-fresh.png` is the face xenia read. `after-fresh.png` and
`after-two-words.png` are the changed face on the same iPhone. It still
ships 327 lines (`apps/magnets/tests/lines.test.mjs`).

## What didn't change, and why

- **No send button.** Two players looked for one. But the link a friend can
  open is the group's ordinary app link, which svrngs knows and the page
  doesn't. On a computer the page's own address is that computer's
  loopback, so sharing `location.href` would send a link no friend can
  open. A send control needs svrngs to tell a page its group's link: a
  platform addition, for whoever places Magnets' platform work.
- **No drag.** xenia only taps, so it can't say whether people reach to
  drag a magnet. The people run records it (`demo/magnets.md`).
- **The changed face had no second set.** One set was the budget. The
  changes are checked by the gate's test and the computed first look. The
  people run decides.

## The instrument's own

- Dev's "taps after that did nothing" in `first-move` is xenia ending a
  part at its goal, not the page.
- Rosa saw Sam's message "turned up on top of a word game": that is xenia's
  chat arriving in a look.
- `findings/` is what xenia kept, one file a finding, with `index.md`.
  `accounts/` holds each account, `<person>-<scene>`, as xenia wrote it.
