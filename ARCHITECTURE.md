# Architecture

The map of svrngs as it is built today. The design it is built toward is
`docs/DESIGN.md`. This file says what exists, and it changes in the same commit
as what it describes.

## Bird's eye

A group's state is one journal: a grow-only set of signed acts in the group's
directory. Everything else either computes from it or writes to it through one
writer. In E0 (DESIGN D7) that is one `svrngs` process on one computer, with a
chat bridge beside it:

```
Discord ──gateway──▶ bridge/ (node) ──loopback HTTP + token──▶ svrngs host ──▶ journal actor ──▶ groups/<id>/journal.jsonl
                     discord.mjs ▶ bridge.mjs ▶ host.mjs        bridge.rs        journal.rs        store.rs
                                                                                  │      │
                                                                   svrngs-core (pure)    runner.rs ──▶ svrngs-engine (QuickJS)
                                                                                                        └─ apps/expenses/expenses.js
```

## Code map

**`crates/svrngs-core`**: the record (DESIGN C1), sans-IO. It never touches a
socket, a disk, a clock or a key, and `scripts/lean_check.py` refuses an I/O
dependency or `async` in it.
- `chain.rs`: an act on the wire. Its id is `H(domain ‖ actor ‖ seq ‖ prev ‖ ts ‖ H(body))`
  and its signature is over the id. `VerifiedAct::parse` is the one way bytes
  become an act. The act kinds are a closed enum: `admit`, `record`, `correct`
  and `guest`.
- `ingest.rs`: `Journal::decide` judges an act and `Journal::apply` holds only
  what `decide` passed. `Journal` also keeps who stands, each key's head, the
  interactions the bridge has taken, and the guest book.
- `guests.rs`: the guest book, which holds the chat speakers the bridge acts
  for. Each entry is keyed by the platform's user id, and only Bridge-role
  acts write it.
- `admit.rs`: `fold` orders acts by `(ts, actor, seq, id)` and works out what
  a `correct` voids.

**`crates/svrngs-engine`**: the fold engine (DESIGN C4), synchronous and
with no I/O dependency. `Engine` is the one way the host reaches an app's
exports — `fold_segment`, `propose`, `say`, `due` — JSON in and out.
- `quickjs.rs`: `QuickJs`, QuickJS (rquickjs) embedded in the host. Every
  call gets a fresh runtime, built without `Date`, `WeakRef`,
  `FinalizationRegistry` or typed arrays, and with a memory budget kept by its
  allocator and a fuel budget counted in QuickJS's interrupt polls.
- `driver.js`: runs before the app's module. It removes `Math.random` and the
  locale methods, runs the fold's loop, and refuses a state that is not exactly
  JSON (`Breach::NotJson`).

**`crates/svrngs`**: the command and the host process. It is the only crate
allowed `anyhow`.
- `home.rs`: the one accessor for each path under `~/.svrngs`: `groups/<id>/`,
  `keys/<id>.json` and `secrets/bridge-token`. It also starts a group: the
  genesis names the Person key, and that key's first act admits the Bridge key.
- `store.rs`: the group's directory, `genesis.json` and `journal.jsonl`.
  Opening it re-ingests every line.
- `journal.rs`: the journal actor. One thread owns the directory, the
  `Journal` and the `Runner`, and everything else holds a `JournalHandle` and
  sends it messages. Every act it writes is signed with the Bridge key.
- `runner.rs`: the handler runner. It holds the built-in apps (their manifests
  and modules, from `apps/`, compiled in), folds an app's acts through the
  engine, and parses what `propose`, `say` and `due` answer into types —
  `Proposal`, `Reply`, `DueItem` — before anything is written or posted.
- `bridge.rs`: the loopback surface the bridge calls. It binds to loopback
  only and requires the capability token. `POST /append` takes an app's op on
  behalf of a speaker, `POST /guest` takes a guest-book event, `GET /group`
  returns the group's name and whether anyone has entered it, `GET /commands`
  lists the apps' commands, `POST /run` answers a command or a tap through the
  runner, and `POST /due` returns the reminders due now.

**`apps/expenses`**: the reference app for a shared record (DESIGN D0a).
`expenses.js` exports `init`, `reduce`, `propose`, `say` and `due`;
`manifest.json` declares its commands (`/paid`, `/owe`, `/settle`).
`laws.test.mjs` is the five-laws bar and `expenses.test.mjs` the arithmetic.

**`bridge/`**: the chat bridge (DESIGN C5), in Node, because discord.js is.
- `bridge.mjs`: the core. It sits behind the adapter's five calls:
  `connect({joined, left, input})`, `post`, `reply`, `commands` and `close`.
  It writes the guest book, welcomes arrivals, greets a group no one has
  entered yet, sends every command and tap to the host's runner and replies
  with what comes back, posts what is due every hour, and tells a person in the
  chat when their write failed or when they have been removed.
