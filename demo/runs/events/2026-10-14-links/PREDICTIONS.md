# The dinner's link, and Sam's computer asleep — setup and predictions, 2026-10-14

Row events-doc-link (`ralph/next/events/STATE.md`): events-run's link and
asleep steps again, now that svrngs tells a lend where it is (events-links,
12d6331, 0c88b2f, db36fec) and caps the dial to a lend's computer at 4 s
(events-asleep, events-wait: cb71f8e, a07a9cc). Written before the driver
and any data. One host, one take: not a rate (AGENTS.md 7). Every person
is a program; no people bar is judged.

## Setup

`W=<scratch> python3 demo/runs/events/2026-10-14-links/run.py`, at this
commit's svrngs, a debug build, n0's public relays, every computer and
phone on this host; data in `data/` beside it, every page a person is
shown saved as `data/shots/<n>-<who>-<step>.png`.

As events-run (`../2026-10-14/PREDICTIONS.md` §1): Sam makes The House
and dWeb, each with `--origin` the page served on this host; Bo's computer
joins The House by its link and Cy's phone by its page; Di's computer
joins dWeb and Eve's phone. The reference app runs on Sam's computer and
is lent with `svrngs share`. Computers' browsers are Chromium keeping no
group (their people joined with svrngs, not in the browser), phones WebKit
as an iPhone. Sam posts Dinner, Friday 7, the house, to both groups.

The steps:

- **L1.** On the dinner's page Sam presses "Copy the link to this event"
  in each group's part; the clipboard read back, and the box beside it.
- **L2.** Each phone opens its group's link as from a chat: Cy The
  House's, Eve dWeb's. Each presses "I'm coming".
- **L3.** Each computer opens its group's link in its browser: Bo The
  House's, Di dWeb's. Where the page says it opens in svrngs, the link it
  shows is pasted on that person's home page under "Open a link from your
  group" and "Open it" pressed. Each presses "I'm coming".
- **L4.** The other group's link: Cy's phone opens dWeb's; Di pastes The
  House's on his home page.
- **L5.** Sam's computer **stopped**, as events-run's step 12 stopped it
  (Ctrl-C): Bo pastes The House's link on his home page again; Bo's home
  page read; Cy, the dinner still open on her phone, opens it again (its
  own address on the lend's origin, as a reload); Cy opens the link from
  the chat again. Then Sam's svrngs started, and Bo and Cy open it again.
- **L6.** Sam's computer **frozen** (SIGSTOP), as a lid closing sends
  nothing: Cy first opens the dinner, so her phone holds a dial to Sam's
  computer; then the same four as L5. Then SIGCONT, and Bo and Cy again.

Each wait is timed from the press, or the address opened, to the page
shown or the words that say it did not answer.

## Predictions

- **P1.** Each group's part of Sam's page holds one link,
  `<page>#open=<label>&at=/g/<group>/e/<event>`: `<page>` the page served
  here, `<label>` the lend's in `/v1/lends`, the event the one posted. No
  token in either. Copy says "Copied", and Chromium's clipboard holds the
  box's link.
- **P2.** Cy's and Eve's phones, at their link, land on the dinner at their
  group's address within 10 s, their own group's part alone, and each
  "I'm coming" counted.
- **P3.** Bo's and Di's browsers, at their link, say "This link opens in
  svrngs on your computer." and show the link. Pasted on the home page, it
  opens a tab on the dinner, at the group's address on the person's own
  computer, within 10 s; their own group's part alone, "I'm coming" counted.
- **P4.** The other group's link: Cy's phone lands on dWeb's address and is
  told "You're not in dWeb, so this event isn't yours to see."; Di's tab
  told the same of The House. Neither is shown the other group's answers.
- **P5 — stopped, within the cap.** Bo's pasted link: 502 `away`,
  `{"error":"the computer that lends this did not answer; try again
  later","kind":"away"}` in braces (events-run's finding 2, not new), in
  4.0–5 s. Cy reopening the dinner: "The computer lending this did not
  answer. Try again later." in 4.0–5 s. Bo's home page says "Sam’s
  computer did not answer, so what they lend is not listed." within 10 s.
- **P6 — stopped, the link anew: not within the cap.** Cy opening the link
  from the chat is told at her group's page, not the dinner's: "Your
  group’s computer did not answer through <relay>. Try again later.", after
  about 30 s (29–35 s). The page dials the computer whose link she joined
  by, Sam's, before it lists anything lent (`web/app.js:314` `enter`), and
  that dial is uncapped: "the dial to the computer a link names waits as
  long as iroh does" (`web/src/lib.rs` `reach`, `dial`; 5c097eb kept the
  joiner's dial uncapped).
- **P7 — awake again.** Within 90 s of Sam's svrngs starting, Bo's pasted
  link and Cy's link each show the dinner, with every answer L2-L3 gave.
- **P8 — frozen.** As P5 and P6: Bo 4.0–5 s; Cy reopening the dinner, her
  dial held, 4.0–5 s (events-wait (b): 4,010–4,271 ms); Cy's link anew
  about 30 s; Bo's home page within 10 s. After SIGCONT, as P7.
- **P9.** No page says key, member, record, act, namespace or origin; no
  page shows a group's part with no one on it while Sam's computer is
  asleep.

What would falsify the doc's story: P2 or P3 failing (the link does not
reach the dinner), P5 or P8's waits past 5 s on the lend's own address.
P6 is a prediction of a wait the row hoped was within the cap; if it holds,
it is svrngs's, written in `NEEDS_HUMAN.md` here, never worked around in
the app.
