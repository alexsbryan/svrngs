# svrngs bash — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and `docs/principles/ARCH_PRINCIPLES.md`
are the compass; where they and this charter disagree, they win and the
decision says so. The operator, 2026-10-12, after Demo C's dry run
(`demo/runs/c/dry-2026-10-12/`, `9d95533`): "I'd like to take all the
learnings from the demo and address as a bug bash". Each row closes findings
of that run's README ("What it found", 1 to 8); finding 5, the phone's typed
name, is the operator's and no row's.

## The name: what was decided, and why

Finding 1: a second member joined `--as ana` was signed in to Ana's Navidrome
as Ana, its admin (`data/name.json`). Two defects, not one:

1. **An app's account follows a name.** The door tells a lend the person's
   name (`crates/svrngs-node/src/door.rs:115-121`, `x-svrngs-person`), and
   an app that makes accounts from a header (Navidrome, Calibre-Web; DESIGN
   §1) keys the account on it. A name is unique nowhere it would need to be:
   a lend reaches every group its computer holds (`lend.rs:3`), a person's
   name is per group, and the lender's own accounts (Ana's admin, `ana`)
   share the app's namespace. Jellyfin is safe only because its middleware
   names each account `<name>-<16 hex of the key>`
   (`apps/jellyfin/mw/src/viewer.rs:73-81`).
2. **A typed name merges a stranger into a person.** The fold binds key →
   person (`membership.rs:111-115`), and a person is their name, so a second
   key admitted as `ana` *is* Ana for everything keyed by person: removal
   takes every key of the name (`crates/svrngs-core/src/register.rs:126-139`),
   so the impostor cannot be removed without Ana — and one named as the
   keeper cannot be removed at all (`Keeps`, `:129-131`); a hand-on to `ana`
   takes the first computer key of the name (`:160-165`); an app's
   `on_behalf_of` names a person by name (`record.rs:34-38`). Nothing asks
   whether a name is taken: the redeem route parses it (`redeem.rs:279-285`)
   and the `Redeem` command it commits checks the link and the key, never
   the name (`register.rs:44-70`). Read from the code; row bash-person
   plants each as a test.

The rule both answer to, which canon reached for its own ledger in signed
acts (`~/dev/canon` `a6960fc`, `docs/SIGNING_DESIGN.md`): **a key is who
someone is; a name is a claim, and a claim on a name someone already holds
counts only with that someone's key.** NORTH_STAR 2's "the person asking,
verified" — only a key is verified. So:

- **An app keys an account on the key, never on a name** (defect 1): the door
  sets `x-svrngs-user`, a name for an account that cannot collide — the
  middleware's rule, moved to the door so it is decided once (AGENTS 8) —
  and header-sign-in apps are told to use it. Structural: no name rule
  anywhere can reopen it.
- **A name binds once where a stranger gives one** (defect 2): the `Redeem`
  command — decided once, at the one place a link holder's name arrives,
  the issuer's write under its lock (`redeem.rs:242-252`) — refuses a name
  a standing person of the group holds, compared case-blind. Until device links exist there is no way
  to show "that's me", so the claim fails closed (canon: "an act that would
  be carried and not applied is never appended").
- **Not in the fold**, where canon put its rule. Canon's attacker writes the
  timestamps, so canon made a second binding contest the name rather than
  lose to the first. Here a link holder writes no act — the issuer's computer
  does — and a fold rule of "first in the rail's order binds" would let any
  member's computer evict a person by a backdated `Admit` under their name.
  The command stops the stranger; the header makes an app account safe even
  when two keys share a name by race or by a member's hand.

What it costs until device links (`~/dev/svrn-docs/rings/current/ONE_PERSON_MANY_DEVICES.md`,
phase 1, the operator's to queue): a person's second device under the exact
same name is refused, and so is a phone whose key was lost (Safari clears a
site's storage after seven days unused) rejoining as itself; each is told how
to join under another name, and a removed person's name is free again.
That design's §8 question 2 ("leave the homonym merge until device links")
is answered by the dry run: not leaving it. Squatting a name before its
person joins stays possible, as in canon; the keeper can remove the squatter,
which defect 2's fix is what makes possible.

## Decide

Row order, splitting and re-scoping a row that cannot execute; the words a
person reads, by DESIGN §1's two rules; the form of `x-svrngs-user`'s name
part (the characters every header-sign-in app DESIGN §1 names accepts),
said; how the home page writes a lend's limit, inside the protections its
`/choices` route already has; whether the round is nudged by the redeem route,
started by a peer that refuses an unknown dialer, or both, by what the row's
own instrument shows; the smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going; a finding that keeps the spirit is a follow-up row, not a
stop.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- the phone's typed name (finding 5);
- device links, or any change to who a person is in the fold;
- changing a rule in DESIGN §6 or a decision in §10 but those this queue's
  rows name;
- raising the budget past 15,350, or svrngs-core's past 3,000 (say the lines
  needed and why, with the count);
- Demo C's bars (`demo/c.md`, the operator's to set);
- deploying the page host (`scripts/deploy-page.sh`) or anything else
  outward-facing — the rerun serves the page locally;
- docker, OrbStack or any tool that is not open software;
- a new crate, or a dependency on a commonwealth-ai or sovereign crate;
- anything that needs a second computer this host cannot give (say what, and
  the command that would run it);
- pushing, history rewrites, `--no-verify`;
- anything whose evidence you could not reproduce.

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
