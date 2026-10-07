# Jellyfin's own client for a screen, through the room door — 2026-10-06

u6-2 (DESIGN §3, "The room", call 4: measured, then told): what
Jellyfin's own client for a screen needs through the room door, measured
before anything about it is written. This section is committed before any
run; the data, and what the lender sets, come after it in later commits.

## The setting

A scratch Jellyfin on this host as u5-4's check makes it
(`apps/jellyfin/check/check.sh up`: the macOS app's server with its own
data dirs on 127.0.0.1:8096, three films, the lender's own account),
lent by Sam's computer with `svrngs share 8096` as it comes, and the
accounts made in Jellyfin's own pages by the line `share` prints: `dee`,
for Dee and Dee's TV, and `ana`, a newcomer. Dee's computer, a data dir of
its own, joins Sam's group by the link and runs `svrngs room jellyfin`,
the room door (u6-1), which prints the address the room reaches it at.

**The screen** is stood in for by jellyfin-web in Playwright's Chromium
at the room address — the web client "Jellyfin for WebOS" wraps (it loads
the server's own jellyfin-web once its address is typed); its version is
recorded beside the reads' 1.2.2. **Across town** is made by the screen's
browser: any request it makes to an address other than the room's is
recorded and refused, as Sam's Jellyfin across town would not answer a TV
at Dee's. **The phones** come by the group's link through the page, as
U5's phones do (`apps/jellyfin/check/person.mjs`, way `phone`): Dee's in
WebKit as an iPhone, Ana's likewise.

## What is measured, and how each is read

- **(a) The address typed once.** The screen opens the room address,
  signs in typing `dee`'s account, plays the film Jellyfin plays as it is,
  and seeks three times through its own seek bar. Passed if it plays and
  each seek plays again within 5 s (Demo A's bar), and every request the
  screen made, page, API, film and socket, was at the room address.
- **(b) Sign-in without typing on a remote.** A second screen, new to it,
  asks for Quick Connect; Dee's phone, signed in as `dee` through the lend,
  types the code the screen shows into jellyfin-web's own Quick Connect
  page. Passed if the screen is then signed in, with nothing typed on it.
  First as the lender's Jellyfin comes; if it is not offered, what the
  lender turns on, and then again.
- **(c) A newcomer's phone plays on the screen.** Ana's phone, signed in
  as `ana`, picks the screen in jellyfin-web's own "Play on", plays a film
  there, pauses it and seeks it. Passed if the screen's `<video>` starts,
  pauses and moves to the seek, each within 10 s of Ana's tap. First as
  the lender's accounts come; if Ana cannot, the permission it takes, set
  as the lender sets it, and then again.
- **(d) Where the screen goes.** What `/System/Info/Public` answers through
  the room door (`LocalAddress`), what the screen keeps of it, and whether
  it asks any address but the room's — through (a)-(c), and when it is
  opened again from what it kept, as a TV's app is the next evening. Kept
  to the room address if it never asks another; else what keeps it — a
  lender's setting first; a door behaviour only if no setting does, sized,
  and then the question goes to the operator.

Beside each: the room door's log (Dee's `svrngs`, its debug trace of
every request it took), Sam's door's log (the person each request came
as), and Jellyfin's log and sessions — so a request of the screen's that
reached Sam's Jellyfin by another way would show.

One host, one run of each: not a rate. The screen is a stand-in; the
real TV is the operator's, in u6-9's package.

## How it ran

Everything above this heading was committed before any run (`5006a5a`).
The instrument is `apps/jellyfin/check/screen.sh` and what it calls, as
committed at `e019551`; the data is `data/`, committed at `7fc471a`;
product code as at `1976fa6`. Jellyfin 12.0.0 (the macOS app's server)
and the jellyfin-web 12.0.0 it serves (`appVersion` in its
`main.jellyfin.bundle.js`; the session's `ApplicationVersion`, `a.json`),
where the reads' TV ran "Jellyfin for WebOS" 1.2.2 around the same
server's web client; Playwright 1.59.1, the screen HeadlessChrome
147.0.7727.15 at 1920×1080, the phones WebKit as an iPhone 15. Sam's,
Alex's and Dee's computers are three data dirs on this host;
`data/steps.log` is the order, 02:45–02:50Z on 2026-10-07 (the evening
of 2026-10-06 here).

```
cargo build -p svrngs
apps/jellyfin/check/check.sh films /tmp/u6-2-films 120
S=apps/jellyfin/check/screen.sh D=demo/runs/u6/2026-10-06-jellyfin-screen/data
MEDIA=/tmp/u6-2-films $S up /tmp/u6-2r $D
$S run /tmp/u6-2r $D a;  $S run /tmp/u6-2r $D b
RUN=as-it-comes $S run /tmp/u6-2r $D c;  $S run /tmp/u6-2r $D control;  $S run /tmp/u6-2r $D c
RUN=same PHONE_AS=dee $S run /tmp/u6-2r $D c
$S run /tmp/u6-2r $D d;  RUN=bounce BOUNCE_DOOR=1 $S run /tmp/u6-2r $D d
RUN=direct DIRECT=1 $S run /tmp/u6-2r $D c
$S logs /tmp/u6-2r $D;  python3 apps/jellyfin/check/screen.py $D > $D/judged.json
$S down /tmp/u6-2r $D
```

`judged.json` is each part read by its bar above; each part's own file
holds the steps behind it, and `<part>-screen-requests.jsonl` every
request the screen made, by time, method, host and path.

**The instrument, failing first.** Trial runs before these (not data)
found three things the driver did that a person does not, each fixed
in `e019551` before the run: jellyfin-web 12's "Play on" is a menu of its
own (`[role=menuitem]` under the button "Cast to Device"), not the older
action sheet the first driver waited for; a play tapped 1.5 s after the
list's first item showed went to the phone itself, three times
(`Active player: Html Video Player` 1.8 s after the tap), where with the
list given 2 s and the pick 3 s it went to the screen every time; and
the lender's page for a person opens from their picture, not their name.
The film maker `check.sh films` wrote its three films into one part file
(`1976fa6`).

**The judge, failing first.** The screen's guard keeps what it refuses:
it kept Google's `cast_sender.js` in every part, so a request off the
room address is seen. That `screen.py` fails one to Sam's Jellyfin was
watched on a copy of `data/` with one planted in each of `a.json`
(`http://127.0.0.1:8096/System/Info/Public`) and `d.json`
(`ws://127.0.0.1:8096/socket`): `every_request_at_room` failed with "1 to
Sam's Jellyfin another way, 1 elsewhere", and `kept_to_room` failed; on
`data/` itself, "0 to Sam's Jellyfin another way" and passed. The parts
fail on their own too: (c) as Jellyfin comes, and its pause and seek.

## What it found

1. **(a) The address typed once: it signs in, plays and seeks, every
   request to Sam's Jellyfin through the room door.** The screen opened
   `http://192.168.1.4:48719/`, the address `svrngs room jellyfin`
   printed (`dee-room.out`), was asked to sign in, signed in as `dee` in
   881 ms, played by DirectPlay in 3,398 ms, and played again 392, 392
   and 482 ms after each of three seeks (`a.json`). Of its 201 requests,
   page, API, film and socket, none went to Sam's Jellyfin by another
   address. One went elsewhere: Google's Chromecast sender,
   `https://www.gstatic.com/cv/js/sender/v1/cast_sender.js`, which
   jellyfin-web asks in Chrome and which the stand-in refused, as in
   every part; the bar as committed counted it, so "every request at the
   room address" is **failed** on it, though it is not Sam's. Whether
   Jellyfin's app on the TV asks it is the TV's read.
   *The doors' logs agree.* The room door took 1,121 requests over the
   run, each from 192.168.1.4, none refused (`dee-room.log`); Sam's
   door let in 1,121 requests as Dee not relayed — 921 guessed home by
   their direct path to 192.168.1.4, 3 direct and let in, 197 before any
   path was observed, so taken as away — and 151 more as Dee relayed:
   Dee's phone in (b), joined by the link, not the room (`sam-run.log`,
   its `relayed` and `addr` fields); Jellyfin saw every
   session from 127.0.0.1, Sam's door (`a.json` `session.remote_end`,
   `jellyfin-sessions.json`).
2. **(b) Sign-in with nothing typed on the screen: Quick Connect, on as
   Jellyfin comes.** Jellyfin 12 offers Quick Connect on its sign-in
   (`/QuickConnect/Enabled` true, the lender having set nothing); the
   screen showed "Enter code … to login", Dee's phone, signed in as
   `dee` through the lend, entered it in jellyfin-web's own Quick Connect
   page in 348 ms, and the screen was signed in as `dee` 3,925 ms later
   (`b.json`). The screen is then whoever approved it.
3. **(c) A newcomer's phone starts the screen's play once the lender
   allows it; it then loses the screen, which is Jellyfin's own.**
   - As Jellyfin comes, Ana's "Play on" lists only "Google Cast
     Unsupported": not the screen, which is `dee`'s (`c-as-it-comes.json`).
   - With "Allow remote control of other users" ticked on Ana's account,
     in the lender's own pages (`control.json`), it lists
     "Chrome - Jellyfin Web dee"; picked, Ana's play started the film on
     the screen in 1,253 ms (`c.json`). Then jellyfin-web dropped the
     screen as Ana's player: its button went back to the cast icon, its
     menu to the list again, with no "Remote Control" — so no pause and no
     seek from Ana's phone (**failed**, each). The screen played on.
   - With no svrngs anywhere — the screen and Ana's browser both straight
     at Sam's Jellyfin — the same: listed, started in 1,760 ms, then no
     "Remote Control" (`c-direct.json`). It is Jellyfin's, not the room
     door's: jellyfin-web drops a remote player whose session a
     `Sessions` update of its socket does not hold
     (`sessionPlayer-plugin.*.chunk.js`, `T`, which calls
     `setDefaultPlayerActive`); which message drops it was not read.
   - A phone signed in as the screen's own user, `dee`, keeps it: started
     in 3,380 ms, then "Remote Control" paused it in 161 ms, sought it to
     half way in 119 ms and played on in 104 ms (`c-same.json`).
4. **(d) The screen keeps to the room address.** Through the room door,
   Jellyfin reports itself at `LocalAddress` `http://127.0.0.1:8096` —
   Sam's own computer, across town — and jellyfin-web keeps it beside the
   typed address (`ManualAddress` `http://192.168.1.4:48719`,
   `LastConnectionMode` 2, manual). Opened again, as the next evening, it
   asked only the room address (`d.json`); with the room door stopped
   16 s and started again while it was open, the door came back at the
   same port (it is the lend's, `room::port_of`), the screen opened its
   socket again there and stayed signed in, and asked nothing else
   (`d-bounce.json`). So nothing keeps it but the address typed: no
   lender's setting and no door behaviour is needed. Not measured here:
   what "Jellyfin for WebOS"'s own shell, which holds the typed address
   before the web client loads, does when the room address does not
   answer; that is the TV's read.

One host, one run of each part: not a rate.

## What the lender sets

Written for the lender and the room in `apps/jellyfin/ROOM.md`, in
DESIGN §1's words: nothing for the TV's sign-in (Quick Connect is on as
Jellyfin comes); for each person who puts a film on the TV from their
phone, "Allow remote control of other users" on their account; and
pausing and seeking from the TV's own remote, since a phone signed in as
someone else loses the TV after it starts. Nothing for the room door:
the door needs nothing it does not do (finding 1 and 4).
