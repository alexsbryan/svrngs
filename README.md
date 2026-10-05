# svrngs

Keep talking where you talk; keep what your group owns — money, plans, files,
the library — on your own computers, not a platform's. And for developers:
anything you can run on localhost, you can share with your group.

That includes the whole self-hosted catalogue — Jellyfin, Navidrome, Immich,
Audiobookshelf, Nextcloud, Paperless-ngx, Actual, Mealie, Forgejo, Home
Assistant. Run it in a container on one computer, `svrngs share <port> --kind
<name>`, and everyone in the group has it (DESIGN §1).

What we answer to: `NORTH_STAR.md`. What it is, how we build it and the order
of work: `docs/DESIGN.md`. What
exists today: `ARCHITECTURE.md`. How agents work here: `AGENTS.md`.

## Run it

On a computer: `svrngs new <name>` makes a group and prints its link,
`svrngs join <link>` joins it from another, `svrngs share <port>` lends what
listens there, and `svrngs` alone keeps it running (DESIGN §8, U1-U3). A
phone joins from the link in its browser (U4, landing). E0 — the first
experiment, a Discord bot that settled expenses in the chat — ran before the
fork and is at tag `e0-bot`.

## Set up

- Rust 1.91 or later (iroh 1.0.2's `rust-version`), with rustfmt and clippy.
- Node.js 20 or later, for the apps' tests.
- A checkout of commonwealth-ai at `~/dev/commonwealth-ai`, or its path in
  `COMMONWEALTH_AI`: `scripts/ports.py` needs it to compare each forked file
  with its original, and `scripts/principles_check.py` compares the pinned
  principles with it when it is there.

Then `scripts/check.sh` — every step passes, or the tree is not done.
