# mw-2: no sign-in, through the middleware — 2026-10-08

mw-2's row (`ralph/next/mw/STATE.md`) under mw-0's rule (`../README.md`,
its bars §3, its predictions §4). Against `check.sh up`'s scratch
Jellyfin 12.0.0 on this host at `/tmp/mw2` (`steps.log`, from 14:15 UTC),
the middleware at `7e0e1c5`, the check at `43c44e2`; Playwright's
Chromium 147.0.7727.15 and WebKit 26.4 (an iPhone on a phone); the
computers and the phones reach Sam's through iroh's public relays, as in
u5-3 and u5-4. `run.sh` reruns it.

    apps/jellyfin/check/check.sh up /tmp/mw2 demo/runs/mw/2026-10-08/mw-2
    apps/jellyfin/check/check.sh mw /tmp/mw2 demo/runs/mw/2026-10-08/mw-2 mw1-again
    JELLYFIN_MW_PLANT=<plant> MW=1 apps/jellyfin/check/check.sh pair /tmp/mw2 <out> computer chromium
    MW=1 apps/jellyfin/check/check.sh pair /tmp/mw2 <out> <computer|phone> <chromium|webkit>
    apps/jellyfin/check/check.sh judge /tmp/mw2 <out> mw-computer-chromium mw-computer-webkit mw-phone-chromium mw-phone-webkit
    apps/jellyfin/check/check.sh down /tmp/mw2 <out>

`mw` (mw-1's check, run again with mw-2's middleware: every bar passed,
`mw-mw1-again.json`) lends the middleware's port as `jellyfin-mw` and
joins Bo as a second computer. Each `MW=1 pair` starts the middleware
afresh and deletes the users it named, so both people are new to it: on
a computer Alex and Bo, each opening their own computer's address of the
lend; on a phone two people who join by the group's link and open
`jellyfin-mw` from the page's list (Ana and Ben in Chromium, Cleo and Dara
in WebKit). Nobody types anything into jellyfin-web. Ana's part plays the
film Jellyfin plays as it is and seeks five times; Ben's plays the one it
converts and seeks three times, as in u5-4.

## The four pairs, judged (`check.sh judge`, 14:30 UTC: all passed)

| Pair | Bar 1: never asked; open + play | seeks (ms) | Bar 2: bytes searched, none found | Bar 3: apart |
|---|---|---|---|---|
| computer, Chromium | Alex 569 + 1,378 ms; Bo 525 + 1,883 ms | 487, 390, 396, 389, 392; 434, 400, 535 | 37,821,127 | passed |
| computer, WebKit | Alex 711 + 1,197; Bo 929 + 1,670 | 420, 595, 480, 549, 407; 406, 407, 427 | 76,970,352 | passed |
| phone, Chromium | Ana 5,480 + 3,833; Ben 8,152 + 6,904 | 748, 747, 687, 1,012, 1,185; 1,129, 395, 1,516 | 74,401,501 | passed |
| phone, WebKit | Cleo 5,520 + 8,049; Dara 19,357 + 15,264 | 404, 1,938, 401, 1,484, 402; 403, 428, 427 | 84,168,430 | passed |

Each line is `judge.py`'s, in `steps.log`. **Bar 1**: every person opened
jellyfin-web signed in — `asked` false, nothing typed, the stored sign-in
naming their own account and the stand-in for a token (`stored`) — and
played within 30 s of play, no second tap; every seek played again within
5 s, the slowest 1,938 ms. **Bar 2**: no byte any person received — every
response header, body and socket frame — holds the key the lender gave the
middleware, the lender's password or token, a password, or the other
person's token; nor does any computer's svrngs output or the middleware's
log (`scan.json`, `logs`: 23 files, no hit). **Bar 3**: in every phase the
person not seeking was left alone (no `emptied`, `loadstart` or new
element, never back, never still); afterwards each was signed in as the
account the middleware made for them, `<name>-<16 hex>`, and Jellyfin
held each browser's own device in a playing session of that account's.
Jellyfin's message to each session showed on that person's page within
11–77 ms (`socket`).

## Each bar watched failing on its plant (computer, Chromium)

