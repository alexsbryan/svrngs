# Movie Night made production, rerun — 2026-10-09

mnp-2 (`ralph/next/compose/STATE.md`) made Movie Night's candidates 1 and 4
(`demo/runs/mn/2026-10-08/README.md` §9): a lent page told the lends it
reads, at its own origin, and a lend opened at a path from a page that
reads it (`ef75c07`); mnp-1 had made candidates 2 and 3 (`1f08560`,
`e02a203`). Movie Night was cut to them (`44784a4`): its `find`, 28 lines,
is one. This is Movie Night run again on them, as mn-1 ran it, by
`run.sh` here (`apps/movie-night/check/check.sh`, each step as written),
judged against mn-0's bars (`demo/runs/mn/2026-10-08/README.md` §3) by
their spirit, mn-0's rule counting the lines (§4 there). One host, one run
of each: not a rate (AGENTS.md 7). A run that could not finish is
could-not-judge, never a pass (AGENTS.md 6).

What changed in the check, and why (`44784a4`): Movie Night names each
library by its lender, so a copy is picked and judged by "Sam" or "Dee"
rather than its lend's label; a person whose computer did not answer
before the list is said as `unanswered` and is no library to wait on; the
computer's journey hands Movie Night nothing (mn-1 handed it the labels
in its address); bar 4 before the list is judged by its letter as well as
its spirit; the count is given for Movie Night's own files beside the
rule's letter, which since su-1 takes in Tonight's Picks in `picks/`; and
the product's diff is recorded, not judged untouched, since mnp-1 and
mnp-2 changed it by design.

## Predictions, 2026-10-09 08:25 UTC, before the run

What the row's check asks, each with what this run is expected to show:

- **Bar 2 passes for a phone that never opened that Jellyfin, in both
  engines.** Chosen from Movie Night, the copy's address goes through the
  group's page, which signs that Jellyfin's pass, and its start page to the
  item; jellyfin-web opens at *Blade Runner*, signed in with the stand-in,
  within 60 s, where all four landed on a 404 in mn-1. Bar 3 from there:
  one press of Back to Movie Night from the item, the group's page and the
  start page having stepped aside, and two from the playing film as in
  mn-1, its query and cards as they were.
- **Bar 4 passes for a library asleep before the list, named by its
  lender**: every search says "Cy’s computer did not answer", never a
  library of Cy's with nothing. Asleep after the list, as mn-1: "Cy’s
  jellyfin did not answer" about 60 s in, the others' answers first.
- **Bar 5's letter passes**: Movie Night's source names none of
  `x-svrngs-`, `svrngs-lend`, `.localhost` or a lend's label, and reads
  nothing of svrngs's store; no secret in any byte a browser received.
- **The computer finds both libraries with nothing handed**, Sam's and
  Dee's, Cy said not to have answered, and opens Sam's copy and plays it.
- **Bar 1 whole on the first search**: with mnp-1's newcomer let in by
  every lender, each phone person's cold search has both libraries
  answering, where in mn-1 Dee's refused them until her round.
- **The count**: Movie Night's own files, own 170, plumbing 12 — `find` 1,
  `ask` 11 (it still tells svrngs's refusals by status; `x-svrngs-refused`
  is not read here), `open` 0, `signin` 0.
- **The limit**: as mn-1's, `WINDOW`'s 30 s after a search.

## The run, against the predictions

Run by `run.sh` here from 08:22 to 08:45 UTC, 2026-10-09, the repository
at `a9e333f`; Playwright's Chromium 147.0.7727.15 and WebKit 26.4 as an
iPhone; the scratch at `/tmp/mnp`, stopped after. Every number below is in
this directory (`8566b86`): the journeys in `phone-*.json`, `asleep-*.json`,
`computer.json`, `limit.json`, `plant-*.json`; the judge's lines in
`steps.log` and `verdicts.json`, 43 passed, 10 recorded, 3 failed — the
three plants, each its bar (`plants: each failed its bar: True`); every
request each lender's door carried in `doors.log`, summed in `doors.json`;
every byte each browser received searched in `scan.json`.

