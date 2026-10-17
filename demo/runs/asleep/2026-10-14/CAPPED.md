# events-asleep (b) — the capped take: predictions, before its data

Written at 883fca8, which gives every dial up after `dial::DIAL_WITHIN`,
2 s, before this take has run. The same driver, the same setup and the
same rounds as `PREDICTIONS.md`, at this commit, its data in
`data/capped-working/` and `data/capped-asleep/`.

Its asleep rounds are events-run's step 10b again in the shape that
matters to the dial (`demo/runs/events/2026-10-14/README.md`, 10b): Sam's
computer stopped by Ctrl-C, Bo's computer and Cy's phone asking at once
for the lend it lends. Bo asks as a program does, not from Chromium, and
the lend is a probe, not the events app: neither is on the dial's path.

1. **A working dial is never said away.** Every one of Bo's 60 requests
   answers 200 and every one of Cy's 12 opens shows the probe, as in
   `PREDICTIONS.md` 1 and 2. Falsified by any `away`.
2. **Stopped: told within the cap.** Bo is answered 502 `away` within
   3 s, the cap and a second, each round; his log says the dial was not
   answered in time, at debug. Cy's page says the computer did not answer
   within 3 s too: its open dial to Sam's last run fails at once, and its
   new dial gives up at 2 s. Falsified by either past 3 s.
3. **Frozen: Bo within the cap, Cy not.** Bo within 3 s, as 2. Cy about
   **32 s**: the worker's open dial gets no goodbye and is given up only
   after iroh's 30 s, which the cap does not reach, then its new dial
   gives up at 2 s. Falsified by Cy within 3 s, or past 35 s.

One host, one take: not a rate (AGENTS.md 7).
