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

**A group** is a record and the computers of the people in it. Each member's
computer runs a node that holds a full copy of the group's journal and syncs it
with the other members' nodes over iroh. There is no server; a member's
always-on computer, when there is one, keeps the group reachable while the rest
sleep.

**An app is a localhost service.** A member shares one with `svrngs share
<port>`. Every other member opens it at a localhost address their own node
serves, tunnelled over iroh to the sharer's node through the **member door**,
which strips any identity header the client sent and adds the verified one. A
lent service — Jellyfin, a svrn daemon's web UI — is an app like any other.

**The record, offered to apps.** An app that wants shared state appends acts to
the group's journal and reads them back over a loopback API on its own node;
every member's copy holds them. An app may keep its own state instead; the
group does not care.

**Joining.** `svrngs new <name>` makes a group and prints its link; `svrngs
join <link>` puts a person's computer in it. A browser with nothing installed comes next: a
stock Chrome tab running iroh in wasm has dialled a live node through the relay
and read the record (commonwealth-ai `ralph/DECISIONS.md`, browser-dial-2,
`HTTP/1.1 200 OK`); what it lacks is a runtime that renders apps through that
dial — fetch, POST and streaming.

**Chats are doors.** A small adapter per chat, in its own package, does one
thing: make a group for this channel and post its link. Discord is first; its
setup code is in `bridge/`. The core never knows a chat exists.

## 4. Domains, and the fork

| Domain | What it owns | Meets svrngs by |
|---|---|---|
| **svrngs** | the record, the node, the doors, the command, plain words, reference apps, demos | — |
| **chat doors** | one package per chat: make a group for a channel, post its link | the `svrngs` command |
| **apps** | anything served on localhost; app-specific code lives only in the app | HTTP through the member door; the record API if it wants one |
| **sovereign (svrn)** | local AI on a member's computer | its OpenAI-compatible endpoint on loopback |
| **commonwealth-ai** | the mesh for one owner's machines, and the ring code svrngs forked | nothing at runtime; `scripts/ports.py` pairs each forked file with its original |

**The fork** (operator, 2026-10-03, over shipping `cw-rails` as a sidecar):
svrngs owns the group's code, forked from commonwealth-ai's ring slice at
`e1a43bcd2` — about 6k lines: the record (`commonwealth-rail-core`), the
journal (`commonwealth-rail` and `oplog`), ring sync, the ring routes and the
rail routes with the default roster they read
(`commonwealth/crates/commonwealth-rails/src/ring_sync`, `ring_routes.rs`,
`rail.rs`), the iroh acceptor they sit behind (`acceptor.rs` there), the member door
(`commonwealth/crates/commonwealth-transport/src/iroh_identity_forward.rs`)
and the sync bodies (`sovereign/crates/sovereign-peer-wire`). How a piece
comes over:

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

- **The group decides who is in it** (U1). Membership comes only from the
  group's own register — never a mesh, a roster file or a list a door keeps.
- **A grow-only set, linked like git** (U2, before any key from outside the
  operator's own computers). Each act names its author's previous
  act; a copy holds an act only with its parent, so copies converge by union.
  Two acts on one parent are a fork, and forks void nothing.
- **Removal has one decider** (U6). Anyone may ask to remove anyone; the group's
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
  never by position or time (U6): today the fold ignores `through_seq` and
  cuts the key at the `Remove`'s place in the order
  (`crates/svrngs-core/src/membership.rs:153`), so a backdated act slips under
  the cut.
- **A signing key per person per group**, so a cut in one group touches no
  other; an endpoint key per device.

## 7. Packaging and reach

Computers first: the node is the `svrngs` process on macOS and Linux. An
always-on member computer is the group's meeting point, its port mapped by
iroh; iroh's relays carry what cannot go direct. A phone app (Tauri 2, the same
node as a library) and Windows wait in §9. A svrn daemon on the same computer
is an optional neighbour for local AI; it holds no group state.

## 8. Order of work

U1-U3 move Demo A, U4-U6 move Demo B. The first three fork the rest of the
ring slice (§4).

