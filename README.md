# svrngs

Keep talking where you talk; keep what your group owns — money, plans, files,
the library — on your own computers, not a platform's. And for developers:
anything you can run on localhost, you can share with your group.

That includes the apps people already self-host — Jellyfin, Navidrome, Immich,
Audiobookshelf, Calibre-Web, Nextcloud, Paperless-ngx, Actual, Mealie, Grocy,
Home Assistant. Run it in a container on one computer, `svrngs share <port> --kind
<name>`, and everyone in the group has it (DESIGN §1).

What we answer to: `NORTH_STAR.md`. What it is, how we build it and the order
of work: `docs/DESIGN.md`. What
exists today: `ARCHITECTURE.md`. How agents work here: `AGENTS.md`. E0, the
first experiment, a Discord bot that settled expenses in the chat, ran
before the fork and is at tag `e0-bot`.

## Run it

Start with `demo/start.md`. In it, a friend lends their model and the group
uses it: you paste one line on each computer, and the friend's chat page
opens on the other's. It gives every line to type and every button to
press, what each step looks like when it works, and what to do when it
does not. It also says how to try both friends on one computer.

## Set up

- Rust 1.91 or later (iroh 1.0.2's `rust-version`), with rustfmt and clippy.
- Node.js 20 or later, for the apps' tests.
- A checkout of commonwealth-ai at `~/dev/commonwealth-ai`, or its path in
  `COMMONWEALTH_AI`: `scripts/ports.py` needs it to compare each forked file
  with its original, and `scripts/principles_check.py` compares the pinned
  principles with it when it is there.

Then `scripts/check.sh` — every step passes, or the tree is not done.
