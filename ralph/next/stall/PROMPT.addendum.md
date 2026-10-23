<!-- section: stall after=intro -->
This queue finds and fixes one stall: a body that stops moving after its
answer's head arrives. Before the row, read this queue's `CHARTER.md` and
`STATE.md`.

Fences the row keeps:
1. *Instrument and reproduce before a fix* (AGENTS.md 2). No product code
   changes until a run stalls at will and a trace names the layer.
2. *Nothing hides it* (AGENTS.md 6). No load bar, no longer limit, no retry,
   and no could-not-judge for this assertion.
3. *Glassbox* (AGENTS.md 1). What the traces add stays, at debug, each line
   saying what its layer waits on.
4. *The machine is shared.* A planted load is the run's own and ends with it;
   no process from a run is left running (`ps`); caches stay under `target/`.
5. *The budget.* Tests and `demo/` are not counted; a product line is, and a
   raise is the operator's.
