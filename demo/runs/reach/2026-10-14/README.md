# A phone's page reaching the computer that made its link — 2026-10-14

Row v1ui-reach (`ralph/next/v1ui/STATE.md`), part (a): measure the members'
dial a phone's page makes to its link's computer (`web/app.js:314`,
`enter`, `connect`), then cap it by the rule written before the data. The
setup and predictions are `PREDICTIONS.md`, committed (e246413) before the
driver (e4cb72b) and the data. Every person is a program; one host, one
take: a spread on this host, not a rate (AGENTS.md 7).

## How it ran

```
W=/tmp/reach-run python3 demo/runs/reach/2026-10-14/run.py < /dev/null
```

- At e4cb72b, nothing dirty (`data/events.jsonl`, `start`); the 1-minute
  load 19.8 at the start and 16.4 at the end on 12 cores (`start`;
  `summary.json`, `end_load`). A debug build; every computer and phone on
  this host.
- **loopback**: iroh's relay at `http://127.0.0.1:64477/`; **n0**: the link
  named `https://usw1-1.relay.n0.iroh.link/`, the one Sam's computer
  reached (`summary.json`, `paths.*.relay`). All four of n0's relays
  answered (`summary.json`, `n0`).
- Each open's row is in `data/events.jsonl` (`"event": "open"`): the whole
  open, the dial and the bind, each in ms, and the status the page showed.
  An asleep open's last page lines are kept beside it.

## Against the predictions

| | predicted | as it ran | verdict |
|---|---|---|---|
| P0 | an n0 relay answers | all four | passed |
| P1 | warm, loopback: 20 in; each dial ≤ 300 ms, median < 100 | 20 of 20 in; dial 9–24 ms, median 15.5; the open 318–340 ms | passed |
| P2 | warm, n0: 20 in; each dial ≤ 1 s, median < 300 | 20 of 20 in; dial 79–184 ms, median 84; the open 826–1,342 ms | passed |
| P3 | cold, both: 5 in each; each dial ≤ 3.5 s | 5 of 5 in each. loopback: dial **3,047–3,057 ms**, every one; n0: 73–187 ms | passed |
| P4 | asleep, both: told 30 s ± 2 s, naming the relay's host | loopback 30,649 and 30,575 ms; n0 30,571 and 32,067 ms; each dial given up by iroh at 30,007–30,009 ms ("connect() failed: timed out"); the words "Your group’s computer did not answer through 127.0.0.1:64477." and "… through usw1-1.relay.n0.iroh.link." | **failed**, by one open: n0's second told at 32,067 ms, 67 ms past the 32 s P4 allowed. Its dial was given up at 30,007 ms like the rest; the two seconds more were the open around it, its bind 505 ms among them. The other three within |
| P5 | the slowest working dial under 1,334 ms, so the cap is `DIAL_WITHIN` | **3,057 ms** (loopback, cold) | **failed** |

## The cap, by the rule

The slowest working dial, 3,057 ms, times 3 is 9,171 ms, rounded up to a
whole second **10 s**. That is not within `dial::DIAL_WITHIN` (4 s), so the
constant does not fit and the cap is one of its own. It is a third of the
asleep wait (30,007–30,009 ms) and under it, so the rule gives a cap, and
part (b) does not stop: the predictions' stop clause reads, as the rule it
copies (`demo/runs/asleep/2026-10-14/PREDICTIONS.md`), whether the slowest
working dial times 3 reaches the asleep wait — 9,171 ms does not reach
30,007 ms.

## What it shows

- **A phone reaches a computer that is on in well under a second**: 9–24 ms
  through a relay on this host, 73–187 ms through n0's, the endpoint's bind
  before it 212–222 ms and 378–627 ms.
- **A computer just woken took 3 s on the loopback path, five times of
  five** (3,047–3,057 ms), and under 0.2 s on n0's. Why is not measured
  here. Its evenness looks like a retry on a fixed schedule — the phone's
  first packets lost while the computer had not yet reached its relay —
  but nothing in this take shows it; a cap below 3.1 s would refuse this
  computer, which answers. It is the same order as the 3.03 s a computer
  just started took to answer a joiner's redeem dial (5c097eb).
- **Today a phone waits 30 s for its link's computer when it sleeps**, on
  either path, and is told the relay's host.

## What this take does not show

- A phone on cellular, a second host, a release build, real people: not
  run, could-not-judge; the operator's (CHARTER.md).
- A frozen computer (SIGSTOP): not run here; the links take measured it at
  30,522 ms (`demo/runs/events/2026-10-14-links/README.md`, P8).
- The redeem dial (`connect_to_join`): not timed; it stays uncapped
  (5c097eb).
