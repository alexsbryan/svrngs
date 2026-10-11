# sup-4's follow-ups run on a computer — 2026-10-10

lean-4-once-and-follow-ups (b) to (d) (`ralph/next/lean/STATE.md`): sup-4's
computer journeys (`demo/runs/sup/2026-10-10/`, follow-ups 1 to 3) run
again on what changed — svrngs's own answers at a lend's origin readable by
a page that reads the lend (b, 9cdfda5), the question and "What apps use"
in one wording (c, 7b10b4a), `stop` beside `ask` (d, c42939a). `run.sh`
is what ran: apps/taste/check/check.sh `up`, then `computer` in Chromium
and in WebKit, then `down`. One host, one run of each: not a rate
(AGENTS.md 7).

## Predictions, before the run

1. In both engines, on Ana's computer from her home page, Tonight's Picks
   says `refused-unasked` before the yes (sup-4 said `refused-unread`:
   its fetch could not read svrngs's refusal), and `refused-stopped` after
   the stop, where sup-4 said `refused-unread` again.
2. The home page asks "Let Sam’s tonights-picks (movie-night) use Ana’s
   taste?" — the kind now kept, as the group's page words it.
3. Tonight's Picks' Stop posts at the `stop` `/svrngs-reads` told it, and
   the stop is kept (`computer-*-choices.json` reads `stop` for
   tonights-picks after it).
4. `/diary` still refused `path` at Ana's door, and the page now reads
   that kind where sup-4's could not.
5. Nothing else moves: Alex's computer, the other app's own yes refused,
   the home page's Stop, each as sup-4 recorded.

## The run, against the predictions

Run by `run.sh` here from 00:59 to 01:04 UTC, 2026-10-11 (17:59-18:04
local, 2026-10-10), through iroh's public relays; the repository at
`5803555` — the product as lean-4 (d) left it (`c42939a`), the driver's
apostrophe fixed after the first take (below); Playwright's Chromium
147.0.7727.15 and WebKit 26.4, as a desktop; the scratch at `/tmp/supf`,
stopped after. Each journey is in `computer-<engine>.json`, the steps in
`steps.log`, what the taste and Ana's computer logged in
`computer-<engine>-taste.log` and `-ana-door.log`, Alex's computer's in
`-alex.log`, the choices Ana's door kept by the end in `-choices.json`.
`-ana-door.log` keeps only lines naming the taste or the home page
(apps/taste/check/check.sh:179), which drops the origin's new trace, so
`ana-readable.log` is Ana's computer's whole log grepped for it, after the
run: `grep "svrngs's own answer, readable\|the lend's own answer to a page
that reads it" /tmp/supf/ana/run.log`. One host, one run of each: not a
rate (AGENTS.md 7).

**Taken again.** The first take (00:48-00:57, kept in `first-take/`)
stopped at the driver in both engines: the home page's lends list now
reads question.js `named`, whose apostrophe is `’`, and the driver looked
for `'s` (`5803555`).

| Prediction | The data | Verdict |
|---|---|---|
| 1, `refused-unasked` before the yes, `refused-stopped` after the stop | Ana's first open, both engines: `refused-unasked`, "Not ranked: you have not let Tonight’s Picks use your taste." (sup-4: `refused-unread`); Yes from the home page's question, ranked Arrival, Brazil, Blade Runner, Alien 3,257 ms (Chromium) and 3,269 ms (WebKit) after. Tonight's Picks' Stop said "Stopped" in 51 and 37 ms; opened again, `refused-stopped`, 3,103 and 3,155 ms after the press (sup-4: `refused-unread`). After a yes again and the home page's Stop (200): `refused-stopped`, 6,287 and 6,333 ms after. Ana's computer traced each readable refusal with its kind, 7 an engine: `unasked` ×3, `path`, `stopped` ×3 (`ana-readable.log`) | **held** in both |
| 2, the home page asks with the kind | "Let Sam’s tonights-picks (movie-night) use Ana’s taste?", from Ana's and Alex's home pages, both engines (sup-4: "Let Sam's tonights-picks use Ana's taste?"); the lends list "Sam’s tonights-picks (movie-night)" (`home_seen`) | **held** |
| 3, Stop at the told `stop` | Ana's door: "a choice said … said=Stop by_app=true", then "a choice kept … Stop" (`-ana-door.log` line 109-111, both engines), the app's own post; "What apps use" then read "Sam’s tonights-picks (movie-night) was stopped using Ana’s taste." (`stopping.uses`). `-choices.json` reads `stop` for Ana's key, but as the run ended, after the home page's Stop too | **held**; the choices file alone could not tell the two stops apart, the door's `by_app=true` does |
| 4, `/diary` refused `path`, now read | 403, `{"error":"taste is lent without /diary","kind":"path"}` read by the page in both engines (sup-4: "Failed to fetch", unread); the taste logged no `/diary` (`-taste.log`) | **held** |
| 5, nothing else moves | Alex: yes from his home page, the taste refused `not-yours`, "Ana’s taste is someone else’s. These films are not ranked." in both, as sup-4. The other app's own `POST /choices` refused by the home page's rule (`site=Some("cross-site") … answers=false`, line 86), no choice kept for it, as sup-4. The home page's Stop answered 200, as sup-4. The lend's own answers to the page carried nothing svrngs added, 20 an engine, each `status=200` (`ana-readable.log`) | **held**, with two moves it did not name (below) |

**What no prediction named.**

1. *Alex's first open reads `refused-unasked` too*, where sup-4's read
   `refused-unread`: the same change as prediction 1, at Ana's door, to
   Alex's page; after his yes the taste's own `not-yours`, as before.
2. *The other app reads its `unasked`*: its ask, then its ask after its
   own yes, `403` `unasked`, `{"error":"only an app you said yes to uses
   taste","kind":"unasked"}` (sup-4: "Failed to fetch"). The other app
   reads the taste's kind, so its page is one the allowance names; its
   yes is refused as before.

Every prediction held in both engines; follow-ups 1 to 3 of sup-4
(`demo/runs/sup/2026-10-10/README.md`, "Follow-up rows") are met on a
computer. Phones were not run: (b) changes only what a computer's origin
answers, and (c) and (d) are held on the phone by their tests.
