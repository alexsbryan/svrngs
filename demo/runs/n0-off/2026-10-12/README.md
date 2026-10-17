# n0 off, on one host — 2026-10-12

Row relay-n0-off (b) (`ralph/next/relay/STATE.md`): a group of three
computers and a phone's page on one Mac with n0 out of the path, to see what
breaks before relay-in-record fixes it. Predictions first, in
`PREDICTIONS.md` (`9322085`); the driver committed before it ran
(`ff86246`). One host, one run: not a rate (AGENTS.md 7).

## How it ran

```
W=/tmp/n0-off-2026-10-12 python3 demo/runs/n0-off/2026-10-12/run.py   # data/, 545 s from the joins to the end
W=/tmp/n0-off-2026-10-12 python3 demo/runs/n0-off/2026-10-12/read.py  # data/end.json, data/read.json
```

The build is `ff86246`'s (`data/events.jsonl`, `start`), 12
cores, the 1-minute load 20.97 at the start and 40.32 at the end
(`start`; `summary.json`, `end`). The relay, iroh's on 127.0.0.1, at
`http://127.0.0.1:54504/`; the page at `http://localhost:54506/`
(`served`). Each computer is the command with that relay alone and
`n0_services: false` (`crates/svrngs/tests/n0_off.rs`), a `HOME` each.
`read.py` ran after, over `data/` and the run's scratch directory, for
each computer's record and folder at the end (`end.json`).

## Against the predictions

| | predicted | as it ran | |
|---|---|---|---|
| 1 | the link carries the relay; each `join` in within 10 s, its folder there | Sam's trace: "no lookup, so the link carries the relay relay=http://127.0.0.1:54504/" (`sam.log.gz`); the link printed in 3,348 ms; Ana in in 408 ms, Alex in 235 ms, "You're in house, as …", each folder there (`summary.json`) | held |
| 2 | the round between Ana and Alex fails both ways, every round; with Sam stopped, Ana's photo not on Alex's disk at 150 s | **failed, and worse than predicted.** Ana never dialled Alex: her record never held Alex's admission (`end.json`: Ana's acts lack `49eb1a43`, Alex's). Alex's 11 dials to Ana each ended in svrngs's own transport, before iroh: "iroh: no dialable path in contact (no relay_url / iroh_direct_addrs) and no address lookup — not dialable … peer=54cbcf5b" (`alex.log.gz`; `read.json`, `alex.no_address_to`). With Sam stopped, the second photo was not on Alex's disk at 150 s, nor in 150 s more with Sam running again (`photo_2_sam_stopped`, `photo_2_sam_again`) | failed |
| 3 | Sam reaches Ana and Alex, and they Sam, while a connection lives | **failed; not predicted.** Every dial between computers after the joins ended "no dialable path": Sam's 25 (13 to Ana, 12 to Alex), Ana's 11 (to Sam), Alex's 22 (11 each); every round reached nobody — Sam's 9, Ana's 11, Alex's 9, each `peers_reached=0` (`read.json`, `rounds`). The only exchanges were each joiner's own join: Sam's trace says "the asker stands in the ring" twice for Ana and twice for Alex (`asked_by`), and no more | failed |
| 4 | Sam's home page lists Ana's `film`; Alex's says Ana's computer did not answer | Sam's: "ana's computer did not answer, so what they lend is not listed. alex's computer did not answer, …"; Alex's: "ana's computer did not answer, … Sam's computer did not answer, …" (`summary.json`, `home`; each read 150,956 and 151,297 ms after Ana's share, after step 3's wait, the first load already saying so). Sam's trace: "node: lends — no dialable path to the computer" for each | failed (Sam's) / as predicted (Alex's) |
| 5 | Ana's photo on Sam's disk within 30 s, Alex's within 90 s, Alex's from Sam | on neither within 150 s (`photo_1`); no "folder: fetched, verified" in any trace (`read.json`, `fetched`); Ana's two file acts (`0d06693d`, `75e68e7a`) in her record alone (`end.json`) | failed |
| 6 | Cy's page asks a name, is in within 30 s, lists Ana's `film`, plays it | asked the name at 1,530 ms, in at 2,401 ms, "You’re in house, as Cy" (`phone`): the page dialled Sam through the link's relay, its console naming it ("LAYER parse: ok — relay http://127.0.0.1:54504/"). **No lend listed in 90 s**: "ana’s computer did not answer … alex’s computer did not answer" (`phone.lent_said`), Sam's computer having no path to either. The film was never opened: the film's server answered nothing (`film_answers`: 0) | failed: the film never-ran |
| 7 | Ana's door asks the ring for Cy, Sam answering | never opened, so no ask: Ana's record did not hold Cy's admission either (`end.json`: `56a6c52b` in Sam's alone) | never-ran |
| 8 | no process opens an address of n0's; no trace names `iroh.link` or pkarr | the relay's and each computer's sockets, 61 distinct ones kept (`lsof.jsonl`): remote addresses 127.0.0.1 and, from each computer once, 192.168.1.1:5000 — this network's router, asked by iroh's port mapper (`iroh-1.0.2/src/socket.rs:710`), not n0's; none of the 16 addresses of n0's 8 hosts (`read.json`, `lsof`). No line of 9,166 traced names `iroh.link`, pkarr or `relay.n0` (`n0_count`: 0 each). **The phone's browser: could-not-judge by lsof** — no socket of it was sampled, WebKit's network process on macOS not being a child of the driver's process; its console names only the link's relay | held for the computers and the relay; the phone could-not-judge |

## What breaks with n0 off, and the dial each names

The prediction was iroh's limit: a bare key dialled from paths iroh holds
for 60 s after a connection (`PREDICTIONS.md`). The run stopped before
iroh. svrngs dials a member by its key alone (`Contact::by_key`,
`crates/svrngs-node/src/iroh.rs:96`), and its transport refuses a contact
with no relay, no address and no lookup before asking iroh
(`IrohTransport::bridge_for`, `iroh.rs:229-234`; the trace at `:283-289`).
So with n0 off:

1. **No computer dials another, ever** — the round, the lends' list, the
   folder's fetch: every one ends "no dialable path", by peer, in the
   dialler's trace (`read.json`, `no_address_first`). A computer's record
   holds what it had when it joined and its own acts, nothing after
   (`end.json`: Sam 4 acts, Ana 4, Alex 3, two in common).
2. **Joining by link holds**, the only path that works: the link carries
   its issuer's relay and the joiner dials the issuer through it.
3. **A phone joins and sees nothing lent**: it reaches the issuer through
   the link's relay, as it reaches any key, but the issuer can list
   nothing it cannot dial.

What relay-in-record (b) and (c) must make green, in this harness: each
member's relay known from the group's record and carried in the contact
it dials (step 3's 25, 11 and 22 dials), so that Ana's photo reaches Sam
and Alex (5), the lends are listed on every home page and the phone (4, 6),
Ana's film plays on the phone (6), Ana's door asks the ring for Cy (7),
and with Sam stopped Ana's photo reaches Alex (2).

## Not judged

- The phone's sockets (8): could-not-judge by lsof on this host. A packet
  capture would judge it and needs `sudo`, the operator's.
- 192.168.1.1:5000's protocol is not named in iroh's traces; that it is
  the port mapper is read from where iroh opens a connection to a router
  under `presets::Minimal`, not traced.
- The phone's console logged "WebSocket connection to
  'ws://127.0.0.1:54504/relay' failed: … Socket is not connected" between
  dials and reported online after each (`phone.logs`); DESIGN §9's row on
  a page whose relay socket breaks is that trace's home, and nothing here
  failed by it.
