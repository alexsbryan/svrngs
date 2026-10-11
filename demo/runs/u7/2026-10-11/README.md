# U7's removal, measured on one host — 2026-10-11

u7-removal (f) (`ralph/next/v1/STATE.md`): Demo C's removal check
(`demo/c.md`, "What is watched", **Removed**) on this host. Three computers
— Sam, the group's maker, so the one who decides who is removed; Ana; Bo —
and Cy's phone (Playwright's WebKit as an iPhone, the group's page served on
`localhost`), in one group, Trip, through iroh's public relays. Sam and Ana
each lend a page (`films`, `music`, each a plain file server); the three
computers share a folder, Trip. Sam removes Bo. `run.py` is the driver;
its data is in `run/`. One host, one run: not a rate (AGENTS.md 7). The
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
