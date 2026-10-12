# The newcomer's first minute, split — this host, 2026-10-12

Demo C's dry run found a newcomer's home page listing both lends 51-56 s
after the paste, while the line was done at about 5-6 s (its README, "What
it found" 8; `demo/runs/c/dry-2026-10-12/data/summary.json`, `newcomers`).
Row bash-first-minute (a), `ralph/next/bash/STATE.md`: show where it goes,
from debug lines on three computers on this host, one the issuer. Not a
people run and not a rate: one host, one run of each (AGENTS.md 7).

## How it ran

```
cargo build -p svrngs
W=/tmp/fm-w SVRNGS=$PWD/target/debug/svrngs python3 demo/runs/first-minute/2026-10-12/run.py base
```

Three svrngs processes, each a `HOME` and data dir of its own, debug
traces on (`run.py`, `LOG`): Sam `new trip --as Sam`, the issuer; Ana
`join <Sam's link> --as ana`; each lends one port (`share <port> --kind
…`). Once Sam's and Ana's `/v1/lends` each list both lends, Alex's `join
<the same link> --as alex` starts — the clock's zero — and Alex's
`/v1/lends` is asked every 250 ms until it lists both. The round's period
is its 60 s (`ring_sync.rs`, `DEFAULT_RING_SYNC_INTERVAL`). The 1-minute
load was 20.3 on 12 cores at the start (`data/base/events.jsonl`): other
work ran on this host.

## Where the minute goes (`data/base/`)

Alex's join started at 15:06:48.235 UTC.

| at | what | where |
|---|---|---|
| +0.674 s | Sam answers Alex's ask of the link | `sam.log`, "redeem: asked" |
| +0.713 s | Sam writes Alex's `Admit` | `sam.log`, "redeem: admitted — the Admit is written" |
| +0.717 s | Sam admits Alex's computer on the members' protocol; Alex pulls 3 acts from Sam, the `Admit` among them | `sam.log`, "node: admitted — the dialer stands"; `alex.log`, "sync_with — one exchange", `pulled=3` |
| +0.804 s | Alex's first look: Sam's lend listed, `unanswered: ["ana"]` | `looks.json`, the first |
| +1.152 s | Ana refuses Alex's computer, closing the connection: "refused — the dialer stands in no group held here" — Alex's ask of what Ana lends | `ana.log` |
| +1.2 to +56.5 s | the same, 100 times, one for each of Alex's asks; every one read on Alex's side as "the computer did not answer" (`error sending request`), never as "refused the ask" (0 of those lines) | `ana.log`, `alex.log` |
| +56.587 s | Sam's round, at its interval (Sam's previous round at 15:06:44.8, when it started), pushes one act — the `Admit`, which Alex already held — to Ana | `sam.log`, "ring sync: round … ops_pushed=1" |
| +56.986 s | Ana admits Alex's computer | `ana.log`, "node: admitted — the dialer stands" |
| +57.04 s | Alex's computer lists Ana's lend | `alex.log`, "lends — the computer answered"; `looks.json`, the last, at 56,831 ms |

Ana's own round ran once, at its start (15:06:45-46), and was next due at
15:07:46; Alex's round ran at its start, before it held the group, and
was next due at 15:07:48. Nothing else ran a round in the window.

## What it confirms, and what it refutes

The reading the row gave, line by line:

- **Confirmed: a computer that has not synced the `Admit` refuses the
  newcomer.** Ana refused every one of Alex's asks for 56 s.
- **Refuted in part: where it refuses.** Not in `ask_lends`'s non-success
  arm (`crates/svrngs-node/src/node.rs`, "the computer refused the ask"),
  which never fired: Ana's accept loop closes the connection
  (`admit_dialer`, "refused — the dialer stands in no group held here",
  then the transport's "dialer not admitted — closing connection"), so
  Alex's computer reads the refusal as no answer. That is finding 6's
  cause too: the home page's "did not answer" was said of a computer that
  answered by closing the door.
- **Confirmed: nothing starts a round when the `Admit` is written.** The
  `Admit` was on Sam's disk at +0.713 s and went to Ana at Sam's next
  interval, 55.9 s later. The newcomer's wait is the issuer's round period,
  less however far into it the join fell.
- **Not the newcomer's round.** Alex's round never dialled Ana in the
  window; what Ana refused were Alex's asks of what it lends.

So the minute is: the join, under 1 s; then waiting on the issuer's next
round, about 56 s here, up to 60 s; then under 0.1 s to list.

## After (b): the redeem route wakes the round (`data/woken/`)

The same run, `run.py woken`, on the commit that makes the redeem route
wake the round once it writes an `Admit` (`crates/svrngs-node/src/redeem.rs`,
`redeem_route`). The 1-minute load was 40.0 at its start
(`data/woken/events.jsonl`). Alex's join started at 15:15:14.356 UTC.

| at | what | where |
|---|---|---|
| +0.625 s | Sam writes Alex's `Admit` and wakes the round | `sam.log`, "redeem: the round woken" |
| +0.643 s | the round, 17 ms, pushes the `Admit` to Ana | `sam.log`, "ring sync: round … ops_pushed=1 round_ms=17" |
| +1.010 s | Ana admits Alex's computer, the first time it dials | `ana.log`; no refusal of Alex's key there |
| +1.053 s | Alex's first look ends listing both lends, `unanswered: []` | `alex.log`; `looks.json`, its one look, begun at 648 ms |

Ana refused Alex's computer 0 times, against 100 before.
