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
