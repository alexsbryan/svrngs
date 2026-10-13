# Two of one person's devices at once — this host, 2026-10-12

Row bash-two-at-once (a) (REVIEW_FINDINGS #125): Ana's laptop and her
phone, each added by her device link, each playing and seeking a film of
its own through Sam's jellyfin at once. Predictions first, in
`PREDICTIONS.md`, committed before any data (`789ea4b`); the driver,
`run.py` and `two.mjs`, at `c22ca7d`, as it ran. Every person is a
program; no people bar is judged (AGENTS.md 7). One host, one run: not a
rate.

## How it ran

```
W=/tmp/two-run python3 demo/runs/two-at-once/2026-10-12/run.py     # data/
```

The build is `c22ca7d`'s (`data/events.jsonl`, `start`); the page built by
`scripts/build-web.sh` and served on this host by `web/tests/serve.mjs`,
nothing published; n0's public relays. Jellyfin 12.0.0 (`summary.json`,
`jellyfin.version`), Demo A's films from `/tmp/onb-e-films`: the laptop
played *Test Pattern Two*, `cabebc6e…`, which Jellyfin converted
(`Transcode`), the phone *Test Pattern One*, `129e6d85…`, as it is
(`DirectPlay`) (`two.films`; `two.sessions`, `method`). The 1-minute load
24.7 at the start, 30.6 as the browsers began and 22.5 at the end
(`events.jsonl`; `end_load`). One take.

## Against the predictions

| | predicted | as it ran | |
|---|---|---|---|
| 1 | both in by the device link, no name; no sign-in asked; one stored device | the laptop's `join`, no `--as`: "You're in two, as Ana." in **1,345 ms** (`laptop_join`); the phone, never asked a name, "You’re in two, as Ana" **5,491 ms** after opening the link (`two.devices.phone.joined`). Neither asked a sign-in (`asked_sign_in: false`); each stored the door's stand-in for a token and the device `svrngs-bf5cb6a9597af715`, the first 16 hex of Ana's `person_key` `bf5cb6a9…` (`devices.*.device`; `people`) | held |
| 2 | each plays within 30 s of its play | the laptop in **1,986 ms**, the phone in **8,402 ms** (`devices.*.play_ms`) | held |
| 3 | neither takes the other's film | over the 180 s window, outside each page's own seeks: no `emptied`, `loadstart` or new element, no still of 5 s, on either; the phone one `stalled` (`devices.*.window`). The laptop played 179.6 s of it, the phone 174.0 s. Every seek played again: the laptop's four in **433–514 ms**, the phone's in **1,054–2,924 ms** (`devices.*.seeks`). The phone's video named its own film at the end (`at_window_end.item`); the laptop's is a converted film's stream, its source no film id (`item: null`), so that check could-not-judge for it — the 22 progress reports and the stop it sent named its own film alone (`devices.laptop.reports`) | held; the laptop's source could-not-judge |
| 4 | one session for both, its film overwritten | each of the 89 reads in the window listed **one** session of Ana's, `c73c58d4…`, device `svrngs-bf5cb6a9597af715`, client `svrngs`, "Ana via svrngs" — the middleware's device, not jellyfin-web's. Its now-playing named the laptop's film in 46 reads and the phone's in 43, changing **24 times** in the window, each time to whichever had last reported, its position the reporting film's (`two.sessions`) | held: the overwrite |
| 5 | stopping one leaves the other | the laptop's back button stopped its film in 39 ms, reported at 935.7 s (`devices.laptop.stop`). The phone over the next 20 s: no `emptied` or `loadstart`, **19.8 s** played (`devices.phone.alone`). Jellyfin's one session named nothing playing at 205,346 ms — **while the phone played** — and the phone's film at 207,344 ms, 4.1 s after the laptop's stop at 203,215 ms (`two.sessions`) | held; the laptop's stop blanked the phone's play in Jellyfin's list for up to 4.1 s |
| 6 | each film's place its own | Ana's account kept the laptop's film at **935.7 s**, where its player stood at 935.92 s, and the phone's at **215.3 s**, its player at 215.46 s (`two.kept`; `devices.*.stop.before`) | held |

The sign-in was asked once for Ana's key (`sign_in_asks`; `data/mw.log`),
and Sam's door gave the kept answer 581 times (`door_sign_in`). Jellyfin's
users after: `Ana-bf5cb6a9597af715` and `sam` (`accounts`).

## What it found

1. **Ana's devices play apart; Jellyfin sees them as one.** What each
   person-facing page does held: each film played, seeked and was kept
   where it was left. But Jellyfin listed one device for both, its
   now-playing overwritten every ten seconds or so by whichever reported
   last, and the laptop's stop cleared it while the phone played on. The
   session is the middleware's device, `client: svrngs`, "Ana via svrngs"
   (`apps/jellyfin/mw/src/person.rs:113`): Jellyfin holds a token's session
   to the device it was signed in under (`apps/jellyfin/mw/src/viewer.rs:42-47`),
   and the door keeps one token a person (`crates/svrngs-node/src/door/sign_in.rs:262`).
   By `PREDICTIONS.md`'s rule, written before the data, prediction 4
   holding sends the row to (b): each device its own Jellyfin device under
   the one account.
2. **One host is not every path.** The laptop's path to Sam's door is this
   host's loopback, the phone's a relay; neither stalled for 5 s at loads
   to 30 on 12 cores. One run, one host: not a rate.
