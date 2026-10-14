# events-asleep (a) — how long a dial to a lend's computer takes, 2026-10-14

Row events-asleep (`ralph/next/events/STATE.md`), part (a). The setup and
predictions are `PREDICTIONS.md`, committed (5b3a7da) before the driver
(db355c5) and any data. One host, one run of each part: a spread on this
host, not a rate (AGENTS.md 7). Nothing in the product changed.

## How it ran

```
PART=working W=/tmp/asleep-run python3 demo/runs/asleep/2026-10-14/run.py   # data/working/
PART=asleep  W=/tmp/asleep-run python3 demo/runs/asleep/2026-10-14/run.py   # data/asleep/
```

- **working** at db355c5, nothing dirty (`data/working/events.jsonl`,
  `start`), 14:01:02 to 14:01:41 UTC; the 1-minute load 12.8 at the start
  and 9.4 at the end on 12 cores.
- **asleep** at db355c5, its only dirt `working`'s data, 14:01:44 to
  14:06:37 UTC; the load 9.4 at the start and 2.6 at the end.
- A debug build, n0's relays, every computer and phone on this host.
- Two dry takes before these, against the driver uncommitted, found one
  fault of the driver's, named in db355c5's body: WebKit answered the
  phone's open from its cache.

## Against the predictions

| | predicted | as it ran | verdict |
|---|---|---|---|
| 0 | some n0 relay answers `/ping` | all four answered (`summary.json`, `n0`) | passed |
| 1 | Bo's 60 requests 200; cold each within 2 s, median under 500 ms; warm within 250 ms; a direct path | all 60 answered 200 with the probe. Cold **9–347 ms**, median 13 ms (rounds 11 and 12: 225 and 347 ms). Warm **3–42 ms**, median 4 ms. Sam's door saw 59 of Bo's requests on a direct path at 192.168.1.10, this host's own address, not 127.0.0.1; 2, rounds 11's and 12's cold ones, with no path selected yet (`relayed=None`, `data/working/sam.log` 14:01:37.25 and 14:01:39.72) | passed |
| 2 | Cy's 12 opens shown; the dial each within 3 s, median under 1 s; the open within 10 s | all 12 shown. The dial, from the worker's trace, **66–103 ms**, median 74.5 ms, one each round, each established. The open 187–272 ms. Sam's door saw every one of Cy's requests relayed, RTT 70–127 ms (`sam.log`) | passed |
| 3 | stopped: Bo and Cy told 502 `away` at 30 s ± 1 s | Bo **30,005–30,008 ms**, 502, `away`, his log "iroh bridge: dial failed … error=timed out" (`data/asleep/bo.log`). Cy **30,048–30,058 ms**, 502, `away`, "The computer lending this did not answer. Try again later."; the worker's dial "connect() failed: timed out" after 30,007–30,012 ms | passed |
| 4 | frozen: Bo as 3; Cy about 60 s, never under 35 s | Bo **30,005–30,007 ms**, as 3. Cy **60,074–60,220 ms**: the request on the worker's open dial waited 30 s, then its dial again 30,004–30,011 ms (`data/asleep/summary.json`, each round's `dials`) | passed |

## What it shows

- **The 30 s is iroh's, on both sides.** The computer's bridge
  (`crates/svrngs-node/src/guest.rs`, `endpoint.connect`) and the phone's
  page (`web/src/lib.rs` `reach`, `bound.ep.connect`) both wait out a dial
  for 30 s before it is "timed out". No svrngs constant is on that path.
- **A working dial is far under it.** On this host the slowest was Bo's
  347 ms, a cold dial before iroh had chosen a path. The phone's, through
  n0's relay, 66–103 ms.
- **A phone holding a dial to a computer that goes without a goodbye
  waits twice.** Its open dial is given up after 30 s, then it dials again
  (4). Capping the dial shortens the second wait, not the first.

## The cap, by the rule committed before the data

The rule (`PREDICTIONS.md`, "The cap"): the slowest working dial measured
in 1 and 2, times 3, rounded up to a whole second.

