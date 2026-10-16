<!-- section: grapevine after=intro -->
This queue builds the grapevine demo app, `apps/grapevine/`, at zero
svrngs lines. It is the last queue before V1 (the operator, 2026-10-15).
Before a row, read this queue's `CHARTER.md`, svrn-docs
`rings/current/GRAPEVINE_DEMO.md` whole (its §12 and §13 change §1-§11),
and `GRAPEVINE.md` §4 and §7. Read `apps/events/` first: the app takes its
shape.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2 and 5). Each law fails
   first on its plant, as GRAPEVINE_DEMO §8 names them, and each commit
   names its plant.
2. *Zero svrngs lines,* except gv-leave (up to +30, core unchanged; the
   CHARTER). Everything else lands under `apps/grapevine/`,
   `demo/runs/grapevine/` and `demo/grapevine.md`. A svrngs line elsewhere
   writes NEEDS_HUMAN.md.
3. *The fold is the one decider* (AGENTS.md 8). The page and the loop read
   only the fold's four answers (`asks`, `carriable`, `threads`, `relay`).
   Neither decides anything of its own.
4. *Never silently substitute* (AGENTS.md 6).
   - The carry decider names each refusal's reason.
   - A computer that doesn't answer is said by its person's name.
   - File mode is labelled a simulation on every page.
   - The labelled shortcut (GRAPEVINE_DEMO §6) is said on every thread and
     first in the README: that a sealed message passed, with its size, its
     handle and its time, is seen by every member of a ring it crosses.
5. *Sealed means sealed.* No record, log, trace (`GRAPEVINE_DEBUG`
   included) or file other than the person's own state file ever holds a
   message's plaintext or a secret key. The seal is tweetnacl's `box`,
   never assembled from parts (§13.5).
6. *Who is asking* (§13.1-§13.3).
   - The process answers only its own person.
   - It refuses any `Host` but its own loopback address, any `Origin` but
     its own, and a write that is not a JSON body.
   - "Me" is read per group from what svrngs answers, never a config
     constant.
   - It is not lent.
7. *A developer's kind is one file and a line.* A kind writes no routing,
   scope, handle, sealing, carrying or name. `kinds/README.md` says so
   first, and a third kind written as it says proves it.
8. *Taps only.* No standing carry, no `/standing`, no bar 8.
9. *The house voice* (svrn-docs `VOICE.md`).
   - The page is in the task register, with no word a reader must look up:
     never key, member, record, act, hash, handle, hop or seal on the page.
     It says "carried by Jo", "one ring away" and "sealed: only you two
     can read this".
   - The README and `kinds/README.md` are for developers, in the task
     register.
10. *Nothing outward-facing.* Every person runs on this host. Dialling n0's
    public relays, as svrngs does by default, is not outward-facing.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
