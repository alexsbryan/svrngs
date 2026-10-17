# svrngs, structurally: where it can be smaller (2026-10-15, at 5bf9941)

The operator's decisions on it are in `CHARTER.md` here, and its rows are
in `STATE.md`.

The audit agent read the code and ran nothing: no build, no test. Counts use
`scripts/lean_check.py`'s rules on an archive of 5bf9941, which has 16,875 code
lines, 3,149 of them in svrngs-core. About 800 lines can go, roughly 175 of them
in the core, and no feature in use is lost. Two moves matter more than their
line counts:

- **The localhost bridge.** Through it, any program on the computer can reach
  every friend's lent apps as this person.
- **The view.** It is signed into every act and read by nothing.

## 1. The map

| Part | Lines | What it must do | Lines that could go |
|---|---:|---|---:|
| The record: core | 3,149 | Order acts, decide who is in, compare copies | ~150–200 |
| The record: runtime | 1,072 | Keep one writer per journal | ~150 |
| Sync and the live lane | 830 | Exchange what each copy lacks | ~30 |
| The record API for apps | 858 | Serve `/v1/rail/*` and `/v1/{groups,lends,sign}` | ~15 |
| Reach (transport) | 764 | Dial a member by key and speak HTTP to its door | ~240 |
| Joining and groups | 782 | Ask, redeem, keep, link, remove | counted under the command |
| The door and lends | 2,356 | Carry a request to a port | — |
| Pages and origins | 898 | Serve the home page, each lend at its own origin, and the room | — |
| Carried apps and folders | 1,452 | The features you chose | ~10 |
| Node wiring | 703 | Start everything | ~35 |
| The phone's page | 1,029 JS + 605 wasm | Join, dial, run each lend in a worker | ~40 |
| The command | 1,750 | Parse, word, start | ~60–100 |

The 5,979 forked lines still carry about 4,000 comment lines of upstream
history. At least 98 of them name the inference daemon, ggml's rpc-server,
corpus-engine, "doctor" or "netops". That is a cost to the reader, not to the
budget.

## 2. Opportunities, ranked by lines saved times confidence

### 1. Speak HTTP on the iroh stream and drop the localhost bridge (~240 net, high confidence)

**How it works today.** To reach another member's computer, the node opens a
listener on `127.0.0.1` for each peer:

- the listener: `guest.rs:252` `HttpBridge`; its accept loop is at `:307-450`
  and its pump at `:516`;
- it is cached for the node's whole life (`iroh.rs:133`);
- its port is derived from the peer's key, in 20000–32767 (`iroh.rs:363`);
- callers reach it with reqwest: `ring_sync.rs:428`, `node.rs:756`,
  `live.rs:120`, `blob.rs:155`, `removal.rs:202`, `origin.rs:249` and
  `redeem.rs:355`.

Upstream built the bridge for ggml's rpc-server (`guest.rs:471-480`) and named
the direct way its upgrade path (`iroh.rs:21-24`). svrngs already speaks HTTP
on the stream on the server side (`iroh.rs:546`) and in the phone's page
(`web/src/http.rs`).

**Why it matters beyond lines.** The bridge is a byte pump with no check on who
connects.

- The friend's door receives its requests as this computer's key. Unless the
  request carries a pass, the door carries it as this person
  (`door.rs:989-1003`, `Ok(None)`; `lend_for` at `:1007`, `Who::Member`,
  `app: None`).
- That reaches every lend and `/choices/{lend}` (`door.rs:330-337`), and it
  skips the per-app scoping that `origin.rs:20-28` keeps.
- `api.rs:133` (`not_from_a_page`) refuses browser pages for the record API.
  The bridge has no such check.
- Any program on this computer can certainly use it. Whether a browser page can
  depends on the browser's local-network protections.
- The test that would show it: in a two-node group, a plain TCP client on A
  sends `GET /lend/<B's lend>/` to A's bridge port and gets B's service
  answering as A's person.

