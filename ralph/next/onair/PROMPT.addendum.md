<!-- section: onair after=intro -->
This queue proves three primitives an app gets, through ON_AIR's proof of
concept (svrn-docs `rings/current/ON_AIR.md` §7). The operator, 2026-10-12:
"what's the minimal primitives we would want to have in svrngs to allow
someone else to realize a vision like this? This is largely just a test of
the infra"; and 2026-10-13: "Add the on air spike too". Before a row, read
this queue's `CHARTER.md`, which holds the decisions, and ON_AIR §3, §4
and §7.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2). The harness runs red on
   svrngs as it is, and that red run is the reproduction. Each primitive
   then turns its own rows of ON_AIR §7's table green, each with its plant
   failing first (AGENTS.md 5).
2. *Never silently substitute* (AGENTS.md 6):
   - a piece no holder answered is said as that, never an empty piece;
   - a piece that lapsed is said apart from one never held;
   - "ended" is said apart from "cannot reach".
3. *The core is pure* (RUST_ARCH; AGENTS.md 10). An app's signature, made
   and checked, lives in svrngs-core: no I/O, the key passed in, its
   domain beside a pass's and a label's (`pass.rs`) with one home for
   domains. The node and its routes only carry it.
4. *One decider* (AGENTS.md 8). Who receives a group's bytes is folder.rs's
   rule, taught a hash an app placed, never a second rule beside it.
5. *Every decision traces* (AGENTS.md 1), at debug, each refusal included.
6. *The nouns gate* (DESIGN §5; `scripts/vocab_check.py`). The app lists
   its nouns in `apps/on-air/nouns.txt`, and svrngs's code uses none of
   them: no station, show, radio or broadcast outside `apps/on-air/`.
7. *The budget is 16,340*, and svrngs-core's 3,190 (the operator,
   2026-10-13, for this queue). The first row past each old cap (16,040
   and 3,130) raises it in its own commit. Each row puts
   `scripts/lean_check.py` before and after in its commit. Going past
   either is the operator's. A raise comes only after an economizing pass,
   its commit naming what the pass cut (the operator, 2026-10-13). `apps/` is not counted
   (`scripts/lean_check.py:14-16`).
8. *Nothing outward-facing.* There is no CDN, no bucket and no public
   door: "the public" is a directory served by `python3 -m http.server` on
   127.0.0.1. The audio is generated here (ffmpeg) or CC0, and said where
   it came from.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
