# Instrument: Ana's acting ends before Kim tells her, so Kim's ask can't reach her

- status: open, for xenia and for the scene's author
- found in: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T03-11-14-983Z` on the Fedora box, xenia `2a10d4d`, the product `ccd48cb4`; `together` runs 1-3
- kind: the scene's choreography with the instrument's ending rule; not a svrngs fault alone

## What happened

In each `together` run Ana's own `tell` ended her acting before Kim had added the pizza, so Kim's split, and Kim's message in the house chat, never reached her (`run-<n>/together/trace.json`, by clock):

| run | Ana's first `tell` | Kim taps "Add it, paid by you" | Kim's `say` to Ana |
|---|---|---|---|
| 1 | 52.3 s | 88.3 s | 92.3 s |
| 2 | 45.3 s | 98.4 s | 102.4 s |
| 3 | 45.5 s | never (spent) | never |

Ana's only opening message is Sam's welcome with the link (`crates/svrngs/tests/xenia/main.rs`, `ana_link`), so after joining she has nothing to look for until Kim writes. In the `ana` scene the pizza names nobody (`ana_world`: the pizza is in before Ana joins, so Kim could not have named her), so "Kim says you were in on pizza" is never on any screen there either. Ana asked about Kim's split: 0 of 6 accounts, by construction in three and by timing in three.

## For xenia

A person's `tell` ends their acting (`src/core/world.ts` `step`, `own`), and a castmate's message that arrives after is never heard. In a scene whose goal for one person needs another's later act, a message arriving after the first has ended could be held as a reason to come back, as a phone's notification is, or the account could say the goal was out of reach when they ended. Either is xenia's design; the operator decides whether the `together` scene's Ana is told to expect Kim's message.

## What svrngs changed on its side

Money's page now draws itself again when the group's money changes (`apps/expenses/index.js`, every 5 s), so an Ana still on Money when Kim names her is asked there. It helps only if she is still acting when Kim writes.
