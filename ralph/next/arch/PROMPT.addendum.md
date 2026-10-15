<!-- section: arch after=intro -->
This queue makes svrngs smaller by structure, following an architectural
audit (the operator, 2026-10-15). Before a row, read this queue's
`CHARTER.md`, which holds the decisions, and `AUDIT.md`, which holds the
evidence. Read every line the audit cites again at HEAD before acting on it.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2). Each change starts from
   a run or a test that shows what happens today, red, named in the
   commit. Every new check fails first on a plant it names (AGENTS.md 5).
2. *A move is its own commit* (AGENTS.md 2). A move that keeps behaviour
   lands apart from any change in behaviour, its tests unchanged.
3. *Existing groups keep working.*
   - What a computer holds on disk is still read.
   - An older build and this one, in one group on this host, still
     exchange and agree.
   - Nothing needs migrating.
   - A row that cannot keep these writes NEEDS_HUMAN.md.
4. *One decider* (AGENTS.md 8).
   - A cut that leaves two makers of one thing is not done.
   - The command and the page ask one rule in the node.
5. *The core is pure* (RUST_ARCH; AGENTS.md 10). Who counts as in, and what
   an act means, is svrngs-core's. The node and the command only carry it.
6. *Never silently substitute* (AGENTS.md 6).
   - A refusal that replaces a path says what it refused and why, and it
     traces at debug.
   - A seal line on disk is read as an act that does nothing, and said so,
     never dropped unread.
7. *The budget* (the operator, 2026-10-15: "Up to +60 inside a row").
   - No raise, except arch-helper: it may raise within the 31 lines v1ui's
     +650 left unspent, up to 16,965, in its own commit.
   - Between its own commits, a row may sit up to 60 code lines over the
     count it started at (`scripts/lean_check.py`), and it closes below
     that count.
   - Each row lowers the repository's cap and svrngs-core's to where it
     closes, in its own commit with the counts before and after.
   - `apps/` is not counted.
8. *Nothing outward-facing.* Every computer and phone runs on this host.
   Dialling n0's public relays, as svrngs does by default, is not
   outward-facing.
9. *The house voice* (svrn-docs `VOICE.md`). Every word a person reads, on
   a page or from the command, is in the task register:
   - second person, active and present;
   - no word a reader must look up: never key, member, record, act, hash,
     origin, port or relay;
   - each button says what it does before it does it.
10. *The computer is the main flow.* Where a row adds to a page, the
    computer's home page comes first, then the phone's.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
