# U7's removal, measured on one host — 2026-10-11

u7-removal (f) (`ralph/next/v1/STATE.md`): Demo C's removal check
(`demo/c.md`, "What is watched", **Removed**) on this host. Three computers
— Sam, the group's maker, so the one who decides who is removed; Ana; Bo —
and Cy's phone (Playwright's WebKit as an iPhone, the group's page served on
`localhost`), in one group, Trip, through iroh's public relays. Sam and Ana
each lend a page (`films`, `music`, each a plain file server); the three
computers share a folder, Trip. Sam removes Bo. `run.py` is the driver;
each take's data is in its own directory, `OUT=take-<n> python3 run.py`. One host, one run: not a rate (AGENTS.md 7). The
30 s bar is the seat's proposal, the operator's to set (`demo/c.md`).

The run, in order (`run.py`, each phase its own `phase` line in
`run/events.jsonl`):

1. **Setup.** Sam makes Trip with its link on the page's origin; Ana and
   Bo join from the link; Cy opens it on the phone. Sam and Ana lend; the
   three computers share the folder. Setup ends when Bo's computer opens
   both lends (200), Cy's phone lists both and opens Sam's, and a file Sam
   puts in the folder (`before.bin`) is on Ana's and Bo's disks.
2. **Ana asks.** Ana runs `svrngs remove Bo --group Trip`; then Bo's
   computer opens both lends again, for 10 s.
3. **Sam removes Bo.** Sam runs `svrngs remove Bo --group Trip`, the
   clock's zero. From then, every 200 ms, Bo's computer asks each lend at
   its origin, and Bo's `/v1/groups`, for 90 s, with nothing put in the
   folder — so the folder's own write cannot carry the cut to Ana sooner
   than the removal itself does.
4. **The folder.** Sam puts `after.bin` in the folder; the time to Ana's
   disk, then 45 s more watching that it never reaches Bo's, nor anything
   else new; `before.bin` still on Bo's disk.
5. **Bo's own removal.** Bo runs `svrngs remove Sam --group Trip` and
   `svrngs remove Ana --group Trip`; then who stands on Sam's and Ana's
   computers (`/v1/rail/people`), and whether Bo's computer's journal holds
   any act of Bo's after the cut.
6. **What each sees.** Bo's home page in Chromium; Cy's phone opens the
   group's page again and Sam's lend.

## Predictions, before the run

Written and committed before `run.py` ran once.

1. **Ana's ask cuts nobody.** Her command says "You asked Sam to remove Bo
   from Trip. Bo stays in it until Sam does." (`Said::Removal`,
   `crates/svrngs/src/cli.rs`), and Bo's computer opens both lends (200)
   throughout the 10 s after it.
2. **Sam's command is the cut**, saying "Bo is no longer in Trip. What Bo
   already has stays with them; nothing new reaches them.", answered
   inside 1 s.
3. **Sam's lend refuses Bo** with svrngs's refusal, 403 `removed`, "You
   are no longer in this group, so nothing of it is open to you.", inside
   2 s of the command: Sam's own computer holds the cut as it writes it.
4. **Ana's lend refuses Bo the same, but only when Ana's computer holds the
   cut, and nothing hastens that.** The removal route writes the cut and
   tells Bo's computer, and wakes no round (`crates/svrngs-node/src/removal.rs`,
   `remove`: no `nudge`, where a folder's write wakes one,
   `folder.rs:434`, and an app's append, `ring_sync.rs:799-805`); rounds
   run each 60 s (`ring_sync.rs:105`). So Ana's door refuses Bo between 0
   and 60 s after the command, wherever Sam's next round falls — past the
   30 s bar about half the time. Until then Ana's lend answers Bo 200. A
   miss of the bar here is a hole of (e), fixed in this row with its test
   red first, and the run taken again.
5. **Bo's computer is told** within 10 s (the removal test's bound,
   `crates/svrngs-node/tests/removal.rs`): its `/v1/groups` answers
   `removed: true` for Trip.
6. **The folder's next file** reaches Ana's disk inside 10 s of Sam's
   write, and never Bo's in the 45 s after; `before.bin` stays on Bo's
   disk. Bo's own folder tries nothing new: its node writes no act.
7. **Bo's own removal counts nowhere.** Both of Bo's commands say "You are
   no longer in Trip.", exit 1, and write nothing (Bo's journal gains no
   act of Bo's); Sam's and Ana's computers list Sam, Ana and Cy standing,
   Bo not.
8. **Bo's home page** says "You are no longer in Trip. What you already
   have stays on this computer; nothing new reaches it." — or "in this
   group", if Bo's computer cannot read the group's name once cut, as the
   browser test's group, named by no seed, read
   (`crates/svrngs-node/tests/browser/main.rs`,
   `the_home_page_tells_a_removed_person`).
