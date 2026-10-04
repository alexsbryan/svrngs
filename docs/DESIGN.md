# svrngs — the design

> **Normative, 2026-10-03.** This is what svrngs is building now, and the only
> design in force. It answers to `NORTH_STAR.md`; where they disagree, this
> file is rewritten. Where code and this file disagree, one of them is wrong; fix
> it in the same commit. The design before the fork — the app SDK and its
> sandbox, the TV path, E0's chat bot, nine red-team rounds on the register —
> is this file at tag `e0-bot` (it opens with a "Direction" section over a body
> it contradicts, the misstep of §2.4). Build from it only what a §8 unit
> names. The record of why — rationale, prior art, history — is
> commonwealth-ai's `docs/RING_APP_LIBRARY.md`, cited as LIB §n. A path in
> backticks that is not in this repository is in commonwealth-ai.

## 1. What svrngs is

**For people:** *keep talking where you talk; keep what your group owns —
money, plans, files, the library — on your own computers, not a platform's.* A
group is a house, a team, a party: about twenty-five people who already share
a chat.

**For developers:** *anything you can run on localhost, you can share with your
group.* The group is the platform. An app gets three things and needs nothing
else: the person asking, verified, in a request header; a shared record that
outlives the computer it runs on; and reach to every member's computer. No
login, no SDK, no bundle.

**What we build:** the platform, and one small reference app per class of app,
for the next developer to copy and bend. Never the finished apps — the
community builds those (operator, 2026-10-03). "The iOS of groups" is an
analogy for being the thing others build on, not a plan to ship an iPhone app
first.

**Two rules for anything a person reads.** No universe to explain: a person
sees their group's name, people's names, apps' names, and *computer*, *phone*,
*the chat* — never ring, member, key, sync, journal, node or relay. And the
most general surface with the fewest features: the system learns by adding
apps, never by adding platform.

## 2. How we build — and the missteps that taught it

Each rule below cost us a day once. With `docs/principles/`, they are how a
choice gets made here.

1. **Take what exists; fork it and own it — never rewrite it.** We wrote a
   record from scratch while commonwealth-ai's `rail-core` already did the job,
   with a hundred tests and a measured browser dial; the rewrite is deleted.
   Before a new type or module, look here, then in commonwealth-ai's ring code.
2. **Take the slice you need, not the system around it.** We nearly shipped
   commonwealth's `cw-rails` as a sidecar: a 30k-line mesh daemon over nine
   crates, to use its 5k-line ring slice, with every fix waiting on a
   commonwealth release. We fork the slice instead (§4).
3. **Build what the next demo needs; everything else waits for its trigger.**
   We designed an app SDK, a wasm sandbox, signed manifests, TV casting and
   group compute in detail before one group had used anything, and the first
   change of direction made most of it moot. Design ahead only as far as it
   takes to keep a door open, and write the rest as a row in §9.
4. **A change of direction rewrites the design.** For a day this file carried a
   section that "overrides everything below it", over a thousand lines it
   contradicted. A design describes one thing; the old one lives in git, at a
   tag this file names.
