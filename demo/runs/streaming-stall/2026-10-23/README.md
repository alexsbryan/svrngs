# streaming-stall (a): the 9 MB range that holds still — what the runs found

Row streaming-stall (`ralph/next/stall/STATE.md`), part (a), measured against
`PREDICTIONS.md` (a53774f6, before any run). One host, a spread, not a rate
(AGENTS.md 7).

## Battery 1 (`battery.sh`, at 17e71761): `a-summary.txt`

| cell | runs | load (1-min) | webkit range red | chromium range red | webkit range | webkit upload | suite |
|---|---|---|---|---|---|---|---|
| alone-burn24 | 10 | 37.6–132.3 | 0 / 10 | 0 / 10 | 20.6–33.6 s | 24.2–40.6 s | — |
| suite | 3 | 25.3–104.6 | 0 / 3 | 0 / 3 | 13.8–15.0 s | 11.1–12.8 s | 127–135 s |

Every range the door sent "to its end", 8,999,000 B (door.rs `Carriage`).
The six gates it is chasing (models-fresh-1..6) ran the suite in
272–311 s, more than twice these 127–135 s, and red 6 of 6.

Against the predictions:

1. *Alone under burn24 does not make it at will* (at most 2 of 10): **held**,
   0 of 10.
2. *The suite makes it* (at least 2 of 3): **falsified**, 0 of 3. Neither
   cell makes it at will.
3. *Where the body stopped*: **could not judge**, no red run.
4. *Which side holds the bytes*: **could not judge**, no red run.
5. *The upload's 36–40 s is a loaded host's*: **held in part**. No green
   upload over 45 s, but alone under burn24 webkit's reached 40.6 s (run 1),
   past arch-stream's 18–31 s: the gates' 36–40 s is inside what a loaded
   host does to a green run, not a second stall.

What the suite here did not have that the gates had: the gate's own command,
`cargo test --quiet --workspace --test browser` (scripts/check.sh:53), where
`run.sh`'s suite cell ran `-p svrngs-node` (the workspace's features unify
differently, so the binary can differ), and a load that held the suite to
272–311 s. Battery 2 plants both.
