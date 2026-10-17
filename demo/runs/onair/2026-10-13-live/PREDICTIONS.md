# On air, the live lane naming who spoke: predictions

onair-live's check: "the harness's 4.3 row green". The harness
`apps/on-air/harness/run.py`, unchanged since onair-red, runs once on this
host at b3bfc0f (onair-live (b)), with the live sender and neither other
primitive:

    W=/tmp/onair-live-w python3 apps/on-air/harness/run.py demo/runs/onair/2026-10-13-live

Written before the harness ran against b3bfc0f. One host, one run: not a
rate (AGENTS.md 7).

| bar | predicted | why |
|---|---|---|
| 4.3 | **passed** | each drained payload is `{payload, key, person_key}` (rail.rs `drain_answer`): every line Bo and Cy drain (10 pieces and `ended`) names Ana's key and her person key as Ana's `/v1/rail/people` names them; Bo's note, pushed before the station starts, is in Cy's drain named Bo's key. Its plant, the drain without its sender, is the red run (`../2026-10-13-red/README.md`, 4.3) and the test's plant in b3bfc0f |
| 4.1 | failed | no `/v1/blob` yet (onair-blob): listen now takes each line by the drain naming the station, and says "not held" on the get's 404, Bo holding 0 of 10 |
| 4.1, carried | failed | as in the red run: every put and get 404 |
| 4.1, lapses | failed | as in the red run |
| 4.1, scoped | failed | as in the red run |
| 4.2 | failed | no `/v1/sign` yet (onair-sign): the page plays 0 of 10 |
| off air | failed | by the page alone: Bo and Cy now say "ended" within one piece, taking the unsigned `ended` by the drain naming the station; the page, which reads no drain, still says "cannot reach". The plant `no-ended` says "cannot reach" |

Anything else that fails is the harness's, and is said as that.
