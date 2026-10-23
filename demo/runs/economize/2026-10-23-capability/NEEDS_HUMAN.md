# capability-economize: one ceiling for the bundle

The row ends here, as it says: an economizing pass, an estimate, and then
the operator's one ceiling for what the pass could not find (CHARTER.md,
"Leave for the operator": "the budget: one ceiling for the bundle, asked
after the economizing pass"; AGENTS.md "Done"). The evidence is
`SURVEY.md` in this directory.

## What the pass did

- Cut 16 counted lines, each its own commit with its behaviour kept and
  its tests unchanged by name: `Key::kid`, which only tests called
  (d2596b56, 3); the door's `Connection` names read once (caf15672, 6);
  the page's IndexedDB opened one way (cda6851f, 3); the page's
  computer key parsed once (93a90fd7, 4).
- Lowered the repository's cap to one past where they land (038f0ff1).
  `python3 scripts/lean_check.py`: **17,634 / 17,635 before, 17,618 /
  17,619 after**; svrngs-core **3,149 / 3,150** throughout, one past
  already.
- Found nothing more: the compiler's other unreachable items, the clone
  report's clusters and the open findings that name a thing said twice
  (#219, #221) are each read and kept, or change behaviour (SURVEY.md §1
  to §4).

So the tree has 1 counted line under its cap, and svrngs-core 1, and the
next row, capability-navigation, cannot land its (b) without a raise.

## The estimate (SURVEY.md §6)

| Row | Repository | svrngs-core |
|---|---:|---:|
| capability-navigation | +5 to +15 | 0 |
| capability-money | 0 (`apps/`) | 0 |
| capability-pass-key | +20 to +80 | 0 |
| capability-open | +55 to +100 | 0 |
| capability-what-acts | +30 to +70 | 0 to +3 |
| capability-authorship | 0 (a design, tests) | 0 |
| capability-signed | +110 to +200 | +55 to +110 |
| capability-keys | +25 to +55 | +5 to +10 |
| capability-release | 0 (docs) | 0 |
| **Total** | **+245 to +520** | **+60 to +123** |

Beside the note's own sizing (svrn-docs
`rings/current/WHAT_CAN_ACT_AS_YOU.md` §10): consent and authorship +215
to +405 / +60 to +123; this pass adds the three fixes it left
unsized (+25 to +95 / 0) and "What acts as you" on the group page as well
as the home page (up to +15). Two ends are open: the pass-key row's,
until it measures the trade-off its row names (a key the page cannot read
against the worker dialling from raw bytes), and the signed row's, until
the operator says yes to the act format.

## The ask

**One ceiling for the whole bundle: +520 repository and +123
svrngs-core over the caps this pass leaves, at most 18,139 and 3,273**,
each row raising only what it lands, to one past where it lands, and a
row past its range stopping for the operator. That is the estimate's top,
so the two open ends sit inside it.

A smaller step, if preferred: +265 repository and +3 svrngs-core, at most
17,884 and 3,153, for the rows before authorship (navigation through
what-acts, +110 to +265 and 0 to +3 by the estimate), with authorship's
ceiling asked again beside its design, when capability-authorship stops
for the operator's yes in any case.
