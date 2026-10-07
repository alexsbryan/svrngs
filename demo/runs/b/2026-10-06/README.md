# Demo B's dry run on this host — 2026-10-06

u6-9 (`ralph/next/u6/STATE.md`, its row; `demo/b.md`). Demo B's steps,
from the link on, taken on one computer by a script, before five people
take them at Dee's: Sam's library lent from one data dir, Dee's computer
running the room door from another, the TV a browser at the room
address, and five phones, one at a time, each opening the link, Sam's
library, and putting a film on the TV. The last section is the package
for the operator. This section is committed before any run; the data and
what it found come after it, in later commits.

## The setting

**Sam's library**: a scratch Jellyfin on this host as u5-4's check makes
it (`apps/jellyfin/check/check.sh up`: the macOS app's 12.0.0 server
with its own data dirs on 127.0.0.1:8096), three films of 600 s
(`check.sh films`), lent with `svrngs share 8096` as it comes; in
Jellyfin's own pages, by the line `share` prints, one account for the
room, `room`, every library ticked (`apps/jellyfin/ROOM.md`, "one account
for the room"); Jellyfin's list of users exported after that. Sam's group
names the page, served on localhost as U5's runs serve it.

**Dee's computer**: a data dir of its own, joined to Sam's group by the
link, running `svrngs room jellyfin`, the room door (u6-1), which prints
the address the room reaches it at.

**The TV** is stood in for by jellyfin-web in Playwright's Chromium at
that address, at 1920×1080, its profile kept on disk as a TV's app keeps
what it stored — the client "Jellyfin for WebOS" wraps (u6-2). Any
request it makes to an address other than the room's is kept and
refused, as Sam's Jellyfin across town would not answer a TV at Dee's.
Before the evening, and not timed, it signs in as u6-2 found and
`demo/b.md` has it: the address typed once, "Use Quick Connect", and the
code typed on Dee's phone, which opened the group's link, Sam's jellyfin
from the list, and signed in as `room` (`night.mjs tv`).

**The five** are five browser profiles as phones, each new to all of it,
on the page by the group's link (`apps/jellyfin/check/person.mjs`, way
`phone`): Ana, Cy and Ed in WebKit as an iPhone 15, Bo and Di in
Chromium as a Pixel 7 (Android). One at a time, each: opens the link,
gives a name and is in; opens Sam's jellyfin from the list; signs in to
Jellyfin's own page with the room's name and password; taps the film
they pick on Jellyfin's home (One, Two, Three, One, Two in turn); picks
the TV in Jellyfin's own "Play on" (the button "Cast to Device"); taps
Play. Then, from their phone, pauses the TV, seeks it to half way and
plays it on (Jellyfin's "Remote Control"). Each phone stays open after
its turn, as phones in a room do; the TV keeps playing the last film
until the next person puts theirs on.

## What is measured, and how each is read

- **The clock**, per person: from the moment their browser is told to
  open the link to the moment the TV's `<video>` plays the film they
  picked — a video started after their tap on Play (a new element, or a
  `loadstart` on it), moving and not paused — and Jellyfin reporting
  the TV's session playing that film. Read against the gate's three
  minutes (180 s). The script's waits are in the clock as a person's
  would be: 1.5 s on Jellyfin's first page, 2 s for the "Play on" list,
  3 s after picking the TV. A script is not a person: this is how long
  the steps take on this host, not the gate's rate (§2, a people bar is
  a recorded run).
- **Every step's time** inside it: the link to the group's list, the
  list to Jellyfin's page, its sign-in, the film's page, "Play on", Play
  to the TV playing.
- **What "Play on" lists** each time, and whether the TV is in it, by
  the name and account the lender's Jellyfin gives the TV's session.
- **Pause and seek from the phone**: each within 10 s of the tap, on the
  TV's `<video>` (u6-2's bar for part c).
- **Each request's path**: every request the TV made, by time, method,
  host and path, and every response each phone received, by time,
  status, host and path; the room door's log (Dee's, every request it
  took), Sam's door's log (the person each request came as, and its
  path guessed home or away), and Jellyfin's log and sessions. Passed
  if the TV asked nothing of Sam's Jellyfin but through the room door.
