# Two of one person's devices at once, after (b) — this host, 2026-10-12

Row bash-two-at-once (b): `../2026-10-12/`'s steps again, on `0e04c99`,
which tells a lend the device asking (`x-svrngs-device`), keeps a
sign-in's answer a device, and names each Jellyfin device from it.
`../2026-10-12/` is the red; this is the green. Predictions first, in
`PREDICTIONS.md` (`9068baa`); the driver, `run.py` and `two.mjs`, is
`../2026-10-12/`'s copied, changed only in the trace words it counts, as
it ran at `9068baa`. Every person is a program; one host, one run: not a
rate.

## How it ran

```
W=/tmp/two-run-b python3 demo/runs/two-at-once/2026-10-12-b/run.py     # data/
```

The build is `9068baa`'s (`data/events.jsonl`, `start`), the page built
by `scripts/build-web.sh` and served on this host, nothing published;
n0's public relays. Jellyfin 12.0.0, the same films: the laptop *Test
Pattern Two*, converted, the phone *Test Pattern One*, as it is
(`two.films`). The 1-minute load 31.2 at the start, 31.0 as the browsers
began and 30.4 at the end (`events.jsonl`; `end_load`). One take.

## Against the predictions

| | predicted | as it ran | |
|---|---|---|---|
| 1 | both in, no name; each its own stored device | the laptop "You're in two, as Ana." in **801 ms** (`laptop_join`); the phone "You’re in two, as Ana" **5,769 ms** after opening the link, never asked a name (`two.devices.phone.joined`); neither asked a sign-in. The laptop stored `svrngs-66bdf92e44bb5ac5`, the phone `svrngs-dd9462cf94827c61` (`devices.*.device`): the first 16 hex of the two keys Sam's register lists for Ana beside her computer's `dbdb7c84…`, her `person_key` (`people`) | held |
| 2 | each plays within 30 s | the laptop in **4,953 ms**, the phone in **10,181 ms** (`devices.*.play_ms`) | held |
| 3 | neither takes the other's film | over the 180 s window, outside each page's own seeks: no `emptied`, `loadstart` or new element, no still of 5 s; the phone one `stalled` (`devices.*.window`). The laptop played 177.3 s, the phone 171.8 s. Every seek played again: the laptop's in **402–2,784 ms**, the phone's in **1,096–3,937 ms** (`devices.*.seeks`) | held |
| 4 | two sessions, each its own film | each of the 89 reads in the window listed **two** sessions of Ana's, `svrngs-66bdf92e44bb5ac5` naming the laptop's film in all 89 and `svrngs-dd9462cf94827c61` the phone's in all 89, never the other's (`two.sessions`) | held |
| 5 | stopping one leaves the other | the laptop's film stopped in 35 ms, reported at 935.7 s (`devices.laptop.stop`); the phone over the next 20 s: no `emptied` or `loadstart`, **19.8 s** played (`devices.phone.alone`); its session named its film in all 10 reads after the laptop's stop, the laptop's named nothing in 9 and its own film in the 1 read before its stop was seen (`two.sessions`, phase `laptop stopped`) | held |
| 6 | each film's place its own | the laptop's film kept at **935.7 s**, its player at 935.91 s; the phone's at **214.4 s**, its player at 214.45 s (`two.kept`; `devices.*.stop.before`) | held |
| 7 | one account, asked once a device | Jellyfin's users after `Ana-dbdb7c84edf72df8` and `sam` (`accounts`); the middleware asked twice for Ana's key `dbdb7c84…`, once with device `66bdf92e…` and once with `dd9462cf…` (`sign_in_asks`); Sam's door asked twice and gave the kept answer 583 times (`door_sign_in`) | held |

## Against the red

`../2026-10-12/`, before (b), on the same steps: one session for both
devices in each of its 89 reads, its now-playing changing 24 times
between the two films, and the laptop's stop blanking it while the phone
played. Here: two sessions in each of 89, each naming its own film
throughout, the phone's untouched by the laptop's stop. What each page
did held in both runs; what Jellyfin saw changed. REVIEW_FINDINGS #125's
question is answered by the two runs.
