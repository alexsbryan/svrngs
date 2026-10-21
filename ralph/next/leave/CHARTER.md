# svrngs leave — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided, 2026-10-21

The operator: "Let's just fix #211 -- let's do the proper due diligence and
design first". The survey is `demo/runs/leave/2026-10-21/SURVEY.md`
(503b891c). Asked, the operator chose:
- **"A alone."** The cut is told with what it covers: the tell first asks
  each computer's digest, then sends the cut with the signer's acts before
  it, and the hand-ons, that the computer lacks. Nobody has left a real
  group yet, so A+ (healing copies already split) is not built.
- **"Yes"**: the leaver's own earlier acts travel with the leave, and
  nobody else's. DESIGN §6's "the leave alone is sent" is rewritten.
- **"One rule"**: a keeper's at-once tell to a removed computer uses the same
  selection that `told_cut` already makes for one that was off.
- **"Pass first, then raise"**: an economizing pass, then at most **+25**
  repository lines over the cap it leaves. svrngs-core is unchanged.

## Why

DESIGN §6 says a copy holds an act only with its parent, so copies converge
by union (`docs/DESIGN.md:944-945`). The leave's tell is the one path that
breaks it (`crates/svrngs-node/src/removal.rs:249-253`). Copies that split
over who left stop counting every earlier act of the leaver's, and never
heal.

## Decide

- The shared function's name and home, and its trace.
- How the tell repeats until the computer's digest covers the cut, so a
  chain past the 4 MB budget is never marked told.
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08). Ask what a bar guards and
judge by that.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **the budget:** past +25 over the pass's cap, any svrngs-core line, or a
  raise without the pass first;
- **existing groups:** a change that alters what an existing record means
  or what a computer holds;
- **anything beyond the record:** any act sent that is not the cut's signer's
  before it, or a hand-on;
- **dependencies:** a new crate;
- **outward-facing steps, git:** publishing, pushing, history rewrites,
  `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
