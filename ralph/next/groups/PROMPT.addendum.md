<!-- section: groups after=intro -->
This queue settles how groups are told apart, how a person leaves one,
and what a computer signs a group with (the operator, 2026-10-14). Before
a row, read this queue's `CHARTER.md`, which holds the decisions, and
svrn-docs `rings/current/LEAVING.md`.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2). Each change starts from
   a run or a test that shows what a person meets today, red, named in
   the commit. Every new check fails first on a plant it names
   (AGENTS.md 5).
2. *Never silently substitute* (AGENTS.md 6):
   - two groups of one name are told apart, never guessed between;
   - a leave is said as a leave, never as a removal;
   - "final from this computer" is said where someone leaves;
   - a computer that did not answer is said by its person's name, never
     by a relay's host.
3. *The core is pure* (RUST_ARCH; AGENTS.md 10). Who counts as cut is
   svrngs-core's fold (`membership.rs`). The node and the command only
   carry it.
4. *One decider* (AGENTS.md 8):
   - one rule for a group's name, in the node, asked by the page and the
     command alike;
   - one cap per dial, by its measurement (`dial.rs`).
5. *Exit governs what stays.* Leaving deletes nothing on anyone's disk and
   sends nothing back. What a person held stays theirs (DESIGN §6).
6. *A person's words* (DESIGN §1). The pages use names, the group,
   computer and phone, never key, member, keeper, record, act or relay.
   The keeper's office is said as "keeps the order of removals".
7. *The budget.*
   - groups-economize, the pass, goes first.
   - After it, each row raises only as far as it needs, in its own commit
     naming what the pass cut.
   - The ceilings are +200 over the repository's cap the pass leaves, and
     +40 over svrngs-core's (the operator, 2026-10-14).
   - `apps/` is not counted.
8. *Nothing outward-facing.* Every computer and phone runs on this host.
   Dialling n0's public relays, as svrngs does by default, is not
   outward-facing.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
