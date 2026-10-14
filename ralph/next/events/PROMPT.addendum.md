<!-- section: events after=intro -->
This queue builds the primitive event and its reference app. The
operator, 2026-10-13: "I want the run after onair to be our event
primitives + reference app (we can layer on this taps concept later)". An
event is a post in a group, and people answer it: three acts in the
group's record. Before a row, read this queue's `CHARTER.md`, which holds
the decisions, and svrn-docs `rings/current/EVENTS.md` §1-§5 and §9.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2). The run's predictions
   are committed before its data. Every check is watched failing first, on
   a plant it names (AGENTS.md 5).
2. *Never silently substitute* (AGENTS.md 6):
   - a group whose record cannot be read is said as that, never shown as
     nobody answering;
   - a computer that did not answer is said as that;
   - nothing records "no", so nothing is ever shown as a refusal.
3. *svrngs is unchanged.*
   - Everything event-shaped lives in `apps/events/`, its nouns in
     `apps/events/nouns.txt`, and svrngs's code uses none of them
     (`scripts/vocab_check.py`; DESIGN §5).
   - Where the app needs what svrngs does not give, that goes in
     NEEDS_HUMAN.md as svrngs's, with its lines estimated. It is never
     worked around in the app.
4. *Machines build, people speak.*
   - The app never answers, sends or invites in anyone's name.
   - "+ Maya" is the member's own word.
   - `came` for a guest is said by the member who brought them, and only
     by them.
5. *Context comes from the record's boundary.*
   - Each group's record holds only its own people's answers.
   - The host's view folds the groups the host's computer holds.
   - A member of one group never reads another group's answers through
     the app.
6. *A person's words* (DESIGN §1). The page uses names, the group, coming
   and came. It never says key, member, record, act, namespace or origin.
7. *No raise.* `apps/` is not counted (`scripts/lean_check.py`), and
   svrngs's code is unchanged. Each row puts `scripts/lean_check.py`
   before and after in its commit, unchanged.
8. *Nothing outward-facing.* Every computer and phone runs on this host.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
