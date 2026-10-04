# svrngs

Keep talking where you talk; keep what your group owns — money, plans, files,
the library — on your own computers, not a platform's. And for developers:
anything you can run on localhost, you can share with your group.

What it is, how we build it and the order of work: `docs/DESIGN.md`. What
exists today: `ARCHITECTURE.md`. How agents work here: `AGENTS.md`.

## Run it

Nothing runs yet. The group's record and journal have landed, forked from
commonwealth-ai; sync, joining by link and sharing an app come next (DESIGN
§8). E0 — the first experiment, a Discord bot that settled expenses in the
chat — ran before the fork and is at tag `e0-bot`.

## Set up

- Rust 1.87 or later (`rquickjs` needs it), with rustfmt and clippy.
- Node.js 20 or later, and `npm ci` in `bridge/` (its tests need discord.js).
- A checkout of commonwealth-ai at `~/dev/commonwealth-ai`, or its path in
  `COMMONWEALTH_AI`: `scripts/ports.py` needs it to compare each forked file
  with its original, and `scripts/principles_check.py` compares the pinned
  principles with it when it is there.

Then `scripts/check.sh` — every step passes, or the tree is not done.