**Two more gains.**

- One dial instead of two. "Not admitted" can't be seen through the bridge, so
  the node dials a second time to learn it (`iroh.rs:325-348`, called at
  `node.rs:770`). A direct stream reports the close reason on the first dial.
- reqwest 0.12 leaves the product (`svrngs-node/Cargo.toml:50-52`). Hyper is
  already the node's client to services (`door.rs:1169`).

**Lines.**

- Removed, about 340:
  - `guest.rs:244-550` (201);
  - `iroh.rs`'s cache, `bridge_for`, `base_url` and the preferred port (85);
  - `closes_unadmitted` (23);
  - the reqwest client (~15);
  - glue at each call site (~20).
- Added, about 50: one module that dials within the cap and sends a request on
  a bi-stream.

**How to land it.**

- First a move: the request helper, with no caller changed. Then each caller in
  its own commit.
- The test that goes red first: no loopback port on this computer reaches a
  friend's door.

**Risk.**

- reqwest's 10 s timeout (`ring_sync.rs:790`) must be kept.
- WebSockets through `origin.rs:266-300` must work on a stream. The page's
  `socket()` already does this.

**Collides with** `door.rs` (`send_to`), which v1ui-polish is editing. Schedule
it after polish.

### 2. Retire the seal and its compaction (~250–320, ~130–160 of them core; medium confidence; your call)

**What it does today.** An app may append `{"op":"seal"}` (`rail.rs:233-239`).
That retires its author's earlier acts on every copy the seal reaches. The
pieces:

- the journal's seal and compaction (`journal.rs:455`, `:559`);
- a prune after every exchange (`ring_sync.rs:566`);
- `process.rs`, whole;
- sealed floors in sync (`sync.rs:127-155`);
- seals and floors in admission (`admit.rs:368-402`, `:675-702`).

No app in `apps/` seals.

**What your drafts already say.** svrn-docs `A_RECORD_FOR_YEARS.md` §2 calls
the seal the wrong cleanup. `WHO_KNOWS_WHOM.md` §3 closes the app's route to
it. `A_RECORD_FOR_YEARS.md` §6 question 2 asks whether copies should stop
honouring a seal.

**The move.** Remove it rather than fence it. Old seal lines are still read, as
acts that do nothing. The original stays in commonwealth-ai.

**Risk.** The floor runs through `sync.rs`'s window of missing ops
(`:176-260`). Read that before cutting. The forked tests in `tests_sealing.rs`
go with it.

### 3. The command asks the running node, for every verb but the bare run (~60–100)

**What it does today.** After install, the running svrngs holds the data dir.
Four verbs refuse with "svrngs is already running on this computer… stop it,
then run this again" (`cli.rs:533`, through `claim` at `:671`):

- `new`
- `link`
- `join`
- `room`

`share`, `remove` and `handover` already work by asking the node
(`cli.rs:868`).

**Two makers each, which already differ.**

| Verb | The command's maker | The node's maker |
|---|---|---|
| `new` | `cli.rs:1675` | `group.rs:246` `Asking::found` |
| `link` | `cli.rs:1722-1800` | `invite.rs:152` `Host::link` |

- The relay differs. `svrngs new` keeps a relay only for `--relay`
  (`cli.rs:1457`), while the page keeps the relay reached now unless it is n0's
  (`group.rs:315-319`).
- They agree only while n0 is the default. When `SHIPPED` flips, they part.

**The move.**

- `join_group` (`cli.rs:1531-1611`) moves into the node, as a move.
- `new`, `link`, `join` and `room` become asks of the running node.
- The home page can then join another group from a link.
- install.sh no longer stops and restarts svrngs around the join
  (`install.sh:95-97`, `:118-121`).

**When.** Before groups-leave, so `svrngs leave` is written as an ask from the
start.

