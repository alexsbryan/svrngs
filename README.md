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

Every step, with what the pages say: [`docs/guide/start.md`](docs/guide/start.md).

Join once from the chat link. Everything your group lends has an address
that never changes. Bookmark it on your phone or your laptop, and it opens.
`https://group.svrnme.sh/#open=films@Sam` opens Sam's films. On an
iPhone, add the group's page to your Home Screen first and join from
there: a Home Screen app keeps its own storage, apart from Safari's.

## What you can do

[The guide](docs/guide/README.md) has a page for each of these, with
examples. New to it? [How it works](docs/guide/how-it-works.md) takes five
minutes.

- **Lend what you run.** Jellyfin, Navidrome, Immich, Actual: anything on
  a port. Your home page lists what's running, and one press gives it to
  the group. Or type `svrngs share 8096 --group <group>`. [Lend](docs/guide/lend.md).
- **Share a model.** Lend your llama-server; friends chat with it, and
  their programs call it at an address that doesn't change.
  [Models](docs/guide/models.md).
- **Play together.** Lend a Minecraft or Factorio server with `--tcp` or
  `--udp`; friends press **Play**. [Give](apps/give/README.md) keeps a game
  for the group and moves it to another computer with one yes.
  [Games](docs/guide/games.md).
- **Watch on the TV.** Open a lend to your WiFi, so the TV's own app opens
  it as you, or put your group on a screen. [TV](docs/guide/tv.md).
- **Split money, plan a night, agree on house rules.** Money, Events and
  House rules come with svrngs, one tap to bring to a group. They're small
  on purpose: where you start building what your group needs.
  [Apps](docs/guide/apps.md).
- **Keep a folder the same on everyone's computer.**
  [Folders](docs/guide/folders.md).
- **Invite, add your phone, have a guest, remove someone, leave.** A guest
  opens what you lend for up to 48 hours, and never reads the group's
  record. What you already have stays yours when you leave.
  [People](docs/guide/people.md).
- **Ask a friend of a friend.** The grapevine carries an ask from one group
  to another, through someone who's in both.
  [`demo/grapevine.md`](demo/grapevine.md).
- **Build an app.** If it runs on localhost, your group can use it.
  [Build](docs/guide/build.md).

Something not working? [Troubleshooting](docs/guide/troubleshooting.md).
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
   with `svrngs join <link>`, and need no page. To move a group later,
   stop svrngs and run `svrngs link <group> --relay <name>`: only its
   keeper may, and the new link carries the new relay.
4. **Host the page, for phones.** Run `scripts/build-web.sh` and
   `scripts/build-release.sh` (each one's top says what it needs), then
   serve `target/web/site/` over HTTPS at your host and at every name under
   it, with `frame-ancestors 'none'` on every answer but `/sign.html`,
   which sends `frame-ancestors https:` so your group's apps can frame it,
   as [`web/vercel.json`](web/vercel.json) does. Add `--origin
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

To work on the group's page, `web/`, run `scripts/build-web.sh`, then
`node web/tests/dev.mjs`. It serves the page on your computer at an https
address, and prints the line that opens Chrome there, where apps open and
save, and how to point a group at it. Safari can't save there
([why](demo/runs/security/2026-11-02-webkit-partition/README.md)).

oxfmt lays out the JavaScript: `web/node_modules/.bin/oxfmt '**/*.{js,mjs,cjs}'`
formats it, and `scripts/check.sh` fails on any file it would change. To
have `git blame` skip commits that only reformat, run once:

```
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

Some of svrngs is forked from commonwealth-ai, and links nothing from it:
each forked file names its original on a `ported from commonwealth-ai`
line. You need commonwealth-ai only to follow those originals: checked out
at `~/dev/commonwealth-ai`, or its path in `COMMONWEALTH_AI`,
`scripts/ports.py` lists each forked file beside its original, and
`scripts/principles_check.py --sync` re-pins `docs/principles/`.

The rest of the map:
- [`docs/guide/`](docs/guide/README.md): how to use svrngs, page by page.
- [`docs/OVERVIEW.md`](docs/OVERVIEW.md): svrngs in three pictures.
- [`NORTH_STAR.md`](NORTH_STAR.md): what we answer to.
- [`docs/DESIGN.md`](docs/DESIGN.md): what svrngs is, and how we build it.
- [`docs/PLAN.md`](docs/PLAN.md): what we build next, and in what order.
- [`ARCHITECTURE.md`](ARCHITECTURE.md): what exists today.
- [`AGENTS.md`](AGENTS.md): how agents work here.

E0, a Discord bot that settled expenses in the chat, is at tag `e0-bot`.

AGPL-3.0-or-later. Fork it; that's what it's for.
