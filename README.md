# svrngs

Your group's things, on your group's computers. The money you split, the
nights you plan, the films you own, a friend's model: kept where you
already are, opened from a link in the chat you already use. No accounts.
No passwords. No landlord.

It holds to three things ([`NORTH_STAR.md`](NORTH_STAR.md)):
- **Ten seconds from a link.** Open it, give your name, and you're in.
  Nothing new to learn.
- **If you can run it on localhost, you can share it with your friends.**
  No login, no SDK, no bundle.
- **No center but you.** No profile, no directory, no feed, nobody ranked.
  Your group's record lives on its own computers, and the group can leave
  together, intact.

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
- **Hack on the examples.** Money splits what you spend; Events plans a
  night and says who came. Bring either to a group in one tap. They're
  small on purpose: not the point of svrngs, just where you start
  building what your group actually needs.
- **Agree on your house's rules.** House rules comes with svrngs too. Someone
  proposes, the rest of you agree or object, and the page says when it
  holds. [canon](https://github.com/alexsbryan/canon) decides; svrngs only
  shows it. See [`demo/house-rules.md`](demo/house-rules.md).
- **Have a guest.** Your home page makes a link for someone outside the
  group. For up to 48 hours they open what you lend, and everyone sees
  they're your guest. They never read the group's record.
- **Open it to your WiFi.** A lend's row on your home page opens it to the
  devices on your WiFi, as you: your TV's own app, a phone, another
  computer. It stays open on that WiFi alone, until you close it.
- **Put your group on a screen.** Your home page opens any group on a TV
  or a projector: who's on now, what each lends, and who arrived since it
  opened.
- **Ask a friend of a friend.** The grapevine carries an ask from one group
  to another, through someone who's in both. See [`demo/grapevine.md`](demo/grapevine.md).
- **Leave.** Whenever you like. What you already have stays yours.

It's early. [`RELEASES.md`](RELEASES.md) says what works and what doesn't
yet.

## What we host, and why

Your group's things live on your group's computers, and nowhere else.
Nothing we run holds them. We run three things, and you can replace each:

- **The page, `group.svrnme.sh`,** static files on Vercel. A phone can't
  run svrngs, so the group's link opens this page, and the phone's key
  stays in its browser. Each app a friend lends opens at an address of its
  own under it. The host, and your phone's name lookups, see which app a
  phone opens; never what's in it.
- **The download,** beside the page: `install.sh` and a build for each
  platform, checked against `SHA256SUMS`, so a link installs svrngs in one
  line.
- **Relays, n0's for now** (the makers of iroh). When two computers can't
  reach each other directly, and always for a phone, a relay carries the
  bytes, encrypted end to end. It sees which computers talk and how much,
  never what they say. n0's address lookup also knows how to reach each
  computer. svrngs's own relay is built, and waits.

## Run it with nobody in the middle

1. **Build it.** `cargo build --release -p svrngs`, then
   `target/release/svrngs`.
2. **Relay for yourselves.** On a computer that a name leads to, with TCP
   ports 80 and 443 and UDP port 7842 open: `svrngs relay <name> --open`.
   Its certificate comes from Let's Encrypt.
3. **Make the group there.** `svrngs new <group> --relay <name>`. Every
   computer its links let in meets it there. Friends on a computer join
   with `svrngs join <link>`, and need no page.
4. **Host the page, for phones.** Run `scripts/build-web.sh` and
   `scripts/build-release.sh` (each one's top says what it needs), then
   serve `target/web/site/` at your host and at every name under it, with
   `frame-ancestors 'none'` on every answer, as
   [`web/vercel.json`](web/vercel.json) does. Add `--origin
   https://<your host>/` to `new`, or run `svrngs link <group> --origin
   https://<your host>/`. Your founder's line is the one above, with your
   host in it.
5. **Leave n0 out too.** Set `SHIPPED` to `MeetAt::Svrngs` in
   [`crates/svrngs/src/cli.rs`](crates/svrngs/src/cli.rs) and build. A
   computer whose groups all keep their own relay then reaches no server
   but yours.

[`docs/DESIGN.md`](docs/DESIGN.md) §7 says what each piece must do.

## Build on it

If it runs on localhost, your group can use it. Your app gets three things:
- the person asking, verified, in its headers;
- a record the group keeps;
- reach to every member's computer.

There's no SDK and no login, and no app store to petition. The platform
learns by adding apps, never by adding platform.
[`docs/DESIGN.md`](docs/DESIGN.md) §3 is the whole surface.

We build the rails and a small example of each kind of app, never the
finished ones. Money ([`apps/expenses/`](apps/expenses/)) and Events
([`apps/events/`](apps/events/)) are each under a thousand lines, written
to be copied and bent. The finished apps are yours to make.

## Work on svrngs

svrngs stays small enough to read in an afternoon:
[`scripts/lean_check.py`](scripts/lean_check.py) holds it to a line budget,
and every raise gives its reason.

You need:
- Rust 1.91 or later, with rustfmt and clippy;
- Node.js 20 or later, then `npm ci --prefix web`.

Then run:

```
scripts/check.sh
```

Every step passes, or it isn't done. On Linux, it also runs whole in a
podman image: [`scripts/gate/README.md`](scripts/gate/README.md).

Some of svrngs is forked from commonwealth-ai, and links nothing from it:
each forked file names its original on a `ported from commonwealth-ai`
line. You need commonwealth-ai only to follow those originals: checked out
at `~/dev/commonwealth-ai`, or its path in `COMMONWEALTH_AI`,
`scripts/ports.py` lists each forked file beside its original, and
`scripts/principles_check.py --sync` re-pins `docs/principles/`.

The rest of the map:
- [`NORTH_STAR.md`](NORTH_STAR.md): what we answer to.
- [`docs/DESIGN.md`](docs/DESIGN.md): what svrngs is, and how we build it.
- [`docs/PLAN.md`](docs/PLAN.md): what we build next, and in what order.
- [`ARCHITECTURE.md`](ARCHITECTURE.md): what exists today.
- [`AGENTS.md`](AGENTS.md): how agents work here.

E0, a Discord bot that settled expenses in the chat, is at tag `e0-bot`.

AGPL-3.0-or-later. Fork it; that's what it's for.
