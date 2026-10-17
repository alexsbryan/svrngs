# Scoped use made production, rerun, the computer first — 2026-10-10

sup-4 (`ralph/next/sup/STATE.md`): scoped use's spike
(`demo/runs/su/2026-10-09/README.md`) run again on what svrngs carries since
sup-1 to sup-3 — the app named on the pass and by a computer's origin; a
person's yes and stop kept and decided by the lend's door; the yes said
only on a page no app runs at, the home page on a computer and the group's
page on a phone (svrn-docs `rings/current/SCOPED_USE_MADE_PRODUCTION.md`;
DESIGN §3, "What an app gets"). The taste and Tonight's Picks were cut to
it first (`da3004d`): the taste keeps `refuse` and `owner` (row 1 waits,
DESIGN §9) and its operations; Tonight's Picks reads `/svrngs-reads`,
asks through `ask` and stops through `svrngs-stop`. Judged against su-0's
bars (su README §3) by their spirit, su-0's rule counting the lines (§4
there). One host, one run of each: not a rate (AGENTS.md 7). A run that
could not finish is could-not-judge, never a pass (AGENTS.md 6).

**What changed in the check, and why** (`517225a`): the computer is judged
first (the queue's fence 1) — Ana's computer, and Alex's for bar 2, each
from its own home page, in Chromium and in WebKit as a desktop; then Ana's
two phones and Bo, as su-1's nights. The plants run on the computer, in
Chromium. The taste is lent `--refuse /diary --ask "the app receives ranked
film suggestions; your viewing history stays where it is stored"` (the
design's §4 example) and lent afresh before each journey, so its door keeps
no one's choice. Two plants changed, since the check each took out is no
longer the taste's: `other-app` is the taste lent without `--ask`, a
lender's declaration as `history` is; `after-stop` is the stop's record put
back to the yes it replaced, after the press — a door that kept no stop.
The door's own checks of both were watched failing in sup-2's tests.
Bar 1's letter is "asked once": until another device for the same person
(DESIGN §9), each of Ana's phones is a key of its own, so she is asked once
on her computer and once on each phone, said so (the queue's row). The
measured gap of su-1, the other app saying Ana's yes itself, is judged now:
"the unasked yes", refused by svrngs.

## Predictions, 2026-10-10 01:35 UTC, before the run

- **The computer, bar 1: passes in both engines.** Ana opens Tonight's
  Picks from her home page; it is invited, not ranked; one press sends her
  to the home page's `/ask`, which asks "Let Sam's tonights-picks use Ana's
  taste?" around the lender's words; Yes, and she is back at the list in
  the plant's order, Arrival, Brazil, Blade Runner, Alien, 3 to 8 s after
  the Yes — each opening on a computer asks every lender for
  `/svrngs-reads`, Cy's asleep one for its 3 s (`node::ANSWER_WITHIN`),
  then both Jellyfins, then the taste. Opened again from the home page: no
  invitation, the list ranked.
- **Before her yes, on a computer, Tonight's Picks cannot read svrngs's
  `unasked`.** The taste is at its own origin, and svrngs's own answers
  there carry no `access-control-allow-origin` (none in
  `crates/svrngs-node/src`), so the page's `fetch` fails and Tonight's
  Picks says it could not read whether the taste lets it rank
  (`refused-unread`), and invites, since the lend `asks`. Likewise `stopped`
  after a stop, and the door's 403 for `/diary`. The taste's own answers
  are read: it allows any origin. On a phone every answer is read: the
  worker answers at the page's own origin.
- **Bar 2: passes on the computer and the phone.** Alex's computer and Bo's
  phone, each invited, say yes on their own page; Ana's door carries their
  ask, and the taste refuses `not-yours` by its 3 lines of owner. Held by
  the taste still, as row 1 waits.
- **Bar 3: passes in its spirit; on a computer its letter is not seen by
  the page.** Ana's door refuses `/diary` ("the lend refuses this path"),
  the taste logs no ask for it, and no planted marker is in any byte a
  browser received. On a phone the page reads 403 `path`; on a computer the
  refusal is unread (above).
- **Bar 4: passes, and the unasked yes is refused by svrngs.** The other
  app's ask refused `unasked` at Ana's door, the taste seeing none of it;
  its own yes refused — on the computer at the home page, whose rule
  answers only itself (`home: … answers=false`), on a phone at the door,
  403 `closed`, "an app may only stop; its person says yes" — and its next
  ask refused `unasked` again.
- **Bar 5: passes.** After Stop in Tonight's Picks, the next ask refused
  `stopped` at the door: on a phone the page says so, 1 to 4 s after the
  press, the page's reload included; on a computer the page reads it as
  unread and the door's log says `stopped`, 3 to 8 s after (the reload
  waits on Cy's 3 s). The home page's "What apps use" says Tonight's Picks
  was stopped. On the computer, yes again then Stop on the home page:
  refused alike. Ana's other phone, its own key's yes kept, still ranked
  after the first phone's stop: said, not judged, until another device for
  the same person.
- **Bar 6: passes.** Each refusal said as itself beside the four films
  unranked; Ana's computer asleep after a phone's list: "did not answer",
  10 to 60 s after opening (su-1: 32,384 ms), the films unranked.
- **Each plant fails its bar**: `bo`, Alex ranked by Ana's taste;
  `history`, `/diary` answered with the five rows (read on the computer,
  since the taste allows any origin); `other-app`, the other app ranked;
  `after-stop`, the ask after the stop ranked; `empty`, Alex's refusal
  shown as no picks.
- **The count**, su-0's rule at `da3004d`: enforce 11 (refuse 8, owner 3;
  su-1: 46), plumbing 16 (find 2, refusal 14; su-1: 39), own 172, plant 3.
- **The product unchanged**: `git diff 502d0e6..HEAD -- crates web` empty;
  `scripts/lean_check.py` 15,272 of 15,350; `scripts/check.sh` names
  neither Jellyfin nor taste.

## The run, against the predictions

Run by `run.sh` here from 02:00 to 02:18 UTC, 2026-10-10, through iroh's
public relays; the repository at `7cbf217`, then `c0c999a` for the three
journeys taken again (below); Playwright's Chromium 147.0.7727.15 and
WebKit 26.4 — as a desktop on the computers, an iPhone 15 on the phones;
the scratch at `/tmp/sup`, stopped after. Every number below is in this
directory (`478fedf`): each journey in `<tag>.json`, the judge's lines in
`steps.log` and `verdicts.json`, what the taste and Ana's computer logged
during each in `<tag>-taste.log` and `<tag>-ana-door.log`, Alex's
computer's in `<tag>-alex.log`, the choices Ana's door kept after each in
`<tag>-choices.json`, every byte each browser received searched in
`scan.json`. One host, one run of each: not a rate (AGENTS.md 7).

**Taken again.** The first takes of `computer-chromium`,
`computer-webkit` and `plant-after-stop` (02:04-02:07, kept in
`first-take/`) were the driver's failure, not svrngs's: after the home
page's Stop the page reloads itself, the driver's next `goto` raced it
("Navigation … is interrupted by another navigation"), and `timed`
(`apps/jellyfin/check/person.mjs:106`) wrote that over the stop step's
record, which su-1's driver kept under the same name. Fixed in `c0c999a`
and run again, 02:08-02:11.

| Bar | Predicted | The data | Verdict |
|---|---|---|---|
| 1, the computer | passes in both | Ana's first open: `refused-unread` and the invitation; one press to the home page's `/ask`, 3,515 ms (Chromium) and 6,988 ms (WebKit), which asked "Let Sam's tonights-picks use Ana's taste?" above the lender's words; Yes, and the list in the plant's order, Arrival, Brazil, Blade Runner, Alien, 3,320 and 3,239 ms after; opened again from the home page, no invitation, ranked, 3,200 and 3,290 ms | **passed** in both |
| 1, the phones | once on each phone | the first phone invited, `refused-unasked`; the group's page asked in 4,987 and 5,053 ms; ranked 3,406 and 4,964 ms after Yes; opened again, no question, 2,400 and 2,836 ms. The other phone asked once and said yes, ranked 3,352 and 4,488 ms after; again, no question, 2,225 and 2,919 ms | **passed** in both, "once" per key, said |
| Before the yes, on a computer | svrngs's refusals unread | every `unasked`, `stopped` and `path` at the taste's origin unread by the page — "Failed to fetch" (Chromium), "Load failed" (WebKit), the console naming `Access-Control-Allow-Origin` — and said as "Tonight's Picks could not read whether Ana's taste lets it rank"; the taste's own answers, its `not-yours` too, read | as predicted |
| 2 | passes, the taste's owner | Alex (computer, both engines) and Bo (phone, both) invited, each said yes on his own page; Ana's door carried the ask; the taste refused `not-yours`, "Ana's taste is someone else's. These films are not ranked.", the four films beside it | **passed**, held by the taste's 3 lines |
| 3 | passes in spirit | Ana's door "refused — the lend refuses this path" `/diary` in all four, the taste no line for it; the phones read 403 `path`, the computers nothing (unread). No marker in any byte the eleven browsers of the kept runs received; all five in `plant-history`'s (`scan.json`) | **passed** in spirit; on a computer the page saw no 403 |
| 4 | passes | the other app's ask `judged=Err(Unasked)` at Ana's door, its app the other's label, the taste seeing none of it; on a phone 403 `unasked`, read | **passed** in all four |
| The unasked yes | refused by svrngs | computers: the other app's `POST /choices` to the home page refused by its rule, `site=Some("cross-site") … answers=false`, its next ask `unasked` again, no choice kept for it. Phones: the app's own endpoint for the pass's holder never came online beside the lend's worker's, which holds the same key on the relay ("did not come online within 30 s"); its next ask `unasked` | computers **passed**; phones **could-not-judge**: the yes never reached the door |
| 5 | passes | the app's Stop kept in 58 and 35 ms (computer), 101 and 101 ms (phone); the next ask refused `stopped` at the door 3,161 and 3,162 ms after the press on the computer — its reload waits 3 s on Cy's computer for `/svrngs-reads` — and 480 and 580 ms on the phone; the computer's page read it as unread, the phone's as "You stopped Tonight's Picks using Ana's taste." The home page said "Sam's tonights-picks was stopped using Ana's taste." On the computer, yes again then the home page's Stop: answered 200, the next ask `stopped` 6,229 and 6,303 ms after (the driver read the home page and opened the app first). Ana's other phone, after the first phone's stop: ranked, its own key's yes | **passed**; the other phone said |
| 6 | passes | each refusal — another person's, the stop's — said beside the four films unranked, in all four; Ana's computer asleep after a phone's list: "Ana's taste did not answer: its computer may be asleep." at 32,628 ms (su-1: 32,384), the films unranked | **passed** |
| The plants | each fails | `bo`: Alex ranked by Ana's taste; `history`: `/diary` answered with the five rows; `other-app`: the other app ranked; `after-stop`: the ask after the stop ranked; `empty`: Alex's first refusal shown as no picks and no films | each **failed** its bar |
| The count | enforce 11, plumbing 16 | enforce 11 (refuse 8, owner 3; su-1: 46), plumbing 16 (find 2, refusal 14; su-1: 39), own 172, plant 3, none unmarked | as predicted |
| The product | unchanged | `git diff 502d0e6..HEAD -- crates web` empty; `scripts/lean_check.py` 15,283 of 15,350 — 11 over the 15,272 predicted, this directory's `run.sh`, which the budget counts; `scripts/check.sh` names neither Jellyfin nor taste | as predicted, but `run.sh` |

**What no prediction named.**

1. *A lend's page asks the home page cross-site, not same-site.* The
   design (§5) has lend pages same-site with the home page; both engines
   sent `Sec-Fetch-Site: cross-site` for the other app's post (each
   `*.localhost` name its own site, `localhost` being on no public
   suffix list). The rule refuses both; nothing changes.
2. *The two pages word svrngs's question apart.* The home page asks "Let
   Sam's tonights-picks use Ana's taste?" (`home/home.js` `named`), the
   group's page "Let Sam’s tonights-picks (movie-night) use Ana’s
   taste?" (`web/app.js` `what`): one question, two sources.
3. *Tonight's Picks finds its stop address in its `ask` address.*
   `/svrngs-reads` gives a lend that asks `ask`, no `stop`, so the app
   reads the label from `ask`'s `lend=` (`picks.js` `stopAt`), 1 of its 2
   `find` lines.

## Where the letter and the spirit part (sup-4)

Every bar passed in its spirit on the computer and the phone, each
judged by what it guards (the queue's fence 6):

- *Bar 1 on a computer passed, and its first words are not the app's
  invitation alone.* Tonight's Picks could not read that the door refused
  `unasked`, so it said "could not read whether Ana's taste lets it rank"
  above the invitation. The person still added the benefit plainly, once,
  in svrngs's words around the lender's, and got it. Follow-up 1.
- *Bar 5 on a computer passed, and the app cannot say "you stopped
  this".* The stop took effect at the door, and nobody was told sooner:
  the app's words at the press are true, and the home page says it was
  stopped. But the reopened app reads the door's `stopped` as unread, so
  it invites again where the design has it say the stop. Follow-up 1.
- *"Asked once" is once per key.* Ana said yes on her computer and once on
  each phone, and a stop on one phone left the other phone's yes. That is
  DESIGN §9's another device for the same person, which row 1 waits on,
  not a gap of this queue's; the choices are keyed by the door's
  `PersonKey`, so it is one function there.
- *The unasked yes on a phone was not exercised in the field.* The door's
  refusal of a holder's yes is held by sup-3's test
  (`crates/svrngs-node/tests/browser`, `web/tests/ask.mjs` step 5), which
  hands a page's pass and holder key to a dial of the test's own; in this
  run the app's page could not bring its own endpoint for that key online
  beside its worker's. Could-not-judge, never a pass (AGENTS.md 6); bar 4
  held, the next ask refused `unasked`.
- *Bar 2 is still the taste's 3 lines.* Row 1, a lend only its lender's
  person reaches, waits (DESIGN §9).

**svrngs carries scoped use now.** Of su-1's 46 enforcing lines the taste
keeps 11: its own refusals' words (8) and its owner (3), as the design
set out (`rings/current/SCOPED_USE_MADE_PRODUCTION.md` §1, "from 46 lines
to 12"). The app, the yes and the stop are the door's, and the plants of
bars 4 and 5 are now a lender's declaration and a door that kept no stop,
not one line of each owner's program. Tonight's Picks' plumbing is 16
lines, 2 of them finding (su-1: 26), none reading svrngs's store.

## Follow-up rows, the operator's to queue

1. **svrngs's answers at a lend's origin readable by the page that reads
   it, on a computer.** svrngs's own answers carry no
   `access-control-allow-origin` (`grep -rni access-control
   crates/svrngs-node/src` finds none), so a page asking another lend by
   `fetch` reads none of `unasked`, `stopped`, `path`, `away` there
   (`computer-*.json`, `refused-unread`; the consoles). The computer's
   origin knows the page's origin and that it reads the lend
   (`origin.rs` `app_of`, `kept_to_reads`): its own answers to that page
   could allow that origin and expose `x-svrngs-refused`. Check: Tonight's
   Picks on a computer says `refused-unasked` before the yes and
   `refused-stopped` after a stop, in both engines; a page that does not
   read the lend still reads nothing.
2. **One wording of svrngs's question.** The home page's and the group's
   page's, from one source (no-prediction 2). Check: both pages ask the
   same words, held by a test of each against the other's.
3. **A stop address beside `ask`.** `/svrngs-reads` gives `stop` for a
   lend that asks, on a computer and a phone (no-prediction 3). Check:
   Tonight's Picks' `find` 1 line.
