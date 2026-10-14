# events-asleep — what stays the operator's

## 1. A phone holding a dial to a computer that sleeps waits 30 s

**What the run showed.** With Sam's computer frozen (SIGSTOP, which sends
nothing, as a lid closing sends nothing), Cy's phone was told it did not
answer after 32,228–32,288 ms, where Bo's computer was told after
2,005–2,008 ms (`README.md` here, the capped take, C3). The phone's worker
keeps one dial to each lender (`web/lend.js` `dialFor`); a request on it
waits until iroh gives the connection up, 30 s, and only then dials again,
which the cap now bounds at 2 s. A computer dials anew for each request,
so it is told within the cap.

**What svrngs could do.** Not tried, each a guess:
- Give the page's endpoint a shorter idle timeout with a keep-alive under
  it, so an open dial to a computer that went silent is given up within
  about the cap: a few lines where the page binds its endpoint
  (`web/src/lib.rs` `bind`). The cost: a keep-alive through the relay for
  every open dial while the phone's page is open.
- Or have the worker bound the wait for an answer's head on a dial that
  has carried nothing for a while, and dial again: about 10–15 lines in
  `web/lend.js`. The cost: a slow app's first byte read as asleep.
- **The operator's call**: whether the phone's wait is worth either cost.

## 2. Cellular, a second host, a release build

The cap is 2 s from this host's dials (`dial.rs`'s doc). A phone on
cellular, a relay farther than n0's `usw1-1`, and a release build were
not run: could-not-judge. The capped take used half the cap once, on a
computer just started (`README.md`, "What the capped take found", 1).
Whether 2 s holds on cellular is the operator's (CHARTER.md).
