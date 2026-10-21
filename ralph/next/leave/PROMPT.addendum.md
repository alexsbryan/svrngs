<!-- section: leave after=intro -->
This queue fixes #211: a leave that outruns the leaver's last act, so the
group's copies never agree again. Before a row, read this queue's
`CHARTER.md` and `demo/runs/leave/2026-10-21/SURVEY.md` whole, and
`docs/DESIGN.md` §6.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2 and 5). The survey's red
   test is the start: it is red at HEAD, and every plant is watched failing.
2. *Structural, not timed* (AGENTS.md 10). Every copy that holds a cut holds
   every act it covers. No fix that only makes the race rarer.
3. *One decider* (AGENTS.md 8). What a cut is told with is chosen in one
   function, which both the at-once tell and `told_cut` call.
4. *Nothing beyond the record.* A leave travels with only the leaver's own
   acts before it, and a keeper's removal with only the keeper's acts before
   it and the hand-ons, never anyone else's. The leaver's computer takes
   nothing more of the group.
5. *Never silently substitute* (AGENTS.md 6). A tell that cannot carry
   its chain is said and asked again, never sent as the cut alone.
6. *Write for the next reader.* DESIGN §6 changes in the commit that changes
   what it describes, rewritten in place.
7. *The budget* (the operator, "Pass first, then raise"): the economizing
   pass goes first, then at most +25 repository lines over the cap it
   leaves, and none in svrngs-core.
8. *Nothing outward-facing.* Every computer runs on this host.
