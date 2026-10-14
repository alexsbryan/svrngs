<!-- section: v1ui after=intro -->
This queue builds the v1 UI: the 20% of the interface that carries 80% of
what the copy promises (the operator, 2026-10-14; svrn-docs
`rings/current/V1_UI_BUILD.md`). Before a row, read this queue's
`CHARTER.md`, which holds the decisions, and V1_UI_BUILD. For 3.4 also read
`rings/current/COMMUNITY_APPS.md` §6-§7.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2). Each change starts from
   a run or a test that shows what a person meets today, red, named in
   the commit. Every new check fails first on a plant it names (AGENTS.md
   5), and V1_UI_BUILD §7 names each one.
2. *Never silently substitute* (AGENTS.md 6):
   - a computer that did not answer is drawn off, never lit and never as
     nobody;
   - a phone reaching the group through someone else's computer says
     whose;
   - a refusal names its person, never a relay's host.
3. *For someone a bit afraid technically.*
   - The pages use DESIGN §1's words, and none a reader must look up:
     never key, member, record, act, hash, origin, port or relay.
   - Each button says what it does before it does it, and the page says
     what happened after.
   - Nothing begun on a page needs the terminal to finish.
   - Presence is only "on now", never "last seen".
4. *The computer is the main flow.* Each page row does the computer's
   home page first, then the phone's group page.
5. *Containment* (COMMUNITY_APPS §6):
   - a carried app reads only its own acts and writes only as the person,
     from its route;
   - it loads nothing from elsewhere, by its headers;
   - nothing on the shelf is a stranger's.
6. *No setting nobody chose, and none hard to change* (DESIGN §10,
   2026-10-12). A group's colour is derived, never asked.
7. *The budget.*
   - v1ui-economize, the pass, goes first.
   - After it, each row raises only as far as it needs, in its own commit
     naming what the pass cut, within +650 over the cap the pass leaves
     (the operator, 2026-10-14).
   - svrngs-core stays at its cap. A row that needs a core line writes
     NEEDS_HUMAN.md.
   - `apps/` is not counted.
8. *Nothing outward-facing.* Every computer and phone runs on this host.
   Dialling n0's public relays, as svrngs does by default, is not
   outward-facing.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