**Starting U1.** List everything `ring_sync`, `ring_routes.rs` and
`acceptor.rs` use outside themselves at `e1a43bcd2`, tests apart: the crates
`commonwealth-core`, `commonwealth-transport`, `commonwealth-media`, `host-kit`
and `sovereign-peer-wire`, and in their own crate `rail.rs` (the rail routes,
and `MembershipRosterSource`, the default roster that asks the mesh) and
`RailsDaemon`, which `RingSyncHost` is implemented for (`ring_sync.rs:700-733`).
Decide per item: fork the piece it needs, or replace it — the mesh with the
register, the daemon with the node. The list and the decisions are U1's first
commit. Until U2, a test admits members by writing `Admit` acts directly, as
upstream's `commonwealth/crates/commonwealth-rail/examples/stage_member_act.rs`
does, and runs two nodes in one process with no relay — the shape of upstream's
`commonwealth/crates/commonwealth-rails/tests/ring_round.rs`, which is built on
the mesh daemon and so is rewritten, not forked.

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U1 a group is not a mesh | ring sync and the ring routes forked into `svrngs-node`; the register decides who may connect and who is synced, where upstream asked the mesh (`commonwealth/crates/commonwealth-rails/src/` at `e1a43bcd2`: `acceptor.rs:47-61`, `ring_sync/journal.rs:153-164`, the default roster at `lib.rs:404`) | a node in the group but no mesh syncs; a node outside the group is refused |
| U2 join by link | `svrngs new <name>` and `svrngs join <link>`; redeeming writes the `Admit`; acts linked by their previous act, ids over everything signed (§6) | one redemption admits one person to one group; a reused link is refused |
| U3 share an app | the member door forked; `svrngs share <port>`; Expenses as a small localhost web app over the record | the app sees the verified person; a header the client typed is stripped |

**Demo A — two computers, one group** (after U3). Sam makes "the house" and
sends Alex a link. Alex's computer joins. Alex opens Sam's Expenses, adds the
pizza, and it shows under Alex's name on both computers.

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U4 the Discord door | the adapter makes a group for a channel and posts its link, from the setup code in `bridge/` | the core imports no adapter |
| U5 the browser | the guest channel — the iroh channel a browser with no key dials, upstream's `cwth/guest/0` — on the member door; a runtime that renders an app through the dial — fetch, POST, streaming | a stock browser tab opens a shared app and writes to the record |
| U6 removal | the keeper and its requests (§6) | a removed key's later acts count nowhere |

**Demo B — split the pizza** (after U6). Five people in a Discord channel who
have never heard of svrngs click the link, and four of them settle an expense
within three minutes, with no account created and no key prompt — the N+1
test: N strangers and one person who knows the system.

After Demo B the classes come one at a time, each with its reference app:
lent services (Films, on Jellyfin), group compute (Ask, on members' svrn
daemons), files (Album), live collaboration (Doc), private coordination (Tap),
spin-off groups (Event).

## 9. Later, each with the trigger that brings it back

| Waits | Trigger |
|---|---|
| an app SDK, a sandboxed fold engine (`svrngs-engine`), signed manifests and install acts | an app needs its state offline on a device that is not its host, or strangers' code runs on members' devices |
| phones and phones-only groups | a group with no computer asks, counted |
| Windows | a group asks |
| a TV in the room (AirPlay, Cast, DLNA from the member door) | Films passes its bars |
| a lent service's credential masked in the door | the first service lent with a credential |
| group compute: asks leased by members' computers and answered as acts | Ask is next |
| files beside the record; sealed payloads, readable only by named people; the live channel | the reference app that needs each |
| a second chat adapter, and the trait it earns | a group on another chat asks |
| compacting the journal | a journal grows past a size measured to matter |
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
- the name is svrngs, for now; machinery waits until an experiment earns it.

## 11. Risks

1. **The always-on computer is the group's availability.** When it sleeps and
   no other member is online, the group is unreachable.
2. **The keeper is trusted to sequence removals** and can delay or refuse one;
   the exit is refounding.
3. **The chat platform** reads the channel and can revoke the adapter. It is
   only the door, so a group survives losing it; its newcomers do not.
4. **Two copies of the record's code.** A signing or canonical-form bug fixed on
   one side can be missed on the other. `scripts/ports.py` is the guard, and it
   is a listing, not a gate.