- `discord.mjs`: the one Discord-specific file, using discord.js 14 on the
  gateway, so the bridge only dials out. Its speaker ids are
  `discord:<user id>`. It registers the apps' commands as slash commands,
  gives a person option to the host as a speaker id, and suppresses mentions in
  everything it posts except the person a reminder names.
  The application needs the Server Members and Message Content intents.
- `host.mjs`: the client for `bridge.rs`.
- `words.json`: everything the bot says, in plain words (DESIGN D0).
- `main.mjs`: reads its five settings from the environment and has no
  defaults, so each value keeps one home.
- `tests/`: the fake adapter (`fake.mjs`), the core's tests, and `harness.mjs`,
  which the host's Rust tests use to drive the bridge.

## A chat event, end to end

A ban in Discord reaches `discord.mjs` as `GuildBanAdd`, and the adapter calls
`left({speaker, name, why: 'banned'})`. `bridge.mjs` posts
`{speaker, name, event: 'banned'}` to `/guest`. The journal actor asks
`GuestBook::change`, gets `Removes`, signs a `guest` act and appends it to
`journal.jsonl` before answering. From then on, `POST /append` on behalf of
that speaker gets 403 `{removed: true}` from `JournalActor::append`, whether it
comes through the bot or is replayed directly. Only a `joined` lifts the
removal; being `seen` again does not.

## A tap, end to end

Sam types `/paid 60 pizza`. `discord.mjs` hands the bridge
`{kind: 'command', command: 'paid', options: {amount: '60', what: 'pizza'}}`,
and the bridge posts it to `/run` with Sam's speaker and the interaction id.
The journal actor checks the interaction is not taken and Sam is not removed,
and the runner folds Expenses' acts, routes the command to it by the manifest,
and asks `propose`, which answers `{op}`. The actor writes the act, and the
runner asks `say` about it: the text and a Confirm button whose id is
`expenses:confirm:<act id>`. When Ama taps it the same path writes a `share`,
and `say` answers who owes whom. A retried tap finds its interaction taken and
gets the same view back, with nothing written.

## Invariants, and where each is held

| Invariant | Held by |
|---|---|
| Bytes become an act only through `parse`, which checks the id and the signature. | `chain.rs`; `crates/svrngs-core/src/tests.rs` |
| One writer per group's directory. | `journal.rs`: the actor owns `GroupDir`; nothing else opens it to append |
| A chat interaction makes one act, across restarts. | `Journal::taken`; `crates/svrngs/tests/double_confirm.rs` |
| A speaker banned or gone from the chat has nothing written for them. | `GuestBook::may_speak` in `JournalActor::append`; `crates/svrngs/tests/removal_latency.rs` |
| The bridge never loads app code. | `crates/svrngs/tests/bridge_holds_no_app.rs` |
| Expenses holds the five laws and the sixth. | `apps/expenses/laws.test.mjs` |
| An app sees no clock, no randomness, no locale and no earlier call. | `QuickJs`; `crates/svrngs-engine/src/tests.rs` |
| What the bot posts is what the journal folds to; a reminder is posted once. | `crates/svrngs/tests/split_the_pizza.rs` |
| The core does no I/O; the repository stays under its size budget. | `scripts/lean_check.py` |

## Not built yet, said so

- **Installing an app.** Expenses is built into the host (`runner.rs`
  `BUILT_IN`); install acts and signed manifests are E3.1.
- **The wasm engine**, with metering inside its hashed bytes, fold segments
  and the engine pinned by SDK version (DESIGN C4, E3.3). The embedded QuickJS
  folds the whole record in one segment, and a stack overflow is still
  catchable inside it.
- **Undo from the chat.** The fold honours a `correct` act, but no command
  writes one yet.
- **A reminder the chat failed to post.** Its fulfilment is written before it
  is posted, so it is said in the bridge's log (`bridge:due_post_failed`) and
  not posted again.
- **Re-adding a removed person** other than by their rejoining the chat.
  DESIGN C5 says "until a person re-adds them". Today the only re-add is a
  `joined` event, which in Discord follows a moderator lifting the ban.
- **An embedded surface.** It is the fifth item in RING_ENTRY's adapter list;
  Discord has one, and nothing uses it yet.
- **A ban that happens while the bridge is down.** The bridge reads no ban
  list when it connects, so a ban made while it was offline does not reach the
  guest book until that person leaves or is banned again with the bridge
  running.
