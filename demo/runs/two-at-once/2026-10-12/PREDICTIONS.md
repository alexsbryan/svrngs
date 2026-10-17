# Two of one person's devices at once — predictions, before any data

Row bash-two-at-once (a) (`ralph/next/bash/STATE.md`; REVIEW_FINDINGS
#125): since bash-devices, `x-svrngs-key` is the person's key, so
Jellyfin's middleware names every device of a person one Jellyfin device,
`svrngs-<16 hex of x-svrngs-key>` (`apps/jellyfin/mw/src/person.rs:113`),
and the door keeps one sign-in answer, one token, a person
(`crates/svrngs-node/src/door/sign_in.rs:262`, `Whose`). Whether two of
Ana's devices can play at once was never run. Every person is a program;
nobody is a person, so no people bar is judged (AGENTS.md 7). One host,
one run: not a rate. Written and committed before the driver ran.

## The setup

- **The build** is this commit's, `cargo build -p svrngs`; every computer
  a data dir and `HOME` of its own under one scratch directory.
- **The page** is `scripts/build-web.sh --out <scratch>/site`, served on
  this host by `web/tests/serve.mjs`; nothing is published (the queue's
  fence 7). n0's public relays, the default.
- **Sam** makes the group, `svrngs new two --as Sam --origin <the page>`,
  and lends a scratch **Jellyfin** (the macOS app's server, its own data
  dirs) through its middleware on 8097: `svrngs share 8096 --sign-in
  8097`. Its library is Demo A's three test films (`/tmp/onb-e-films`,
  1,200 s each): *Test Pattern One*, an mp4 a browser plays as it is, and
  *Test Pattern Two*, an mpeg-2 mkv Jellyfin converts.
- **Ana** joins by Sam's link, `svrngs join <link> --as Ana --no-run`, then
  makes a link for her devices, `svrngs link two --device`, which runs on
  as her computer.
- **Ana's laptop** is a second computer, `USER=analaptop`: `svrngs join
  <Ana's device link>`, no `--as`. Its browser is Playwright's headless
  Chromium opening the laptop's own address for Sam's jellyfin, as the
  laptop's home page would.
- **Ana's phone** is Playwright's WebKit as an iPhone (`web/tests/page.mjs`)
  opening Ana's device link on the page, then Sam's jellyfin from the
  page's list.
- **Each plays a film of its own**: the laptop *Test Pattern Two*, the
  phone *Test Pattern One*, each by its page's play button, as
  `apps/jellyfin/check/person.mjs` plays one. Once both play, a **180 s
  window** at once: each seeks four times through jellyfin-web's own seek
  bar (`person.mjs`'s `seek`), on schedules apart, while each page's
  `<video>` is watched (`person.mjs`'s `watch`, `trio.mjs`'s `over`) and
  Sam's Jellyfin is asked for its sessions every 2 s (`GET /Sessions`, as
  Sam). Then the laptop's film is stopped by its player's back button; the
  phone plays on 20 s alone; then the phone's is stopped the same way, and
  Jellyfin is asked where each film was left (Ana's account's `UserData`
  for each).

## Predictions

1. **Both in by Ana's device link, no name typed.** The laptop's `join`
   says "You're in two, as Ana." within 60 s; the phone reads "You’re in
   two, as Ana" within 60 s of opening the link, never asked a name. Each
   opens Sam's jellyfin without a sign-in asked, and each page's stored
   device (`_deviceId2`) is the same: `svrngs-<first 16 hex of Ana's person
   key>`.
2. **Each film plays.** Within 30 s of its play button, each page's video
   plays (`currentTime` past 0.5 s, not paused).
3. **Neither takes the other's film.** Over the window, outside each
   page's own seeks: no `emptied`, no `loadstart`, no new `<video>`
   element, no still of 5 s or more; each of the eight seeks plays again
   within 30 s; each page's video still plays its own film at the end (its
   `currentSrc` names its own film's id).
4. **Jellyfin sees one device, its film overwritten.** Every `/Sessions`
   read lists **one** session of Ana's account for the two devices — one
   `DeviceId`, the one `_deviceId2` of prediction 1 — and its
   `NowPlayingItem` names each film in turn over the window, never both at
   once: whichever device reported last.
5. **Stopping one leaves the other.** After the laptop's film stops, the
   phone's plays on: over the next 20 s, no `emptied` or `loadstart`, and
   it plays at least 15 s of them. Jellyfin's session names the phone's
   film within 15 s of the laptop's stop.
6. **Each film's place kept as its own.** After both stop, Ana's account's
   resume position for each film is within 15 s of where its own player
   stood when stopped, and neither is the other's.

## What sends the row to (b)

(b) — each device its own Jellyfin device under the one account — runs if
the run shows one device stopping, taking or overwriting the other: a
film stopped, started over or replaced outside its own seeks (3, 5), a
place kept for one film taken by the other's (6), or Jellyfin's sessions
naming one device's film in the place of the other's while both play (4).
Prediction 4 says the last will hold; if it does, (b) runs, red first on
these steps. If 3, 5 and 6 hold and 4 does not — Jellyfin lists two
sessions, each naming its own film — #125 closes with this run as its
evidence.

A step the run cannot finish is could-not-judge for it, never a pass
(AGENTS.md 6).
