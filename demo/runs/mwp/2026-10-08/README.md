# mwp-3: Jellyfin through the sign-in, the proxy gone — 2026-10-08

mwp-3's row (`ralph/next/mwp/STATE.md`) judged by DESIGN §3 "The room"
call 6's bars 1 and 2, beside mw-2's numbers
(`demo/runs/mw/2026-10-08/mw-2/README.md`). Against `check.sh up`'s
scratch Jellyfin 12.0.0 on this host at `/tmp/mwp3` (`steps.log`, from
18:10 UTC), svrngs and the middleware at `a81178d`; Playwright's Chromium
147.0.7727.15 and WebKit 26.4 (an iPhone on a phone); the computers and
the phones reach Sam's through iroh's public relays. `run.sh` reruns it
(`apps/jellyfin/check/check.sh mw2`, mw-2's sequence as one step); it ran
as written, 7 min 26 s, exit 0.

What changed from mw-2: the middleware is no longer lent. Sam lends
Jellyfin itself, `svrngs share 8096 --name jellyfin-mw --sign-in 8097`
(`share` asked no password, and said "Each person is signed in to it as
themselves by what answers on port 8097, with nothing typed",
`steps.log`), and the middleware answers the door's `POST /` alone
(`apps/jellyfin/mw/src/answer.rs`). The door sets each person's token,
swaps it for the stand-in both ways and puts the script first in their
pages (mwp-1, mwp-2). One more step than mw-2's: before each computer
pair, after the accounts are deleted, one request through Alex's and
Bo's addresses was answered 401 by Jellyfin, so the door forgot their
answers (`check.sh` `mw_forget`; ten `forgotten` in `door-sign-in.log`).

## The four pairs (`check.sh judge`, 18:16 UTC: all passed)

| Pair | Bar 1: never asked; open + play (ms) | seeks (ms) | bytes searched, none found | apart | mw-2's seeks, for comparison |
|---|---|---|---|---|---|
| computer, Chromium | Alex 1,568 + 1,474; Bo 1,184 + 3,116 | 481, 460, 433, 359, 401; 479, 474, 446 | 37,813,180 | passed | 487, 390, 396, 389, 392; 434, 400, 535 |
| computer, WebKit | Alex 998 + 1,447; Bo 1,088 + 2,227 | 415, 509, 500, 554, 409; 409, 411, 435 | 82,492,001 | passed | 420, 595, 480, 549, 407; 406, 407, 427 |
| phone, Chromium | Ana 4,354 + 3,593; Ben 9,503 + 7,164 | 622, 616, 584, 878, 1,176; 1,065, 385, 1,468 | 76,490,707 | passed | 748, 747, 687, 1,012, 1,185; 1,129, 395, 1,516 |
| phone, WebKit | Cleo 5,459 + 5,975; Dara 17,713 + 9,277 | 404, 407, 4,008, 411, 406; 406, 410, 427 | 79,172,532 | passed | 404, 1,938, 401, 1,484, 402; 403, 428, 427 |

**Nothing typed**: every person opened jellyfin-web signed in — `asked`
false, the stored sign-in naming their own account and the stand-in for a
token — and played within 30 s, no second tap. **Every seek within 5 s**:
the slowest Cleo's third, 4,008 ms (WebKit, a phone), where mw-2's
slowest was 1,938. **No lender's key in a byte**: no response header,
body or socket frame either person received holds the key the lender
gave the middleware, the lender's password or token, the passwords of
the accounts the lender made, or the other person's token; nor do the
computers' svrngs output or the middleware's logs (`scan.json`, `logs`:
22 files, no hit). Each person's own token was in no byte they received
either (`own_token` 0 in every pair), as in mw-2, now by the door's swap.
The passwords the middleware invents are kept in its memory alone, so no
file holds them to search for, as in mw-2. **The two apart**: neither was
disturbed while the other seeked; afterwards each was signed in as the
account made for them, `<name>-<16 hex>`, and Jellyfin held their device
in a playing session of that account's. Jellyfin's message reached each
session's page in 13–71 ms (`socket`).

## Each plant caught (computer, Chromium)

| Plant (`JELLYFIN_MW_PLANT`) | What failed | What held |
|---|---|---|
| `no-sign-in`: answered with no script (18:10:39) | **no sign-in**: both "Please sign in", stopped; seeks and apart could-not-judge | no lender's key |
| `lender-key`: the script's stored sign-in holds the lender's key (18:11:24) | **no lender's key**: `mw_key` in `/web/` for both | they played, seeked and were apart |
| `one-device`: everyone one account on one device (18:12:35) | **apart**: both `everyone-eeeeeeeeeeeeeeee`, both on `svrngs-mw-one-device` | neither play disturbed |

As in mw-2, each caught by the bar it plants against.

## mw-1's check, again (`mw-mw1-again.json`)

Every bar passed through the sign-in: each a user of their own made by
their first ask, a second ask makes none, each person's session on the
device named from their key (`svrngs-e0bbe6435b6ec390`,
`svrngs-c702768d92865076`), no lender's key in 76,864 and 76,859 bytes, a
keyless ask at the middleware's port refused 403, a body sent, a range,
the page and the socket carried. Recorded: a process on Sam's computer
asking the port with Alex's key got a secret (200), as DESIGN §9's tie
leaves it.

## What it found

1. **One ask a person a run.** `door-sign-in.log`: 16 asks — Alex and Bo
   in mw-1's check, then two people in each of the seven pair runs — and
   10 forgets, all `mw_forget`'s; the middleware was asked twice in each
   pair's log (`mw-logs.txt`). No browser's request was refused 401 and
   asked again.
2. **The device is the person's, as DESIGN said it would cost.** jellyfin-web
   is told its device by the script (`_deviceId2`), so the socket names
   the device the door signs in, and mw-2's `On::Last` is not needed: every
   socket carried its message. Jellyfin's sessions name the client
   `svrngs`, not `Jellyfin Web`; nothing judged here read it.
3. **The fence.** The door learned no app: `scripts/vocab_check.py`, 0 app
   nouns in infra code; svrngs ran Jellyfin's middleware as nothing but a
   port it asks.

## Against the bars' predictions (DESIGN §3 call 6)

- **Bar 1**, predicted all pass, one phone seek in WebKit perhaps
  missing: all passed; none missed, the slowest 4,008 ms; each person's
  own token in no byte, as predicted.
- **Bar 2**, predicted about 430 (380–480) from 1,039: **669** (viewer.rs
  241, answer.rs 112, main.rs 99, person.rs 88, people.rs 78, page.rs 51;
  the awk in `run.sh`'s header). The letter misses by 189. Its spirit —
  the door carries the general pieces, the middleware keeps Jellyfin's
  nouns — holds: no forward, hop list, swap or page rewrite is left in
  it. The excess is the five plants the checks use (about 55 lines), the
  credential's form (`Device`, about 35, not in the prediction), and the
  refusals and traces as rustfmt breaks them.

One host, one run of each: not a rate.
