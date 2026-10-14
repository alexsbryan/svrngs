# svrngs v1ui — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and `docs/principles/ARCH_PRINCIPLES.md`
are the compass; where they and this charter disagree, they win and the
decision says so.

## What the operator decided, 2026-10-14

- **The design.** "Based on the marketing copy -- what would you propose is
  the UI build that hits 80% of the use cases with 20% of the effort?",
  then "Just document this as V1_UI_BUILD". The design is svrn-docs
  `rings/current/V1_UI_BUILD.md`. Its four moves are §3, its order §5, its
  bars §7. It narrows `rings/current/COMMUNITY_APPS.md`, whose §6
  (containment) and §7 (what it asks of svrngs) the rows read.
- **The order.** "v1ui first, then groups". This queue runs before the
  slimmed groups queue (`ralph/next/groups/`: leaving, a key per group on
  computers). Two of the groups rows move into this queue:
  - **groups-join-wait joins 3.3.** The page gives up on the link's
    computer within a measured cap, then reaches another member's.
  - **#138 joins 3.1.** It is the operator's "Allow; tell them apart"
    (`ralph/next/groups/CHARTER.md`): one rule for a group's name, in the
    node, and same-named groups told apart where 3.1 draws them.
- **The budget.** "+650 repo, core unchanged". There is one economizing
  pass first (AGENTS.md "Done"). Then each row raises only as far as it
  needs, in its own commit naming what the pass cut, within +650 over the
  cap the pass leaves. svrngs-core's cap stays where it is, and a row that
  needs it goes to NEEDS_HUMAN.md.
- **The shelf** (3.4): "Money and Events". Both of svrngs's reference apps
  are carried: `apps/expenses/` and `apps/events/`.
- **A group's colour**: "Derived from the group". It comes from the
  group's id, so different groups look different, with no setting and
  nothing to choose.
- **The house voice**: "Can we make sure the copy reflects the house voice?"
  (svrn-docs `VOICE.md`). New copy follows it (fence 9), and v1ui-voice
  rewrites what came before.
- **The look is merged** (`efadec2`, from the branch `look`): every page
  wears `crates/svrngs-node/src/home/look.css`, by DESIGN §1's rule that
  nothing imitates anything else.

## Why

The copy's selfish hooks have to be true for a lazy person on a phone
(V1_UI_BUILD §2):

- who is around;
- whatever runs on your laptop, lent without a terminal;
- the phone reaching someone awake;
- the gas bill split in one tap.

The bar is `ONE_PAGER.md`'s N+1: "Four of five strangers settle an expense
and open a title in the library within three minutes of joining the chat,
with zero accounts and zero key prompts."

## Decide

- Row order and splitting, within V1_UI_BUILD §5's order (3.3 → 3.1 → 3.2
  → 3.4).
- **3.1's layout.** It lands on "Draft 2 · native", screens 1 and 2, from
  the operator's canvas, as V1_UI_BUILD §6 describes it, minus what §4
  defers. The phone's group page is in the same row, after the computer's.
- **3.4's shapes.** The carried path's shapes, by COMMUNITY_APPS §6-§7: a
  lend's source, the route and its stamp, the headers.
- The colour's derivation.
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going; a finding that keeps the spirit is a follow-up row, not a
stop.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **the budget**: past the ceiling, a raise without a pass first, or any
  svrngs-core line;
- **anything V1_UI_BUILD §4 defers**: strangers' app links, *read by*, the
  closed window, versions, `svrngs dev`, the scan, ⌘K. A stranger's code
  reaches nobody in this queue;
- **removing someone from the page**, which stays "Done in a terminal"
  (V1_UI_BUILD, "Not decided");
- **people's own judgement**: the N+1 bar run with real people, a phone on
  cellular, a second computer;
- **dependencies and tools**: a new crate, a dependency on a commonwealth-ai
  or sovereign crate, docker, OrbStack, or a tool that is not open
  software;
- **outward-facing steps**: publishing, `sudo`, a system service, a port
  opened on this host's network;
- **git**: pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
