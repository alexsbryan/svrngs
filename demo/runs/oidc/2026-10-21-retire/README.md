# Jellyfin's retirement — the screen and Movie Night still find it, 2026-10-21

Row oidc-retire (`ralph/next/oidc/STATE.md`), part (b), the trial's bar 3
(svrn-docs `rings/current/SIGN_IN_WITH_SVRNGS.md` §7 3). The program per app
retired in `337a7c63`: `apps/jellyfin/mw/` and the well-known table's helper
row are gone, and a kind is a port or its lender's word. This run asks
whether the two things that used Jellyfin still find it, **run as they were
before the retirement**: the checks are not changed by this row.

This section is committed before any data below it.

## What is run

- **The screen.** `apps/jellyfin/check/screen.sh` (u6-2's instrument, as
  committed since `e019551`): `up` lends Sam's scratch Jellyfin with
  `svrngs share 8096` — the table's kind for the port, no program beside
  it — then `run a`, `run b` and `run control`, judged by `screen.py`. The
  bars are `demo/runs/u6/2026-10-06-jellyfin-screen/README.md`'s and are
  not changed here: (a) the screen types its account once, plays, and seeks
  three times within 5 s each, every request at the room address; (b) a new
  screen asks for Quick Connect, a phone approves it, and nothing is typed
  on the screen; `control` shows the setting the run changes goes false then
  true.
- **Movie Night.** `apps/movie-night/check/check.sh up`, the first step of
  its run (a40fe81d), run as it is kept.

## The predictions, before any data

1. **The screen's a, b and control pass.** `check.sh up` never called the
   retired program: it lends the port with `share 8096 $SHARE_FLAGS`, and
   `share` names the kind from the table's `8096 jellyfin` row
   (`crates/svrngs-node/src/well_known.txt:24`), which this row keeps. The
   account the screen types is made by the lender in Jellyfin's own pages
   (`lender.mjs`, the `accounts:` clause), not by the program. Falsified by
   any part failing, or any request of the screen leaving the room address.
2. **Movie Night is could-not-judge, not failed.** Its `up` runs the
   retired program's binary (`apps/movie-night/check/check.sh:144`, built
   before by `apps/jellyfin/check/check.sh mw`) and lends with `share 8096
   --sign-in` (`:215-216`), the flag the hook's cut already refuses
   (`crates/svrngs/src/cli.rs:393-395`, `d78ec302`). The check therefore
   cannot set up here at HEAD, before or after this row's cut, and its bars
   are not judged; this row does not rewrite it, because the row says to run
   the checks as they were. Falsified if `up` gets past the program's start
   and the sign-in lend.

One host, one run of each: not a rate (AGENTS.md 7). A run that does not
finish is could-not-judge, never a pass (AGENTS.md 6).

## How it ran

Everything above this heading was committed before any data (`899dfba7`).
svrngs at `e349e3c0`, with the retirement in (`337a7c63`); the instruments
are unchanged. Jellyfin 12.0.0 (the macOS app's server, `$JF_APP`), the
screen HeadlessChrome 147.0.7727.15 at 1920×1080, the phone WebKit as an
iPhone 15.

```
cargo build -p svrngs
S=apps/jellyfin/check/screen.sh D=demo/runs/oidc/2026-10-21-retire/data
$S up /tmp/oidc-retire-2026-10-21 $D
$S run /tmp/oidc-retire-2026-10-21 $D a
$S run /tmp/oidc-retire-2026-10-21 $D b
$S run /tmp/oidc-retire-2026-10-21 $D control
$S logs /tmp/oidc-retire-2026-10-21 $D
python3 apps/jellyfin/check/screen.py $D > $D/verdicts.json
$S down /tmp/oidc-retire-2026-10-21 $D
apps/movie-night/check/check.sh up /tmp/mn-retire-2026-10-21 $D/movie-night
```

## Against the predictions

`data/verdicts.json` is `screen.py`'s; the numbers are from `data/a.json`,
`data/b.json` and `data/control.json`.

| | predicted | as it ran | verdict |
|---|---|---|---|
| a | passed | **the screen found Jellyfin and played**: it opened the room address in 497 ms, was asked to sign in, signed in as `dee` in 240 ms typing once, played by DirectPlay in 1,475 ms, and played again 380, 362 and 375 ms after three seeks (bar 5,000 each). Its session is Jellyfin's, `dee`, Chrome, `Test Pattern One`. Of its requests, none went to Sam's Jellyfin another way; one went elsewhere, Google's Chromecast sender `https://www.gstatic.com/cv/js/sender/v1/cast_sender.js`, which jellyfin-web asks in Chrome and the stand-in refused — the same request u6's run reported, so `every_request_at_room` is **failed** on it, and it is not Jellyfin's | typed_once passed; every_request_at_room failed on `cast_sender.js` |
| b | passed | **could-not-judge**: the second screen offered Quick Connect and showed a code (`code_shown` true, enabled as it came), and the phone's journey stopped at the group's join page in 120 s: "Someone in House already goes by Dee. If that's you, open your home page on your computer, press 'Make a link to add your phone or another computer'…". The phone joins by the group link as Dee (`apps/jellyfin/check/person.mjs:182-190`); since start-home (ab6ef1fa) the product sends a phone to a device link instead, and the instrument predates it. The judge's letter says failed, because it has no `stopped` branch for b; by AGENTS.md 6 the run did not finish, so it is could-not-judge | could-not-judge |
| control | passed | the setting `Allow remote control of other users` read false, then true | passed |

The lend the screen used was named by the table, not by a program: `share`
printed `You lend jellyfin (port 8096)` and the accounts line
(`data/sam-share.out`), and the room door took 351 requests, none refused,
all from 192.168.1.11 (`data/dee-room.log`). Sam's door let all 351 in as
Dee, not relayed (`data/sam-run.log`, `data/verdicts.json`'s `logs`).

## Movie Night: could-not-judge, as predicted

`check.sh up` (`data/movie-night/up.out`, `steps.log`) started Sam's and
Dee's Jellyfins and their films, then could not start the retired program:

    nohup: /Users/alexsbryan/dev/svrngs/apps/jellyfin/mw/target/debug/jellyfin-mw: No such file or directory

(`data/movie-night/mw-sam.log`), and its lend was refused in a person's
words:

    svrngs: --sign-in is gone: svrngs no longer runs a program of yours to
    sign people in; sign-in with svrngs is the door's own

(`data/movie-night/share.log:1`), the refusal the hook's cut put at
`crates/svrngs/src/cli.rs:393-395`. The check stopped there, before Movie
Night searched anything, so its bars are could-not-judge here — as they
were at HEAD before this row's cut (the same refusal) — not failed. Its
Jellyfin lends and its `--reads jellyfin` were reached by nothing this run;
a later run through the app's own accounts or through the SSO plugin is
not this row's.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