- **What each phone was asked**: a name on the page, and Jellyfin's
  sign-in; any prompt for anything DESIGN §1 keeps from people is a
  `key_prompts`.
- **Jellyfin's users**: the export after Sam made the room's account,
  and the list after the run; `accounts_created` is the difference.
- **No secret of Sam's** in any byte a phone received (`scan.py`, u5-4's).

One host, one run: not a rate. The TV is a stand-in; the real TV, Dee's
computer, Sam's library on another network and five people are the
operator's, in the package below.

## How it ran

Everything above this heading was committed before any run (`eadf35f`).
The instrument is `apps/jellyfin/check/screen.sh` `night` and what it
calls (`night.mjs`, `night.py`, `tv.mjs` moved from u6-2's `screen.mjs`
with its behaviour kept, `52d8168`), as committed at `2f3f2c9`; the data
is `data/`, committed at `2e9116d`; product code (`crates/`, `web/` but its tests) unchanged since
`1ad52a8`. Jellyfin 12.0.0 (the macOS app's server) and its jellyfin-web
12.0.0; Playwright 1.59.1; the TV HeadlessChrome 147 at 1920×1080; Ana,
Cy, Ed WebKit 26.4 as an iPhone 15, Bo and Di Chromium 147 as a Pixel 7.
Sam's, Alex's and Dee's computers are three data dirs on this host;
`data/steps.log` is the order, 05:22–05:28Z on 2026-10-07 (the evening
of 2026-10-06 here).

```
cargo build -p svrngs
S=apps/jellyfin/check/screen.sh D=demo/runs/b/2026-10-06/data
PEOPLE=room MEDIA=/tmp/u5-9-films $S up /tmp/u6-9r $D
$S night /tmp/u6-9r $D tv;  $S night /tmp/u6-9r $D five
$S logs /tmp/u6-9r $D;  python3 apps/jellyfin/check/scan.py /tmp/u6-9r $D
$S down /tmp/u6-9r $D;  python3 apps/jellyfin/check/night.py $D > $D/judged.json
```

**Not as the setting said.** The films are u5-9's three of 1,200 s
(`/tmp/u5-9-films`), not 600 s ones: this host's disk was full while
they were being made (544 MiB free), and those already there were used.
`2e9116d`'s body counts 92,752,083 bytes as what "the five phones and
Dee's" received; that is the five's alone, and with Dee's phone it is
103,772,888 (`data/scan.json`).

**The instrument, failing first** (`2f3f2c9`'s body, trials on a scratch
copy, not data): the clock given a person whose film the TV never plays
did not stop; the judge given a 190 s clock and a TV request to
`127.0.0.1:8096` failed both. Trials found four things the driver did
that a person does not, each fixed before the run: the TV in "Play on"
matched by the item's whole words; the TV's session looked up by its
device; the clock stopping on the TV's `<video>` alone, before Jellyfin
named the film (trial t3: 622 ms after Play); and the seek bar dragged
before the phone's Remote Control showed the film's length (finding 4).

## What it found

1. **Five of five got from the link to their film on the TV well inside
   three minutes.** Each clock, from the link opened to the TV playing
   the film they picked and Jellyfin naming it as the TV's
   (`data/night-five.json`, `judged.json`):

   | who | phone | film | link → list | list → Jellyfin | sign-in | film's page | Play → TV | **clock** |
   |---|---|---|---|---|---|---|---|---|
   | Ana | WebKit, iPhone | One, played as it is | 5,534 | 4,437 | 866 | 1,797 | 3,875 | **25,063** |
   | Bo | Chromium, Pixel 7 | Two, converted | 7,474 | 5,786 | 1,798 | 1,470 | 11,127 | **39,240** |
   | Cy | WebKit, iPhone | Three, as it is | 7,079 | 6,838 | 1,588 | 797 | 3,115 | **29,928** |
   | Di | Chromium, Pixel 7 | One, as it is | 6,603 | 4,748 | 4,260 | 786 | 1,365 | **28,203** |
   | Ed | WebKit, iPhone | Two, converted | 5,583 | 4,660 | 876 | 672 | 4,960 | **25,578** |

   In ms. The rest of each clock is the script's own pauses, as a
   person's: 1.5 s on Jellyfin's first page, 2 s for the "Play on" list,
   3 s after picking the TV. A script is not a person, and one run is not
   a rate: this says the steps fit in the time on one computer, not that
   four of five people will find them.
2. **The TV reached Sam's Jellyfin only through the room door.** Signed
   in by Quick Connect before the evening — the code typed on Dee's
   phone, by the link and signed in as `room`, in 392 ms, the TV signed
   in 4,734 ms after with nothing typed on it (`night-tv.json`) — it
   kept the room address (`ManualAddress` `http://192.168.1.4:48220`)
   though Jellyfin reports itself at `http://127.0.0.1:8096`. Of its
   406 requests (159 signing in, 247 through the five), none went to
   Sam's Jellyfin another way; two went elsewhere, Google's
   `cast_sender.js` once each time it opened, refused, as in u6-2. The room
   door took 413 requests, every one from 192.168.1.4 (`dee-room.log`).
   Sam's door let in 125 of them as Dee before a path was known and
   guessed 288 home, not counted, the path ending at a private address
   (`sam-run.log`): on this host Dee's computer is in Sam's home; at
   Dee's it is across town, and counted as one person. Each phone went
   through the page and n0's relay to Sam's door as itself (1,163 lines
   `relayed=Some(true)`: Dee's phone 151, Ana 216, Bo 220, Cy 192, Di
   198, Ed 186), so Sam's door counted up to five at once (`people=5`,
   203 lines) — the phones, not the TV.
