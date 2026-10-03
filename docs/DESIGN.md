# svrngs — the design

> **Normative for building, 2026-10-03.** svrngs is its own repository, standing
> apart from commonwealth-ai by operator direction of 2026-10-03, and this is the
> design it is built to. It moved here from commonwealth-ai's
> `docs/RING_SPEC.md` Part I, where Part II remains as the inventory of the ring
> code svrngs replaces. The design record — rationale, prior art and history —
> stays in commonwealth-ai's `docs/RING_APP_LIBRARY.md`, cited as LIB §n. A path
> in backticks that is not in this repository is in commonwealth-ai: code this
> design ports from, cites or replaces, never code it links.

## D0. What it must do well, and nothing else

**The pitch, which every person-facing surface is held to:** *Keep talking
where you talk; keep what your group owns — money, plans, files, the library —
on your own devices, not a platform's.*

**The product is the iOS of groups.** A runtime, an SDK, an app model with
permissions, and distribution, so that developers build the apps a group uses
(operator, 2026-10-03: "we want community to build the apps, we want to provide
the platform"). It runs on every device a person has. We build one **seed** per
primitive — the smallest app that proves it — and never the finished apps.

Two rules hold every surface to the pitch.

- **No universe to explain.** A person sees the group by its own name ("the
  house", "the party"), people by their names, apps by their names, and
  *phone*, *computer*, *the chat*. Internal words — ring, member, guest, key,
  sync, journal, host, relay — appear in nothing a person reads (D9
  plain-words). The app on the home screen is **svrngs** — svrn and rings — for
  now (D11).
- **The most general surface, the fewest features.** Everything a group does is
  an app on D0a, so the system learns by adding apps, never by adding platform.

### D0a. The SDK — what every app gets, and nothing more

An app is a static bundle with a signed manifest and a module of pure exports.
Its screen runs sandboxed and reaches only `window.group`:

| Term | What it gives | Seed |
|---|---|---|
| `group.me`, `group.people` | who is asking, and who is here — each person tagged by how they arrived (phone or chat) | every seed |
| `group.add(payload, {to})` | write to the group's record; `to` seals it so only those people can read it | Expenses, Tap |
| `group.undo(id, replacement)` | correct an earlier act of this app, never erasing it | Expenses |
| `group.state()` | the app's state — its exported reducer folded over its own acts by the runtime, the state of record — delivered as changes: `{value, complete, asOf}`; a screen may also receive this app's acts in the order they arrive on this device and keep its own view — a cache, never the state of record (the Doc's editor document) | every seed |
| `group.live` | an ephemeral channel, within this app, to the people who have this group open right now — cursors, presence; checked against the register, never written to the record; a declared capability | Doc |
| `group.files.put/get`, and `add(payload, {files})` | bytes beside the record, named by hash in the act's envelope (so the runtime knows which act holds which file), within the group's limits | Album |
| `group.start(name, people)` | a new group: an invite act in this group; each person accepts when they next open the app, on that device, completing when someone already in the new group next syncs; this app comes installed; a person with no key (chat only) is refused by name | Event |
| `group.ask(kind, input \| inputs)` | work for the group's computers — a model's answer, or a batch over many inputs — submitted as acts, leased by every computer that lends the kind, and answered by acts signed by the computer that ran it, naming the model; the reducer folds answers like any act | Ask |
| `localStorage` | private storage — a preloaded, capped shim over the shell — on this device, for this app in this group only | Tap |
| `group.open(name)` | a service someone in the group lends (the Library), opened in its own view through the member door | Library |
| `group.request(name, path)` | read a lent service's API as this person — GET only, by construction — through the shell's port and the member door, never the frame's network | Films |
| `group.play(name, paths, {on: here \| screen})` | the shell's own player for a lent service's media — an HLS playlist and a plain file — here, or on a TV in the room; the app sends play, pause and seek and sees state, never the TV or the URL | Films |
| exports `init`, `reduce`, `propose`, `say`, `due` | pure functions the runtime calls: the app's state from its acts; a proposal from a chat command; a text view; what is due at a time | every seed; Expenses for `due` |

Terms arrive with the seed that earns them (D7); SDK v1 is what exists when the
first outside developer starts (E3).

**The manifest**, signed by the app's developer key, names the app, its version,
the SDK version it targets — which pins the fold engine: the QuickJS-wasm bytes
by hash — metering, memory maximum, stack size and the fold's own loop all
inside those bytes — and the segment size K and per-segment fuel budget — the kinds of act it writes, the earlier versions'
acts it skips (cumulative: a version that drops a skip is flagged), its chat
command schema (the one place commands are declared), and **the capabilities it
uses** — sealed payloads, files, private storage, the live channel, starting
groups, lent services and their APIs, playing on a TV, asking the group's
computers, the chat. The runtime refuses any capability not declared, and the
group sees each one as a plain sentence when the app is installed.

**Install and update.** Anyone in the group may install an app; everyone sees
who did. Every install act — the first one too — carries the manifest signed by
the app's lineage key (its developer key and name), the version and the bundle
hash, and every runtime verifies it before running the app; the bundle travels
as a file. A different key is a different app. Before an update runs, the
runtime folds the group's acts under both versions and shows each person, on
their next open, what would change; an update that adds a capability asks each
person again. Each person consents
the first time they open an app, and the app's frame shows its name, version
and who installed it. Reverting is installing the previous version; it voids
nothing — acts a newer version wrote stay in the record, and people see "some
entries need a newer version of Expenses"; a later version may skip acts by their
version stamp, which its preview shows.

**What an app sees.** Every act is stamped by the runtime with the app's lineage
and version. An app reads and corrects only its own lineage's acts, plus the
people list. It reports "N acts from a newer version" rather than guess.

**Classes of app, each proven by one reference app.** The platform is proven by
classes, not by apps: each class below is a capability a developer can build a
whole family of apps on, and its reference app is the smallest one that uses it
end to end — the worked example the next developer copies and bends.

| Class | Proves | SDK terms | Reference app |
|---|---|---|---|
| shared record | state every copy folds the same, corrected without erasure, with reminders | `add`, `undo`, `state`, `due` | Expenses |
| lent services | someone's running service, used by the group through its own screen, its API, or a TV | `open`, `request`, `play` | Library, Films |
| group compute | work run on the group's own computers — distributed inference — with answers kept in the record | `ask` | Ask |
| files | bytes beside the record, fetched on demand, erased when voided | `files` | Album |
| spin-offs | a new group started from this one | `start` | Event |
| live collaboration | people working on one thing at once, presence never written | `live`, a screen's own cache | Doc |
| private coordination | inputs only their author sees, and payloads only named people can read | seals, `localStorage` | Tap |

Expenses and Doc are built and move onto the SDK. Ask answers a question about
the group's record with a model one of its members lends, citing the acts it
used, and runs a batch — a caption for every photo in an album — fanned out
across every computer in the group that lends a model.

**WHY_THEY_JOIN on this surface:** the post-party question is Tap — sealed
payloads that name no recipient and are padded to one size, uniform
participation, a two-party AND in Tap's reviewed library, and per-event keys in
private storage (the two-tap cap is a social rule, not one code can enforce
over hidden inputs). The address is the chat plus `group.start`. Money is the
record plus `due` for reminders without a model — the bridge posts and
fulfils them; the app, when opened, shows the person named what is due to them
and writes nothing, so in a phones-only group a reminder reaches only someone who
opens the app. Free things are lent services.
Context is one record per group and storage scoped to a group. The grapevine
needs acts across groups, a person-scoped store and the local model — additive
in a later SDK version, precluded by nothing here.

### D0b. Six jobs

- **J1 Keep the group's state** in an order every copy computes the same way,
  correctable without erasure. No platform holds it.
- **J2 Let anyone in the group's chat act on it**, with no account and no key.
- **J3 Let a person act from their phone, anywhere, with their own key.**
- **J4 Let a person lend a service they already run.**
- **J5 Tell the truth about absence** — a closed group, a missing act, the age
  of a copy, a partial balance.
- **J6 Let a developer build, install and run an app** on the SDK, on every
  device, without changing the platform.
- **J7 Make leaving cheap.** A group's whole state is a plain directory every
  member's device already holds. If a group goes wrong — its keeper lost,
  captured, frozen — its people refound it from any copy, within hours, with
  their apps, history, balances and files, and without whoever they leave out.
  Recovery inside a group is not attempted; leaving it together is the exit.

**Not goals.** Groups open to strangers. A service that keeps groups for people
(E0's test groups excepted, and said so). A device keeping a group its owner is
not in. Keeping someone's server-side app alive after they leave. A browser as a
client off the house WiFi. Push. Moving money. Recovering a group from inside
it — J7 is the exit. A node protocol published for other implementers, or agent kits, before
an outside builder asks.

## D1. The shape

Each group has exactly one durable shared thing — its journal, with the files
it names — held by the devices of the people in it. Everything else is a pure
function of the journal, a disposable cache, a stateless pipe, or someone
else's software lent through a door.

**One kind of node.** A phone, a laptop and an always-on box run the same node
core: a key, a full copy, the same ingest rule, the same sync, dialling and
accepting while running. They differ only in services, which follow from the
device:

| Service | Where |
|---|---|
| hold a copy, sign, sync both ways, run apps | any node while running |
| keep the group open | a node that is always on |
| be the keeper, sequencing removals | one Person key; the always-on node's when there is one |
| be the meeting point (its direct address in the invite) | an always-on node reachable from outside |
| lend a service | a node running that service |
| lend compute: run work units of the kinds it declares | a computer running the work — the svrn daemon serves `infer:v1` |
| the bridge, and running apps' `propose`, `say`, `due` for it | the always-on node holding the chat's token |

A group of phones alone is open while at least two of its apps are open. A
group with an always-on computer is open while that computer is.

## D2. Layers

| Layer | Owns | Durable state |
|---|---|---|
| L0 reach | iroh endpoints, port mapping, the project's relay (the only relay) | none |
| L1 record | canonical form, signatures, the ingest rule, order, void, gaps, the register; journals and files | the journal and its files |
| L2 app logic | apps' reducers and handlers, run by the runtime's sandbox | none |
| L3 doors | the member door to lent services; every group route checking its group's register | none |
| L4 nodes | the node, packaged for a computer or inside the app | a device's key, its copy, declared secrets for its services |
| L5 apps | bundles; lent services | lent services only, on the lender's device |

**Rule:** no layer above L1 holds state the group would lose if that component's
disk were wiped.

**Order** is decided once, in `admit`, by `(ts_unix, actor, seq, id)` on the
author's clock. Every signer stamps `ts_unix = max(now, its last, the ts of the
`Admit` that admitted it)`. Removal is sequenced by the group's keeper and
names the removed key's acts by hash, never by time.

## D2a. Code architecture

Held to `docs/principles/RUST_ARCH.md` and `docs/principles/ARCH_PRINCIPLES.md`,
pinned from commonwealth-ai and read by every agent here (`AGENTS.md`): **types
encode the domain, ownership encodes the architecture, I/O lives at the edge.**

**A sans-IO core.** `svrngs-core` is one deep crate with a narrow interface: a
pure, synchronous state machine over a group's journal, in `quinn-proto`'s
shape. `decide(&journal, msg) → event` and `apply(&mut journal, &event)` cover
ingest; `fold(&journal) → Fold` covers order, voids, the register and
completeness; `sync` returns what to send and in what order. `now` and
verification arrive as arguments; it never touches a socket, a disk, a clock or
a key. Its modules — `chain`, `ingest`, `register`, `admit`, `sync` — are
internal; tests drive it with golden journals and no async runtime.

**Types make the illegal states unrepresentable:**
- *Parse, don't validate.* Bytes from the network become a `VerifiedAct` only
  through `parse`, which checks the signature over the id and derives the id;
  `ingest` accepts nothing else. Ids, actors, persons and groups are newtypes.
- *Closed enums for the domain.* Act kinds, membership acts, ingest outcomes
  (`Held`, `OfferAgain(reason)`, `Refused(reason)`), standing, completeness,
  capabilities and screen protocols (`AirPlay`, `Cast`, `Dlna`) are enums.
- *Keeper acts are their own type.* `Remove` and `Keeper` are variants of a
  `KeeperAct` enum that parses only from the keeper's chain.
- *A redemption is built from its invite.* `Admit::redeeming(&invite, …)` copies
  the `Invite`'s person and role.

**Ownership is a tree.** A node owns one **journal actor** per group — a task
that owns the store and an `mpsc` receiver, behind a cheap cloneable handle —
and it is the one writer. Routes, the bridge and the runtime send it messages;
nothing shares the store through `Arc<Mutex<_>>`. Async lives only in the
shells.

| Crate | What it is | State |
|---|---|---|
| `svrngs-core` | the sans-IO state machine and fold above | none |
| `svrngs-node` | the journal actor and its store; iroh accept and dial; the group, claim and serve-to-a-screen routes; the member door as a `tower` stack — the library outside builders build on | the journal |
| `svrngs-engine` | the `Engine` trait — `fold_segment(state, acts) → (state, complete)` and the handler calls — with QuickJS as its first implementation | none |
| `svrngs` | the command and the host process: the handler runner, the bridge, `start`, `refound`, `app dev` | declared secrets |
| `app/` | the shell on iOS, Android and desktop | keys, in the keystore |
| `apps/`, `providers/` | reference apps as bundles; providers' declarations as data | none |

**Abstraction only where it is real.** A trait appears where a second
implementation exists or at a genuine boundary — the keystore, the fold engine
(which the operator wants swappable), a chat adapter when the second one is
written — and nowhere else. Errors are `thiserror` enums in the crates and
`anyhow` only in the command and the app.

**Classes made impossible, not guarded.** C1's three choices remove the
register's recurring holes by structure; the types above remove the
field-binding and wrong-chain cases.

**Lean, and kept lean.** The repository depends on no commonwealth-ai crate and
no sovereign crate — `scripts/lean_check.py` refuses either from `Cargo.lock` —
and
`svrngs-core` and `svrngs-engine` take no I/O dependency. Its size budget is set
before its first line: `svrngs-core` under 2,500 lines of code and the whole
repository under 12,000 at the end of E1, tests and bundles apart, raised only
by the operator with the reason in the commit. The crates, how a message flows
from an iroh stream through the journal actor to a fold, and the invariants
each owns are written down in `ARCHITECTURE.md`, rust-analyzer style.

## D2b. Domains

svrngs is the product people install and the platform developers build on, in
its own repository. It shares no code with commonwealth-ai and meets the rest of
the world only at the wire.

| Domain | What it is | Meets svrngs by |
|---|---|---|
| **svrngs** | the record and its rules, the node and its door, the runtime, the SDK, the app, the bridge, the reference apps | — |
| **apps** | `apps/<app>/` bundles and `providers/<provider>/` declarations; the only place app- or provider-specific code lives | the SDK |
| **sovereign (svrn)** | local AI on a member's computer | its OpenAI-compatible endpoint, called on loopback by that member's own node (C3a) |
| **lent services** | anything a member runs — Jellyfin, a svrn daemon's web UI | HTTP through the member door |
| **commonwealth-ai** | the mesh for one owner's machines, and the ring code svrngs replaces | nothing; its code is ported by citation when a unit needs it |

- **App-specific code has one home.** Expenses' fold is in `apps/expenses/`,
  never in `svrngs-core`. Jellyfin's viewer account, secret and refused paths
  are its provider declaration, never strings in the door. Discord is an adapter
  module, never in the bridge's core. A model belongs to the svrn daemon.
- **"svrngs" names the product.** A person reads svrngs, their group's name and
  apps' names; the code says group, act, register and keeper.
- **The mesh is not a group.** commonwealth-ai's mesh decides which machines
  belong to one owner; a group's membership is its register (D4).

## D3. Packaging and reach## D3. Packaging and reach

**The node** is `svrngs-node` over `svrngs-core`. It builds for iOS, Android,
macOS and Linux (D9 same-core), and it is published as a library: the iroh
client a builder uses for a group of people rather than a pair of keys.

- **On a computer:** the `svrngs` host process is the node plus the services the
  computer qualifies for — the bridge among them. A svrn daemon on the same
  computer is an optional neighbour for inference; it holds no group state and
  mounts no group route.
- **The app** is the node plus a shell (Tauri 2) plus the runtime, on iOS,
  Android and desktop alike.

**Keys and reach.** A person signs in each group with a key of that group's own
(RING_SPEC U26), so a cut in one group never touches another and the house
cannot link the party by signing key; someone in both groups can recognise the
same device by its endpoint, and no one else can see either register. Each device has one iroh endpoint key, carried
on every `Admit` as `endpoint` and co-signed, so the register is also the
address book: nodes dial a person's devices by endpoint. The endpoint key lives
in the keychain this-device-only without a biometric gate (iroh needs it in
process at launch); signing keys may ask for one. The invite carries the group
id, the starter's endpoint, the meeting point's direct address if there is one,
the project's relay, and an `Invite`'s id with the private half of its one-time
key.

The always-on box is the meeting point, with its port mapped by iroh. The
project's relay is the only relay: a small server running iroh's relay, seeing
only ciphertext, swappable by changing the invite, and proven non-essential for
direct paths by being turned off. Two phones on one WiFi meet through it; local
discovery would need Apple's multicast entitlement (D8).

## D4. State

| State | Lives on | Written by | If lost |
|---|---|---|---|
| journal, beginning with its genesis act | every device of the people in the group | their keys; the bridge's key for chat speakers | rebuild from any full copy |
| the group's directory: `genesis.json`, `journal.jsonl` (one signed act per line, append-only), `files/<hash>` — bundles are files | every device, readable and copyable with no tool; no key or private storage in it | the journal actor | it is the copy (J7) |
| files | fetched on demand; the always-on node fetches all; bytes kept only while an unvoided act names them in its envelope | `group.files.put` | refetch from any holder; else the app says it is missing |
| group id | `H(domain ‖ starter key ‖ nonce ‖ name ‖ H(legacy roster))`, the last term only for a migrated group; carried in the genesis act | the starter, once | — |
| a person's signing key per group | the device's keystore, this device only | the device | another device of theirs admits a new key; a cut key is never re-admitted |
| per-group encryption key | the device; its public half on `Admit`, co-signed | the device | rotate by an act its signing key signs |
| the device's endpoint key | the keychain, this device only | the device | a new endpoint, announced by an act each of its signing keys signs |
| the bridge's key | the always-on node, admitted with role Bridge | setup | cut and admit a new one; the guest book follows the role |
| declared secrets: chat token, capability token, channel map | the node running the bridge | the operator | re-issue |
| the guest book, keyed by platform user id | the journal | the bridge, from people who act in the chat | rebuild from the journal |
| private storage | the device, keyed by group and app lineage | the app | the app's own loss |
| installed apps | install acts; bundles as files | anyone in the group | the journal |

**Must not exist**, each a second decider or copy:

1. membership anywhere but the register — no roster file, no mesh, no list a
   door keeps of its own;
2. a waiting store for acts that cannot be held yet;
3. a membership view inside every act — the sync digest keeps heads, acts do
   not;
4. two fold traversals — the runtime owns the one;
5. two copies of an app's code.

## D5. Components and contracts## D5. Components and contracts

**C1 the record (`svrngs-core`).** Each part below is one module's contract (D2a).

*Three choices that make whole classes of edge cases impossible.* Nine rounds of
red-teaming kept finding holes in rules that defended assumptions the structure
did not need. These three remove the assumptions instead:

1. **The journal is a grow-only set of acts, linked like git commits.** Each act
   names `prev`, the id of its author's previous act; `seq` is its height. A copy
   holds an act only once it holds the act's `prev`, as git holds a commit only
   with its parent. Copies converge because set union does — there are no slots,
   holes or floors to reason about, and a withheld branch can never be spliced in.
2. **Forks void nothing.** Two acts by one key with the same `prev` are two acts,
   both kept and both folded, as if written one after the other. A reinstalled
   phone, a restored backup or a deliberate equivocation makes a second branch
   and changes nothing else — so there is no fork marker, copy bound, fork
   evidence or reinstall path.
3. **Removal, the one act that does not commute, has one decider.** Admitting,
   inviting and writing commute and stay decentralised. Each group has one
   **keeper** — a role held by one Person key, the starter's at first — and only
   the keeper's chain carries `Remove`s. Anyone may ask for a removal; the keeper
   sequences every valid request mechanically, in the order it receives them,
   naming the removed key's heads as it holds them. So removals have one order:
   no mutual removal, no cycle of removals, no race between removers, no cut to
   correct.

Compaction is cut in v1 (D8), so nothing deletes the history removal reads.

*Ids and the chain (`chain`).* An act's id is
`H(domain ‖ actor ‖ seq ‖ prev ‖ ts ‖ H(body))`, 32 bytes, and the signature is
over the id. The ring has its own constructor; `Op::new`'s short id is never an
act's id. Acts migrated from a legacy journal keep their legacy ids, so a
`Correct` still finds what it voids.

*The ingest rule (`ingest`)*, the same on every path:
1. the signature verifies over the id, and the id derives from the act — else
   refused;
2. the actor is bound: named in the genesis or by an `Admit` from a bound signer;
3. its `prev` is held, or it is the key's first act, and `seq` is `prev`'s plus
   one;
4. `ts_unix ≤ now + skew`, and `ts_unix` is at least its `prev`'s and the least
   among held `Admit`s naming the key.

An act failing 2-4 is not held and is offered again. Once a key's `Remove` is
held, ingest holds only that key's acts its heads reach.

*Admit (`admit`).* Order is `(ts_unix, actor, seq, id)`; `Correct` voids. A
removed key's acts count only where the `Remove`'s heads reach them by `prev`;
acts of a key whose every `Admit` fails count nowhere. Both are the **refusal
class**, separate from absence, so they never make a group incomplete. A head a
`Remove` names that is not yet held is absence.

*The register (`register`).* Membership acts, none of which can be voided:
- `Invite{redeem_key, role: Person | Bridge, via, person_id, expiry}` — what a
  link hands out; `redeem_key` is a one-time public key, and the link carries the
  act's id and its private half. `via` binds the redeeming device to a chat user
  id; `person_id` makes it another device of that person and counts only when the
  issuer is that person's key.
- `Admit{person_id, name, key, role, enc_key, endpoint, invite}` — signed by a
  standing key; co-signed by the admitted key, its device's endpoint key and,
  when it redeems an `Invite`, that `Invite`'s one-time key. Without `person_id`
  it mints a person whose id is the `Admit`'s own id. A redeeming `Admit` carries
  exactly its `Invite`'s `person_id` and `role`; otherwise naming an existing
  person needs that person's key as the signer. An `Admit` naming an unminted id
  is refused, and a key named by several `Admit`s belongs to the person of the one
  with the lowest id. So no one can become, or pre-claim, someone else.
- `RemoveRequest{key}` — any standing person's act, which the group sees.
- `Remove{key, heads, request}` — only in the keeper's chain, naming the removed
  key's heads as the keeper holds them when it sequences the request. A request
  whose signer an earlier `Remove` cut is not sequenced. A request to remove the
  keeper is sequenced by first handing the role to the requester.
- `Keeper{key}` — only in the keeper's chain: hands the role to another
  standing Person key; setup hands it to the always-on node when there is one.

An act of the register counts when its signer is standing at it: not removed,
or reached by the heads of the `Remove` that removed it. Rotating an `enc_key` or
`endpoint` is an act signed by the signing key and co-signed by the new endpoint
key. A Bridge-role key signs only `Invite`s with `role: Person` and `via` set. A
person with no usable key is admitted again as a new person, which the group
sees. A cut key is never re-admitted.

**Redeeming an invite.** A node holding a standing Person key signs the `Admit`
for a device that presents an `Invite`'s id and an `Admit` body co-signed by the
new key, the endpoint key and the one-time key — if the `Invite` is unexpired by
its clock and not already redeemed in its fold. The fold counts the `Admit` only
while the `Invite` counts. Two redemptions signed concurrently on different nodes
both count, and the group sees both.

**The line the register holds:** copies converge; outsiders cannot act; a
removed key can neither extend its standing nor admit anyone; no one can become
an existing person; removals have one order. Accepted, and said in D10: the
keeper is trusted to sequence and can delay or refuse a request (insider
capture, whose exit is refounding, J7); a removed key's acts the keeper had not received
when it sequenced the removal do not count; while the keeper's device is away,
removals wait, and if it is lost, removals stop — admitting and writing go on —
until the group refounds (J7).

*Genesis and continuation.* The genesis act carries only the id's preimage — the
starter's key, a nonce, the name — so it cannot be equivocated; everyone else
arrives by `Admit`. A node refuses a group until it holds a genesis that hashes
to the group id.

A genesis may **continue** an earlier group, and this one mechanism is both
refounding (J7) and migrating a group from before genesis. The preimage then
also names the earlier group's id, its heads as the founder holds them, and the
people carried over by their earlier `person_id`s. The new group folds the
earlier journal up to those heads — read-only history — before its own acts, so
every app's state, every balance and every file carries over unchanged, and
nothing after the heads does. A carried person joins by redeeming an invite with
an `Admit` co-signed by one of their keys from the earlier group, which proves
they are the same person; someone without one joins as a new person, and the
group sees it. The founder chooses who is carried over, and the people choose
whether to come: a group escapes a captor by not inviting them. A legacy roster
is the same thing with no earlier journal.

**C2 node core.** The journal store, files, and:
- **sync** over iroh, the one route between nodes. The acceptor admits the union
  of the standing endpoints in the registers the node holds, and every group route —
  sync, checkpoint, live — checks its own group's register. The sender sends in
  causal order: genesis; each act after its `prev` and after the `Admit` that
  binds its key; and it rotates the actor it starts from each chunk, choosing
  among acts the receiver can hold. Progress is proved by "each chunk's first act
  is holdable by the receiver". The digest lists each actor's heads, so only what
  is missing travels;
- **the claim route**, the only prefix the acceptor serves to an endpoint in no
  register: a new device redeems an `Invite` there (C1). The issuer's own device
  can always redeem its `Invite`; any other node can once the `Invite` has
  synced to it, so a link names the issuer's endpoint and the meeting point. The
  always-on node holds its owner's Person key in each group, since the Bridge
  key signs only `Invite`s. It
  is rate limited. An unstamped loopback caller is refused;
- **serve to a screen**, accepted by an always-on node only from devices of its
  own person: serve title T, for TV address A, through the member door;
- **`log`**, whose people are the register plus the guest book;
- on the node running the bridge, the **bridge append** on loopback:
  `{op, on_behalf_of}` with a capability token, signed by the Bridge-role key.
  Folds honour `on_behalf_of` only from a Bridge-role key.

**C3 member door.** `group.open(name)` opens a lent service in its own view, for
a person the group's register admits, tunnelled to the lender's node over iroh
(the forwarding body in `iroh_identity_forward.rs`), stripping client `x-mesh-*`
and injecting the verified ones. Video over the project's relay is capped, and
the app says so.

*A TV in the room.* Every way a TV takes a film from a phone — AirPlay (Apple TV
and the AirPlay 2 TVs from Samsung, LG, Sony, Vizio and Roku), Google Cast
(Chromecast, Google TV, Android TV) and DLNA's "play to" — ends with the TV
fetching an HTTP URL itself. That URL is the lingua franca, and the platform
makes exactly one: `group.play(..., {on: screen})` has a node on the TV's WiFi
serve one title, at a path no one can guess, over plain http on its real
address (never loopback, never through a resource loader), with CORS for Cast's
receiver, until playback stops; behind it is the member door's tunnel to the
lender. The phone is always the sender — the AirPlay route, the Cast session,
the DLNA control point — and the remote.

The URL's origin is the person's own always-on node when it is on the same WiFi,
so the phone can sleep; otherwise it is the phone. The phone hands the origin to
its own node over the group route below and finds it by the register's endpoint
and a direct dial, which brings iOS's Local Network alert (D10.12). With the
phone as the origin, AirPlay keeps playing through a lock under the app's
declared background mode for media playback, while Cast and DLNA stop when iOS
suspends the app about thirty seconds after locking — so the app says "keep
Films open — the film comes through this phone" before it starts. The URL serves
the one TV picked: by its address for Cast and DLNA; for AirPlay, which gives
the app no receiver address, the first device other than the phone to fetch it.
A route change mints a new URL.

The three protocols are adapters, as chat adapters are: the system route picker
for AirPlay, the Cast SDK's dialog (Bonjour; no multicast entitlement), and a
UPnP control point for DLNA — Android only, since SSDP on iOS needs Apple's
multicast entitlement (D8). The app gives an HLS playlist for AirPlay and Cast
and a plain file for DLNA, where seeking restarts the file at a time; Cast falls
back to the plain file if its receiver refuses plain-http HLS. For Jellyfin both
are the lender's own transcode endpoints, with the bitrate in the URL, capped to
the measured path, trickplay off, text subtitles as a WebVTT track and image
subtitles burned in, and a play session id per play, so one person's seek never
stops another's transcode. The lender's door counts its own concurrent plays
and refuses past the lender's limit. The TV needs no app, account, code or key.
A TV that takes no push — a PlayStation, a Fire TV Stick, a TV with none of the
three — is named as such, with "play here" offered instead.

*A lent service's credential never leaves the lender's node.* The lending
declaration names the secret itself, not only the header that carries it. The
door injects the header, refuses any request that carries the secret, asks the
service for uncompressed bodies, and replaces the secret wherever it appears in
a response body with a same-length mask — across reads and chunk boundaries, so
lengths and byte ranges stay exact. Jellyfin writes its token into playback
info and into the HLS playlists' subtitle, trickplay and variant URLs, and reads
the `Authorization` header before any `ApiKey` in the URL, so a masked URL still
plays. A lending declaration also names the paths the door refuses; Jellyfin's
refuses `/Sessions/Logout`, which would sign the whole group out.

**C3a group compute.** `group.ask` puts a request in the group's journal as an
act. Each computer that lends a model holds a lease on a request by an act, runs
it against its own svrn daemon's OpenAI-compatible endpoint on loopback, and
answers with an act carrying the answer and the model's identity — the shape of
commonwealth-ai's work plane (`commonwealth-work`: submit, lease, renew,
complete), ported, not linked. The fold takes one answer per request, so every
copy folds the same answer and a fold never calls a model. Distribution is the
leases': every lending computer leases in parallel, and a lease left to expire
is offered again. A lend is an act naming the kinds, models and limits; the
lender's node counts and refuses past them. Only computers the register admits
lease, so a removed person's computer runs nothing. A group with no computer
lending a model has no `infer`, and Ask says so.

**C4 the runtime.** One sandbox model on every device.
- *The fold runs headless, in one engine build, on every device:* the app's
  exported `init` and `reduce`, and `propose`, `say` and `due`, run in one
  pinned QuickJS compiled to wasm with its own maths library, inside wasmtime
  (Pulley's interpreter where JIT is forbidden; NaNs canonicalised), with no
  imports and JSON in and out. Fuel is metered by instructions injected into the
  module when it is built, so the cost table is inside the hashed bytes and
  wasmtime's own fuel is off; the memory maximum is declared in the module; and
  QuickJS's own stack limit is the one that binds — the host's stack running
  out means "could not run here".
- *The engine build is pinned,* because each of these defaults breaks
  determinism silently: bulk memory operations are metered by length (or the
  engine is built without them); a failed memory grow traps inside the
  allocator, so any out-of-memory is a breach whatever the reducer catches; the
  shadow stack is sized and placed first, so an overflow traps instead of
  corrupting data, with QuickJS's stack limit below it and the host's stack
  above what either backend needs; and the fold's own loop — removals, seed
  parse, per-act calls, the exactness check — runs inside the hashed bytes, not
  in shell code. A breach is read from the engine's own fuel counter. `Date`, random, `Intl`, the
  locale-dependent string methods, `WeakRef` and `FinalizationRegistry` are
  removed. State is JSON only, and exactly: a state that does not survive a JSON
  round trip unchanged (`-0`, `NaN`, `Infinity`, `undefined`, `Map`, `Set`,
  bytes) at a segment boundary is a breach, named for the developer; the dev
  harness folds at K = 1 to catch one that a boundary would hide.
- *Segments, so completeness is a function of the journal alone:* the fold runs
  in fixed segments of K acts by order position, each in a fresh instance seeded
  with the previous segment's JSON state and given its own fuel and memory
  budget. Each segment's seed state is a disposable cache on the device. A new
  act re-runs the last segment; a late act re-runs from its own.
  A segment that breaches its budget halts and is marked incomplete — the fold
  never skips the act. A device that cannot instantiate the engine at all (its
  memory is reserved at the fixed size, with no guard region) says "could not
  run here", which is not a breach.
- *The engine is part of the SDK version.* Every shell build carries every engine
  that any released SDK version names, and engines arrive only inside the shell
  — never as a group file, since loading a precompiled module is unsafe on
  untrusted bytes. A shell that lacks the engine an app's SDK version names
  shows "needs a newer version of the app" and folds nothing, rather than fold
  with another engine; a shell may retire an engine found unsound, with the
  same message. Precompiling for Pulley, and QuickJS's start-up snapshot, are a
  per-build cache. Folds run one at a time, and the memory maximum fits the
  oldest supported iPhone's headroom. JavaScriptCore is
  not used for folds: it has no public execution or memory cap, and its
  locale-dependent built-ins would let two phones compute different balances.
  The fold is the state of record; a screen receives changes and may keep a
  cache.
- *QuickJS is the first engine, not the only one.* The runtime reaches an engine
  through one `Engine` trait whose contract is the SDK's — JSON in and out, the
  five laws, the budgets — and an SDK version names its engine by hash. Another
  engine replaces QuickJS under a new SDK version once it passes `engine-parity`
  and `fold-speed` on the golden journals; apps on the older version keep the
  engine they named.
- *With a screen:* every bundle is served from one URI scheme,
  `ringapp://<install-id>/`, with a CSP that allows no network, frames, forms or
  navigation. Each app runs in `<iframe sandbox="allow-scripts">`. `window.group`
  is a MessageChannel port, and the app's identity comes from the shell's own
  map, never from the message. Camera access is `<input type=file>`.
- The runtime owns the one fold traversal. Golden journals fold to byte-identical
  state on every platform, under a swapped locale too (D9 fold-parity).
- A lent service's view gets no shell command, like a bundle's frame (D9
  frame-cannot-ipc covers both).

**C5 bridge.** An adapter and nothing else: discord.js on the gateway, so it is
outbound-only, behind a five-method seam. It renders command schemas from manifests and sends each invocation to the host's handler runner. It posts the
results with embeds, `@everyone` and role mentions suppressed (a `due` item may
mention the person it names), and posts each `due` item once with a fulfilment
act. It never loads app code. It keeps the guest book from people
who act or are mentioned in the chat, and a removal there sticks until a person
re-adds them.

Binding works like this:
- The bridge writes an `Invite` with `via` set to the speaker and sends them its
  link, which only they can see.
- The app redeems it at the always-on node.
- The chat shows "Ama's phone joined".

A bound speaker acts through the chat as themselves, labelled chat-origin. The
bridge checks that its own acts were admitted and says in the chat when they
were not. Its greeting says it reads the channel, whose computer keeps the group
open, and, for E0, that the data sits on the operator's computer.

**C6 the app.** The runtime with a screen, on iOS, Android and desktop, holding
several groups and never showing one inside another. It:
- keeps the key in the keystore;
- syncs both ways while open;
- installs by hash after verifying the signed manifest, with consent on first
  open and again when an update adds a capability;
- shows each app's name, version and who installed it;
- previews an update's effect before it runs;
- shows each app's `due` items to the person they name when it opens, and writes
  nothing for them.

After a reinstall or a restore from backup, the app syncs and keeps signing with
the same key from its synced heads; an act it lost that a peer still holds is a
second branch, which voids nothing (C1). Android deletes keystore keys on
uninstall, so a person whose only device is an Android phone is admitted again
by their own other device or, failing that, as a new person; the app says so at
setup, since an app cannot see an uninstall coming.

On Android, the sandbox ships only once frame-cannot-ipc passes; otherwise each
app gets its own native web view.

**C8 browser.** A static install page with no group data.

**C11 setup.** `svrngs start`, five nominal steps:
1. install;
2. create the Discord app and paste its token;
3. run setup, which finds the application, prints the invite link, lists the
   channels, enables the service and warns if the computer sleeps;
4. open the link;
5. pick the channel.

The host-setup bar measures the real count.

`svrngs refound <directory>` — or "Start this group again" in the app — makes
the continuing genesis from a copy, lists the people to carry over, and prints
one invite link per person for the chat; the always-on node is set up as in the
five steps above.

**C12 the developer surface.**
- *The reference:* the manifest and permissions; every SDK call with its limits;
  what order, undo, complete and missing mean; determinism and the five laws
  (determinism, environment independence, non-interference, idempotence,
  totality — RING_APP_LIBRARY §4) and a sixth, no module globals: `reduce`
  depends only on the state and the act; handler budgets;
  what a seal hides and does not hide; storage scope; file limits (per file and
  per person, set by the group) and deletion — the act's author may undo a
  file, which erases its bytes everywhere;
  who keeps a started group; install, update and consent; the sandbox; the dev
  loop; the test harness.
- *The seeds* as worked examples.
- *A dev loop that is the install path:* `svrngs app dev` puts the bundle and
  writes an install act on save, in a dev group with the developer's phone, which
  reloads on sync.
- *A harness:* the five laws, the update preview, and fold-parity.

## D6. The narrow waist and the edges

**Closed:**
- the journal wire (canonical form, signing bytes, derived ids, the sync digest,
  gaps);
- the ingest rule, the refusal class, and the register's rule;
- admit, order and void;
- origin by role;
- SDK v1 (`window.group`), the manifest schema and permissions;
- the sandbox rules.

`commands`, `propose`, `say` and `due` are provisional until a second chat
adapter exists.

**Open:** apps; chat adapters; lent services.

## D7. Adopt, adapt, build, avoid

**Adopt:** iroh 1.x and its relay server; Tauri 2 with its barcode and biometric
plugins; AVKit's AirPlay route picker and the platform players; the Google Cast
sender SDK; a UPnP AVTransport control point; wasmtime with QuickJS (Javy); discord.js 14; Yjs in pages; age's X25519
recipient stanzas for seals; a pure-JS curve library for app cryptography; the
platform keystores.

**Adapt:** p2panda-auth's strong removal; the hash-linked author log (SSB,
p2panda); LiveStore's
versioned event names; webxdc's two sandbox requirements; meshapp's pack and
install.

**Build, by experiment.** Each experiment tests one assumption with the least
machinery that can test it, and builds only what its bars need. Everything else
waits in D8 for the signal that earns it.

**Demos keep the cadence.** No more than three units pass without a demo a
future end user would want to watch: a scene on real phones and a real chat,
recorded, with a run-of-show in `demo/` written before its first run and a record of
every run in `demo/runs/` (`demo/README.md`). Each demo's gate is a bar id, and it is shown when that bar passes.
A unit names the demo it moves; one that moves none waits.

*E0 — will a group settle up through the bot, in the chat it already uses?* On
the operator's computer, as one `svrngs` process: no iroh, no phone, no sandbox.
It writes the wire format of C1 from its first act.

| Unit | Builds | On | Bars |
|---|---|---|---|
| E0.1 bridge | discord.js on the gateway behind its five calls; the greeting; the guest book keyed by Discord user id; a ban removes from it | `svrngs`, the host | removal-latency, bridge-holds-no-app |
| E0.2 journal and bridge append | `svrngs-core`'s chain and fold, one writer's journal in the group's directory, and the loopback append with a capability token, signed by the Bridge-role key; Confirm taken once by interaction id | `svrngs-core`, `svrngs` | double-confirm |
| E0.3 Expenses | one `expenses.js`, ported; its `propose`, `say` and `due` run by an embedded QuickJS on the host | `apps/expenses/`, from commonwealth-ai's copies in `sovereign/apps/ring-doc/`, `ring_cmd/templates/` and `My-doc/` | five-laws, N+1 |
| E0.4 origin gate | the bridge renders only what the chat wrote | the bridge | origin-gate |
| E0.5 setup | `svrngs start`, five steps | `svrngs` | host-setup, plain-words |

| Demo | After | What a person watches | Gate |
|---|---|---|---|
| Split the pizza | E0.3 | Five people in a Discord channel who have never heard of svrngs: one types the pizza, the others tap Confirm, and the bot says who owes whom — no app, no account | N+1 |
| Your own group in fifteen minutes | E0.5 | A friend installs svrngs on their computer, pastes a Discord token, picks a channel, and their friends are settling up | host-setup |

*E1 — will people move from the chat to their own key?* A phone signing beside
the host is the first second writer, so this is where the register (C1) is
earned. The keeper is the host, and E0's groups carry straight over: E0 already
wrote the wire format.

| Unit | Builds | On | Bars |
|---|---|---|---|
| E1.1 sync | iroh accept and dial, causal ingest, the digest by heads — the multi-writer half of C1 | `svrngs-node`, porting commonwealth-ai's iroh patterns | converge, causal-ingest, forks-void-nothing, fresh-node-progress |
| E1.2 continuation | a genesis that continues an earlier group, and the group directory; `svrngs refound` | `svrngs-core`, `svrngs` | refound-from-a-copy, group-is-a-directory, genesis-certifies |
| E1.3 register | `Invite`, the co-signed `Admit` and person ids, `RemoveRequest` and the keeper's `Remove` | `svrngs-core` | removal-is-sequenced, refusal-not-absence, invite, person-id, admit-co-signed |
| E1.4 doors by register | the acceptor and every group route admit by the register | `svrngs-node` | one-decider, cross-group, first-contact |
| E1.5 the app | the Tauri shell with the node, a key per group in the keystore, two-way sync with the host, Expenses and the Library built in, the claim route from the bot's link | `app/`, porting commonwealth-ai's `sovereign-mobile` shell | same-core, phone-to-own-key, reinstall-keeps-person, rebuild-from-a-phone |
| E1.6 one engine | the `Engine` trait, and one embedded QuickJS build behind it on the host and the app; JSON-only state | the runtime | fold-parity |
| E1.7 the door | the member door to a lent service, porting commonwealth-ai's forwarding body (`iroh_identity_forward.rs`); lent views under CSP; refused paths, Jellyfin's `/Sessions/Logout` first | `svrngs-node` | peer-text-is-text, lent-service |
| E1.8 the library | `svrngs-node` published, with a two-node example: the iroh client for groups that builders asked for | `svrngs-node` | node-library |

| Demo | After | What a person watches | Gate |
|---|---|---|---|
| Leave together, keep everything | E1.2 | Copy one member's svrngs folder to a new computer, start the group again, and every balance and every expense is back | refound-from-a-copy |
| From the bot's link to your own phone | E1.5 | Tap the bot's link, svrngs opens, the chat says "Ama's phone joined", and Ama's next expense is signed by her phone, offline included; then wipe the computer and the group comes back from her phone | phone-to-own-key, rebuild-from-a-phone |
| Movie night from a friend's shelf | E1.7 | Open Jo's library in svrngs and start a film from Jo's house on your phone | lent-service |

*E2 — does the platform carry every class of app?* One reference app per class
(D0a), each built only as far as proving its class needs, in Phase 1's twenty
groups — each live once a second copy has synced, and each choosing which apps
it runs, which is the second-app rate. The order puts first the classes whose
machinery is already built.

| Unit | Builds | On | Bars |
|---|---|---|---|
| E2.1 second copy | commands register only after another node of the group has synced | the bridge | two-copies-before-live |
| E2.2 Films | `group.request` and `group.play`; a TV by AirPlay from the phone with the credential mask at the door, then Cast, DLNA (Android) and the person's own always-on node serving, in the order households ask | the door (E1.7); Jellyfin's provider declaration, porting commonwealth-ai's read-only viewer account (`mesh_media/viewer.rs`) | lent-service, play-on-a-tv, finishes-a-film, credential-stays-home, films-journey |
| E2.3 Ask | `group.ask`: requests, leases and answers as acts (C3a); each lending computer's node calls its own svrn daemon on loopback; lends with kinds, models and limits | `svrngs-node`, `svrngs-engine`; the svrn daemon, unchanged | ask-is-an-act, ask-fans-out |
| E2.4 Album | `group.files`, fetched on demand, erased when voided | the file store | file-erased |
| E2.5 Event | `group.start`: an invite act, accepted on each phone | the register | start-a-group |
| E2.6 Doc | `group.live`, and a screen's own cache over Yjs | Doc (built) | live-doc |
| E2.7 Tap | sealed payloads to named people's encryption keys, padded and naming no one; private storage | the register's `enc_key` | seal-hides |

Each reference app is its own demo:

| Demo | After | What a person watches | Gate |
|---|---|---|---|
| Jo's film on your TV | E2.2 | Pick a film from Jo's library on your phone and it plays on your living-room TV, with nothing installed on the TV | play-on-a-tv |
| Your group's own AI | E2.3 | Ask "what do I owe Sam, and why?" and a model on Jo's computer answers, citing the expenses; then caption forty album photos and watch the work split across two friends' computers | ask-is-an-act, ask-fans-out |
| Photos that leave when you say | E2.4 | Share an album; delete one photo and it is gone from every phone | file-erased |
| Spin off the party | E2.5 | Vote on when and where, and the party's own group starts on three phones | start-a-group |
| Write it together | E2.6 | Two phones edit one doc at once, each seeing the other type | live-doc |
| The post-party question | E2.7 | Two people who both tap see the match, and nobody else ever learns who tapped | seal-hides |

*E3 — will outside developers build on it?* Only now does strangers' code run
on people's phones, so only now is the platform machinery earned.

| Unit | Builds | On | Bars |
|---|---|---|---|
| E3.1 SDK v1 | `window.group` published with the terms the seeds use by then; the manifest, capabilities and consent; install and update by signed manifest and bundle hash; the update preview | the app, porting commonwealth-ai's meshapp pack and install | permissions, update-preview |
| E3.2 sandbox | a frame per app, its CSP, the MessageChannel port | the app | frame-cannot-ipc |
| E3.3 an engine for strangers' code | QuickJS in wasm under wasmtime as a second implementation of `Engine`, metering inside the hashed bytes, fold segments, the engine pinned by the SDK version (C4) | `svrngs-engine` | engine-parity, fold-speed |
| E3.4 the developer surface | the dev loop, the reference with the seven reference apps as worked examples, the harness | `svrngs app dev` | developers |

| Demo | After | What a person watches | Gate |
|---|---|---|---|
| An app in an afternoon | E3.4 | A developer outside the project copies a reference app, bends it into their own, and their group is using it that evening | developers |

**Avoid:**
- an existing local-first system as the substrate;
- Chat SDK; Dex and lldap; matterbridge; Matrix as a hub; Discord Activities;
- a home-screen web app as the client; browser keys;
- a tunnel that sees plaintext; a VPN app on the phone;
- Bun or Deno permissions as the sandbox for community code;
- Tauri's iframe isolation on Android without frame-cannot-ipc;
- yrs; server-held expense apps.

## D8. Cut, each with the trigger that brings it back

| Cut | Trigger |
|---|---|
| the town: deeds, keepers, insurance, handover, first refusal | a lent service a group would miss, counted |
| the LAN guest door | a group asks for browser guests on its WiFi |
| phones-only groups: a phone as keeper, phones accepting connections, the relay as the only path, keeper handoff, refounding from the app | a group with no computer asks, counted, and E1's key-holding share holds |
| another device for the same person | someone asks to add a second device |
| a second chat adapter, and the trait it earns | a group on another chat asks |
| the wasm engine, metering and segments for first-party apps | strangers' code (E3) |
| compacting act bodies | a group's journal on a phone passes a size measured to matter; it returns as a keeper-signed checkpoint of the fold, not a floor per key |
| a TV's own Jellyfin app (Fire TV, Xbox, webOS, Tizen, non-AirPlay Roku) pulling from a door the person's own computer serves on its WiFi — an app and a login on every TV | people whose TV takes no push ask, counted |
| `Remote-User` and per-person accounts on lent services | a named service that needs them |
| acts across groups, a person-scoped store, local-model access (the grapevine) | a grapevine seed is wanted |
| the clerk, and the Elder answering in the chat | Ask passes its bars |
| the wire published as a protocol for a second implementation; agent kits | someone sets out to write a second implementation |
| erasure coding, gossip, streams, push | an app needs one, named |
| a remote browser view | install friction at first contact, measured |
| public App Store listing | the age question is answered and 4.7.4's public index has an answer for group-private apps (TestFlight and a signed APK until then) |
| local discovery on one WiFi | Apple's multicast entitlement is granted |
| host packaging for StartOS and Umbrel | host setup is the measured blocker |

## D9. Bars

Each bar is watched failing first, on the input named after the arrow. A stage's
bars are only those its build needs; the rest wait with the machinery they test.
Bars against the pure core are golden-journal tests and run in seconds; bars
about people are runs, counted by hand.

**E0 — the bot, in the chat**

| Bar | What must hold | Failing input |
|---|---|---|
| N+1 | Four of five strangers in the chat settle an expense within three minutes, with no account and no key prompt. | measured against today's flow |
| host-setup | Three of four people, unassisted, go from install to a first chat act within fifteen minutes. | measured against today's flow |
| double-confirm | A Confirm retried after a bridge restart produces one act. | a Confirm with no interaction id |
| bridge-holds-no-app | The bridge never imports app code. | an import of an app module |
| origin-gate | Nothing written in the group by a key renders into the chat unless the chat wrote it; a planted violation fails the build. | a planted violation |
| removal-latency | Someone banned in the chat cannot have an act admitted sixty seconds later. | the bridge's leave handler disabled |
| five-laws | Expenses' reducer holds determinism, environment independence, non-interference, idempotence, totality and no module globals. | a reducer reading `Date.now()` |
| plain-words | An allowlist covers the shell, the seeds, the bot, setup output and the iOS permission strings. | "ring" in the greeting |

**E1 — own keys.** The register's bars are tests against `svrngs-core`:

| Bar | What must hold | Failing input |
|---|---|---|
| converge | Deliver the same acts in any order and any batching, forks included: every copy holds the same set and reaches the same fold and completeness. | today's first-seen-wins at a taken seq |
| causal-ingest | An act whose `prev` is not held is not held; a branch withheld under a published act and released after its key's removal counts nowhere. | `prev` carried but unchecked (v12) |
| forks-void-nothing | A key that signs two acts on one `prev` — by reinstall, restore or on purpose — voids neither, and every copy folds both. | the fork marker swallowing an `Admit` |
| removal-is-sequenced | Concurrent requests, two people asking to remove each other, and a request signed after its signer's removal: the keeper's order decides, no one else is cut, and the removed key's later acts count on no copy. | today's two removers racing (v13's third-party cycle) |
| refusal-not-absence | A removed key that mints keys never leaves the group incomplete. | `UnknownSigner` counted as absence |
| genesis-certifies | A forged genesis for a group id, or a migrated genesis whose roster does not hash into the id, is refused. | a group with no genesis being accepted |
| refound-from-a-copy | From one member's directory, a 25-person group whose keeper is lost — and one held by a captor — is refounded with its apps, history, balances and files; carried people prove continuity with their earlier keys; the captor, not invited, can act in neither the new group nor its history past the heads. | today: no path but starting empty |
| group-is-a-directory | A group's state on a device is `genesis.json`, `journal.jsonl` and `files/`, readable with a text editor and copyable with `cp`; a copy folds to the same state on another device, and holds no key or private storage. | a journal readable only through the daemon |
| invite | An `Invite` that does not count, an expired one, a second redemption at an honest node holding the first, or an `Admit` redeeming it without its one-time key admits no one; two concurrent redemptions both count and the group is shown both. | a claim token outside the record (v10) |
| person-id | Neither a newcomer nor someone already in the group can become or pre-claim an existing person — including by redeeming a link with another `person_id` or `role`. | first admission decided by timestamp (v10) |
| admit-co-signed | An `Admit` that re-labels a person, swaps their encryption key, or grants Bridge without the admitted key's signature, or an endpoint rotation without the new endpoint key's, is refused. | today's admitter-only `Admit` |
| one-decider | Every door, `log` and fold answers with the register, and nothing else decides who is in — the guest book included. | a door that keeps its own list |
| cross-group | On a shared node, someone in group A cannot read group B's checkpoint or live lane. | today's checkpoint route |
| first-contact | A new phone's first claim lands only by redeeming an `Invite`; an unstamped loopback caller is refused. | today's open join route |
| fresh-node-progress | A fresh phone syncs a group whose Bridge history is over 4 MB and whose `Admit` sits at a high seq, with a fork early in it. | today's hex-ordered sender |

And the people's bars:

| Bar | What must hold | Failing input |
|---|---|---|
| same-core | A closure test passes, and `cargo check --target aarch64-apple-ios` succeeds. | a phone syncing through `HttpBridge` |
| fold-parity | Golden journals fold to byte-identical state on the host and on iOS and Android, under a swapped locale and timezone. | transcendental maths from the platform's library |
| phone-to-own-key | Four of five phone-only people go from the bot's link to their first act signed by their own key within five minutes, TestFlight included. | measured against today's flow |
| reinstall-keeps-person | Reinstall the app, restore the phone from its own backup, move it by Quick Start with the old one still in use, or join from a phone whose clock is behind its admitter's, then act: the same person, nothing voided, nothing silenced. | the fork marker; a same-device restore |
| rebuild-from-a-phone | Wipe the always-on node. One phone's copy restores the group, genesis included. | membership held in `roster.json` |
| peer-text-is-text | A payload carrying `<img onerror>` renders as text in every view, lent ones included. | `serve_file` without CSP |
| lent-service | A person outside the group is refused at the member door; video over the relay is capped and the app says so. | a door that admits by mesh membership |
| key-holding share | Half of active people hold a key by week eight. | counted by hand, with consent |
| node-library | A developer outside the project runs two nodes that admit each other, sync a journal and open a door, from the crate's documentation alone, within an hour. | an API that needs the app or the host |
| flip rate | At least a quarter of bridged groups move admission into the group within six months; kill under 5% (RING_ENTRY). | counted by hand, with consent |

**E2 — every class of app** (Phase 1, twenty groups). `play-on-a-tv` and
`finishes-a-film` grow a column per protocol as households ask for it, AirPlay
first.

| Bar | What must hold | Failing input |
|---|---|---|
| two-copies-before-live | A group's commands register only after another of its nodes has synced. | registering immediately |
| second-app rate | At least 40% of groups still active at eight weeks run a second app; kill under 15% (STRATEGY_RINGS). | counted by hand, with consent |
| play-on-a-tv | On an Apple TV, an AirPlay 2 TV, a Chromecast with Google TV and (from Android) a DLNA-only TV, a film from a library lent from another house starts from the phone within ten seconds, with nothing installed or typed on the TV; its URL serves only the TV picked and is gone when playback stops. | today's member door, loopback and relay-capped |
| finishes-a-film | For each of those TVs, with and without the person's own always-on node, a two-hour film plays to the end while the phone locks at minute 5 and switches app at minute 20 — or the app said beforehand, in plain words, that it would not. | the phone as the only server |
| credential-stays-home | Grep every byte the TV, the phone's app frames and the record receive during browsing and playback: the lender's token appears nowhere, and a sign-out from the lent view leaves the library playable. | today's door, which passes bodies through |
| films-journey | Four of five people who own one of those TVs start a friend's film on it within two minutes of first opening Films. | measured against today's flow |
| ask-is-an-act | Ask's answer lands as an act signed by the computer that ran it, naming the model, and every copy folds the same answer; with no computer lending a model, Ask says it is waiting for one; a removed person's computer leases nothing. | a reducer that calls the model |
| ask-fans-out | A batch captioning 40 photos, on two computers lending a model, finishes in at most 60% of one computer's time, each photo answered once in the fold; a lease left to expire is offered again and finishes. | one lender only; a lease that never expires |
| start-a-group | Event's vote starts a new group; three people accept on their phones and its first act lands; a chat-only person is refused by name. | no `group.start` |
| live-doc | Two phones edit one Doc at once and each sees the other's edit within a second; presence is never written to the record; the Doc's acts fold to the same text on every copy. | no `group.live` |
| seal-hides | A sealed act is unreadable on any device of a person it does not name, and nothing in the journal shows whom it names; two people who both tap see the match, and no one else does. | a plaintext payload |
| file-erased | Void a photo, sealed or not, and sync: no node still holds its bytes. | bytes kept forever |

**E3 — outside developers**

| Bar | What must hold | Failing input |
|---|---|---|
| frame-cannot-ipc | On iOS and Android, a planted bundle — and a lent service's view — reaches no shell command, network or storage; `group.live` is the one path out, and only for a bundle that declares it. | a bundle loaded in the window |
| permissions | An undeclared capability is refused, and an install whose manifest is not signed by the lineage key it names never runs. | no manifest check; an install naming another developer's key |
| update-preview | A hostile update's effect on past acts is shown before it runs, and one that adds a capability asks everyone again. | silent activation |
| engine-parity | Golden journals fold to byte-identical state on every platform and on two builds of the app, under a swapped locale and timezone and any batching and arrival order, with a fold that breaches its budget marked incomplete everywhere; folding at K and at K = 1 agrees where both complete. | transcendental maths from the platform's library; a fold near the memory cap; one instance per arriving batch; a wasmtime upgrade between the two builds; recursion at the stack limit; a reducer that catches out-of-memory |
| fold-speed | On the oldest supported iPhone, under Pulley with injected metering, a full refold plus an update preview of 3,000 Expenses acts finishes within a time set before the first measurement, and so does the first fold after a shell update. | unmeasured today; a reducer looping `TypedArray.set` over a large buffer |
| developers | At least two of three outside developers each run an app that is not a seed, riffing off a reference app and using at least two classes of D0a, in a real group of three people for a week, with no platform change and no more than five questions the reference should have answered. | today's docs |

**Waiting with their machinery** (D8):

| Bar | What must hold | Failing input |
|---|---|---|
| phones-only | Two phones and no computer start a group, write on both, and fold the same, through the relay. | a phone that can only dial |
| relay-off | With the relay off, direct paths keep syncing and the rest say so. | a relay hard-coded in the app |
| keeper-handoff | After `Keeper`, removals continue under the new keeper; with the keeper's device gone, admitting and writing continue, and the app says removals wait and offers to refound. | a keeper hard-coded to the starter |
| refound-in-hours | A group of five, its keeper's phone gone, refounds from the app and is back to writing within two hours, every person carried over. | measured against today's flow |

## D10. Risks

1. The always-on node is the group's availability. When it is asleep, the chat
   shows Discord's own "did not respond". A phones-only group is open only while
   two apps are open.
2. Whoever holds the Bridge key speaks for every chat speaker, and can write
   `Invite`s that admit new phones in their name.
3. Discord reads the channel, can revoke the bot, and is the only path for
   people without the app.
4. A deep link from a stolen bot token can point a phone at another node. A
   leaked invite link admits whoever redeems it; concurrent redemptions both
   count, and the group sees each.
5. TestFlight expires at 90 days; external testing and public listing meet Apple
   4.7.
6. The node on iOS and accepting connections in the foreground are
   unmeasured; Delta Chat's foreground accept is the precedent. So is wasmtime's
   Pulley on iOS ("supported but less well tested", no iOS CI) and its speed
   (wasmtime's own doc: about ten times slower than native).
7. Clock skew past the bound refuses acts.
8. The project's relay is a cost and a dependency, kept swappable.
9. Files are as safe as the devices holding them.
10. A frozen or captured group is not recovered from inside; its people refound
    it from any copy (J7), losing only what came after the heads they carry.
11. The keeper sequences removals: it can delay or refuse one, removals wait
    while its device is away, and a lost keeper stops removals (not admitting or
    writing) until the group refounds (J7). A removed key's acts the keeper had
    not received when it sequenced the removal do not count.
12. iOS shows a Local Network alert for direct paths on one WiFi, and macOS 15
    applies the same rule to launch agents, which this project installs.
13. Debugging an app inside a TestFlight build may be blind (`isInspectable` is
    unmeasured).
14. Install flapping: reverting and re-installing moves acts in and out of the
    fold for everyone; the brake is removing whoever does it.
15. Apple's current terms permit downloaded code in any engine (DPLA 3.3.1(B))
    but bind mini apps to 4.7: filtering, reporting and blocking (4.7.1), no
    native APIs (4.7.2), consent each time (4.7.3), a public index with universal
    links (4.7.4 — unresolved for group-private apps), and age gating (4.7.5);
    the App Store build carries no catalogue of apps.
16. A film on a TV plays at the lender's upload speed (about 30 Mbps on an
    average US line, so about three 1080p viewers) and transcode capacity (a
    lender without hardware transcoding serves few), and over the relay it is
    capped. A burned-in image subtitle costs a full transcode. Without the
    person's own always-on node on the WiFi, Cast and DLNA need the phone kept
    open; AirPlay through a lock needs the media background mode, measured by
    finishes-a-film. Whether third-party AirPlay 2 TVs fetch the URL themselves,
    and whether Cast plays plain-http HLS, are unmeasured.
17. A long-lived app's state grows toward the fixed memory size, or its reducer's
    cost grows with the state until a segment runs out of fuel, and then it stops
    folding for the whole group; one pathological act can do the same. The remedy
    is a new app version or a new SDK version. The reference says so, and the dev
    harness reports the memory and fuel headroom left per segment.
18. Someone in the group can post acts timestamped far in the past (no earlier
    than their own admission), and each forces every open device to refold from
    that act's segment.
19. iOS keeping keychain items across an uninstall is observed behaviour, not a
    documented guarantee; if Apple changes it, iOS reinstalls work as Android's
    do.
20. Asking spends the lender's machine; the lend's limits and the door's count are
    the brake. An answer is the named model's, kept in the record as that.

## D11. Decisions

By the operator, 2026-10-03:
- **Copies:** one for E0 only, and two before a Phase 1 group goes live.
- **Cut:** the town; the kit and the node protocol; the clerk and the Elder;
  per-person service accounts.
- **Removal:** anyone may remove anyone, a removed key cannot remove, and there
  is no recovery inside a group.
- **Classes, not apps:** the platform is proven by a reference app per class of
  app, for the next developers to riff off, and group compute — distributed
  inference on members' own computers, as svrn does — is one of the classes.
- **Exit is cheap:** "If your ring gets fked you should have the state to
  reconstitute it (with dotfile elegance) within hours." A group's state is a
  directory every member holds, and refounding from it is J7. With that, one
  keeper per group sequencing removals is accepted.
- **Install:** anyone may install.
- **Push:** none.
- **The pitch and its two rules.**
- **Scope:** WHY_THEY_JOIN must be possible; the product is the platform, not
  the apps; we are building the iOS of groups (an analogy, not "an iPhone app
  first").
- **A file primitive; keep the Library.**
- **E0** runs on the operator's always-on computer.
- **Reach:** the box is the meeting point, and the project runs the only relay.
- **Devices:** a server and a phone are the same kind of thing. Phones-only
  groups were placed in the first stage; the lean order (2026-10-03, "no complex
  machinery before earning it") moves them behind a trigger in D8, which the
  operator confirmed as following from the principles.
- **Apps declare their capabilities, and the group sees them.**

The app is named svrngs, for now (operator, 2026-10-03).

By this design, open to the operator's veto:
- one node core in every package;
- a group kept only by the devices of the people in it;
- the Bridge role, with the bridge holding no app code;
- the journal is a grow-only set of git-like acts, and forks void nothing;
- removal has one decider per group, the keeper, handed off by act;
- membership acts cannot be voided; an `Admit` is signed by a standing key and
  co-signed by the key it admits;
- no compaction in v1;
- a signing key per person per group, and an endpoint key per device;
- svrngs is its own repository, linking no commonwealth-ai or sovereign crate;
  it meets sovereign at the svrn daemon's endpoint and lent services through
  the door, and ports commonwealth-ai code by citation (operator, 2026-10-03;
  D2b);
- the LAN guest door is cut;
- `commands`, `propose`, `say` and `due` are provisional.

## D12. Order of work

E0 (the bot, on the operator's computer) → E1 (own keys) → E2 (a second app,
twenty groups) → E3 (outside developers). An experiment starts when the one
before it passes its bars. A kill bar that fires stops the line and goes to the
operator. Machinery that no experiment lists waits in D8 for its trigger.

The design loop stops here. It ran nine rounds; the TV path converged, the
runtime's last findings were engine build settings, and the register's recurring
holes were removed by structure in v16 rather than patched. A finding during the
build is fixed in the one module that owns its bar.