- The slowest working dial: **347 ms**, Bo's cold dial in round 12.
- Times 3: 1,041 ms. Rounded up to a whole second: **2 s**.
- Neither stop the rule names holds: no working dial failed, and 1,041 ms
  is far under the 30 s asleep wait.

## What this run does not show

- **A computer's dial through a relay.** Both computers are on this host,
  and iroh carried 59 of Bo's 60 requests direct. The relay's path was
  measured from the phone's page, which dials only through one.
- **A phone on cellular, a relay farther than the one n0 gives this host,
  a second host.** Not run: could-not-judge. Cellular stays the
  operator's (CHARTER.md).
- **A release build.** The debug build's dials are the slower of the two,
  if either.

## The capped take — events-run's step 10b again

At 883fca8, which gives every dial up after `dial::DIAL_WITHIN`, 2 s. Its
predictions are `CAPPED.md`, committed (e18090e) before the take; the
driver as above with `TAKE=capped` (d0cb319). Data in
`data/capped-working/` and `data/capped-asleep/`.

```
TAKE=capped PART=working W=/tmp/asleep-run python3 demo/runs/asleep/2026-10-14/run.py
TAKE=capped PART=asleep  W=/tmp/asleep-run python3 demo/runs/asleep/2026-10-14/run.py
```

- **working** at d0cb319, nothing dirty, 14:21:36 to 14:23:05 UTC, the
  1-minute load 12.0 at the start and 13.6 at the end on 12 cores.
- **asleep** at d0cb319, its only dirt `capped-working`'s data, 14:23:09
  to 14:25:14 UTC, the load 12.7 and 2.9.

| | predicted | as it ran | verdict |
|---|---|---|---|
| C1 | a working dial never said away | all four n0 relays answered. Bo's 60 requests all 200: cold 7–1,013 ms, median 11 ms; warm 2–19 ms, median 6 ms. Cy's 12 opens all shown, 198–275 ms; the dials 69–101 ms, each established. No `away` (`data/capped-working/summary.json`) | passed |
| C2 | stopped: Bo and Cy told within 3 s | Bo **2,008–2,010 ms**, 502 `away`, his log "iroh bridge: the dial was not answered in time, so the computer is taken as asleep … within=2s" at debug (`data/capped-asleep/bo.log`). Cy **2,051–2,058 ms**, 502 `away`, "The computer lending this did not answer. Try again later."; the worker's dial "REFUSED — not answered in 2s: asleep" after 2,005–2,011 ms | passed |
| C3 | frozen: Bo within 3 s; Cy about 32 s, not within 3 s nor past 35 s | Bo **2,005–2,008 ms**. Cy **32,228–32,288 ms**: the request on the open dial given up after iroh's 30 s, then the new dial at 2,007–2,011 ms | passed |

Step 10b, events-run's one failed bar, said within 30 s and measured at
30,019 ms and 30,024 ms: told within the cap here, 2,008–2,010 ms on a
computer and 2,051–2,058 ms on a phone, for a computer stopped as
events-run stopped Sam's. One host, one take: not a rate.

## What the capped take found

1. **A just-started computer's first dial took 1,013 ms**, half the cap.
   Round 3: Sam's svrngs bound its endpoint at 14:22:43.40 and answered
   the record API, Bo asked at 43.71, and Sam admitted Bo's dial at 44.72
   (`data/capped-working/sam.log`, `bo.log`), under a load of 12–13.6 on
   12 cores. The measurement's slowest, 347 ms, was the same kind of dial
   (README above, 1). The cap held it, with a margin of 2 where the rule
   asked 3. A computer waking under a heavier load, or a slower path, could
   come past 2 s and be said away; a second host or a release take would
   say whether the rule's input moves.
2. **A phone with an open dial to a computer that sleeps without a
   goodbye still waits 30 s** (C3). The cap bounds a dial; the worker's
   open connection is given up only by iroh's own timeout. A lid closing
   sends nothing, as SIGSTOP sends nothing, so on a phone this is the
   common case when the phone opened the lend shortly before.
   svrngs's, written in `NEEDS_HUMAN.md` here.