### 4. The core's single-use traits (~30 core lines, ~55 if the seal stays; 0.7)

- `Projection` (`project.rs:40`) has two implementations, and one is just
  `membership().clone()`. Both become inherent methods on `Admission`.
- `ProcessManager` (`process.rs:43`) has one implementation, `Compact`. It goes
  with #2, or becomes one function if the seal stays.
- `RingSigner` (`lib.rs:818-835`) has one implementation, `SigningKey`, which
  can replace it.

The acts are private to the core already (`project.rs:52-57`), so the traits'
sealing guards nothing extra. These lines are worth more than most: the core
sits at 3,149 of 3,150, and groups wants +40.

### 5. One version check instead of compatibility per field (~25–40; 0.5; your call)

- **What it does today.** "A computer before X" branches sit in
  `web/app.js:354`, `:360`, `:377-380` and `:384`; `question.js:136-139`; three
  serde defaults in `lend.rs:298-316`; and `live.rs:169`.
- **The move.** The people answer carries one version, and the page says once
  that a computer runs an older svrngs.
- **Your question.** What is the oldest build a page still has to talk to?

### 6. One seed file per group (~25–30; 0.5)

- **What it does today.** Each group keeps `seed.json` and `roster.json`,
  written together (`group.rs:171-189`). The roster is just `Seed::roster()`
  (`seed.rs:75`), and the runtime reads it as "the seed"
  (`journal.rs:253-258`, `:665-694`).
- **The move.** The runtime reads `seed.json` and derives the roster.
- **When.** Inside groups-keys.

### 7. One rule for what an app reads and writes (~10–20; 0.5)

- **What it does today.** `Append` decides what an app may write
  (`record.rs:32`). The log answers everything (`rail.rs:380-389`).
  `carried.rs:225-262` scopes an app's acts by indexing JSON strings
  (`o["payload"]["kind"]`).
- **The move.** One function, given a kind prefix, that all three ask.
- **When.** After v1ui-polish, which is editing `carried.rs`.

### 8. One state for the node's routers (~30–40; 0.4)

- **What it does today.** The node builds six overlapping state structs
  (`node.rs:237-375`): `rail` is held five times and `nudge` three. Accessors
  fill `node.rs:377-455` and `:504-554`.
- **The move.** One `Arc` of the node's handles, with axum's `FromRef` handing
  each router its part.

### 9. Stop signing the view into every act (~3–5 lines; many bytes)

- **What it does today.** Every commit computes a digest of everything held
  (`command.rs:235`) and signs it into each act (`lib.rs:654`). Only signature
  checking reads it (`sig.rs:181`).
- **Why it can stop.** It existed to weigh cuts, and `Remove` now carries its
  own `through` (`lib.rs:640-641`). DESIGN said it "goes with U7"
  (`docs/DESIGN.md:2393`), and U7 has landed. The field is optional, so nothing
  on the wire changes.
- **What it saves.** It is 25–39% of journal bytes, measured on journals left
  in `/private/tmp`. Stopping it also takes one full pass over the record off
  every write.

### 10. Small ones (~50)

- **The page's dial entry points.** `web/src/lib.rs:305-434` has eight around
  one `reach`, and `connect` is called only by tests.
- **`asleep` passed as text.** The wasm writes `"asleep: …"`
  (`web/src/lib.rs:258`), and the worker tests for it with `/asleep/`
  (`lend.js:345`). A typed refusal would replace it.
- **Upstream's proxy and egress logging** (`guest.rs:157-215`).
- **The sync loop over a one-item list** (`ring_sync.rs:427-470`), which goes
  with #1.

## 3. Draft rows: an `arch` queue after v1ui-polish, with arch-command before groups-leave

