# svrngs groups — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and `docs/principles/ARCH_PRINCIPLES.md`
are the compass; where they and this charter disagree, they win and the
decision says so.

## What the operator decided, 2026-10-14

- **Two groups of one name**, `ralph/REVIEW_FINDINGS.md` #138: "Allow; tell
  them apart". The home page refused a group name the person already holds
  (`crates/svrngs-node/src/group.rs`, `Asking::found`), while `svrngs new`
  allowed it (`crates/svrngs/tests/new.rs`,
  `new_twice_with_one_name_makes_two_groups_with_two_ids`).
  - The decision becomes one rule, in the node.
  - A second group with a name already held is allowed, and said plainly:
    "You're already in Trip (Sam's, since 3 Oct). This makes a second group
    called Trip."
  - Wherever two groups share a name, each is told apart by who made it
    and when, as DESIGN §6 tells two people of one name apart.
- **Leaving a group** (svrn-docs `rings/current/LEAVING.md`, its answers
  in §6):
  - "Nobody's yes; keeper hands on first". A person's own `Remove` of
    their own keys is the cut, by them alone. A keeper is asked to hand on
    first, by the `HandOn` that exists. The last person in a group can
    simply leave.
  - "Final for now; per-group keys as a row". Leaving is said plainly as
    final from that computer, and a key per group on computers is a row of
    its own.
  - "Record only". "Ana left on <date>" shows where removals show, in
    their wording, and nothing else is sent.
- **The phone's wait when the link's computer sleeps**, which events-doc-link
  found (`demo/runs/events/2026-10-14-links/NEEDS_HUMAN.md`). It takes the
  rule the operator chose for the lend's dial: "Measure first, then cap",
  by three times the slowest working dial, rounded up (`dial.rs`). And it
  words the refusal by the person, not by a relay's host.
- **The budget**: "+200 repo, +40 core". There is one economizing pass
  first (AGENTS.md "Done"). Then each row raises only as far as it needs,
  in its own commit naming what the pass cut, within +200 over the cap the
  pass leaves and svrngs-core within +40 over its own. The per-group keys
  row stops and asks if its survey says it needs more.

**The V1 cut line** (the operator, 2026-10-15). They said: "And then I think
that's our V1 cut line", of the grapevine demo app. Asked, they chose "arch,
groups, grapevine; V1 on n0" and "arch → groups → grapevine".
- This queue runs after the arch queue and before the grapevine queue
  (`ralph/next/grapevine/`).
- Its first row, groups-v1, rewrites DESIGN §8 for the cut line: the own
  relay comes after V1.

**Moved into the v1ui queue, which runs first** (the operator, 2026-10-14:
"v1ui first, then groups"). #138 is now v1ui-door, and the phone's wait is
now v1ui-reach (`ralph/next/v1ui/`). This queue keeps leaving and a key
per group on computers, and its leave lands on the home page v1ui-door
leaves.

- **Per-group keys, and the leaver's other computers** (the operator,
  2026-10-16).
  - On groups-keys' survey: "Final from this computer, through V1". Way C
    would have cost about 220–330 repository lines against 41 left.
    groups-keys closes as surveyed, and per-group keys on computers go to
    DESIGN §9.
  - On the audit's #179–#181: "Yes, a row within the 41 left". That row is
    groups-leave-others.

## Why

- **Groups mean things** (the operator, 2026-10-13: "Ostrom-like commons").
  So their names are told apart rather than refused.
- **Exit governs what stays** (`rings/current/MOONSHOTS.md`). So leaving
  needs no one's yes.
- **One decider** (AGENTS.md 8). The page and the command decide one rule.
- **A key per device per group** is DESIGN §6's decision
  (`docs/DESIGN.md:1431`). Phones keep one (`web/store.js:22`). A computer
  signs for every group with one key
  (`crates/svrngs/src/identity.rs:17-23`), so someone who leaves or is
  removed can never rejoin from that computer.

## Decide

- Row order, and splitting or re-scoping a row that cannot be done as
  written.
- How same-named groups are told apart where each page shows a group.
- How the self-cut is read in `membership.rs`'s fold, and how a keeper's
  leave is refused until they hand on.
- **Per-group keys.** The shape, by the row's own survey:
  - a signing key derived per group from the device's;
  - an `Admit` that carries both the signing key and the dialled key;
  - or another the survey finds.

  It must keep "one key signs and dials" true where it must (DESIGN §8,
  "Four facts"), and it must leave every existing group's members who
  they are.
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going; a finding that keeps the spirit is a follow-up row, not a
stop.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **the budget**: past the ceilings, or a raise without a pass first;
- **existing groups**: any change that alters an existing group's keys,
  makes an existing member someone new, or needs a migration of what a
  computer holds;
- **a new kind of act** in the record;
- **people's own judgement**: cellular, a second computer, or a run with
  real people;
- **dependencies and tools**: a new crate, a dependency on a commonwealth-ai
  or sovereign crate, docker, OrbStack, or a tool that is not open
  software;
- **outward-facing steps**: publishing, `sudo`, a system service, a port
  opened on this host's network;
- **git**: pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
