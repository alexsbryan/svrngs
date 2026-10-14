# svrngs events — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and `docs/principles/ARCH_PRINCIPLES.md`
are the compass; where they and this charter disagree, they win and the
decision says so.

## What the operator decided, 2026-10-13

- "ideally for initial launch we have a good story for how to handle event
  invites in svrngs, so many stories start with an event, who was invited
  and who actually came".
- "probably not something built into svrngs, but like expenses a svrngs
  native app that is able to run independently as selfhosted event
  concept".
- "we want groups to mean things and to build roots over time … We're
  striving to build Ostrom-like commons over time in digital space". So no
  event makes a group. A one-time occasion such as a wedding may be a group
  of its own, a keepsake, and that is a group someone makes, not something
  the app does.
- "I would like to make a very clear delineation between the svrngs
  primitives and the product decisions … someone else should be free to
  take all the same primitives and build their own idea of a great events
  app".
- Of the simplified concept: "I like the elegance of it and would rather
  start here as the primitive event concept". Then: "I want the run after
  onair to be our event primitives + reference app (we can layer on this
  taps concept later)".

The design is svrn-docs `rings/current/EVENTS.md`:

- §1, the rule;
- §2, nothing new in svrngs;
- §3, an event drawn from several groups;
- §4, guests are names;
- §5, the reference app's opinions;
- §9, this queue;
- §7 and §8, what waits.

## Why

**An event is a post in a group, and people answer it**, as three acts in
the group's record, folded as expenses folds its acts
(`apps/expenses/expenses.js:1-30`):

- `event`, written by whoever hosts;
- `coming`, each person's own, optionally "+ Maya";
- `came`, each person's own, or a member's for the guests they brought.

What follows from it:

- **Who was invited** is whoever is in the group.
- **Nothing records "no."**
- **An event drawn from several groups** is posted to each, in one step.
  Each group's record holds only its own people's answers, and the host's
  view folds them together.
- **svrngs gives all of it already** (EVENTS.md §2): the person on each
  request, verified; the record over loopback; who stands in a group; a
  lend; an app opened at a path; a correction.

## Decide

- Row order, and splitting or re-scoping a row that cannot be done as
  written.
- The acts' exact shapes, and the app's routes and page, beginning from
  EVENTS.md §1.
- **One id across groups.** The id the host's computer mints for an event
  posted in several groups, so the host's view joins them (EVENTS.md, "Not
  decided").
- **What "invited" means.** Whoever is in the group when the event is
  posted, or now. Decide by what the run shows, and say why.
- **How the app knows which group a request is for.** The event's path
  names it, and the app checks that the asker stands in it through
  `/v1/rail/people`. Never from the key: a computer signs for every group
  with one key (`crates/svrngs/src/identity.rs:17-23`), and a phone keeps
  one per group (`web/store.js:22`).
- **The store.** The group's record first. A file of the app's own, for
  running self-hosted alone, as a small adapter behind the same fold.
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going; a finding that keeps the spirit is a follow-up row, not a
stop.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **anything svrngs lacks** that the app needs. Write it as svrngs's, with
  its lines estimated, never worked around in the app. svrngs's code is
  unchanged by this queue, and a change to it is a row of its own and the
  operator's budget (AGENTS.md "Done": a raise only after an economizing
  pass);
- **what EVENTS.md keeps for later**: the guest pass, an invitation in the
  host's own words, the door code, a lend for named groups, links that
  last longer, a secret two people share and the question after the party
  (§7, §8);
- **people's own judgement**: a run with real people, a phone on cellular,
  a second computer;
- **outward-facing steps**: publishing, `sudo`, a system service, a port
  opened on this host's network;
- **dependencies and tools**: a new crate, a dependency on a commonwealth-ai
  or sovereign crate, docker, OrbStack, or any tool that is not open
  software;
- **git**: pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
