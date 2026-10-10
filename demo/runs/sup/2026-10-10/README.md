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
