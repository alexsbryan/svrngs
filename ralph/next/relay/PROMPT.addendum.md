<!-- section: relay after=intro -->
This queue gives svrngs its own relay before v1 and takes n0 out of the
path — its relays and its address lookup (the operator, 2026-10-12: "I do
think we need to host our own relay (can't rely on n0) before our v1 ship.
And ideally however we do that is something packageable to make it easy
for anyone who wants to run it themselves"). Read this queue's
`CHARTER.md`, which holds the decisions, and svrn-docs
`rings/current/GROUP_RELAYS.md`, the design, before a row. Then the
operator, 2026-10-13: "Can we actually just park this here right at the
gate and keep using n0 until I'm actually ready to start paying / flip the
switch for production?" — n0 the default again, svrngs's relay one switch
away (`CHARTER.md`, 2026-10-13).

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2): what breaks with n0 off
   is a run or a test that shows it on this row's base before its fix,
   named in the commit with its red output.
2. *Never silently substitute* (AGENTS.md 6): a computer or phone that
   reaches no relay says which it tried; a relay that refuses a key says so
   in its trace; n0 is the operator's chosen default until the switch,
   and never a fallback when svrngs's relay or a group's fails.
3. *The core is pure* (RUST_ARCH; AGENTS.md 10): where a computer's relay is
   kept in the record is decided in svrngs-core's acts and fold; the node
   and the command only carry it.
4. *Every decision traces* (AGENTS.md 1), at debug, each refusal included.
5. *A person's words* (DESIGN §1): a person reads names, the group, computer
   and phone — never key, member, keeper, ring, act or journal. A relay's
   operator reads its flags and its words too.
6. *The budget is 15,800*, svrngs-core's 3,130 (the operator, 2026-10-12, for
   this queue): the first row past the old caps raises each in its own
   commit; each row puts `scripts/lean_check.py` before and after in its
   commit; past either is the operator's.
7. *Nothing outward-facing*: no server deployed, no DNS changed, no port
   opened on this host's network, no `sudo`; a run serves its relay on this
   host. Dialling n0's public relays, as svrngs did before this queue, is
   not outward-facing.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