| The row's check | Predicted | The data | Verdict |
|---|---|---|---|
| Bar 2, a phone that never opened that Jellyfin, both engines | passes | Ana and Ben (Chromium), Cleo and Dara (WebKit) chose a copy before their phone ever opened its Jellyfin: through the group's page (`#open=<label>&at=%2Fweb%2F%23%2Fdetails…`) to jellyfin-web at *Blade Runner*, served by that Jellyfin's worker, signed in with the stand-in, in 15,816, 16,304, 17,178 and 17,215 ms (`never_opened`); mn-1 landed all four on the page's host's 404. Opened from the list after: at the item in 15,666-18,702 ms, playing 2,550-3,077 ms after Play | **passed** |
| Bar 3 | one press from the item, two from the playing film | one press of Back from the never-opened item, 4 of 4, two from the playing film, 4 of 4 and the computer's, query and cards the same every time (`never_opened_back`, `opened_back`) | **passed** |
| Bar 4, asleep before the list, named by its lender | passes | every search of all four phone people, 5 each, said "Cy’s computer did not answer, so no library of theirs is here.", none shown empty; the group's page said the same | **passed**, letter and spirit (mn-1: letter failed) |
| Bar 4, asleep after the list | as mn-1 | "Cy’s jellyfin did not answer: its computer may be asleep." at 60,349 ms (Eli) and 60,219 ms (Fay), the others' first answers at 283 and 296 ms | **passed** |
| Bar 5's letter | passes | 0 lines of Movie Night's source name `x-svrngs-`, `svrngs-lend`, `.localhost` or a lend's label; 0 set a credential outside the plants; no lender's secret nor anyone's own token in 277,070,313 bytes eleven people received outside the token plant | **passed** (mn-1: failed) |
| The computer, nothing handed | finds both, plays | Alex's Movie Night, handed nothing: "Dee’s jellyfin: 1 film", "Sam’s jellyfin: 1 film", "Cy’s computer did not answer…"; Sam's copy at the item in 648 ms, playing 1,282 ms after Play, two presses back. Its first card at 10,248 ms (below) | **passed** |
| Bar 1 whole on the first search | passes | Ana's and Cleo's cold searches had both libraries; Ben's and Dara's had "Dee’s jellyfin would not let you in." at 10,384 and 10,368 ms — Dee's door refused 7 passes in the run (`doors.json`, `refused`). Every phone person's next search was whole (`settled.searches` 1; mn-1: 5, 3, 3, 3) | **failed** for 2 of 4, below |
| The count, Movie Night's own files | own 170, plumbing 12 | own 170 (cards 109, match 51, choose 10), plumbing 12 (`find` 1, `ask` 11), plant 7; by the rule's letter, Tonight's Picks with it: own 344 | as predicted |
| The limit | `WINDOW`'s 30 s | Hal counted on both Jellyfins 5.5-25.8 s after Gus's last search, 2 people, and 1 at 35.5 and 45.8 s; the most at once 2; Alex guessed home on every request (149 at Sam's Jellyfin) | as predicted |

**What no prediction named.**

1. *Every lender's round waits on an asleep one.* mnp-1's door, meeting a
   pass whose signer it does not know, asks the group's ring and waits up
   to 10 s for a round to end. Dee's round dials Cy, asleep, whose dial
   gives up at about 10 s: for Ben the round began at 08:31:34.866 and
   the door gave up at 08:31:44.869 (`ended=false known=false`), refused,
   and let him in at 08:31:45.043, the round having ended (Dee's log, the
   scratch's `dee/run.log`; `doors.log`). So with a computer asleep in
   the group a newcomer's first request at a second lender can still be
   refused, once, at about 10 s. mnp-1's piece, not this row's; written
   down, not fixed here.
2. *The computer's answer waits on an asleep lender too.* On a computer
   `/svrngs-reads` asks every lender now, as `/v1/lends` does
   (`crates/svrngs-node/src/surface.rs`, `reads`, through
   `Asking::lent`), so Cy's dial held Alex's every search's first card to
   10,248 ms; on a phone the group's page asked once, when the list was
   made, and the first card came in 90-324 ms. Movie Night asks before
   each search (`app.js`, `libraries`).
3. *A computer just started is a stranger to its peer's next ask.* In
   the first plant's run Sam's computer still held a connection to Alex's
   computer's last process, restarted by `up` 13 s before, and said Alex
   did not answer (`failed closing path err=LastOpenPath`, 08:25:38); the
   group's page and Movie Night both said so, as they should. Gone by the
   second plant's run.
4. *Tonight's Picks still reads svrngs's store.* `apps/movie-night/picks/`
   (su-1) reads `pass <label>` from `svrngs-lend` and builds each address
   as `baseOf` does (`picks/picks.js:20-49`); the row cut Movie Night, not
   it. Its own check is su-1's, not run here.

One host, one run of each: not a rate (AGENTS.md 7).
