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

## What the operator decided, 2026-10-18, on a lean pass

The first draft (203ac18b) built GUESTS whole. The operator chose the
smallest build that tests, with people, whether a member has a friend over
and whether the house is fine seeing it (STATE.md says the bets):
- **"A link in any chat."** A visit is made the way a person joins. The
  grapevine's two taps wait.
- **"+150 repo, +60 core",** over the caps this queue's economizing pass
  leaves: the small version.
  - Every fact of GUESTS §3 is kept. A lean cut drops features, never what
    was entrusted.
  - The bounds are fixed at the decided two guests per host and 48 hours,
    each one constant in svrngs-core. `GuestRules` and House rules' say
    over it wait.
  - A guest opens only their host's own lends. The welcome waits.
  - `apps/` is not counted.
- **"At the run."** The queue builds it, runs it on one host and stops. The
  words for people wait for a real visit, whose bars the operator sets
  before it.
- **The host chooses how long.** "Let's make that duration configurable,
  but with reasonable default (48hr)", and, asked how long a host may
  choose, "Up to 48 hours". 48 hours is chosen to start, and a host can
  choose less. The house's 48 hours stays the bound.

## What the operator decided, 2026-10-20, of guests-link's raise

guests-link parked over the ceiling
(`demo/runs/guests/2026-10-19-link/NEEDS_HUMAN.md`): its draft measures +118
repository lines and +7 svrngs-core, about +17 still to come, and
guests-pages after it. Asked, the operator chose "Cover both rows": the
guests rows' ceiling is now at most **16,905** for the repository and
**3,076** for svrngs-core, for guests-link with the reopened link kept
(option 1 there) and about 40 for guests-pages. Each raise is still one
past where its commit lands, in its own commit. Past these is
NEEDS_HUMAN.md again.

## Why

The operator: "the main thing svrngs must do is to never violate the trust
of what has been entrusted to it". Bo lent Jellyfin to the group's members;
opening it to Kim's guest without Bo's yes breaks what Bo trusted svrngs
with. A guest carried as their host is the substitution this queue exists
to prevent (`crates/svrngs-node/src/door.rs:67-72`).

## Decide

- Row order and splits, within the order in STATE.md.
- The `Visit` act's exact fields, within GUESTS §5, so `GuestRules` and the
  welcome fit later with no new act.
- A guest's account name, so it can't collide with a member's.
- How the visit link reuses the join link: the token, the redemption, and
  how the issuer knows what it issued.
- The guest's page: the phone's page (`web/`) already keeps a key per group
  (`web/store.js`) and dials a computer by key, which is the likely home.
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08). Ask what a bar guards and
judge by that. Where its letter and its spirit part, rewrite the letter
yourself, saying both in the commit, and keep going.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **the budget:** past 16,905 for the repository or 3,076 for svrngs-core
  (2026-10-20), or a raise without the pass first;
- **a wait built early:** the welcome, `GuestRules`, House rules' say over
  it, the grapevine's path, or the words for people (STATE.md, "Waits");
- **the trust floor:** any widening of a page's policy or a listener;
- **canon's repository:** anything canon-core needs;
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
