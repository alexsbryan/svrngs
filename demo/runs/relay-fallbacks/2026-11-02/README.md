# A group's relays in tiers, its relays stopped and started: this host, 2026-11-02

The question (PREDICTIONS.md, `d3e0a31b`): a group names two relays of its
own, as two laptops' `svrngs relay --at` would be on a local network, and
the build's, n0's, sits beside them. When one of the group's relays stops,
does the group keep working through the other? Does a computer still reach
a member that has moved? Does it go to n0's only when both stop, and come
back when they return? How long does each move take? Then the 2026-11-01
modem-off run again on this build.

One host, one run of each final take, so not a rate (AGENTS.md 7). Nobody
acts in it, so no people bar is judged.

## How it ran

```
cargo build --release -p svrngs          # main, clean tree, CARGO_INCREMENTAL=0
BIN=target/release/svrngs W=<scratch> python3 demo/runs/relay-fallbacks/2026-11-02/run.py
BIN=target/release/svrngs W=<scratch> python3 demo/runs/relay-fallbacks/2026-11-02/modem-off/run.py
```

The final takes ran at `98e3712e`, whose code is `f15002be`'s, from a clean
tree (`data/summary.json` and `modem-off/data/summary.json`, `start`).
- **Take 4** (`data/`) ran 02:00:36–02:04:50 UTC. The 1-minute load was 19.1
  at its start: another session's work ran on this host, so its times are
  slower than an idle host's.
- **The modem-off rerun** (`modem-off/data/`) ran 02:04:50–02:05:57 UTC, load 6.1.

The earlier takes are kept. Each found something fixed before the next:

| take | build | data | what it found | fixed in |
|---|---|---|---|---|
| 1 | `d3e0a31b` | `take-1/data/` | A joiner took a build's relay from its link into the group's own tier: n0's `usw1-1` stayed in Ana's endpoint at tier `Own`. The driver also missed the home trace. | `eaca0b3c`, `c0487881` |
| 2 | `c0487881` | `take-2/data/` | A joiner asked only the link's one own relay for a minute, so A's stop moved Ana and Alex to n0's though B answered ("own=1", "from=Own to=Unanswered" at 01:44:55 and :59). | `a40ab3ea` |
| 3 | `453ccc38` | `take-3/data/` | Every computer started at n0's for about 3 s, and kept connections opened through n0's relays then. | `f15002be` |
| modem-off 1 | `453ccc38` | `modem-off/take-1/data/` | Joins took 6.1 s against the 5 s bound, a computer in no group waiting out n0's QUIC probes offline. | `f15002be` |

## The instrument

`no-direct.sb` refuses UDP to every one of this host's addresses.
- **The plants:** a send to 127.0.0.1, ::1, 192.168.1.11, 100.104.36.28 and
  fd7a:115c:a1e0::a3a:241c, each `PermissionError(1, 'Operation not
  permitted')`.
- **The controls:** a UDP send to 1.1.1.1 "sent"; the lent app on loopback
  200; n0's `usw1-1/ping` over HTTPS 200.

Every probe held (`summary.json`, `instrument`).

What it does **not** cut, found after the run, finding 4: UDP to this
network's **public** address. Once a computer is at n0's tier, n0's QUIC
address discovery tells it that address, and the router sends it back
here. Ana's connection to Sam was "Ip(69.181.177.160:53818) Active" 34.6 s
after both relays stopped (`data/summary.json`, `traces.ana-1.dials_held`).
At tier `Own` no relay serves address discovery, so no computer learned that
address, and only relays carried anything. Each dial line in take 4 before
both relays stopped lists relays alone.

## Against the predictions (take 4)

Times are from each stop or start. Homes and tiers come from each computer's
trace; the lend from the driver's fetch every 0.5 s.

| | predicted | as it ran | verdict |
|---|---|---|---|
| 0 | instrument holds | every plant refused, every control through | passed |
| 1 | link ≤ 15 s; joins ≤ 15 s; lend listed ≤ 60 s and works; every computer `Own`, homed at A or B, no n0 relay in its endpoint | link 3,300 ms; joins 3,299 and 3,299 ms; listed 4 ms after the share, works at once; no computer changed tier (built at `Own`, `f15002be`); homes Sam A, Ana B, Alex A | passed |
| 2 | B (Ana's home) stops: tier stays `Own`; Ana's home moves ≤ 30 s; lend works again ≤ 60 s on both, through the dial reaching Ana at A | no tier changed; Ana homed at A at 17,948 ms; the lend worked on Sam's at the first fetch (4 ms), and on Alex's at 4,511 ms after one fetch timed out | passed; whether it was the moved Ana reached: could-not-judge (below) |
| 3 | B back: no home returns within 90 s | Ana stayed at A; Sam, who had never been at B, moved there at 47,169 ms; the lend worked | failed: one home moved to the restarted relay |
| 4 | both stop: `Unanswered` ≤ 13 s; home at n0's ≤ 30 s; lend works ≤ 90 s | `Unanswered` at 1,256 / 4,547 / 7,856 ms (Sam, Ana, Alex); homed at n0's `use1-1` at 2,559 / 5,997 / 14,405 ms; the lend worked again on Alex's at 22,546 ms and on Sam's at 78,100 ms | passed, Sam's lend at 78 s of 90 |
| 5 | both back: `Own` ≤ 13 s, n0's taken away; homed at A or B ≤ 30 s; lend works | `Own` at 5,200 / 8,494 / 1,801 ms; homed at a group relay at 8,203 / 11,498 / 4,803 ms; the lend worked on both | passed |

**A dial reaching a member that moved.** Take 3, at `453ccc38`, shows it.
A was stopped at 01:52:13.9 UTC, and Alex was homed at B from 01:52:42.9.
Sam's dial of Alex at 01:52:43.7 held "Relay(http://127.0.0.1:63938/)
Active", which is B (`take-3/data/summary.json`, `traces.sam-1.dials_held`,
peer `9a1b6a214e`). With no direct path at tier `Own`, Sam reached Alex
where Alex had moved: **passed**.

