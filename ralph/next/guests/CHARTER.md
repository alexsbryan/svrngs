# svrngs guests — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided, 2026-10-16

- **The design.** svrn-docs `rings/current/GUESTS.md` (231df13), read
  whole, its §10 decided. The guide: "As much as possible this should look
  like inviting a guest into a shared home. You shouldn't be able to invite
  so many guests that membership becomes diffuse … I just want to make
  blatant abuse fairly inexpressible that's it (and to maintain trust)".
- **The order.** "Ok so the queue next is: loopback, canon, then guest".
  GUESTS's settings are the first rule a house decides in canon
  (CANON_IN_SVRNGS §4, §9 step 5).
- **Guests per host:** "+2 can be a default that can be configurable".
- **A visit's longest:** "The visit ceiling is configurable and starts at
  48hrs", and "ceiling at most 7 days".
- **Welcome:** "Lend by lend". It starts closed.
- **Who changes the settings:** any member, seen by all, until the house
  adopts canon for them.
- **A lend's limit:** "Yes they count the same". A guest is one person
  against it.

## The budget

Not set yet: the operator's. GUESTS §9 guesses 50-80 lines of svrngs-core,
about 40 in the door and about 60 in the pages; CANON_IN_SVRNGS §5 adds
about 2 of core and 20 of page for the link. svrngs-core stands at 3,008 of
3,009 and the repository at 16,561 of 16,562, so every row past the
economizing pass needs a raise.

## Why

The operator: "the main thing svrngs must do is to never violate the trust
of what has been entrusted to it". Bo lent Jellyfin to the group's members;
opening it to Kim's guest without Bo's yes breaks what Bo trusted svrngs
with. A guest carried as their host is the substitution this queue exists
to prevent (`crates/svrngs-node/src/door.rs:67-72`).

## Decide

- Row order and splits, within the order in STATE.md.
- The `Visit` and `GuestRules` acts' exact fields, within GUESTS §5.
- A guest's account name, so it can't collide with a member's.
- The guest's page: the phone's page (`web/`) already keeps a key per group
  (`web/store.js`) and dials a computer by key, which is the likely home.
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08). Ask what a bar guards and
judge by that. Where its letter and its spirit part, rewrite the letter
yourself, saying both in the commit, and keep going.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **the budget:** past the ceiling, or a raise without the pass first;
- **the trust floor:** any widening of a page's policy or a listener;
- **canon's repository:** anything canon-core needs, such as a typed value
  on a commitment, if 0.5.0 has none;
- **existing groups:** a change that alters what a computer holds or makes a
  member someone new;
- **a new office or policy** in svrngs past GUESTS §3 and §5;
- **dependencies and tools:** a new crate in the svrngs workspace;
- **outward-facing steps:** publishing, `sudo`, a system service, a port on
  this host's network;
- **git:** pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
