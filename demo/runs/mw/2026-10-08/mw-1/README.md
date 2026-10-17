# mw-1: a user per person, through the middleware — 2026-10-08

mw-1's row (`ralph/next/mw/STATE.md`) under mw-0's rule (`../README.md`).
Against `apps/jellyfin/check/check.sh up /tmp/mw1 demo/runs/mw/2026-10-08/mw-1`'s
scratch Jellyfin 12.0.0 on this host (`steps.log`, 13:44 UTC).

## Two probes, before the middleware signs anyone in

Run by hand with `curl` against the scratch Jellyfin at 127.0.0.1:8096, a
user `probe-p` made for them with an API key minted from `/Auth/Keys`,
both deleted afterwards (204, 204; `GET /Users` then
`["ana", "ben", "sam"]`, `GET /Auth/Keys` then `[]`).

**1. A token is held to the device it was signed in under.** `probe-p`
signed in by `AuthenticateByName` under `DeviceId="dev-a"`, then
`GET /Users/Me` with that token under `Device="Dev B", DeviceId="dev-b"`:

    me under B: 200
    GET /Sessions:  probe-p dev-a Dev A c
    GET /Devices:   device dev-a Dev B probe-p

The session stays `dev-a`; only the device's name follows the asker. So a
token signed in under one device and used under another shows the first in
Jellyfin's sessions: mw-0's prediction for bar 3 named this case (README
§4), and the middleware signs a person in once per `DeviceId` the client
names.

**2. Resetting a password revokes the person's other tokens.** With
`dev-a`'s token held, `POST /Users/Password?userId=…` by the API key, then
the old token:

    reset: 204
    dev-a token after reset: 401
    dev-b token: 200
    dev-b token after dev-a signs in again: 200

So upstream's way — a password reset by the elevated key before each sign
in (`viewer.rs:378-396` at `d4356181c`) — would end a person's first
device when their second signs in. The middleware keeps the password it
invented in memory (never on disk) and signs each device in with it;
signing in again does not end another device's token (the last line).

A request with a token to a static or public path (`/web/index.html`,
`/System/Info/Public`, with `dev-b`'s token under `DeviceId="dev-c"`)
answered 200 and made no device: `GET /Devices` after it listed `dev-a`,
`dev-b` (its name now `Dev C`) and Sam's two, no `dev-c`.

**3. A device the middleware makes up opens a session of its own.** A
smoke run of the middleware's first version, straight at its port with a
made-up key: a request naming no device (`GET /web/`) was signed in on a
device the middleware named for the person, and that sign-in put
`Smoke-ffffffffffffffff svrngs-ffffffffffffffff Smoke via svrngs` in
`GET /Sessions` beside the person's own device. So a request that names
no device goes on signed in as nobody (`person.rs`, `Device::of`); the
user and key the smoke run made were deleted (204, 204).

## The check, and its runs

    apps/jellyfin/check/check.sh up /tmp/mw1 demo/runs/mw/2026-10-08/mw-1
    JELLYFIN_MW_PLANT=one-user    apps/jellyfin/check/check.sh mw /tmp/mw1 demo/runs/mw/2026-10-08/mw-1 plant-one-user
    JELLYFIN_MW_PLANT=let-keyless apps/jellyfin/check/check.sh mw /tmp/mw1 demo/runs/mw/2026-10-08/mw-1 plant-let-keyless
    apps/jellyfin/check/check.sh mw /tmp/mw1 demo/runs/mw/2026-10-08/mw-1 green
    apps/jellyfin/check/check.sh down /tmp/mw1 demo/runs/mw/2026-10-08/mw-1

`mw` mints the middleware a key in Jellyfin's API Keys, once, and starts
it on 127.0.0.1:8097 with the key on standard input; the first time, Sam
lends it (`svrngs share 8097 --name jellyfin-mw --kind jellyfin --limit
8`, Enter alone at the password), Bo joins House as a second computer,
and Alex's and Bo's bare commands are started again to list the lend
(`alex-bare-command-mw.out`). Each run starts the middleware afresh and
first deletes the users it named (`<name>-<16 hex>`), so each person's
first ask in a run is new to Jellyfin. `mw.py` asks through each person's
own address of the lend, over HTTP as an app would (mw-2 opens it in
browsers), and judges (`mw-<tag>.json`; every line in `steps.log`):

| Bar | plant one-user | plant let-keyless | green |
|---|---|---|---|
| each a user of their own, made by their first ask, not an administrator | **failed**: both `everyone-eeeeeeeeeeeeeeee` | passed | passed: `Alex-24e2646f6325dcc3`, `Bo-82fe24fec739be58` |
| a second ask makes none | passed | passed | passed |
| each person's own device in Jellyfin's sessions | **failed**: both devices under one user | passed | passed: `mw1-alex`, `mw1-bo` |
| neither receives the lender's key (the middleware's key, Sam's token and password, in every header, body and socket byte) | passed | passed | passed: none in 76,499 and 76,496 bytes |
| no `x-svrngs-key` at the middleware's port is refused | passed | **failed**: 200, as `everyone` | passed: 403, "This port answers only what svrngs's door forwards" |
| carried: a body sent (204), a range back (206, 65,536 bytes), the page (200), the socket (101, Jellyfin's `ForceKeepAlive` frame) | passed | passed | passed |

`steps.log` keeps every attempt. The three before 13:58:16 stopped before
any ask, on the check's own setup: `share` refused a port it does not
know without `--kind`; the bare command names the lend `jellyfin-mw
(jellyfin)`, which the first pattern missed; and Bo's join, run again
over a running one, found its lock held. The first one-user run that
asked (13:58:16) read `GET /Sessions` after closing both sockets and found
neither person: Jellyfin ends a session when its socket closes. `mw.py`
now holds them open until it has read the sessions; the plant runs and
the green one in the table are after that change (13:58:58, 13:59:06,
13:59:15). The fence: `git diff 5e877c9..HEAD -- crates
web` prints nothing.

## Recorded, not judged, for mw-3 (README §5)

**Demo A's unchanged list of users.** `users-before.json`
`["ana", "ben", "sam"]` (the lender's own and the two `up` makes by hand);
`users-after.json` adds `Alex-24e2646f6325dcc3` and `Bo-82fe24fec739be58`:
one per person, made by their first ask, under the name the door gives
them and 16 hex of their key; a second ask made none. The list changes,
by construction.

**The per-process tie.** A process on Sam's computer asking the
middleware's port: with no `x-svrngs-key`, refused 403; with Alex's key
(read from the middleware's own log, where a key it sees is named) and a
device of its own, answered 200 as `Alex-24e2646f6325dcc3`, and Jellyfin
then held a session for Alex on that device (`mw1-mallory`,
`mw-green.json`, `sessions_after_anothers_key`), as mw-0 predicted
(README §5). Nothing past the door's port tells its request from any
process's.

**What `share` tells a lender of the middleware.** Lent as `--kind
jellyfin`, `share` prints Jellyfin's account advice (`sam-share-mw.out`):
"asks each person to sign in once, with an account you make them" —
which the middleware now does instead. No change made (fence 1); for the
verdict.
