# Stock Jellyfin as `svrngs share 8096` lends it — u5-4's check, 2026-10-05

u5-4 builds sign in once, as u5-3 decided (DESIGN §3, "Lent apps", call
4): the lend carries no secret, and each person signs in once with an
account the lender made, which `svrngs share` tells the lender how to make.
This is its check (`ralph/next/u5/STATE.md`, u5-4's row), run against a
scratch Jellyfin on this host. `scripts/check.sh` runs none of it: it needs
Jellyfin. What it does run is the command's half, against a stand-in port
(`crates/svrngs/tests/share.rs`,
`a_service_with_its_own_accounts_is_lent_with_no_password`).

## What was built

`svrngs share 8096` (`0f15cb4`) asks no password for a service the
well-known table says keeps its own accounts, keeps none though one is
typed, and prints how to make one, in DESIGN §1's words
(`data/sam-share.out`):

```
Lent jellyfin (port 8096) to everyone in your groups, up to 8 people at once.
jellyfin asks each person to sign in once, with an account you make them. In jellyfin: your picture at the top right, then Dashboard, Users and +; give it a name and a password, tick the libraries they may watch, and Save. Then tell each person theirs.
```

Nothing else changed in svrngs: the door already carries a lend with no
secret whole, the sign-in, the player and the live socket included (u5-3).

## The bars, and how they are read

`apps/jellyfin/check/judge.py`, per run of two people (u5-3's pair: Ana
plays the film Jellyfin plays as it is and seeks it five times while Ben's
is watched; Ben plays the one it converts for every browser and seeks it
three times while Ana's is watched):

1. **list to playing** — each, new to Jellyfin, from the list (the bare
   command's address on a computer; the page's list on a phone) to a
   playing film, asked to sign in and signing in once with the account
   the lender made;
2. **seeks** — every seek, through jellyfin-web's own seek bar, playing
   again within 5 s (Demo A's bar);
3. **left alone** — while one seeks, the other's play never started over,
   sent back or held, and each still signed in as themselves after;
4. **no secret** — no byte either browser received (every header, body
   and socket frame) holds a secret the lend keeps, the lender's password
   or token, or the other person's password or token.

Each is passed, failed or could-not-judge; a run with no file never ran.

## How it ran

Instrument `apps/jellyfin/check/` at `509ca15`; Jellyfin 12.0.0 (the macOS
app's server); Playwright's Chromium and its WebKit as an iPhone (as u5-3);
both computers' `svrngs` and the phones through iroh's public relays.
`data/steps.log` is the order, 03:00–03:15 UTC.

```
cargo build -p svrngs
C=apps/jellyfin/check D=demo/runs/u5/2026-10-05-jellyfin-in/data
$C/check.sh up    /tmp/u5-4 $D
$C/check.sh pair  /tmp/u5-4 $D computer chromium
$C/check.sh plant /tmp/u5-4 $D token chromium        # and one-device; both again in webkit
$C/check.sh pair  /tmp/u5-4 $D computer chromium     # and webkit; phone chromium; phone webkit; RUN=2 computer chromium
$C/check.sh judge /tmp/u5-4 $D <run>...
$C/check.sh down  /tmp/u5-4 $D
```

**The lender, by share's words.** Sam's Jellyfin starts with Sam's own
account alone (`users-before.json`). `share` was given Sam's token at its
prompt and kept none (no `secrets/lends/jellyfin`; `scan.json`
`lend_kept: []`). `lender.mjs` then made Ana and Ben accounts in
Jellyfin's own pages, finding each step by what the printed line names:
seven steps each, two libraries ticked (`lender.json`). Its first two
runs stopped at "tick the libraries they may watch": the boxes are drawn
over, and the libraries come a moment after the form; Jellyfin starts
every one unticked, so an account made without ticking one sees nothing.

## What it found

**Each plant, red.** A plant makes the lend keep a token where `share`
keeps one, as a lender who pasted one before `0f15cb4` would have. Through
it nobody signing in as themselves gets past their sign-in: the first
token plant (03:02, overwritten by the next; its lines are in
`steps.log`) left both people on "Please sign in", each asking their
own settings as the lend's user and answered 403. So each plant also gives
the browsers what its failure was first seen with (`check.sh`, `plant`):

| plant | run | the bar it fails | what failed |
|---|---|---|---|
| the lend's token reaching a browser: the lend keeps `zed`'s token, both browsers signed in as zed by a stored sign-in (u5-3's probe) | `pair-computer-chromium-plant-token.json`, `-webkit-` | no secret | the lend's token in Ben's `PlaybackInfo`, `master.m3u8` and `main.m3u8`, both engines |
| one person's sign-in ending another's: the lend keeps Ana's token on its one device, Ana playing by a stored sign-in, Ben signing in with Ana's account (U3's finding 2) | `pair-computer-chromium-plant-one-device.json`, `-webkit-` | left alone | Ana's `/Users/Me` 401 after Ben's sign-in; Ben left on "Please sign in" |

The one-device plant needs one account for everyone: on this host Jellyfin
12 ended a sign-in only when the same account signed in again on the same
device (one device and two accounts: `/Users/Me` 200 and 200; one account
twice: 401, then 200; one account on two devices: 200 and 200).

**Then green, in both engines, from a computer and from a phone.**
`judge.py` exit 0 over `computer-chromium-2`, `computer-webkit`,
`phone-chromium` and `phone-webkit`:

| run | open + sign in + play, Ana · Ben (ms) | Ana's 5 seeks (ms) | Ben's 3 seeks (ms) |
|---|---|---|---|
| computer, Chromium (`-2`) | 432 + 357 + 1,374 · 533 + 493 + 4,135 | 640, 429, 452, 404, 421 | 427, 445, 405 |
| computer, WebKit | 1,945 + 980 + 2,082 · 2,707 + 877 + 6,041 | 406, 497, 402, 642, 403 | 1,869, 811, 404 |
| phone, Chromium | 4,280 + 1,596 + 5,730 · 9,617 + 817 + 13,737 | 1,171, 1,792, 1,323, 608, 1,296 | 1,332, 427, 499 |
| phone, WebKit | 4,489 + 861 + 6,910 · 15,612 + 2,863 + 16,195 | 402, 823, 412, 1,318, 403 | 410, 411, 431 |

In each, the person not seeking was left alone (no `emptied`, no
`loadstart`, no new element, never back, never still 5 s), both were
signed in as themselves after, each browser kept its own device, and the
lender's message reached each page over the live socket in 15–74 ms. No
byte either browser received held a secret the lend keeps (it keeps
none), the lender's password or token, or the other's password or token,
in 38–91 MB per run (`scan.json`); neither computer's `svrngs` output held
any (`scan.json` `logs`). Jellyfin's users after: `ana`, `ben`, `sam`, the
accounts the lender made (`users-after.json`).

**One seek missed.** `pair-computer-chromium.json` (03:08), the green
computer Chromium run again after the plants: Ben's first seek on the
converted film played again in 6,389 ms, against 5 s; his play took
11,197 ms to start. This host's load average was then 33.63 on 12 cores
(`uptime`, 20:09 local), other builds and Spotlight running; at 03:14,
the load 11, the same run (`-2`) seeked in 405–640 ms. The miss stays a
miss: one host, one run of each, not a rate; Demo A's seek bar is judged
as a rate on the carriage in u5-9 (DESIGN §3, call 4). The first green
computer Chromium run (03:01, every bar passed) was overwritten by this
one; its lines are in `steps.log`.
