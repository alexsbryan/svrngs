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
