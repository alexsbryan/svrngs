# canon-spike (d) — standing from the roster, for the operator

**The question:** canon-core 0.3.0 has no way to take the house's members as
its standing, which CANON_IN_SVRNGS §3 and §8 decided. Do you change
canon-core, in canon's repository, so `derive` takes an outside roster? Or
does House rules take standing some other way?

The rest of the row is done. canon-core builds to wasm and through `wasm2js`
to plain JavaScript unchanged. 18 of 18 canon replays are equal across
canon-cli, the wasm and the JavaScript (`MEASURE.md`, `build.log`). The
JavaScript is vendored and pinned (`apps/canon/vendor/`, `e0996d7f`). So the
floor needs nothing: row (d)'s first case does not arise.

## The evidence

All of it is in canon at `49eaf31` (release 0.3.0):

- `derive` takes the acts and nothing else:
  `pub fn derive(acts: &[Act]) -> Canon` (`crates/canon-core/src/fold.rs:621`),
  and `derive_at(acts, now)` (`:924`).
- Who has standing is read from `Canon::grants` alone:
  - `seat` (`crates/canon-core/src/ratify.rs:543-600`) filters `self.grants`;
  - `holders_at` (`:608-627`), whose people are the only ones counted
    toward a proposal (`tally`, `holder_at`), filters `self.grants`;
  - `Canon::grants` is filled by the fold from `grant` acts
    (`crates/canon-core/src/act.rs:209`, `ActKind::Grant { holder, scope, … }`).
- A canon with no grants is open (`seat` returns `Seat::Open`), and in it no
  objection counts: `consent` needs "no objection from a holder", and
  nobody holds anything. So, without grants, "anyone proposes and it holds
  after two days unless someone objects" would hold every proposal.
- svrngs gives a carried app the people in the group now, `{you, people}`
  (`apps/expenses/index.js:28`). It gives no joining or leaving times and
  no one who admitted them.

## The ways

| | What | Where | Cost | Keeps |
|---|---|---|---|---|
| A | `derive` takes a roster, `(person, scope, from, until)`, read as standing beside the grants, and said as coming from outside (a new field, so `why` and the record say "a member of the house" and never a grant nobody wrote) | canon's repository, a release after 0.3.0; then this row's build again at that commit | in canon, unknown here; in svrngs 0 lines (the page passes `people`) | the decided design, and fence 4: no act is written in anyone's name |
| B | members give each other a say with real `grant` acts: whoever brings House rules grants every member, and any member's page offers "Give Jo a say in House rules" when someone joins | `apps/canon/` only | 0 svrngs lines, 0 canon lines | fence 4. **Changes the decided design**: joining is not having a say until someone grants it, and leaving does not end it until someone withdraws it |
| C | the page writes canon `grant`s from svrngs's own membership acts, each in the name of the member whose computer admitted the person, at the time it did | svrngs-core and the node, to give a carried app the membership history (who admitted whom, when; who left, when); then `apps/canon/` | svrngs-core and node lines, guessed 40–80; 0 canon lines | the decided design. Strains fence 4: the grant is inferred from an admission, never written as a grant |

## What I recommend

**A.** It is what CANON_IN_SVRNGS §10 already names ("If not, it is a small
input in canon, never a second decider in the page"). It keeps the
decided design and writes no act in anyone's name. It costs svrngs nothing.
It is canon's repository's, so it is yours, and canon-house waits for it.

If canon's release must wait, **B** gives a house its rules now with zero
lines anywhere, at the price of a step the decision meant to remove.

## What would falsify this

An input in canon-core 0.3.0 that seats an actor with no `grant` act. I read
`fold.rs`, `ratify.rs`, `standing.rs` and `scope.rs`, and found none.

## What I did not do

- I changed nothing in canon's tree. The build reads it by `git archive`.
- I wrote no grants in anyone's name, and no roster in the wrapper.