| Plant (`JELLYFIN_MW_PLANT`) | What failed | What held |
|---|---|---|
| `no-sign-in`: the page passed on without its stored sign-in (14:24:08) | **no sign-in**: both asked "Please sign in" and stopped; seeks and apart could-not-judge | no lender's key |
| `lender-key`: the stored sign-in holds the lender's key for its token (14:25:03) | **no lender's key**: `mw_key` found in `/web/` by both | everything else: they played, seeked and were apart — only the scan sees it |
| `one-device`: both one account, on one device, as `check.sh plant one-device`'s lend (14:25:45) | **apart**: both signed in as `everyone-eeeeeeeeeeeeeeee`, and neither browser's device in any session | both plays were left alone in every phase |

The row named the third plant's shape as "one's play disturbed". In
u5-4's plant a person's sign-in on the lend's one device ended the other's
token; through the middleware nobody signs in from a browser, the
middleware signs the one device in once, and neither play was disturbed.
The bar caught it by the account and the device.

## What it found

1. **jellyfin-web's socket names no device, and was refused until the
   middleware chose one.** The first pair (14:21, kept as
   `pair-mw-computer-chromium-socket-403.json`) passed every judged bar,
   yet its socket was refused 403, 17 times for each person
   (`mw-logs.txt`): jellyfin-web 12 opens `/socket?ApiKey=<its token>`
   with no `authorization` and no `deviceId`, so the middleware, which
   signs a request in on the device it names (mw-1), passed it on signed in
   as nobody, and Jellyfin's message reached nobody (`message_shown_ms`
   null). The middleware now signs a request that asks to be signed in by
   its query and names no device on the person's last device that named
   itself (`people.rs`, `On::Last`; `7e0e1c5`); in every run after, each
   socket opened and carried the message. One person on two browsers at
   once would have their socket on the last one's session; no run here
   has that.
2. **The token swap takes exactly u5-3's 43 places.** In every run that
   played the converted film the middleware swapped the person's token
   for the stand-in 43 times: `PlaybackInfo` 1, `master.m3u8` 1,
   `main.m3u8` 41 (`mw-logs.txt`) — u5-3 finding 3's count. No person's
   bytes hold their own token, in any run (`scan.json`, `own_token` 0
   everywhere): the browser holds only the stand-in, and the middleware
   puts each person's token back in what they ask.
3. **What a lent page already runs carries it unchanged.** On a phone the
   lend's worker puts its shim at `<head>` (`web/lend.js:334-341`) ahead
   of the middleware's script, and the shim carries jellyfin-web's socket
   (`web/shim.js:10-11`); the stored sign-in is written at the phone's
   own origin for the lend (`stored.address`), since the script reads
   `location.origin`. No header beyond those the door sets was needed:
   `x-svrngs-key` and `x-svrngs-person` name the person, the client's
   own `authorization` names the device.
4. **The fence.** `git diff 5e877c9..HEAD -- crates web` prints nothing
   (0 lines, after the last run); `scripts/check.sh` runs none of this.

## Against mw-0's predictions (README §4)

- **Bar 1**, predicted to pass, a phone's WebKit seek perhaps missing:
  passed in all four; no seek missed, the slowest 1,938 ms (WebKit, a
  phone). Opening on a phone in WebKit took Dara 19.4 s to the lend and
  15.3 s to play, the slowest here; within the bar.
- **Bar 2**, predicted to pass, with a first version missing a place the
  token is written: passed, and the first version missed none — the
  person's own token was in no byte from the first run on.
- **Bar 3**, predicted to pass, the device not known: passed; Jellyfin's
  sessions show each browser's own device, as mw-1's sign-in per device
  made them. Not predicted: the socket naming no device (found 1).
- **K1, K2**, predicted not to fire: neither fired (found 3, 4).

## Recorded, not judged, for mw-3 (README §5)

**Demo A's unchanged list of users.** Each pair made two users, one per
person, by their first ask: `Alex-26861c44e0d207e3` and
`Bo-dfc73ea4540cb1c0` on the computers (the same in each run, their keys
being the same), and on the phones `Ana-09e0d9fd0aa4ab34`,
`Ben-d560aa7d6f790c3e`, `Cleo-2c0ca0c97d0fbd71`, `Dara-486cee158da4a1cd`,
a phone joining by the link being a person of their own. Each run deleted
the last run's; `users-before.json` `["sam"]`, `users-made.json` the two
`up` makes by hand, `users-after.json` adds the last pair's two. No
person here came through both a computer and a phone, so whether that is
one user is not measured.

**The per-process tie.** In none of the eight pair runs did the
middleware refuse a request for carrying no `x-svrngs-key`
(`mw-logs.txt`, "refused, not the door's: 0"): everything it was asked
came with a key. That counts only keyless requests; a process on this
computer naming a person's key is not told apart (mw-1's record).
