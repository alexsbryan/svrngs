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
