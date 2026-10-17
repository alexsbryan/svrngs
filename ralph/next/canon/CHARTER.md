# svrngs canon — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided, 2026-10-16

- **The designs.** Read these whole, each with its decided section:
  - svrn-docs `rings/current/WHAT_RUNS_AS_YOU.md` (§8);
  - `rings/current/CANON_IN_SVRNGS.md` (§8);
  - `rings/current/GUESTS.md` (§10). GUESTS is the queue after this one.
- **The order.** "Ok so the queue next is: loopback, canon, then guest".
  The trust floor and pinned carried code come first in this queue: "Yes,
  floor and pin first". canon's code decides a house's rules, so no member
  may quietly replace it.
- **The budget.** "+150 repo, +40 core", over the caps this queue's
  economizing pass leaves.
  - The pass goes first (AGENTS.md "Done").
  - Each row raises only as far as it needs, in its own commit naming what
    the pass cut.
  - `apps/` is not counted.
- **The invariant** (WHAT_RUNS_AS_YOU §1): what runs as you, and what can
  reach you, is what you or your group chose to bring. svrngs never widens
  it silently.
- **A new version of a carried app is each person's yes, by default.** "New
  versions of the carried app I think should follow 'what runs as you is
  yours' by default then yeah canon can allow different configuration."
- **canon is carried, with no lent step.**
  - It is House rules on the shelf.
  - Membership is standing.
  - svrngs shows canon's decisions and doesn't enforce them.
  - It is built through `wasm2js`, so carried pages get no
    `'wasm-unsafe-eval'`.
  - The strategy: "decouple the binary from the data so that the data can
    distribute and the binary runs pure against the data on any machine (so
    we don't have to have an always on server)".

## What the operator decided, 2026-10-17

- **Standing from the members, by canon's own grants.** canon-core 0.3.0
  takes no roster (`demo/runs/canon/2026-10-16/NEEDS_HUMAN.md`). The
  operator chose "A: canon takes the members". Then, asked whether canon's
  own primitives could carry it, "Yes, reuse": canon-core stays as it is,
  with no release. The house's membership is canon grants and withdrawals,
  written in memory by a `house:<group>` actor from svrngs's `spans`, and
  never recorded. svrngs-core stays the one decider of who is a member;
  canon reads its result in canon's own acts. This tree never writes
  canon's repository.
- **Joined and left.** svrngs gives House rules each person's time in the
  house, those who left included, "so past decisions never shift". A raise
  past the ceiling for it is asked of the operator, never taken.

## Why

The operator: "the main thing svrngs must do is to never violate the trust
of what has been entrusted to it". Library rules make blatant abuse
inexpressible and leave governance to the people ("I also don't want to
overburden the library level with governance rules that should be decided
by people living in and among these commons"). canon is where a house
governs itself. svrngs carries it, and never governs.

## Decide

- Row order and splits, within the order above.
- The trust floor's file format, and how the gate reads each page's policy.
- How a carried app's pin and each person's yes are recorded and folded.
- The canon wrapper's shape, JSON in and JSON out, under `apps/canon/build/`.
- The page's layout, within CANON_IN_SVRNGS §3.
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going. A finding that keeps the spirit is a follow-up row, not a
stop.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **the budget:** past the ceilings, or a raise without the pass first;
- **the trust floor:** any widening of a page's policy or a listener, once
  the floor is pinned. That includes `'wasm-unsafe-eval'` for carried pages,
  if `wasm2js` cannot carry canon;
- **canon's repository:** any change canon-core needs, such as an outside
  roster as standing. It is canon's, never written from this tree;
- **existing groups:** a change that alters what a computer holds or makes a
  member someone new. A carried app already brought, pinned at what its
  folder holds when the pin lands, is not one;
- **dependencies and tools:**
  - a new crate in the svrngs workspace, or a tool that is not open software;
  - canon-core at its pinned release, built only by `apps/canon/build.sh`
    and never by the workspace, is allowed;
  - so is binaryen's `wasm2js`, which is open software;
- **outward-facing steps:** publishing, `sudo`, a system service, a port on
  this host's network;
- **git:** pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
