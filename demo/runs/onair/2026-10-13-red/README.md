# On air, red: ON_AIR §7 on svrngs as it is

onair-red (c). The harness `apps/on-air/harness/run.py` ran once on this
host at `3c43877`, with no primitive yet:

    W=/tmp/onair-red-w python3 apps/on-air/harness/run.py demo/runs/onair/2026-10-13-red

The predictions (`PREDICTIONS.md`) were committed in `a92ae3f`, before the
harness first ran against svrngs. This is the reproduction (AGENTS.md 2):
each of the seven bars **failed, for the reason predicted**, and nothing
the harness itself caused failed. One host, one run: not a rate (AGENTS.md
7). Everything below is in `data/`: `verdicts.json` (each bar as `judge()`
read it), `evidence.json`, `station.jsonl`, `bo-listen.jsonl`,
`cy-listen.jsonl`, `events.jsonl`, and each computer's printed words
(`*.out`) and trace (`*.trace`, its lines on the lane, blobs, folders,
removal and refusals). The link's token is never written there.

## The bars

| bar | verdict | what the run said |
|---|---|---|
| 4.1 | failed | the station's `PUT /v1/blob` answered **404** for each of the 10 pieces (`station.jsonl`, `put.status`); Bo held **0 of 10**: every line refused "unsigned: POST /v1/sign answered 404; and the drain names no sender" (`bo-listen.jsonl`) |
| 4.1, carried | failed | Ana's puts of Y and Z **404**, Bo's get of Y **404**; with Ana stopped, Cy's get of Y **404** with no body and no holder in Cy's trace; the plant Z **404** with no body: "no holder answered" not said, and not told apart from a route that is not there |
| 4.1, lapses | failed | 65.1 s after the last put, Bo's and Cy's gets of piece 0, of Y and of a hash never held each answered **404** with no body: lapsed not told apart from never held |
| 4.1, scoped | failed | Dee's `GET /v1/blob/<Y>?group=<onair>` answered **404** with no body: a route not there, not a refusal; no provider refused Dee in Bo's trace |
| 4.2 | failed | `POST /v1/sign?app=on-air` answered **404** for all 11 lines; the page on `evening` played **0 of 10**, refused every line "unsigned: POST /v1/sign answered 404", and said "cannot reach: no line 11 in 12 s". Plants: `flipped` and `bo-signed` refused as unsigned, before any hash or key was asked; `act-sig` refused lines 0 and 1 "its signature does not check under the station's key fbeae5a9…", the one plant refused for its own reason |
| 4.3 | failed | the drain's `payloads` are bare strings: Bo drained 11 and Cy 12 (the 11 lines and Bo's note), each kept as "the drain names no sender", Bo's note among them. The lane carried: each of the 11 pushes answered 200, `delivered: 2`, peers Bo and Cy |
| off air | failed | Bo and Cy refused the unsigned `ended` and stopped at `--for`, "no ended from the station in 80 s"; the page said "cannot reach", never "ended". The plant `no-ended` said "cannot reach: no line 2 in 12 s", the same as the bar's red, as predicted |

## What worked, so no failure above is the harness's

- The group formed: Ana's `/v1/rail/people` named Ana, Bo, Cy and Dee, then
  Ana, Bo and Cy after `svrngs remove Dee` ("Dee is no longer in onair"),
  and Bo's and Cy's own records held the removal before anything was
  pushed (`evidence.json`, `removal_seen`).
- The station cut 40 s into 10 pieces on frame boundaries (nine of 4.023 s,
  the last 3.84 s), aired them over 40.2 s, and wrote 11 lines and 10 pieces
  to the public directory.
- The page loaded every module, with no console error in any of its five
  tabs, and never said "cannot play": Media Source took `audio/mpeg`. The
  stranger exited 0 both times.
- Every process the harness started was stopped at its end.

## Where the harness parts from ON_AIR §7's letter

- **The act-sig plant's signature is read from Ana's journal**, not from
  `/v1/rail/log`: the admitted acts the log gives carry no signature
  (`crates/svrngs-core/src/admit.rs:309`, `AdmittedOp`; this run's log
  fields were `actor, id, person, seq, ts_unix, voided`). The plant's
  point stands: one of Ana's acts, signed by the key that signs for the
  app, offered as an app's line.
- **The public directory is `harness/public.py`**: http.server's handler on
  127.0.0.1 with a listen backlog of 128. `python3 -m http.server`'s
  backlog of 5 reset Chromium's thirty-odd module loads at once, and the
  page never ran (`2a37770`).
- **`show` is not in `apps/on-air/nouns.txt`**: svrngs prints it as a verb
  (`crates/svrngs/src/cli.rs:125`, `:1238`; `register.rs:316`), and the
  gate cannot tell the two.

## The sound

40 s generated here by ffmpeg's `aevalsrc`: a sine stepping up a fifth
every 2 s (start: 220 Hz), mono, 128 kbit/s MP3, 640,731 bytes. Nobody holds
rights to it. Nobody heard it in this run, and no bar asks that.

## What the primitives' rows turn green

- **onair-blob** (4.1, carried, lapses, scoped): the station's put answers
  a hash, listen's get answers bytes, a silent holder is said, a lapse is
  said apart from never held, and a provider's refusal of Dee shows in a
  member's trace. Dee's node may refuse her ask itself, since it no longer
  holds the group. That alone is not the provider's check, and the
  harness's scoped bar looks for the provider's refusal.
- **onair-sign** (4.2, off air): every line is signed, so the page plays and
  says "ended". Then `flipped`, `bo-signed` and `act-sig` each fail for a
  reason of their own.
- **onair-live** (4.3): each drained payload names its sender's key and
  person key, so listen takes Ana's lines by the drain alone and sees
  Bo's note named Bo.
