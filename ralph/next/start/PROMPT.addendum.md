<!-- section: start after=intro -->
This queue makes svrngs easy to start with, before v1. The operator,
2026-10-13: "nail down the svrn / llama-server on svrngs story and make it
part of an early demo document for anyone getting started"; "making sure
our localhost dashboard really makes the system easy to navigate for
someone who is a bit afraid technically"; and "#126–#129 and #131". Read
this queue's `CHARTER.md`, which holds the decisions, and
`demo/runs/llama/2026-10-11/README.md` before a row.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2). What a newcomer meets is
   a run or a test, on this row's base and before its fix, that shows it.
   The commit names it, with its red output.
2. *Never silently substitute* (AGENTS.md 6). An address that moved, a
   program that could not reach a lend, a service svrngs could not name:
   each is said, in a person's words, never left for the person to find.
3. *For someone a bit afraid technically*:
   - The home page uses DESIGN §1's words (names, the group, computer,
     phone), and no word its reader must look up. Never key, member,
     keeper, ring, act, journal, port, namespace or origin.
   - Each button says what it does before it does it, and the page says
     what happened after.
   - Nothing begun on the page needs the terminal to finish.
   - What still needs the terminal is said there, with the one line to
     type.
4. *The computer is the main flow.* The phone is how people start;
   nothing on the computer's side waits for the phone's.
5. *No setting nobody chose, none that is hard to change* (DESIGN §10,
   2026-10-12). What a person changes, they change on the home page, not
   by a flag only.
6. *The core is untouched*, #131 apart, which changes one signature in
   svrngs-core and adds no lines there. Nothing of llama-server's is named
   in svrngs's code outside the well-known table (DESIGN §5, the nouns
   gate).
7. *The budget is 16,000*, and svrngs-core's stays at 3,130 (the operator,
   2026-10-13, for this queue: 15,950, raised in c27521f, then +50 after
   start-measure for the home page). The first part past 15,950 raises the
   cap to 16,000 in its own commit. Each row puts `scripts/lean_check.py` before and after
   in its commit. Going past either cap is the operator's call.
8. *Nothing outward-facing.* No server is deployed, no DNS is changed, no
   port is opened on this host's network, and no `sudo` is used.
   llama-server runs on this host, on 127.0.0.1, with a small model:
   Qwen2.5-0.5B-Instruct Q4_K_M, as in the 2026-10-11 run, fetched by
   llama-server's own `-hf` if it is not cached. Its answers are not
   judged.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
