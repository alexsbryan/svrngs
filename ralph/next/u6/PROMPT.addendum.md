<!-- section: u6 after=intro -->
This queue is U6 (`docs/DESIGN.md` §8): a film from a library across town
plays on the TV in the room, with svrngs giving the room reach and nothing
more. The operator, 2026-10-05, after five reads on a real TV
(`demo/runs/u6/2026-10-05-screen/reads.json`): the screen is the app's own
client — Jellyfin's app on the TV — and svrngs gives devices in the room
reach to a lend through a member's computer on that network; "we're adding
value, but we're not taking media on tvs and end user devices into our
scope." The infra row adds the reach; the Jellyfin row measures the app's
own client through it and tells the lender what to set; Demo B's dry run
ends in the operator's package.

The plan for every row is DESIGN §1, §3, §8 "U6's inventory" (written by u6-0
from `ralph/next/u6/UPSTREAM.md`, the seat's survey, which was written for a
handoff the operator has since taken out of scope) and §10. Upstream is
commonwealth-ai at `e1a43bcd2`; read a file there with
`git -C ~/dev/commonwealth-ai show e1a43bcd2:<path>`. Jellyfin is
GPL-2.0-only: svrngs talks to it over HTTP and its API, and copies none of
its code.

**The calls** (2026-10-05; u6-0 writes them into DESIGN):
1. *The screen is the app's own client; svrngs gives it reach.* A TV plays a
   lend through the app's own client for it — "Jellyfin for WebOS" 1.2.2
   signed in and played by DirectPlay from a LAN address (the reads) —
   pointed at an address on the TV's network; a phone as the remote is the
   app's own (Jellyfin's "Play on"). Nothing in svrngs knows AirPlay, Cast,
   DLNA, a TV, a player or what a lend serves. AirPlay from a phone's
   browser showed no video three ways (the page's worker-served address;
   jellyfin-web, whose hls.js sets `disableRemotePlayback`; a plain LAN
   file the TV never fetched), and old TV browsers are not supported (the
   LG's Chrome 87 could not open the page; operator: "a losing cause").
   AirPlay and Cast from a page wait in §9: an app with no client of its own
   for a screen is asked for on one, counted.
2. *A computer in the room.* The address comes from a member's computer on
   the screen's network (operator, 2026-10-05), not from a phone (a browser
   cannot listen) and not from a public address per lend (§9: a relay that
   forwards bytes it cannot read to the lender, who holds a certificate per
   lend — a new service and Let's Encrypt's per-domain limit; trigger: a
   group with no computer in the room asks, counted). Its hop to the
   library is computer to computer over iroh, which can go direct, where a
   browser's is relay-only (8.1 Mbit/s through n0's relay in the reads).
3. *The room door.* A member's computer serves the lends its person names
   to its own network while the command runs: each at a port of its own on
   this computer's private address, the lend's root its own as u5-2's
   origins on loopback are (one body with them, not a second), reached as
   this computer's person — dialled under this computer's key, the way this
   computer's own browser reaches the lend. Call 4 of the browser holds: the
   screen is this person's own device, not another person's dial, and no
   node proxies a lend for anyone else. It carries what u5-2's origins carry
   (Range, both bodies streaming, a WebSocket — the app's remote rides its
   socket). The address is picked as upstream's `advertised_base` picks one:
   the private LAN before tailscale, never loopback, a wildcard or a public
   address, and absent, said so, when there is none. Anyone on that network
   reaches the named lends as this person while it runs, which the command
   says when it starts, in §1's words; a request from off this network is
   refused. The lender's door counts the room as one person and guesses its
   path as it does now.
4. *Measured, then told.* What Jellyfin's own client for a screen needs
   through the room door is measured (u6-2) before any of it is written:
   the server address typed once; sign-in without typing on a remote
   (Jellyfin's Quick Connect: the TV shows a code a signed-in phone
   approves); a newcomer's phone, signed in as another Jellyfin user,
   playing on the TV's session (a permission the lender sets); and whether
   the client keeps to the room address or moves to the address the
   lender's Jellyfin reports for itself (`/System/Info/Public`
   `LocalAddress`, which is across town). What the lender sets is written in
   `apps/jellyfin/`, never in the infra; anything the door would need is
   general (any app), or it is a finding for the operator.
5. *Demo B as it now runs* (operator, 2026-10-05: the library is across
   town; the people in the room are on the TV's WiFi). Five friends at
   Dee's; Sam's library is across town; Dee's computer runs the room door
   and Dee's TV has Jellyfin's own app, each set up once by Dee before the
   evening; the five open Sam's link on their phones and put a film on the
   TV; nobody installs or signs up. The gate stands byte-identical: four of
   five newcomers get from the link to the film on the TV within three
   minutes, unassisted.

The size ceiling set aside about 2,100 lines for U4's page and U6's handoff
(DESIGN §5); U6 is now a door and a measurement, and should land well under
its share. A raise is the operator's.

Tests that need Jellyfin run a scratch Jellyfin on this host as u5-3's
script does (the macOS app's server with its own data dirs, on a port of its
own); scripts/check.sh stays free of Jellyfin, so what runs there uses the
conformance canary (u5-2), and the Jellyfin runs are their own script beside
it. The screen's client is stood in for by jellyfin-web in a browser, the
client Jellyfin for WebOS wraps; the real TV is the operator's, in u6-9's
package. Anything a person reads uses only DESIGN §1's words.
