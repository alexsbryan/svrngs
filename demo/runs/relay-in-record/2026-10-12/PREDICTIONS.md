# relay-in-record, on one host — predictions, 2026-10-12

Row relay-in-record (c) (`ralph/next/relay/STATE.md`): relay-n0-off's run
again, in the same harness — `run.py`, `read.py`, `film.mjs` and `n0.mjs`
copied byte for byte from `demo/runs/n0-off/2026-10-12/` (`cmp`), so its
data lands beside this file and n0-off's stays as it ran — on the build
that carries each computer's relay in the group's record (`b18f187`, (b))
and every dial of a member through it (`42e36ca`, (c)). Each computer is
still `RelayConfig { relay_urls: [one iroh-relay on this host],
n0_services: false }` (`crates/svrngs/tests/n0_off.rs`). Written and
committed before the driver ran. One host, one run: not a rate (AGENTS.md
7). A step that could not finish is could-not-judge, never a pass
(AGENTS.md 6).

## The red, already run

n0-off's README ("Against the predictions", "What breaks with n0 off"):
every dial between computers after the joins ended "no dialable path"
(Sam's 25, Ana's 11, Alex's 22), every round `peers_reached=0`; no lend
listed on any home page or the phone; no photo moved; the film never
opened; Ana's door never asked for Cy.

## Predictions, each a step that was red

1. **Joining by link still holds**: each `join` in within 10 s, its folder
   there.
2. **The rounds reach each other**: no trace of Sam's, Ana's or Alex's
   says "no dialable path" for a member's key after both joiners are in
   and each has said its relay; rounds reach their peers
   (`peers_reached` > 0 in `read.json`'s `rounds`), and each computer's
   record ends holding all three admissions and three `Relay` acts.
3. **The photo**: Ana's first photo on Sam's disk and Alex's within 60 s
   (n0-off's bar for Alex was 90 s, through Sam; Alex now dials Ana too).
4. **The lends**: Sam's and Alex's home pages each list Ana's `film`
   within 90 s of her `share`, neither saying her computer did not answer.
5. **The phone**: Cy is in within 30 s of the name, Ana's `film` listed,
   and the film plays (`currentTime` past 1 s), the page dialling Ana
   through the relay her group names (the console's "group names its
   relay").
6. **Ana's door asks the ring for Cy**, if Cy's first request reaches her
   before her record holds Cy's admission — or her record already holds
   it; either way the film plays.
7. **With Sam stopped**, Ana's second photo on Alex's disk within 150 s:
   Ana and Alex dial each other through the relay the record names.
8. **No n0**: as n0-off's 8 — no address of n0's sampled for the
   computers and the relay, no trace naming `iroh.link`, pkarr or
   `relay.n0`; the phone's browser could-not-judge by lsof, as before.

Would falsify (c): any of 2–7 failing with "no dialable path" for a
standing member's key, or a dial carrying no relay for a computer whose
record holds its `Relay`.
