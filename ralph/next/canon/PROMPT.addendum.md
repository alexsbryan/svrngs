<!-- section: canon after=intro -->
This queue puts a house's rules on svrngs, after V1 (the operator,
2026-10-16). Its first rows pin what runs as you. Before a row, read this
queue's `CHARTER.md` and svrn-docs `rings/current/WHAT_RUNS_AS_YOU.md` and
`CANON_IN_SVRNGS.md` whole. For canon itself, read
`~/dev/canon/docs/PRIMITIVES.md` and `crates/canon-core/src/ratify.rs`.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2 and 5). Each change starts
   red, named in its commit, and every new check fails first on a plant it
   names.
2. *Never widen silently* (WHAT_RUNS_AS_YOU §1).
   - Once the floor is pinned, no page's policy and no listener widens
     without NEEDS_HUMAN.md.
   - Carried pages never get `'wasm-unsafe-eval'`.
   - A carried page runs only its pinned files.
3. *One decider* (AGENTS.md 8).
   - canon-core is the only thing that decides a house's rules. The page
     never re-implements a canon decision in JavaScript; it calls the build.
   - The floor file is the only home for what a page or socket may do.
4. *Who said it, truly.* canon's actor is the record's person, never the
   act's own text. An act written through someone else's computer says
   whose, with `via`.
5. *What runs as you is yours.* A new version of a carried app runs for a
   person only after their own yes. Until then the version they said yes to
   keeps working.
6. *For someone a bit afraid technically.*
   - The page says "House rules", never canon's internal words: scope,
     ratification, commitment, act, actor.
   - Each button says what it does before it does it.
   - Nothing begun on a page needs the terminal to finish.
7. *The house voice* (svrn-docs `VOICE.md`): second person, active, present.
8. *The computer is the main flow.* The home page comes first, then the
   phone's page.
9. *The budget* (the operator, "+150 repo, +40 core").
   - The economizing pass goes first.
   - Each row raises only what it needs, in its own commit.
   - `apps/` is not counted.
10. *Nothing outward-facing.* Every computer and phone runs on this host.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
