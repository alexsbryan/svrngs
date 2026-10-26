# give-app-keep: who may keep an app?

**The question, for the operator** (CHARTER "Awaiting the operator"; GIVE.md
§12 item 5; STATE.md give-app-keep (b)): which computers may keep an app
for a group?

1. **Any member** whose computer has Podman and Give's program. This is the
   default until you answer, and it is what has landed.
2. **A successor the house names**: the group says, as an act of its own,
   whose computers may keep, and the fold refuses the rest.

Nothing waits on the answer. Moving and taking over (give-app-move,
give-app-takeover) build on either one.

## What landed under the default

The fold counts a computer as one that can keep once it writes `ready`
(`apps/give/fold/src/lib.rs`, `step`, the `ready` arm). Give's program
writes it in each group once podman answers rootless (`apps/give/src/keep.rs`,
`Step::Ready`). A start, move or takeover by any other computer is refused
as `cannot_keep`, and so is a move to one (`fold/cases.json`, "a start from a
computer that never said it can keep is refused…" and "a move to a
computer that never said it can keep is refused"). svrngs admits only
members' acts to the record, so "any member" is the default with no
check of give's own.

## What option 2 would take

Option 2 is one more act and one more refusal in the fold, both give's
alone, with no change to svrngs:

- an act such as `keepers {computers | people}`, written by whoever the
  house's rules let decide it. House rules (canon) already decides a
  house's rules, so the act could be a rule there, which the fold reads;
- a refusal, `not_named`, for a start, move or takeover by a computer the
  latest such act leaves out.

The page and the program would both refuse the same way, because they run
the one fold (`apps/give/fold.test.mjs`, `fold/tests/cases.rs`).

## What I would choose, and why

Any member, as landed. The groups this is for are people who know each
other and keep a group for years. Every move and takeover already needs a person's yes
(CHARTER), and a keeper's state becomes a copy rather than being
overwritten. A list of keepers would guard against a member the group
already trusts to write its record. Option 2's trigger could be the
first group that asks, the way the queue's other waits are triggered.
