# On air, a signature for an app

onair-sign's check, "the harness's 4.2 row green with its three plants".
The harness `apps/on-air/harness/run.py`, unchanged since onair-red, ran
once on this host at `63057d3` (the predictions' commit; its code is
c03b6db's, onair-sign (c), which the predictions' commit body misnames
"6f-onair-sign (c)"), in 1 min 54 s (`events.jsonl`, `start` to `end`),
exit 0, the load 35.3 at its start and 11.5 at its end:

    W=/tmp/onair-sign-w python3 apps/on-air/harness/run.py demo/runs/onair/2026-10-13-sign

The predictions (`PREDICTIONS.md`) were committed in `63057d3`, before it
ran. Every bar came out as predicted. One host, one run: not a rate
(AGENTS.md 7). `data/` is what the harness writes (see
`../2026-10-13-red/README.md`), and `data/<who>-sign.trace`, each
computer's `/v1/sign` lines, taken from its whole log after the run (the
harness's own trace keeps lines naming `/v1/sign`, which the route's
trace, target `sign`, does not):

    sed 's/\x1b\[[0-9;]*m//g' /tmp/onair-sign-w/<who>.log | grep "sign: "

## The bars

| bar | verdict | what the run said |
|---|---|---|
| 4.2 | **passed** | every `/v1/sign` the station asked answered 200 (`verdicts.json`, `signs`), each with Ana's computer's key `d8b34eed…`; the page on `evening`, given that key in its link, played 10 of 10 and said "ended, after 10 pieces". The plants: `flipped` played 0 and 2 to 9, refusing piece 1, "its BLAKE3 is afdedf6b6058…, not the line's c225526d54ff…"; `bo-signed`, whose lines Bo's computer signed (key `61a033d4…`, 3 signatures, each 200), played none, "its signature does not check under the station's key d8b34eed…; it names 61a033d4…"; `act-sig`, whose lines 0 and 1 carry the signature of Ana's act `e8da6260…` (seq 0) read from her journal, played 2 to 9, refusing 0 and 1, "its signature does not check under the station's key d8b34eed…" (`verdicts.json`, `plant_refusals`; `evidence.json`, `plants`) |
| 4.3 | passed | Bo drained 11 and Cy 12, each line named Ana's key and person key; Bo's note in Cy's drain named Bo's key, as in `../2026-10-13-live/` |
| off air | passed | Bo and Cy said "ended" within one piece, and now the page did too, checking the signed `ended`; the plant `no-ended` said "cannot reach", "no line 2 in 12 s" |
| 4.1 | failed | no `/v1/blob` (onair-blob): Bo held 0 of 10 |
| 4.1, carried | failed | Cy's get of Y with Ana stopped 404, as in the red run |
| 4.1, lapses | failed | as in the red run: lapsed not told apart from never held |
| 4.1, scoped | failed | Dee's get 404, no provider's refusal, as in the red run |

4.2's plants were red before this row: in `../2026-10-13-live/` the page
played 0 of 10 and every plant 0, no line signed (`/v1/sign` 404). Here
the page plays what Ana's key signed and each plant refuses what it
should. The plants' refusals by a signature that does not check are
`air.mjs`'s `checkLine`, which the run proves agrees with svrngs-core's
bytes: the station's lines, signed by the node, checked in Chromium.

The traces: Ana's node signed 13 lines (`evening`'s 10 and `ended`, and
`no-ended`'s 2), Bo's 3 (`bo-signed`'s 2 and `ended`), Cy's and Dee's
none; none was refused (`data/<who>-sign.trace`).

## What this run cannot show

The act-sig plant offers an act's signature over the line's text, the
act's id in hex; an act signs `svrngs-act-id-binding:` and the id's 32
bytes, so no page could ever take it. That an act's signature over its own
message fails as an app's, for the app named after the act's domain, is
`crates/svrngs-core/src/app_tests.rs` `an_acts_signature_is_no_apps`, and
its plant (no domain) — not this run. `/v1/rail/log` carries no signature
(`log_op_fields`: actor, id, person, seq, ts_unix, voided), so the harness
reads the act from Ana's journal, as onair-red wrote it.

The sound is the red run's: 40 s generated here by ffmpeg's `aevalsrc`,
nobody's rights, heard by nobody.

## scripts/check.sh at the close

Once, at 1d4f31a, exit 0 in 12 min 56 s, every step passed: principles
pinned, lean and RUST_ARCH, app nouns gate, app nouns, rustfmt, clippy,
tests, canaries, ceiling canary, web rustfmt and clippy, browser, joining
in a browser, conformance canary, app laws; WebKit's seeks a rate, each
passed. `scripts/lean_check.py`: crates/svrngs-core 3149 / 3150, the
repository 16014 / 16015.