5. **Draw the line by what each side owns.** A group is not a mesh: the people
   in a group decide who is in it, not whose machines they are. App-specific
   code lives in the app. A chat is a door, not the core. Our library holds no
   integration code (operator: "the only responsibility should be creating the
   ring from discord, in the most generic way, and something of `exec`-level
   simplicity for running whatever is on the ring").
6. **Demos keep us honest.** No more than three units without a scene a future
   user would want to watch, recorded, with its run-of-show written before its
   first run (`demo/README.md`). People bars are judged by strangers.
7. **Ship code, not prose.** A record is written when something lands, never
   for an attempt.

## 3. The shape

**A group** is a record and the computers and phones of the people in it. Each
member's computer runs a node that holds a full copy of the group's journal and
syncs it with the other nodes over iroh. There is no server; a member's
always-on computer, when there is one, keeps the group reachable while the rest
sleep, and is the meeting point a phone dials.

**An app is a localhost service** — an *rngapp*. A member lends one with
`svrngs share <port>`. Every other member opens it through the **member door**,
tunnelled over iroh to the lender's node, which strips any identity header the
client sent and adds the verified one. A lent service — Jellyfin, a svrn
daemon's web UI — is an app like any other; what the group may do with it is a
lend declaration, kept as data: a secret the door injects and never lets out, the
paths it refuses, how many people may use it at once.

**What an app gets — the whole surface, for our apps and a stranger's alike:**
- the person asking, verified, in a header the door sets;
- the people in the group, each with a public key the app may seal to;
- the record, over a loopback API on the member's own node: append, read, live;
- the services the group's members lend, each by name and an opaque kind the
  app interprets (`jellyfin`), and how many people are using each now;
- on each request through the door, the share of the lender's upload it may
  use right now;
- the screen handoff: a node on a TV's network serves one stream at an
  unguessable local address the TV fetches, while a phone is the remote.

An app that needs anything else is a decision, not a convenience.

**Joining, phones first.** `svrngs new <name>` makes a group and prints its
link. On a computer, `svrngs join <link>` makes it a member node. On a phone the
same link opens in the browser and the person is in: the page holds the
person's own key (WebCrypto, never extractable) and reaches the group by
dialling a member node over iroh in wasm — measured working from a stock Chrome
tab (commonwealth-ai `ralph/DECISIONS.md`, browser-dial-2, `HTTP/1.1 200 OK`).
A phone holds no copy of the record; it reads and writes through the node it
dials. What the browser still lacks is the runtime that renders a lent app
through that dial: fetch, POST, streaming and byte ranges.

**Chats are doors, and the last thing we build.** A small adapter per chat, in
its own package, makes a group for a channel and posts its link. Onboarding
comes after the core (§8). The core never knows a chat exists.

## 4. Domains, and the fork

| Domain | What it owns | Meets svrngs by |
|---|---|---|
| **svrngs** | the record, the node, the doors, the command, plain words, reference apps, demos | — |
| **chat doors** | one package per chat: make a group for a channel, post its link | the `svrngs` command |
| **apps (rngapps)** | anything served on localhost; app-specific code lives only in `apps/<app>/` | the surface in §3, and nothing else |
| **sovereign (svrn)** | local AI on a member's computer | its OpenAI-compatible endpoint on loopback |
| **commonwealth-ai** | the mesh for one owner's machines, and the ring code svrngs forked | nothing at runtime; `scripts/ports.py` pairs each forked file with its original |

**The fork** (operator, 2026-10-03, over shipping `cw-rails` as a sidecar):
svrngs owns the group's code, forked from commonwealth-ai's ring slice at
`e1a43bcd2`: the record (`commonwealth-rail-core`), the journal
(`commonwealth-rail` and `oplog`), ring sync and its route
(`commonwealth/crates/commonwealth-rails/src/ring_sync`, `ring_routes.rs`),
the sync bodies (`sovereign/crates/sovereign-peer-wire`), the dial-by-key
tunnel and the member door (`commonwealth/crates/commonwealth-transport/src/`
`iroh.rs` and `iroh_identity_forward.rs`, `mesh-reach/src/guest.rs`), and in
U3 the rail routes an app writes through (`rail.rs`). What each upstream piece
becomes — forked, replaced, or left behind — is U1's inventory (§8). How a
piece comes over:

- **Behaviour kept first, then changed.** A piece lands with only the edits it
  needs to build here — paths rewritten, a small dependency inlined, a test
  re-asserted without a crate we do not carry — each named in its commit and in
  `ARCHITECTURE.md`; its edition, rustfmt policy and clippy allowances stay
  upstream's, and every file carries `ported from commonwealth-ai
  <path>@<commit>` on every source file. Behaviour changes only in later
  commits (ARCH 2), so a diff against upstream shows what svrngs changed. Where upstream code is tied to
  something we do not fork — the mesh — the tie is cut in the unit that
  replaces it, and that unit's commits say which lines changed meaning.
- **A fix shows its twin.** commonwealth-ai keeps `rail-core` for its work
  plane and the door for its mesh. The two systems share no wire, so the
  copies are a fork and may diverge; `scripts/ports.py` lists each pair and
  whether the original changed since — "unchanged" means upstream has not
  moved, not that the fork still matches it — so a signing fix on one side is
  seen on the other.
- **The arrow points one way.** svrngs links no commonwealth-ai or sovereign
  crate (`scripts/lean_check.py` refuses them from `Cargo.lock`).
  commonwealth-ai's ring-facing code — `ring_cmd`, ring-doc, ring-runtime,
  `ring_shim.js`, the phone's ring shell — retires as svrngs replaces each
  piece (its `docs/RING_SPEC.md`, retirement list).

## 5. Code

| Crate | What it is | State |
|---|---|---|
| `svrngs-core` | the record: acts, admission into one order, membership, the sync digest. Pure — no filesystem, clock or network; time and keys are arguments | none |
| `svrngs-node` | a group's journal on disk; next, sync, the routes and the member door over iroh | the journals |
| `svrngs` (to come) | the command and the process: `new`, `join`, `share`, and the loopback record API | a device's keys |
| `svrngs-engine` | a QuickJS fold engine, waiting in §9 | none |

Held to `docs/principles/RUST_ARCH.md`: types encode the domain, ownership
encodes the architecture, I/O lives at the edge. Parse, don't validate; closed
sets are enums; one writer per journal, and no shared mutable state; a trait
only at a real boundary; `thiserror` in the crates, `anyhow` only in the
command and the app. Every decision branch traces at debug; the engine, which
is pure, returns its decisions for its caller to trace. New code is held to
these from its first line; forked code is held to them as it becomes ours.
Upstream's shape stays until a unit changes it: the node's map of open
journals sits behind a `Mutex`, which an actor per journal replaces.

**Two gates keep the apps out of the infra.** Every app lists its own nouns in
`apps/<app>/nouns.txt`, and `scripts/vocab_check.py` refuses any of them in
infra code — comments and tests apart, where an example is fine. So the infra
never learns what a film, an expense or a crush is. And the canaries of §8,
Expenses and Tap, run against the infra in `scripts/check.sh`: a change that
serves Films and breaks one fails the build.

**Lean, and kept lean.** `svrngs-core` stays under 2,500 lines of code and the
repository under 12,000, tests and `apps/` apart (`scripts/lean_check.py`);
a raise is the operator's, with the reason in the commit. `svrngs-core` stands
at 2,297, so the rules of §6 will ask for one. `ARCHITECTURE.md` maps what
exists and changes in the commit that changes it.

**Words.** The *record* is a group's signed acts; its *journal* is the file
that holds them on one computer; the *register* is the part of the record that
says who is in the group — its `Admit` and `Remove` acts and their fold.
Forked code keeps upstream's words until a unit renames them: *ring* and
*namespace* for a group, *roster* for a seed list of members, and names such as
`commonwealth-rail` and `MeshStore` in its comments. Upstream comments cite
ARCH principles by old section number (ARCH §18.1); the table at the end of
`docs/principles/ARCH_PRINCIPLES.md` maps them.

## 6. The record

**Today**, as forked: signed acts — `Record`, `Correct`, `Seal`,
`Admit{person, key}` and `Remove{key, through_seq}` — admitted into one total
order by `(ts_unix, actor, seq, id)`; `Correct` voids without erasing; a
membership fold over `Admit` and `Remove`; a per-actor digest, so sync sends
only what is missing. The roster that seeds it comes from a registered source,
a file, or a default, which upstream set to the mesh.

**Where it is going**, by operator decision, each rule landing as an edit to
the fork with the unit named beside it (§8). The full rules as designed are at
tag `e0-bot`, `docs/DESIGN.md` §D5, "C1 the record".

- **The group decides who is in it** (U1). Who may connect and who is synced
  come only from the group's register — the key that seeds it, and its
  `Admit` and `Remove` acts as `membership` folds them — never a mesh or a
  list a door keeps. Until U2's link carries the seed, it is the founder's key
  alone, in the journal's `roster.json`.
- **A grow-only set, linked like git** (U2, before any key from outside the
  operator's own computers). Each act names its author's previous
  act; a copy holds an act only with its parent, so copies converge by union.
  Two acts on one parent are a fork, and forks void nothing.
- **Removal has one decider** (U7). Anyone may ask to remove anyone; the group's
  keeper — one person's key, handed on by an act — sequences each request. A
  removed key cannot remove. There is no recovery inside a group.
- **Leaving is cheap.** A group's whole state is a directory every member's
  computer holds. A group that goes wrong refounds from any copy, within
  hours, without whoever it leaves out (operator: "with dotfile elegance").
- **An invite admits one person once** (U2): a one-time key in the link;
  redeeming it writes one `Admit`, co-signed by the new key.
- **Ids cover everything signed** (U2): the id hashes the actor, seq, previous
  act, timestamp and body, and the signature is over the id. Today the
  timestamp is outside the id, so one act can be copied with two timestamps and
  folded in two orders, and the previous act is outside the signature, so a
  relaying node can rewrite it. A removal names the removed key's acts by hash,
  never by position or time (U7): today the fold ignores `through_seq` and
  cuts the key at the `Remove`'s place in the order
  (`crates/svrngs-core/src/membership.rs:153`), so a backdated act slips under
  the cut.
- **A signing key per person per group**, so a cut in one group touches no
  other; an endpoint key per device.

## 7. Packaging and reach

Phones first, through the browser: no install, no store, one link. Computers
run the node — the `svrngs` process on macOS and Linux — and an always-on one is
the group's meeting point and the origin a TV fetches from, its port mapped by
iroh; iroh's relays carry what cannot go direct. A native phone app and
Windows wait in §9. A svrn daemon on the same computer is an optional neighbour
for local AI; it holds no group state.

## 8. Order of work

**One experience, done delightfully: Films — the group's films, one shelf,
on any phone and the TV in the room.** Everyone's lent library on one shelf,
each title marked "from Sam's"; no accounts, no port forwarding; play it here
or on the TV; the lender's password never leaves their computer. Nothing else
is built until Demo B passes, except the two canaries, which keep Films from
bending the infra into a media platform.

**Many people, one library — first class, not an edge case.** Three friends in
three homes on Sam's library at once is the normal evening. What breaks, and
who fixes it:
- *Sam's upload is shared* — infra. The door measures Sam's upload and each
  viewer's path, divides the upload evenly among the people watching, and tells
  each request its share; shares change as people start and stop. Sam's own
  home, on the local network, is never counted against it.
- *Sam's computer has one processor* — Films. It asks Jellyfin for no more
  than the share (`MaxStreamingBitrate`), plays the file as it is whenever the
  share and the phone allow — which costs Sam nothing — and transcodes only
  when it must.
- *Everyone comes through one viewer account* — Films. Jellyfin kills running
  transcodes by device id and play session (`DynamicHlsController.cs:1504` in
  the source read), so each person's device and each play carry their own, and
  Bo seeking never stops Cy. Progress and "watched" live in the group's record
  under each person's name, so Continue Watching is theirs and follows them
  across every member's library. The lend's limit sits below the account's own
  session cap (`MaxActiveSessions`), so the door, not Jellyfin, says no.
- *Too many at once* — the door refuses past the lend's limit with a reason;
  Films turns it into a choice: "Sam's library has three watching — Dee has
  Arrival too", or "you're next". A title two members lend plays from whoever
  can give you the bigger share.
- *The same room* — one stream to the TV, however many phones hold the remote.

**U1's inventory.** What `ring_sync`, `ring_routes.rs` and `acceptor.rs` use
outside themselves at `e1a43bcd2`, tests apart, and what each becomes. Paths
are under `commonwealth/crates/commonwealth-rails/src/` unless they name a
crate.

| Upstream | What it does there | Becomes |
|---|---|---|
| `commonwealth-rail` | the journals | forked already: `svrngs-node` |
| `sovereign-peer-wire` | the sync bodies; the 8 MiB body limit and the 4 MiB exchange budget | forked whole into `svrngs-node` |
| the mesh: `commonwealth-core::mesh`, read by `RoundMembers::of` (`ring_sync/journal.rs:146`), `member_check` (`acceptor.rs:49`) and `MembershipRosterSource` (`rail.rs:113`, installed at `lib.rs:404`) | who the round dials; who may connect; every ring's default roster | **the register**: a group's members are the keys standing in `membership` over its seed and its `Admit` and `Remove` acts (`crates/svrngs-core/src/membership.rs`), read through `RingJournal::admit`. The round offers a group to its standing keys, the acceptor admits a key standing in any group the node holds, and the sync route asks the group the request names. No default roster is installed |
| `commonwealth-core::ids` | `NodeId`, the mesh's id for a machine; `NodePubkey`, a verified key | a member is the record's actor, its key's lowercase hex (`actor_of`), read from the key iroh verified; `NodeId` goes with the mesh |
| `commonwealth-transport::mesh_proof` | proves the mesh secret on a plain-IP hop | left behind: svrngs dials only over iroh, whose handshake proves the dialer's key |
| `commonwealth-transport::iroh` and `mesh-reach::guest` | dial by key: a loopback bridge per peer for the dialer (`IrohTransport`, `HttpBridge`, `pump`); for the dialled, an accept loop that copies each stream into the node's loopback listener (`IrohAcceptor`); the relayed endpoint and its discovery setting | forked, the dial-by-key tunnel only: one ALPN, no IP transport, no forward by ALPN or path prefix, no bench relay pin |
| `mesh-reach`'s port: `PeerTransport`, `PeerContact`, `PeerEndpoint`, `TrafficClass` | one seam over IP and iroh; a port per class of traffic | replaced: one transport, so a concrete type (RUST_ARCH: a trait only at a real boundary); a contact is a member's key; one ALPN constant, and an enum when a second protocol comes |
| `commonwealth-transport::iroh_identity_forward`, with `kernel-types::member`'s header scheme | stamps the dialer's verified key on every request head and strips any the client typed | **forked in U1**: the sync route is the member door's first origin and U3's lent services its second, so there is one identity mechanism, not two |
| `commonwealth-media::origins` (`OriginRegistry`, `admit_spliced_origin`) and `commonwealth-transport::iroh_routed_forward` | a table of many programs' origins by ALPN and path prefix | replaced: U1's node serves one origin, its sync route, to members; a table waits for a second |
| `host-kit::shell::RouteBundle` | named route bundles a shell mounts | replaced by an `axum::Router` |
| `rail.rs`: `err` | the refusal body | forked with the route |
| `rail.rs`: `append`, `RailQuery`, `LiveBuffer`; the live route (`ring_routes.rs:163`) | the record API an app writes through; the nudge that wakes the round after a write; the live lane | U3, as the loopback record API. Until then nothing nudges, and the round runs on its interval |
| `rail.rs`: the sync doors (`digest` to `compact`) and `attest` | the inference daemon reading this process's journals over loopback | left behind: one process per computer |
| `ring_routes.rs`: the checkpoint route (`:208`) | exports one ring as a document | waits (§9). It checks no ring's roster, so any member the acceptor admits can export any ring the node holds |
| `RailsDaemon` and `RailsNode` (`lib.rs:232-664`): `RingSyncHost`'s implementor (`ring_sync.rs:700-733`) and the wiring in `start` and `run` | the journals, endpoint, membership, nudge, loopback listener, acceptor and round, in one process | replaced by the node: a `svrngs-node` type holding the rail, endpoint, transport, loopback listener, acceptor and round. Gossip, join, the mesh store, the work plane, presence, the endpoint's self-heal and `rails.toml` stay behind |
| `commonwealth-transport::identity` | the node key on disk | U2, with the `svrngs` command, which owns a device's keys; the node takes its key as an argument |

Four facts follow. *One key signs and dials:* upstream signs a ring's acts
with the key its endpoint proves (`lib.rs:400-402`), so a member's actor is
its iroh id and the register names every node the round may dial; when §6's
signing key per person per group lands, the register carries each device's
endpoint key beside it. *A bare key is dialled:* upstream dials the relay and
addresses the mesh gossiped, and refuses a bare key
(`commonwealth/crates/commonwealth-transport/src/iroh.rs:182-237`); with no
gossip, iroh's address lookup, upstream's default
(`mesh-reach/src/guest.rs:79-88`), resolves a member's key, so the fork accepts
a bare key when lookup is on, and tests seed addresses as upstream's do. *No
presence:* the round dials every standing member, and one that is offline
costs a dial timeout each round. *The door asks the register too:*
`RingJournal::append` refuses a signer the seed does not name
(`crates/svrngs-node/src/journal.rs:225`), which would leave two answers to
who is in a group (ARCH 8); in U1 it asks the same standing the round, the
acceptor and the sync route ask, so a member the founder admits can write.
`svrngs-node` gains tokio, axum,
reqwest and iroh, and iroh 1.0.2 raises the toolchain floor to Rust 1.91.

Until U2, a test admits members by writing `Admit` acts directly, as
upstream's `commonwealth/crates/commonwealth-rail/examples/stage_member_act.rs`
does, and runs two nodes in one process with no relay — the shape of upstream's
`commonwealth/crates/commonwealth-rails/tests/ring_round.rs`, which is built on
the mesh daemon and so is rewritten, not forked.

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U1 a group is not a mesh | ring sync, its route, the dial-by-key tunnel and the member door forked into `svrngs-node`; the register decides who may connect and who is synced, where upstream asked the mesh (`commonwealth/crates/commonwealth-rails/src/` at `e1a43bcd2`: `acceptor.rs:47-61`, `ring_sync/journal.rs:153-164`, the default roster at `lib.rs:404`) | a node in the group but no mesh syncs; a node outside the group is refused |
| U2 join by link | `svrngs new <name>` and `svrngs join <link>` on a computer; redeeming writes the `Admit`; the link carries the group's seed; each person's public key in the register; acts linked by their previous act, ids over everything signed (§6) | one redemption admits one person to one group; a reused link is refused |
| U3 lend a service | the member door opened to lent services; `svrngs share <port>` with its lend declaration; streaming and byte ranges through the door; each request's share of the lender's upload; the people-at-once limit; the loopback record API (the rail routes and the live lane, forked); the list of what members lend | the app sees the verified person; a header the client typed is stripped; the lender's secret appears in no byte a member receives; with three streams, each is told a third of the measured upload, and a fourth person is refused with a reason |

**Demo A — Sam's films, three homes at once** (after U3). Sam lends Jellyfin
with one command and sends a link. Alex, Bo and Cy join from three homes and
each plays a different film from Sam's library in their own browser at the
same time, seeking freely; none stalls another, and each plays at the best
quality Sam's upload can share. Sam's password never left Sam's computer;
nobody made an account.

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U4 the browser | a phone opens the link and is in: its own key, a dial to the meeting point, and the runtime that renders a lent app through it — fetch, POST, streaming, byte ranges | a stock phone browser plays a lent film and seeks |
| U5 Films | `apps/films`: one shelf across every member's lent library, "from Sam's", search, play here; a device and play-session id per person and play; progress per person in the record; busy turned into a choice — on the §3 surface alone | a title two members lend shows once, with both; Bo's seek never restarts Cy's play; Films calls nothing outside §3 |
| U6 the screen handoff | the meeting point serves one stream to a TV on its network at an unguessable address; AirPlay from Safari, Cast from Chrome; the phone is the remote and may lock | a film plays on the TV from a phone browser; the address serves only the device it was handed to |

**Demo B — movie night** (after U6). Five friends at Sam's, phones only. Sam
texts the link; each opens it, browses one shelf across three members'
libraries, and a film they pick plays on the TV. Nobody installs or signs up.
Gate: four of five newcomers get from the link to the film on the TV within
three minutes, unassisted.

**The canaries** run in `scripts/check.sh` from the moment their surface
exists, each a small app in `apps/` using only §3:
- **Expenses** (after U3) — no server, all state in the record: two nodes,
  everyone writes, a correction voids, and both fold to the same balances.
- **Tap** (after U2) — private inputs and a ring born from a ring: two groups
  on one node; an app starts a group of two from inside a group; it seals its
  own payloads to members' public keys.

**Then:** removal (U7, the keeper of §6), before any group of strangers; then
the chat doors, Discord first — onboarding, last. After that the core grows one
class at a time, each chosen by the north star's tests.

## 9. Later, each with the trigger that brings it back

| Waits | Trigger |
|---|---|
| an app SDK, a sandboxed fold engine (`svrngs-engine`), signed manifests and install acts | an app needs its state offline on a device that is not its host, or strangers' code runs on members' devices |
| a native phone app, and groups of phones only | the browser cannot keep what a group needs — its key, or a copy — measured |
| Windows | a group asks |
| DLNA, and TVs that take no push | people whose TV takes neither AirPlay nor Cast ask, counted |
| watching together from different homes: one transcode, shared, in step | a group asks to watch the same film apart at the same time |
| sealing in the infra | two apps need the same seal |
| group compute: asks leased by members' computers and answered as acts | Ask is next |
| files beside the record; the live channel beyond what U3 carries | the reference app that needs each |
| a second chat adapter, and the trait it earns | a group on another chat asks |
| compacting the journal | a journal grows past a size measured to matter |
| exporting a group as one document (upstream's checkpoint route), checking the asker is in that group | a group refounds from a copy, or someone asks to export one |
| the endpoint's self-heal (upstream's `self_heal.rs`, `iroh_watchdog.rs`) | a node's endpoint wedges in a recorded run |
| dialling a round's members at once, or skipping the ones known offline | a round's measured time nears its interval |
| another device for the same person | someone asks |
| the wire published for a second implementation | someone sets out to write one |
| local discovery on one WiFi | Apple's multicast entitlement is granted |

## 10. Decisions

By the operator, 2026-10-03:
- the product is the platform; the community builds the apps; we build one
  reference app per class (classes, not apps);
- anything on localhost can be shared with the group; chats are doors, outside
  the core; no integration code in the library;
- svrngs owns the group's code, forked from commonwealth-ai's ring slice — not
  a cw-rails sidecar, not a rewrite;
- removal: anyone may ask, a removed key cannot remove, one keeper sequences,
  no recovery inside a group; leaving is cheap, so refounding is the exit;
- no push notifications; anyone in a group may install an app;
- the name is svrngs, for now; machinery waits until an experiment earns it;
- one experience done delightfully first — Films — with Expenses and Tap kept
  as canaries so the infra stays general; the screen handoff is infra; phones
  first, through the browser; chat doors are onboarding, built last.

## 11. Risks

1. **The always-on computer is the group's availability.** When it sleeps and
   no other member is online, the group is unreachable.
2. **The keeper is trusted to sequence removals** and can delay or refuse one;
   the exit is refounding.
3. **A media flagship pulls the infra toward media.** The nouns gate and the
   canaries are the guard. Members lend their own servers to their own group;
   svrngs hosts nothing, and its relays carry only ciphertext.
4. **Phone browsers forget.** Safari may clear a site's storage, and the
   person's key with it; a web app added to the home screen keeps it, and a
   lost key means joining again by link.
5. **Members find each other through iroh's address lookup**, n0's DNS by
   default, as upstream's (§8, U1's inventory): it maps a key to its relay,
   never to a person, and a group is unreachable through it while it is down.
   Upstream's `discovery = "none"` with the group's own relay is the way out,
   kept by the fork.
6. **Two copies of the record's code.** A signing or canonical-form bug fixed on
   one side can be missed on the other. `scripts/ports.py` is the guard, and it
   is a listing, not a gate.