9. **Cy's phone is unmoved**: the group's page lists Sam's films and Ana's
   music, and Sam's lend opens, after the removal as before.

## Take 1 — 18:31-18:35 UTC, the repository at `7412588`

`run.py` as committed with this section, the repository otherwise as
u7-removal (e) left it (`cf6ea9c`); its data in `take-1/`: every event in
`events.jsonl` (each `since_cut_s` from the clock's zero, Sam's command),
the measures in `summary.json`, each computer's log kept to the lines
naming a removal, a refusal, a cut, the folder or a round (`<who>.log`),
and what each node printed (`<who>.out`). Through iroh's public relays;
load averages near 5 (`events.jsonl`, `load`). Setup: Bo's computer opened
both lends 52.7 s after the lends were made (the computers learn each other
at their rounds); `before.bin` reached Ana's and Bo's disks 0.65 s after
Sam put it; Cy's phone was in 5.6 s after the link was opened, and opened
Sam's films.

| Prediction | The data | Verdict |
|---|---|---|
| 1, Ana's ask cuts nobody | "You asked Sam to remove Bo from Trip. Bo stays in it until Sam does.", exit 0, 18 ms; Bo's computer then read `films 200`, `music 200` for 10 s (`after-ask`) | **held** |
| 2, Sam's command is the cut, inside 1 s | "Bo is no longer in Trip. What Bo already has stays with them; nothing new reaches them.", exit 0, 37 ms | **held** |
| 3, Sam's lend refuses Bo inside 2 s | 403 `removed`, "You are no longer in this group, so nothing of it is open to you.", first at 0.052 s (`events.jsonl`, `lend`, films) | **held** |
| 4, Ana's lend refuses Bo only once Ana holds the cut, 0-60 s | `music` answered Bo 200 from 0.053 s until 43.32 s, then 403 `removed` with the same words (`events.jsonl`, `lend`, music); Ana's computer's round at 18:33:29.46 is when it took the cut from Sam (`ana.log`, `round membership`), 43.0 s after the command | **held**, and with it the 30 s bar **failed**: one lender of two refused Bo at 43.3 s |
| 5, Bo's computer told within 10 s | `/v1/groups` answered `removed: true` for Trip at 0.341 s (`summary.json`, `cut.bo_told`); Sam's log: "the removed computer told its cut … told=true" | **held** |
| 6, the folder's next file to Ana inside 10 s, never to Bo | `after.bin`, put by Sam 90.2 s after the command, on Ana's disk 0.35 s later; Bo's disk held `before.bin` alone 45 s after, as before the cut (`folder-measured`) | **held** |
| 7, Bo's own removal counts nowhere | `svrngs remove Sam --group Trip` and `… Ana …` from Bo's computer: "svrngs: You are no longer in Trip.", exit 1, each; Bo's log "removal: refused … NoStanding" twice; no act of Bo's key in Bo's journal before or after (`bo_acts_before`, `bo_acts_after`, both empty: Bo never wrote one); standing on Sam's and on Ana's computers: Ana, Cy, Sam | **held** |
| 8, Bo's home page | Chromium: heading "Trip", "You are no longer in Trip. What you already have stays on this computer; nothing new reaches it." — the group's name read, not "this group" | **held** |
| 9, Cy's phone unmoved | 141 s after the cut, the group's page listed "Ana’s music", "Sam’s films" and opened Sam's films ("Sam's films", 1,652 ms) and Ana's music ("Ana's music", 2,383 ms) | **held** |

**Removed, by demo/c.md's check: failed.** Every lender's refusal inside
30 s did not hold: Ana's door served Bo for 43.3 s after Sam's command,
because a removal waits for the keeper's next round to reach any computer
but the removed one's. The rest of the check held: the refusal is
svrngs's in a person's words, Bo's home page says so, the folder's next
file reached everyone but Bo, Bo's own removal counted nowhere, and
`before.bin` stayed on Bo's disk.

**What no prediction named.**

1. *Bo's computer goes on logging rounds that "exchanged" with Ana and
   Sam* after the cut (`bo.log`, 18:33:29, 18:34:29, 18:35:29), while
   Sam's computer refused every one of Bo's dials, 268 in all, "the
   dialer stands in no group held here" (`sam.log`, after 18:32:46). The
   round's trace counts a peer it dialled, not one that took anything;
   Bo's disk and register gained nothing. A trace that reads as an
   exchange where every dial was refused is a follow-up, not this row's.

Fix, then take 2 (below): the removal route wakes the round as the folder's
write and an app's append do.