3. **With one account for the room, "Play on" lists every phone signed
   in as it, and a film starts where the last person left it.** Ana's
   list was the TV ("Chrome - Jellyfin Web room"), Dee's phone ("Safari
   iPhone - Jellyfin Web room") and Google Cast; Ed's, the TV and five
   phones. The TV is told apart by its name only: here "Chrome", the
   stand-in's; on a TV, the name its app gives. Di's One started at
   631.6 s and Ed's Two at 633.7 s, where Ana and Bo had left them: the
   room shares where each film stands, as `apps/jellyfin/ROOM.md` says it
   shares what is played. Nothing here is svrngs's: it is Jellyfin's, for
   one account.
4. **Pause and seek from each phone, each within 10 s** (u6-2's bar):
   paused 109–505 ms, sought half way 133–900 ms, played on 436–2,935 ms.
   Cy's phone (WebKit, Three) never showed the film's length on its
   Remote Control within 15 s (`length_shown_ms` null); dragged after
   that wait it sent `SeekPositionTicks=6000105000` and the TV went half
   way. In trials, dragged at once, the same phone sent
   `SeekPositionTicks=0` and the TV went to the start (t5, t9; Sam's door
   carried what it was sent), where with no svrngs anywhere the same
   drag at once sent half way (t9-direct, one run). So a phone through
   the page holds the length of a film played as it is later than one
   straight at Jellyfin, for WebKit and Matroska here; the instrument
   waits as a person does, and the lead is not pursued here: it moves
   no demo until a person drags before the length shows (the operator's
   run reads it).
5. **Nothing a person must not hold reached them.** None of Sam's
   password, Sam's token or the room's password was in any of the
   103,772,888 bytes the five phones and Dee's received, headers, bodies
   and socket frames; each held only the token Jellyfin gave it at its
   own sign-in; the lend keeps nothing (`scan.json`). Jellyfin's users
   were `["room", "sam"]` after Sam made the room's account and after the
   run (`users-made.json`, `users-after.json`): `accounts_created` 0.

The record, for a dry run (`judged.json` `record`): `participants` 5
browser profiles, `succeeded_in_time` 5, `accounts_created` 0;
`key_prompts` and `assisted` are null, since a script answers only what it
is asked and there was nobody to help: the recordings of the operator's
run say those.

## The package: the operator's run

`operator-only: anything that needs a service this host cannot run` —
Dee's computer and Jellyfin's app on a real TV, Sam's library on another
network, five people. The package is `NEEDS_HUMAN.md` beside this file.
