# The examples ladder, from release v1.11.0 — one host, 2026-11-18

PLAN §1, "Examples, a ladder", unit 4. `PREDICTIONS.md` was committed first
(`20121649`). One host, one run: not a rate.

The binary was release v1.11.0's `svrngs-linux-x86_64.tar.gz`, its sha256 as
`SHA256SUMS` lists it (`svrngs 1.11.0 (08441174)`). Ana and Bo were each a
program with a HOME of their own, and Ana's steps were taken by the words
`docs/guide/examples.md` gives (`ana.mjs`, `bo.mjs`). The guide's fetch
lines and Spot's start line were read from the files, not retyped
(`drive.py`). Results: `data/responses/seen.json`.

    python3 drive.py

## The predictions

1. **Sandbox made, Money brought: passed. See how they’re made: failed.**
   1.11.0 was cut before the link (`howLink: 0`).
2. **Folder location names the folder: passed.** It holds `apps/Money/index.js`.
3. **A sentence changed shows on reopening: passed.** *Add what you paid*
   became *Add what you paid for* (`a-sandbox-brought.png`).
4. **`svrngs check` exits 0: passed.** It said *"Nothing in … apps/Money
   that apps can't load."*
5. **Copied into Elm, Bo sees Money: passed** (`b-money.png`).
6. **Node 24, `svrngs version` and the three lines: passed.** They left Ana
   in `svrngs-1.11.0/`, and `apps/spot/server.mjs` was there.
7. **Spot started and lent from Already running here, its header added:
   passed.** The page said *"svrngs adds authorization to each request your
   friends make to deepseek-flash."* (`a-lend-lent.png`, `a-lend-header.png`)
8. **Bo answered, the key reached the provider: passed.** *"Hello, Bo. This
   came from the provider through Ana’s account."* (`b-chat.png`)

## What it found

- **1.11.0 predates the ladder.** Its source has no `examples.md`, and
  Spot's README has no **Run it**: its line is the old one, a server put
  in the background and then `share`. The guide's *"Its README's Run it
  has the line"* is true only from the next release.
- **Already running here listed 14 programs here.** That's every program
  this computer runs, each line with its own **Its name:**. The guide
  didn't say the line to use is the one naming your port.

## What the driver got wrong first

Five tries stopped on the driver, not the guide:
- two fields named "Your name in it", and hidden copies of "Add an app";
- an open link used twice: it is spent once used, and lasts a minute;
- Spot's page typed into before it had asked who Bo is;
- the stand-in started before its key was set.

Those tries' outputs were not kept. This README is the sixth try.
