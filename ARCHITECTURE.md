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
                                                                                     │
                                                                              svrngs-core (pure)
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

**`crates/svrngs`**: the command and the host process. It is the only crate
allowed `anyhow`.
- `home.rs`: the one accessor for each path under `~/.svrngs`: `groups/<id>/`,
  `keys/<id>.json` and `secrets/bridge-token`. It also starts a group: the
  genesis names the Person key, and that key's first act admits the Bridge key.
- `store.rs`: the group's directory, `genesis.json` and `journal.jsonl`.
  Opening it re-ingests every line.
- `journal.rs`: the journal actor. One thread owns the directory and the
  `Journal`, and everything else holds a `JournalHandle` and sends it messages.
  Every act it writes is signed with the Bridge key.
- `bridge.rs`: the loopback surface the bridge calls. It binds to loopback
  only and requires the capability token. `POST /append` takes an app's op on
  behalf of a speaker, `POST /guest` takes a guest-book event, and `GET /group`
  returns the group's name and whether anyone has entered it.

**`bridge/`**: the chat bridge (DESIGN C5), in Node, because discord.js is.
- `bridge.mjs`: the core. It sits behind the adapter's five calls:
  `connect({joined, left, input})`, `post`, `reply`, `commands` and `close`.
  It writes the guest book, welcomes arrivals, greets a group no one has
  entered yet, and tells a person in the chat when their write failed or when
  they have been removed.
- `discord.mjs`: the one Discord-specific file, using discord.js 14 on the
  gateway, so the bridge only dials out. Its speaker ids are
  `discord:<user id>`, and mentions are suppressed in everything it posts.
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

## Invariants, and where each is held

| Invariant | Held by |
|---|---|
| Bytes become an act only through `parse`, which checks the id and the signature. | `chain.rs`; `crates/svrngs-core/src/tests.rs` |
| One writer per group's directory. | `journal.rs`: the actor owns `GroupDir`; nothing else opens it to append |
| A chat interaction makes one act, across restarts. | `Journal::taken`; `crates/svrngs/tests/double_confirm.rs` |
| A speaker banned or gone from the chat has nothing written for them. | `GuestBook::may_speak` in `JournalActor::append`; `crates/svrngs/tests/removal_latency.rs` |
| The bridge never loads app code. | `crates/svrngs/tests/bridge_holds_no_app.rs` |
| The core does no I/O; the repository stays under its size budget. | `scripts/lean_check.py` |

## Not built yet, said so

- **The handler runner** (E0.3). Commands and taps reach the bridge, and for
  now they only enter their speaker in the guest book.
- **Re-adding a removed person** other than by their rejoining the chat.
  DESIGN C5 says "until a person re-adds them". Today the only re-add is a
  `joined` event, which in Discord follows a moderator lifting the ban.
- **An embedded surface.** It is the fifth item in RING_ENTRY's adapter list;
  Discord has one, and nothing uses it yet.
- **A ban that happens while the bridge is down.** The bridge reads no ban
  list when it connects, so a ban made while it was offline does not reach the
  guest book until that person leaves or is banned again with the bridge
  running.