- [ ] arch-view — depends [] — (operator first: A_RECORD_FOR_YEARS §6 q1) a commit stops signing its view (`command.rs:235`, `lib.rs:654`); old acts verify unchanged — check: red first (plant: the view stamped), a new line holds no `view` and is the same size at 2 and 25 members; lean_check; check.sh once
- [ ] arch-peer — depends [v1ui-polish] — (a) a move: `send_to` over any stream, plus a dial-and-request helper on a bi-stream (b) each caller moved, one commit each (`ring_sync.rs:428`, `node.rs:756`, `live.rs:120`, `blob.rs:155`, `removal.rs:202`, `origin.rs:249`, `redeem.rs:355`); `closes_unadmitted` replaced by the stream's close reason (c) the bridge goes (`guest.rs:244-550`, `iroh.rs:133-163/236-318/356-373`), and reqwest 0.12 with it — check: red first (plant: the bridge as at HEAD), two nodes, and no loopback port on A reaches B's door; the callers' tests unchanged; the cap lowered where (c) lands
- [ ] arch-seal — depends [] — (operator first: A_RECORD_FOR_YEARS §6 q2) (a) an app's seal refused as `NotAnAppAct` (b) a seal held retires nothing: floors, `Retire`, compaction and prune removed; old seal lines still read; DESIGN §5 and §9 rewritten in place — check: WHO_KNOWS_WHOM bar 10; red first (plant: copies honour a seal), a seal another member signs retires nothing here; a HEAD build and this one stay in step; both caps lowered
- [ ] arch-command — depends [v1ui-polish] — (a) a move: `join_group` into the node (b) `new`, `link`, `join` and `room` ask the running node and start one only when none runs; `Asking::found` the only relay rule; the home page joins another group from a link; install.sh's stop and start around the join goes — check: red first (plant: the command claims the data dir), with svrngs running, `svrngs link` prints a link and `svrngs new` keeps the page's relay
- [ ] arch-core — depends [] — moves, behaviour kept: `Projection` → `Admission::membership()`/`invites()`; `RingSigner` → `SigningKey`; `ProcessManager` → one function unless arch-seal removed it — check: the core's tests unchanged; the `compile_fail` privacy test kept; the core's cap lowered
- [ ] arch-version — depends [v1ui-polish] — (operator first: the oldest build) one version in the people answer, and the per-field compatibility goes — check: red first, a browser test where the page says once that a computer runs an older svrngs
- [ ] arch-app-reads — depends [arch-seal or WHO_KNOWS_WHOM §3's row] — one core function, with an optional kind prefix, asked by append, the log and `carried.rs`; no `o["…"]` indexing left in `carried.rs` — check: the app laws and canaries green; red first (plant: the carried route reads another kind's act)
- [ ] (inside groups-keys) the runtime reads `seed.json`, and `roster.json` is no longer written

## 4. Looked at, nothing to cut

- **The door.** Every kind of caller runs through one `door::lend_for`
  (`door.rs:1007`).
- **The lists of lends.** They all ask `Asking::lent` and `lends_in`.
- **The two pages.** They share `question.js` and `look.css`.
- **The static secret and the sign-in helper.** They are one `Signing` enum
  already (`door/sign_in.rs:155`).
- **The command's parser.** Dense rather than bloated.
- **The sync round.** It already dials every member at once
  (`ring_sync.rs:394`).
- **`Oplog`'s generic tenant.** Small, and not worth the churn.
- **Features you chose, kept:**
  - folders and blobs;
  - the live lane;
  - the room;
  - scoped use;
  - the parked own relay (`crates/svrngs/src/relay.rs`, 142 lines).
- **`membership.rs` and `admit.rs`.** Nothing structural beyond the seal and
  the view.

## 5. Not judged

- **The bridge's exposure** is from reading. arch-peer's red test decides it.
- **The seal's line count** is from reading, not a build.
- **The tests** are 27,082 lines and uncounted by the budget. The browser suite
  (2,716 + 1,147 lines) cost three check.sh runs today over misses of 9 and
  180 ms. #168's single driver is where that suite's own economy starts.
