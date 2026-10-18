# svrngs

Your group's things, on your group's computers. The money you split, the
nights you plan, the films you own, a friend's model: kept where you
already are, opened from a link in the chat you already use. No accounts.
No passwords. No landlord.

## Start a group

On a Mac or a Linux computer, paste this in Terminal:

```
curl -fsSL https://group.svrnme.sh/install.sh | sh -s -- 'https://group.svrnme.sh/'
```

Your home page opens. Name the group, make a link, and drop it in the chat.
Friends on a computer paste the line the link shows them. Friends on a
phone type their name and tap **Join**. Nobody signs up, and nobody asks
us.

Every line and every button: [`demo/start.md`](demo/start.md).

## Then

- **Lend what you run.** Jellyfin, Navidrome, Immich, Actual, llama-server:
  anything on a port. Your home page lists what's running, and one press
  gives it to the group. If you'd rather type: `svrngs share <port> --kind <name>`.
- **Split it, plan it.** Money and Events come with svrngs. Bring them to a
  group in one tap. They keep going when you close your laptop.
- **Agree on your house's rules.** House rules comes with svrngs too. Someone
  proposes, the rest of you agree or object, and the page says when it
  holds. [canon](https://github.com/alexsbryan/canon) decides; svrngs only
  shows it. See [`demo/house-rules.md`](demo/house-rules.md).
- **Ask a friend of a friend.** The grapevine carries an ask from one group
  to another, through someone who's in both. See [`demo/grapevine.md`](demo/grapevine.md).
- **Leave.** Whenever you like. What you already have stays yours.

It's early. [`RELEASES.md`](RELEASES.md) says what works and what doesn't
yet.

## Where your things go

On the group's computers, and nowhere else. We host a page and a download,
both static. When two computers can't reach each other directly, n0's
relays carry the bytes, encrypted. A relay sees that you're talking, never
what you say.

## Build on it

If it runs on localhost, your group can use it. Your app gets three things:
- the person asking, verified, in its headers;
- a record the group keeps;
- reach to every member's computer.

There's no SDK and no login, and no app store to petition.
[`docs/DESIGN.md`](docs/DESIGN.md) §3 is the whole surface. The reference
apps are in [`apps/`](apps/).

## Work on svrngs

You need:
- Rust 1.91 or later, with rustfmt and clippy;
- Node.js 20 or later;
- commonwealth-ai checked out at `~/dev/commonwealth-ai`, or its path in
  `COMMONWEALTH_AI`. `scripts/ports.py` compares every forked file with its
  original there.

Then run:

```
scripts/check.sh
```

Every step passes, or it isn't done.

The rest of the map:
- [`NORTH_STAR.md`](NORTH_STAR.md): what we answer to.
- [`docs/DESIGN.md`](docs/DESIGN.md): how we build, and in what order.
- [`ARCHITECTURE.md`](ARCHITECTURE.md): what exists today.
- [`AGENTS.md`](AGENTS.md): how agents work here.

E0, a Discord bot that settled expenses in the chat, is at tag `e0-bot`.

AGPL-3.0-or-later. Fork it; that's what it's for.
