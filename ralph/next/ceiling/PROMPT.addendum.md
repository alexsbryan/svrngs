<!-- section: ceiling after=intro -->
This queue is the ceiling (`docs/DESIGN.md` §8 :1069-1085, "Then" :1085):
an idle round at 25 members and 10,000 acts under 0.1 s of processor in
the instrument, where today it is 73 s. The mistake is one, made in
several places — a fold rebuilt from the whole journal when it could be
caught up from its tail. svrngs already does it for writes (`Held`,
`crates/svrngs-runtime/src/journal.rs:36-58`); the rows apply that
pattern to the digest, the signatures and the register, one row each,
and the bar lands with the last.

The survey of record is svrn-docs `rings/current/THE_CEILING.md`. It is
a DRAFT, not in force; its fixes were prototyped behind switches
(`SVRNGS_BENCH_*`) in a scratch clone of svrngs at `15a9de7` and
measured there — 73 s → 10 ms at 25 × 10,049, five of ten written
predictions refuted, each failure finding the next cost. This queue does
not port the prototype's switches: it makes the real folds, behaviour
kept, each row naming what byte-identical means for it. Its line numbers
were taken at `15a9de7`; re-verify each at HEAD before the row that
cites it runs.

Fences the rows keep:
1. *Dialling at once is not this queue's.* It stays §9's row (:1079);
   THE_CEILING.md's fix 4. An asleep member's ten-second timeout stays
   until that row lands, and the bar is set so it must: the ceiling-4
   canary measures processor, not wall time.
2. *The view each act carries goes with U7* (THE_CEILING.md's fix 5).
   ceiling-4 touches it only if the bar needs it, as its row says; if it
   does, that is a finding for the operator before the row lands, since
   it changes the line format's byte-compatibility promise.
3. *One computer, no other machines* (operator, 2026-10-05,
   THE_CEILING.md "How we know"). A fix is judged in the instrument,
   `crates/svrngs-node/tests/scale.rs`, which stays ignored and promoted
   — it runs when named, never in scripts/check.sh's default pass,
   except the ceiling-4 canary, which is the one scale number
   scripts/check.sh holds.
4. *A prediction first.* As THE_CEILING.md's runs did: the number
   written and timestamped before the run that tests it, five of ten
   refuted there and none hidden. ceiling-4's check repeats it at HEAD.

The bar's shape (THE_CEILING.md "What the one computer must show", item
6): the canary's plant is today's code, watched failing at 73 s, and
passes on the fixed folds. A run that could not finish is
could-not-judge, never a pass with a smaller number (AGENTS.md 6).