In take 4's step 2, the lend worked again before Ana's home moved. So
another relay she was connected to carried it, and the trace does not say
which: could-not-judge.

## Take 3's numbers, the same steps at `453ccc38`

Load was 38.6. Every computer there had started at n0's, and its kept
connections had been opened through n0's.
- **A stopped:** the lend worked again at 4,511–4,513 ms on both computers.
  Homes moved at 17,558 / 27,841 / 29,008 ms.
- **A back:** Ana returned at 51,685 ms.
- **Both stopped:** the lend worked at the first fetch, 8 ms, through
  connections already open to n0's `usw1-1` ("Relay(https://usw1-1…)
  Active", `traces.ana-1.dials_held`, 01:52:37). `Unanswered` at
  2,858–9,509 ms; homed at n0's at 4,127–10,737 ms.
- **Both back:** `Own` at 1,359–8,009 ms; homed at a group relay at
  4,361–11,011 ms.

## The modem-off rerun, against the 2026-11-01 predictions

Every bound that run held still holds. The numbers below are from
`modem-off/data/summary.json`, with the 2026-11-01 figures in brackets:
- link 3,109 ms (3,109); joins 3,106 and 3,107 ms (3,118, 3,089);
- 1 MB file 207 and 210 ms;
- lend listed 3–4 ms and fetched 6–7 ms after the share;
- restart API 105–111 ms; first rounds reaching both 104–1,090 ms;
- Dee in 3,125 ms, both lends and both files on her disk within 33 ms;
- every process exited within 4 ms;
- `off_host` empty.

The build's relay is noise:
- pkarr failed every publish (2–8 per computer).
- Address lookups failed in 2,410–3,599 ms.
- No dial was a bare key's.
- QUIC probes timed out 4 times across the first starts, against 20 at
  `453ccc38`'s take 1, since a computer starting with `new --relay` or a link
  no longer probes n0's relays first.

Prediction 6: **passed**.

## What it found

1. **A dial carries the build's tier, and so a connection can ride it.**
   Every dial lists n0's four relays beside the group's (`Contact::named`),
   as §7 asks. In take 3, connections opened through n0's stayed there
   ("usw1-1 Active") while the group's own relays answered, and the lend
   never visibly stopped when both died. §7's trade says a group whose own
   relays answer keeps the build's relay off its path "apart from the check".
   That holds only for connections opened at tier `Own`. Take 4, where
   every computer started at `Own`, held to it.
2. **A move to the fallback costs the lend 22–78 s here**, though tiers move
   in 1–8 s and homes in 3–14 s. Sam's fetches each timed out at 4 s, one
   at 12 s, until 78.1 s (`data/summary.json`, `both_stopped.lend.sam`).
   Each of his dials of Ana listed every relay "Inactive", 4.5 s apart, until
   36.1 s (`traces.sam-1.dials_held`). Why Ana was not reached through n0's
   sooner is not in the trace: could-not-judge. Load was 19.
3. **Homes move to a restarted relay sometimes.** One of three did, in
   take 4 (47 s) and in take 3 (52 s). iroh moves home only to a relay a
   third faster (net_report.rs:810-816), and two relays on one host are
   near alike.
4. **The instrument's hole is the public address**, as above. It matters
   only at the build's tier.
5. **No computer was ever at tier `Unowned` here**: the driver's computers
   hold one group. The home page's line for it is checked by
   `web/tests/fallback.test.mjs` alone.

## What it leaves out, each said

- **Real laptops and a radio**: every relay and computer was on one host.
- **A relay with QUIC address discovery** (`svrngs relay <name>`): never
  ran.
- **A phone**: it binds one relay of its link. Not in this slice.
- **The 10 s ask interval and 3 s answer timeout tried at other values**:
  one setting, `tiers::ASKED_EVERY` and `ANSWERS_WITHIN`.
