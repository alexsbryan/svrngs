# On air, the three primitives together: predictions

onair-run. The harness `apps/on-air/harness/run.py`, as of `1b281c6`
(4.1, lapses now reads the stores too), runs on this host at the commit
after this one's, with `/v1/blob` (db6ba51), `/v1/sign` (c03b6db) and the
live sender (b3bfc0f). First each code plant, then the whole run:

    # each plant: applied, run, taken out (plants/<name>.patch)
    git apply plants/<name>.patch
    W=/tmp/onair-run-<name>-w python3 apps/on-air/harness/run.py demo/runs/onair/2026-10-13/plants/<name>
    git apply -R plants/<name>.patch
    # the whole run, on svrngs as it is
    W=/tmp/onair-run-w python3 apps/on-air/harness/run.py demo/runs/onair/2026-10-13

Written before any of these ran. One host, one run of each: not a rate
(AGENTS.md 7). A run that could not finish is could-not-judge, never a
pass (AGENTS.md 6).

## The bound, set before the run

4.1's bound is `BOUND_MS = 4000` (`run.py`, since onair-red's `a92ae3f`):
one piece's time on air, from its line's arrival in Bo's drain to Bo's
listener holding the piece, checked. "Within one piece" for off air is
`PIECE` = 4 s after the station's `ended`, the page given a second more
for its poll.

## Each bar's plant, red first

| bar | its plant | where it runs | predicted |
|---|---|---|---|
| 4.1 | — (ON_AIR §7 names none) | its red is `../2026-10-13-red/`: no `/v1/blob`, Bo held 0 of 10 | not rerun |
| 4.1, carried | Bo not serving: a computer that fetched a hash stops carrying it (`plants/bo-not-serving.patch`, blob.rs) | the harness | **failed**: with Ana stopped, Cy's get of Y 502 `unanswered`, "no holder answered", Ana's "did not answer within 3s" and Bo's `never_held` — never an empty piece. 4.1, scoped, 4.2, 4.3 and off air passed; 4.1 passed (each listener holds what it gets, all from Ana); 4.1, lapses failed too, Bo and Cy having carried nothing to say lapsed of (502, not 410) |
| 4.1, lapses | `keep` ignored: a put kept a day (`plants/keep-ignored.patch`, blob.rs) | the harness | **failed**: 65 s after the last put, Bo's and Cy's gets of piece 0 and of Y answer 200; Bo's and Cy's stores each hold the 10 pieces' files. Ana's restarted store holds none, what apps placed being kept in memory alone. Every other bar passed |
| 4.1, scoped | the provider's group check out (`plants/group-check-out.patch`, folder.rs) | svrngs-node `tests/folders.rs` `bytes_an_app_put_in_one_group_are_refused_to_a_member_of_another` | **failed** at "Ana refuses Cy her films bytes: Ok(50000)". The harness cannot show it: Dee, removed, is refused by her own computer before any provider is dialled (`../2026-10-13-blob/README.md`), so its bar would pass on the plant |
| 4.2 | a byte flipped in a public piece; a line Bo signed; an act's signature offered as an app's | the harness, in every run | each plant's tab refused for its own reason: `flipped` plays all but piece 1, `bo-signed` none, `act-sig` 2 to 9 |
| 4.3 | the drain without its sender (`plants/drain-without-sender.patch`, rail.rs) | the harness | **failed**: every drained payload a bare string, "the drain names no sender", Bo's note among them. Listen still takes each line by its signature, so 4.1, carried, lapses, scoped, 4.2 and off air passed |
| off air | no `ended` signed | the harness, in every run | the `no-ended` tab says "cannot reach", never "ended" |

## The whole run

| bar | predicted | why |
|---|---|---|
| 4.1 | **passed** | 10 of 10 pieces held by Bo, each within 4,000 ms of its line's arrival (onair-blob's: 26 to 120 ms) |
| 4.1, carried | **passed** | Ana stopped, Cy's get of Y 200 from Bo, named in Cy's trace; Z 502 `unanswered`, no empty piece |
| 4.1, lapses | **passed** | 410 `lapsed` of piece 0 and Y at Bo and Cy, 502 of a hash never held, told apart; no piece's file in Bo's or Cy's store; Ana's stopped store holding the 10 pieces' files, and none once restarted and swept (onair-blob's `ana-restart.txt`) |
| 4.1, scoped | **passed** | Dee refused Y by her own computer, 403, `blob: refused` in her trace |
| 4.2 | **passed** | the page on `evening` plays 10 of 10 in Chromium, its refusals none; the three plants as above |
| 4.3 | **passed** | every line Bo and Cy drain names Ana's key and person key; Bo's note in Cy's drain names Bo |
| off air | **passed** | Bo, Cy and the page each say "ended" within 4 s of the station's; `no-ended` says "cannot reach" |

Where the run could part from this: the host's load (onair-blob's run
began at 8.3 and ended at 17.3); Bo asking Cy for a piece Cy is still
fetching, a reset and a second holder (onair-blob's piece 3, 120 ms).
A miss past the bound is said with its milliseconds, not hidden.

Anything else that fails is the harness's, and is said as that.
