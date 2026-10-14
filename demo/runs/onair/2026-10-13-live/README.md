# On air, the live lane naming who spoke

onair-live's check, "the harness's 4.3 row green". The harness
`apps/on-air/harness/run.py`, unchanged since onair-red, ran once on this
host at `53a386c` (the predictions' commit; its code is b3bfc0f's, onair-live
(b)), in 2 min 25 s, exit 0:

    W=/tmp/onair-live-w python3 apps/on-air/harness/run.py demo/runs/onair/2026-10-13-live

The predictions (`PREDICTIONS.md`) were committed in `53a386c`, before it
ran. Every bar came out as predicted. One host, one run: not a rate
(AGENTS.md 7). `data/` is what the harness writes (see
`../2026-10-13-red/README.md`), and `data/<who>-live-sender.trace`, each
computer's live route lines, taken from its whole log after the run (the
harness's own trace keeps `rail live` lines, not the route's `ring live`):

    sed 's/\x1b\[[0-9;]*m//g' /tmp/onair-live-w/<who>.log | grep "ring live: buffered, with its sender"

## The bars

| bar | verdict | what the run said |
|---|---|---|
| 4.3 | **passed** | Bo drained 11 and Cy 12 (`verdicts.json`): the 10 pieces' lines and `ended`, each named `key` and `person_key` `02cddd3a…`, Ana's as her `/v1/rail/people` names her (`events.jsonl`, `group`); and in Cy's drain Bo's note, named `4a508238…`, Bo's, which listen refused as "not a line; sent by 4a508238…, not the station" (`cy-listen.jsonl`). Bo's note went to Ana and Cy, `delivered: 2`; each station push `delivered: 2`. The routes traced each payload they buffered with its sender: Bo 13 and Cy 13 from Ana (`evening`'s 11 and the no-ended plant's 2), Cy 4 and Ana 4 from Bo (his note and the bo-signed plant's 3), Dee 0, removed before anything was pushed |
| 4.1 | failed | no `/v1/blob` (onair-blob): listen took each line by the drain naming the station and said "not held" for all 10 on the get's 404, in Bo's and Cy's records; Bo held 0 of 10 |
| 4.1, carried | failed | every put and get 404, as in the red run |
| 4.1, lapses | failed | as in the red run: lapsed not told apart from never held |
| 4.1, scoped | failed | Dee's get 404, no provider's refusal, as in the red run |
| 4.2 | failed | no `/v1/sign` (onair-sign): the page on `evening` played 0 of 10, and every plant 0 |
| off air | failed | by the page alone, as predicted: Bo and Cy each said "ended" within one piece, by "the drain naming the station" (`cy-listen.jsonl`, `ended`, `by`); the page, which reads no drain, said "cannot reach"; the plant `no-ended` said "cannot reach" |

4.3's plant, the drain without its sender, is the red run's 4.3
(`../2026-10-13-red/README.md`: the drain's `payloads` bare strings, every
line kept as "the drain names no sender"), and the test's plant in
b3bfc0f's body.

## What this run cannot show

Every computer here is one device of its person, so each `person_key` is
its `key`. A second device naming its person's first key is shown by
`crates/svrngs-node/tests/record_api.rs`
`a_payload_drained_names_its_sender_and_their_person`, where Bo's second
device, key 6, drains named key 6 and person key 2 — not by this run.

The sound is the red run's: 40 s generated here by ffmpeg's `aevalsrc`,
nobody's rights, heard by nobody.
