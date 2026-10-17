# On air, a signature for an app: predictions

onair-sign's check: "the harness's 4.2 row green with its three plants (a
byte flipped in a public piece; a line Bo signed; an act's signature from
`/v1/rail/log` offered as an app's)". The harness
`apps/on-air/harness/run.py`, unchanged since onair-red, runs once on this
host at the commit after this one's (onair-sign (c), `POST /v1/sign`, and
the live sender of onair-live), with no `/v1/blob` yet:

    W=/tmp/onair-sign-w python3 apps/on-air/harness/run.py demo/runs/onair/2026-10-13-sign

Written before the harness ran with `/v1/sign`. One host, one run: not a
rate (AGENTS.md 7).

| bar | predicted | why |
|---|---|---|
| 4.2 | **passed** | every line the station signs is 200 from `/v1/sign` under Ana's computer's key, which the link's fragment carries; `air.mjs` now spells the core's message (`svrngs-app:on-air:` and the line, checked against svrngs-core's frozen vector in `laws.test.mjs`). The page on `evening` plays all 10 pieces. Its plants: `flipped` plays 9, refusing piece 1 by its BLAKE3; `bo-signed` plays none, each line's signature not checking under Ana's key; `act-sig` plays 2 to 9, refusing lines 0 and 1, which carry the signature of one of Ana's acts from her journal (its id as the line, then line 0's text) |
| 4.3 | passed | as in `../2026-10-13-live/` |
| 4.1 | failed | no `/v1/blob` yet (onair-blob): Bo holds 0 of 10 |
| 4.1, carried | failed | every put and get 404 |
| 4.1, lapses | failed | as in the red run |
| 4.1, scoped | failed | Dee's get 404, no provider's refusal |
| off air | passed | Bo and Cy say "ended" within one piece, as in the live run; the page now checks the signed `ended` and says "ended" within one piece and a second of it; the plant `no-ended` says "cannot reach" |

Where the off-air bar could part from this: the page says "ended" when it
reads the line, and its reading lags the station's by up to half a second's
poll plus a piece's append. A miss there is said as that, not as 4.2's.

Anything else that fails is the harness's, and is said as that.
