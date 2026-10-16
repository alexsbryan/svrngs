# Grapevine

**First, what everyone in a group sees.** Replies to an ask are sealed:
only the person who asked and the person who offered can read them. But
here they travel as entries in each group's shared history, so everyone
in a group a reply passes through sees that a sealed message went by,
its size, its handle (which ask it answers) and its time. They never see
what it says. Every conversation on the page says so. This is the
app's one shortcut (svrn-docs `rings/current/GRAPEVINE_DEMO.md` §6); it
goes when svrngs carries notes between two computers (§9 there).

Grapevine lets you ask your groups for something, and lets the people in
them carry your ask to their other groups, one tap a group. Sam asks
1400 Page for a tile saw; Jo, in 1400 Page and Waller block, taps
**Carry to Waller block**; Kim, in Waller block, sees "A neighbour of
Jo’s needs: tile saw, Saturday — carried by Jo", and offers hers. The
carry names Jo and never Sam. Sam and Kim see each other's names only
once both have said yes. When Sam takes the ask back, Jo's grapevine
takes the carry back too.

It is a reference app: one way to do it, for you to copy and bend. It
needs nothing svrngs doesn't already give an app, and svrngs's code holds
none of its words (`nouns.txt`, `scripts/vocab_check.py`).

## Try it alone

From your copy of svrngs, with Node.js 20 or later, one line:

```
d=$(mktemp -d) && cp apps/grapevine/demo.json $d/town.json && echo "{\"file\": \"$d/town.json\", \"acting\": [\"Sam\", \"Jo\", \"Kim\"], \"data\": \"$d\"}" > $d/config.json && node apps/grapevine/server.mjs $d/config.json
```

It prints `{"port":…,"at":"http://127.0.0.1:…/","store":"file"}`. Open
that address: you choose to be Sam, Jo or Kim, all in one file on your
computer, and every page says it's a simulation. `demo.json` is the
town: 1400 Page with Sam and Jo, Waller block with Jo and Kim, and Kim's
tile saw listed. Nothing else is needed: no svrngs, no second computer.

To watch the whole scene without a browser: `node apps/grapevine/canary.mjs`.
It plays Sam, Jo and Kim through the same code, and prints each step's
time.

With svrngs, three people on one computer: `demo/grapevine.md`, every
line and every button. Its recorded run
(`demo/runs/grapevine/2026-10-16/README.md`) took 20.4 s from Sam's ask
to the ask gone from Kim's page, and found no sealed word in any file
but a person's own.

## Write a kind

A kind is what people ask for: a tool, a room, a question. You write one
file in `kinds/` and add its name to the list in `kinds.mjs`. You don't
write routing, handles, sealing, carrying or names; the app does all of
that, the same for every kind. `kinds/README.md` says how, in one page.

`kinds/rooms.js` is one written that way, in 24 lines: a place to stay,
with a date, a number of nights and a choice of who for. Its whole
change to the app was that file and one line:

```
 apps/grapevine/kinds.mjs      |  2 +-
 apps/grapevine/kinds/rooms.js | 24 ++++++++++++++++++++++++
```

The canary then carries a room ask one group over, with no line of the
canary changed.

## What's here

- `grapevine.js`: the fold, the one place anything is decided. From the
  person's groups' records and their own state file, it answers what they
  can see (`asks`), where they may carry it and why not elsewhere
  (`carriable`), their conversations, opened (`threads`), and what their
  computer must pass on or take back (`relay`). Pure; no I/O.
- `relay.mjs`: each person's loop, every 5 seconds: it passes each sealed
  message on to the next group, and takes back each carry whose ask was
  taken back. It keeps the person's state file, the one file holding a
  secret.
- `seal.mjs`: tweetnacl's `box` behind `seal` and `open`, a key pair per
  ask. `vendor/tweetnacl/` is tweetnacl 1.0.3, whole and pinned.
- `server.mjs`: the page and its routes. It answers only at its own
  address on this computer, to its own page, and only for the person this
  computer's svrngs says it is in each group.
- `store.mjs`: Events' store, unchanged: a group's record through svrngs,
  or a file of the app's own.
- `kinds.mjs`, `kinds/`: the kinds there are.
- The tests, which `scripts/check.sh` runs: `grapevine.test.mjs` and
  `laws.test.mjs` (each law caught on a planted change), `relay.test.mjs`,
  `who.test.mjs`, `page.test.mjs` (Chromium and WebKit), `canary.test.mjs`.

Under svrngs each person runs their own, on their own computer:
`{"record": "<their svrngs's api>", "data": "<a folder>"}` as its config.
It isn't lent to the group.
