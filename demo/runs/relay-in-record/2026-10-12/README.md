# relay-in-record, on one host — 2026-10-12

Row relay-in-record (c) (`ralph/next/relay/STATE.md`): relay-n0-off's run
again, in its own harness — `run.py`, `read.py`, `film.mjs` and `n0.mjs`
copied byte for byte from `demo/runs/n0-off/2026-10-12/` (`32785ec`) — on
the builds that carry each computer's relay in the group's record and
dial every member through it. Predictions first, in `PREDICTIONS.md`
(`32785ec`). Each computer is the command with one iroh relay on
127.0.0.1 and `n0_services: false` (`crates/svrngs/tests/n0_off.rs`).
One host, three takes, each one run: not a rate (AGENTS.md 7).

## How it ran

```
W=/tmp/relay-in-record-2026-10-12[-take2|-take3] python3 demo/runs/relay-in-record/2026-10-12/run.py
W=<the same> python3 demo/runs/relay-in-record/2026-10-12/read.py
```

run.py writes `data/`; take 1's and take 2's were moved to `take-1/` and
`take-2/` after `read.py` ran over each. `data/` is take 3's.

| take | build | what it changed | load, start → end (12 cores) |
|---|---|---|---|
| 1 | `32785ec` | every dial of a member through the relay its group names (`42e36ca`) | 21.0 → 89.7 |
| 2 | `e16d52e` | a joiner waits for its home relay before its join's exchange | 39.6 → 33.5 |
| 3 | `6cd1727` | a push landing its asker's own relay wakes the round | 25.2 → 35.9 |

## Against n0-off's red

Each step n0-off's run failed (`demo/runs/n0-off/2026-10-12/README.md`), by
take; ms from the step's start (`summary.json`).

| step | n0-off | take 1 | take 2 | take 3 |
|---|---|---|---|---|
| joins, ana / alex | 408 / 235 | 219 / 272 | 3,387 / 3,395 | 3,380 / 3,308 |
| Ana's photo on Sam's disk / Alex's (5) | never / never | 60,611 / 61,319 | 363 / 56,937 | **266 / 268** |
| Sam's home page lists ana's film (4) | "did not answer" | listed, 62,279 | listed, 57,640 | **listed, 1,469** |
| Alex's home page lists ana's film (4) | "did not answer" | "ana's computer did not answer" | listed, 58,283 | **listed, 2,641** |
| Cy in / film listed / playing (6) | in / nothing lent / never-ran | 2,388 / 2,390 / 4,632 | 2,248 / 2,254 / 4,568 | **3,231 / 3,241 / 5,502** |
| the film's server answered (6) | 0 | 12 | 11 | 11 |
| Sam stopped: Ana's 2nd photo on Alex's disk (2) | never in 300 s | 515 | 654 | **580** |
| dials "no dialable path" after the joins (3) | Sam 25, Ana 11, Alex 22 | Sam 5, Ana 1, Alex 3 | Sam 2, Ana 1, Alex 0 | **Sam 2, Ana 0, Alex 0** |
| n0 addresses sampled, n0 names traced (8) | none, 0 | none, 0 | none, 0 | **none, 0** |

The home pages are read after step 3's wait, so take 1's and take 2's
times carry the photo's.

Take 3's two "no dialable path" are Sam's, each the round its redemption
woke dialling the newcomer before the newcomer's relay reached it
(`data/sam.log.gz` :157, :367; `read.json` `sam.no_address_to`: ana 1,
alex 1); the newcomer's push of its relay wakes the round again, so each
costs nothing a person sees. Ana's and Alex's rounds reach both peers
(`read.json` `rounds`), each computer's record ends with the same nine
acts and both photos where they were put (`end.json`).

## What each take showed

- **Take 1**: a joiner's join exchange ran before iroh named its home
  relay — "node: no relay reached yet, so the record keeps what it says of
  this computer" at 04:21:51.994, `net_report` naming the preferred relay
  at 04:21:54.920 (`take-1/ana.log.gz`) — so its relay reached the record
  only at its first round, ~60 s on: what (b) named as its falsifier.
- **Take 2**: with the join waiting for its relay, Sam reached Ana at
  once; but Sam's redemption woke its round and told Ana of Alex at
  04:28:40.60, 3 s before Alex's relay reached Sam in his join's second
  exchange, which woke nothing; Ana's photo-woken round then dialled Alex
  with no relay named (`take-2/ana.log.gz` :284), and Alex heard of the
  photo only at his own round.
- **Take 3**: all green.

## Not judged

- **Which relay the phone's lend worker dialled Ana through**: one relay
  serves the host, so the link's and the group's are the same URL, and the
  console the harness keeps is the page's, not the lend origin's worker
  that dials (`web/lend.js`). The page's own dials say their relay
  ("LAYER parse: ok — relay http://127.0.0.1:49696/"). That the page is
  handed each lender's relay is the door test's
  (`a_member_asks_what_the_group_s_people_lend_each_with_its_lender_s_key`).
  Two relays on the host would judge it.
- **The phone's sockets**: could-not-judge by lsof, as in n0-off.
- **Sam's stop at take 3's end**: SIGTERM after SIGCONT went unanswered
  for 20 s (`events.jsonl`, `stop`); after every step, and not seen in
  n0-off or takes 1 and 2. One run: not judged.
- **The joins' 3.3 s**: the wait for a home relay iroh's net report
  takes (`ONLINE_WAIT`, up to 10 s); on this host, against a relay on
  127.0.0.1. Not measured against a relay across a network.
