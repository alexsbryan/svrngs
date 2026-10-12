# One person, many devices — predictions, before any data

Row bash-devices (e) (`ralph/next/bash/STATE.md`): Ana's computer, a
second computer of hers and her phone as one person, on this host, after
(a)–(d) landed (`aa052a1`..`c2e074a`). Every person is a program; nobody
is a person, so no people bar is judged (AGENTS.md 7). One host, one run:
not a rate. Written and committed before the driver ran.

## The setup

- **The build** is this commit's: `cargo build -p svrngs`, the binary at
  `target/debug/svrngs`, every computer a data dir and `HOME` of its own
  under one scratch directory, each `svrngs` started by the driver.
- **The page** is `scripts/build-web.sh --out <scratch>/site`, served on
  this host by `web/tests/serve.mjs` at `http://localhost:<port>/`; nothing
  is published (the queue's fence 7). The computers and the phone reach
  each other by n0's public relays, the default (`RelayConfig::default()`).
- **Sam** makes the group, `svrngs new devices --as Sam --origin
  <the page>`, and lends three things, each by `svrngs share`:
  - a scratch **Jellyfin** (the macOS app's server, its own data dirs, no
    films) through its middleware (`apps/jellyfin/mw`) on 8097:
    `svrngs share 8096 --sign-in 8097`;
  - Homebrew's **Navidrome** on 4533 with `ND_EXTAUTH_USERHEADER=X-Svrngs-User`
    and `ND_EXTAUTH_TRUSTEDSOURCES=127.0.0.1/32`, as `svrngs share 4533`
    prints, Sam its admin: `svrngs share 4533`;
  - a **shelf**: a stand-in on this host answering every request with the
    `x-svrngs-key`, `x-svrngs-user` and `x-svrngs-person` it was told, lent
    to one person at once: `svrngs share <port> --name shelf --limit 1`.
- **Ana** joins by Sam's link, `svrngs join <link> --as Ana --no-run`, then
  makes a link for her devices, `svrngs link devices --device`, which runs
  on as her computer.
- **Ana's laptop** is a second computer, `USER=analaptop`: `svrngs join
  <Ana's device link>`, no `--as`.
- **Ana's phone** is Playwright's WebKit as an iPhone (`web/tests/page.mjs`)
  opening Ana's device link on the page.
- **Bo** joins by Sam's link as `Bo`: the one other person, to show the
  shelf's limit is one.
- Each of Ana's computers asks each of Sam's lends through its own lend
  origin, as a page there would (`<label>.localhost`, its `Host`), as Demo
  C's dry run's `name.py` asked Ana's Navidrome. The phone opens each lend
  from the page's list.
- Then Sam removes Ana: `svrngs remove Ana --group devices`.

## Predictions

1. **Each added by a device link, no name typed.** Ana's `link --device`
   prints a link that says it adds another computer or phone of hers, to
   send to nobody else. The laptop's `join` prints "You're in devices, as
   Ana." with no `--as`, within 60 s. The phone is never asked a name: it
   reads "Adding this phone to devices as Ana’s…", then "You’re in devices,
   as Ana", within 60 s of opening the link. Sam's register lists five
   keys — Sam's, Bo's and Ana's three — and Ana's three share one
   `person_key`, Ana's computer's key.
2. **One account in each app across the three.** The shelf is told one
   `x-svrngs-key` (Ana's computer's) and one `x-svrngs-user`
   (`Ana-<its first 16 hex>`) from all three devices. Jellyfin's users after
   are `sam` and that one name (and Bo's, if Bo asked it; Bo does not);
   Navidrome's are `sam` and that one name.
3. **Asked once by the sign-in.** The middleware's log has one "the door
   asks who a person is to Jellyfin" for Ana's person key, however many
   requests her three devices make.
4. **Counted once by a lend's limit.** The shelf lets in each of Ana's
   three devices in turn within its 30 s window; Bo, asking then, is
   refused 503 `full`, "shelf is lent to 1 person at once, and 1 is using it
   now: …".
5. **Removed together.** `svrngs remove Ana --group devices` says Ana is no
   longer in devices; within 30 s each of Ana's two computers asking the
   shelf is refused 403 `removed`, and the phone, opening the page again,
   reads "You are no longer in devices." Sam's register lists Sam's and
   Bo's keys alone.

A run that cannot finish a step is could-not-judge for it, never a pass
(AGENTS.md 6).
