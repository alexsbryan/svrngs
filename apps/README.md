# Apps

None of these is svrngs. Each uses only what any app can
([DESIGN §3](../docs/DESIGN.md)), and is yours to copy and bend. How to use,
open up, run and change one: [Examples](../docs/guide/examples.md).

**Carried**: a page in each person's browser; the group keeps its data.
- On the shelf, one tap from the home page: `expenses/` (Money),
  `events/` (Events), `canon/` (House rules), `movie-night/` (Movie Night).
- Copied into a group's folder: `taste/` (Taste).

**Lent**: a program on your computer, opened by your group through svrngs.
`spot/` (a provider's model for a night, your key kept by svrngs) and
`doc/` (a document written together).

**Yours alone**: a program on your computer, for you, that asks svrngs.
`pantry/` (every friend's model behind one address) and `grapevine/` (asks
carried group to group).

**Kept by the group**: `give/`, which keeps an app on one member's computer
at a time.

**Waiting on svrngs**: `magnets/` (Magnets), a sentence of word magnets a
group shares, in a database svrngs doesn't give apps yet
([PLAN §1](../docs/PLAN.md)). Until then it plays against a stand-in:
`node apps/magnets/tests/standin.mjs`, and open the address it prints.

**Notes for a program you already run**: `jellyfin/`, `holyarrs/`.

**For your own program**: `client/svrngs.mjs`, the one way a program on
your computer asks svrngs as you, never sending svrngs's token. Copy it
beside your code and `import { ask } from './svrngs.mjs'`; pantry, Events
and Grapevine each carry a copy, byte for byte. A shell or Python script
asks through `svrngs ask <route> [<json>]` instead.

**For svrngs's own checks**: `web-conformance/`, `on-air/`, `tap/`,
`pinned.mjs`, and `client/standin.mjs`, a svrngs stand-in for an app's
tests.
