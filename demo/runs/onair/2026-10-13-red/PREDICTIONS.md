# On air, red: predictions

Written before the harness first ran against svrngs (onair-red (c); AGENTS.md
2, 6). The run is the reproduction: ON_AIR §7's seven bars on svrngs as it is
at this commit, before any primitive. Each is predicted **failed**, for the
reason said, and nothing else is to fail that the harness itself caused.

## The run

`apps/on-air/harness/run.py`, one host, one run (not a rate, AGENTS.md 7):

- **Who.** Four svrngs, each a data dir and HOME of its own under a scratch
  directory, started as `demo/runs/devices/2026-10-12/run.py` starts them:
  Ana makes the group `onair` (`svrngs new`); Bo, Cy and Dee join by her
  link (`svrngs join`); Ana removes Dee (`svrngs remove Dee`), so Dee is a
  computer outside the group that once stood in it. Ana is the station; Bo
  and Cy listen.
- **The sound.** 40 s generated here by ffmpeg's `aevalsrc` (a sine stepping
  up a fifth every 2 s), mono, 128 kbit/s MP3, no Xing or ID3: nobody's
  rights. Cut into 4 s pieces on frame boundaries: 10 pieces.
- **The public directory.** `apps/on-air/harness/public.py` on 127.0.0.1
  (http.server's handler, its backlog raised: 2a37770), with `page/` in it.
- **The stranger.** `harness/stranger.mjs`: headless Chromium (Playwright
  1.59.1), one tab per link.

The order: Bo's note on the live lane; the station's show `evening` (10
pieces at 4 s, `keep=60`, a signed `ended`), Bo's and Cy's `listen` and the
page on it; then the plant shows; then two blobs Y and Z put on Ana, Y got
by Bo, Ana stopped, Y and Z asked of Cy; Dee asks Y; and 60 s after the
last put, Bo and Cy ask a piece, Y and a hash never held.

The bound for 4.1, set before the run: **4,000 ms**, one piece's time on
air, from the line's drain on Bo to Bo holding the piece. A listener slower
than that falls behind the station.

## The bars, each predicted failed

| bar | predicted verdict | the red output predicted |
|---|---|---|
| 4.1 | failed | the station's `PUT /v1/blob` answers **404** for each of the 10 pieces; Bo's `listen` refuses every line — unsigned (no `/v1/sign`) and the drain names no sender — so holds **0 of 10**; asked directly, Bo's `GET /v1/blob/<hash>` answers **404** |
| 4.1, carried | failed | Y and Z's puts on Ana **404**; Bo's get of Y **404**; with Ana stopped, Cy's get of Y **404** and no holder in Cy's trace. The plant (Z, which no running computer holds) also answers **404**: "no holder answered" is not said, and cannot be told from a route that is not there |
| 4.1, lapses | failed | after `keep=60`, Bo's and Cy's gets of piece 0 and Y, and of a hash never held, all answer **404**: "lapsed" is not told apart from "never held". The plant (`keep` ignored) cannot be told from it either |
| 4.1, scoped | failed | Dee's `GET /v1/blob/<Y>?group=<onair>` answers **404**, a route that is not there, not a refusal; no provider's refusal of Dee in Ana's or Bo's trace |
| 4.2 | failed | Ana's `POST /v1/sign?app=on-air` answers **404** for all 11 lines; the page on `evening` plays **0 of 10**, refuses all 11 as "unsigned: POST /v1/sign answered 404", and says "cannot reach: no line 11 in 12 s". Of the plants: `flipped` (a byte flipped in piece 1) and `bo-signed` (Bo's `/v1/sign`, also 404) are refused as unsigned, before any hash or key is asked; `act-sig` (one of Ana's acts from `/v1/rail/log`, its signature on a line) is refused as "its signature does not check under the station's key", the one plant refused for its own reason |
| 4.3 | failed | `GET /v1/rail/live` gives `payloads` as bare strings: every line Bo and Cy drain is kept as "the drain names no sender", and Bo's note in Cy's drain names no one. The lane itself carries: each station push says `delivered: 2` (Bo and Cy; Dee removed) |
| off air | failed | Bo and Cy refuse the unsigned `ended` and stop at their `--for` saying "no ended from the station"; the page says "cannot reach", never "ended". The plant `no-ended` (two signed-if-it-could pieces, no end) says "cannot reach" too: in red, the bar and its plant say the same |

## Nothing else

Predicted to work, so that a failure among them is the harness's, to fix
before the run is read:

- the group forms: Ana's `/v1/rail/people` names Ana, Bo and Cy (Dee gone
  after the removal);
- the station cuts 10 pieces, writes 11 lines and 10 pieces to the public
  directory, and each live push answers 200;
- the page loads every module, Media Source takes `audio/mpeg` (the page
  never says "cannot play"), and the stranger reads `window.air` from each
  tab;
- every process the harness started is stopped at its end.

The data, the counts and each bar's verdict as the run judged it go in
`README.md` and `data/` beside this file, after the run.
