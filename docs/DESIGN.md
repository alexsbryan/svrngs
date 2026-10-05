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

**And the apps already exist.** Everything people self-host is a web app on a
port: Jellyfin and Navidrome for the library, Immich for photos,
Audiobookshelf and Calibre-Web for books, Nextcloud and Paperless-ngx for
files, Actual and Firefly III for money, Mealie, Vikunja and Grocy for plans,
Forgejo, BookStack, Home Assistant. Run one in a container on any member's
computer, `svrngs share <port> --kind <name>`, and everyone in the group can
open it — no port opened, no tunnel, nothing for them to install. An app that
takes the signed-in person from a request header lets each of them straight
in; one that keeps its own accounts asks them once. Jellyfin has run through
the door (`demo/runs/a/dry-2026-10-04/`).

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

**Joining: the client first, the browser a door a group opens** (operator,
2026-10-04). `svrngs new <name>` makes a group and prints its link,
`svrngs://join/<token>`. On a computer the svrngs program redeems it
(`svrngs join <link>`) and the computer is a member: running the client is the
way in, and nothing depends on a hosted URL. A group that wants phones
without an install names a page origin, one it hosts or ours, and its links
carry the same token as `<origin>#<token>`. On a phone that link opens in the
browser and the person is in: the page makes the person's own key and reaches
the group by dialling over iroh in wasm, relay-only. A dial from a page was
measured once, in headless Chrome for Testing 147 on Linux driven by
Playwright, through n0's relay (commonwealth-ai `ralph/DECISIONS.md:9909-10013`,
browser-dial-2, `HTTP/1.1 200 OK`); no phone has dialled. A phone holds no
copy of the record; it reads and writes through the nodes it dials. U4 builds
the rest, by five calls (the seat, 2026-10-05, from its survey of upstream;
the evidence is U4's inventory, §8):
1. *The page holds the key's bytes, sealed.* The person's key is 32 bytes,
   sealed by a non-extractable WebCrypto AES-GCM key, both kept in IndexedDB.
   It cannot be a non-extractable WebCrypto signing key: iroh signs its
   handshake synchronously, and the issuer redeems only for the key that
   dials it.
2. *A browser's key dials and is never dialled.* The redemption names whether
   its new key is a computer's or a browser's, and the register answers it for
   each standing key; the round, the live fan-out and the list of what members
   lend skip a browser's key. A phone's endpoint lives only while its page
   runs, so without this every round would wait out a dial per phone.
   (Landed, u4-1: `Device`, `Membership::device`, and `Device::dialled`
   asked by the node's `standing_peers` and the record API's `live_peers`.)
3. *The service worker holds the endpoint* and serves every request under its
   scope — a lent service's page, its fetches, a `<video>`'s byte ranges —
   through the dial; the page joins and lists. If a recorded run shows the
   worker stopped under playback, the endpoint moves to the page and the
   worker relays its requests to it. (Landed, u4-6: every lend has one
   address under the scope, `<scope>l/<lender>/<lend>/<path>`, the lender's
   key in full, so a page under it reaches any lend by a relative path; the
   worker holds one endpoint per group, opened from the sealed key on the
   first request after it starts, and dials each lender from it.)
4. *The phone dials whoever lends a service*, directly by key, since it
   stands in the group. The issuer, the node the link names, only redeems and
   answers what the group's people lend, each with its lender's key. No node
   proxies another's lend.
5. *One body for the link.* The command and the page parse a token and sign
   the ask and the redemption with the same code, in svrngs-core: bytes and
   keys in, bytes out.

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
tunnel and the accept loop (`commonwealth/crates/commonwealth-transport/src/iroh.rs`,
`mesh-reach/src/guest.rs`), and in U3 the rail routes an app writes through
(`rail.rs`). What each upstream piece
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
| `svrngs-core` | the record: acts, admission into one order, membership, the sync digest; and what a browser's page spells as the command and the node do (u4-2): a link's token, the ask and the redemption a joiner signs with it, the issuer's answer and the seed it is believed by (u4-5), the protocols' names, and a relay's URL without the DNS root's trailing dot, which WebKit refuses (u4-9). `no_std`, so the compiler refuses filesystem, clock, network and threads; time and keys are arguments | none |
| `svrngs-runtime` | the only owner of each group's journal on disk, its projections and its writes: the journal and its write index, the open-journal map, the register's standing | the journals |
| `svrngs-node` | the adapters: sync and its route over iroh, redemption, the loopback record API with its live lane (landed, u3-2 and u3-3), the member door (landed, u3-4), the list of what members lend, asked of each (landed, u3-5), and answered to a member by the computer it dials, each with its lender's key (landed, u4-5), and the door's limit and shares, a pure decision the door's task owns (landed, u3-6); every relay URL it makes or reads dotless where it enters — n0's relays, a relay it is told, a lookup's answers — in `relay.rs`, which the page compiles too (landed, u4-9) | each lend's declaration and secret; who is using each lend now |
| `svrngs-web` | the page's dial, wasm only, in `web/` with its static shell and a workspace of its own: a browser's endpoint, bound with the person's key, dialling a group's computer by key over a relay on the members' protocol (landed, u4-3); HTTP over the dial (landed, u4-4); joining — the ask and the redemption by svrngs-core's bodies, the key sealed in the page's store, and the list of what the group's people lend (landed, u4-5); the runtime, a service worker serving every request at a lend's address under its scope from the lender's door, one endpoint for every lender (landed, u4-6); an adapter | none: the page's store holds the key, sealed |
| `svrngs` | the command and the process: `new`, `join`, `link`, `share` and `unshare` (landed); the bare command prints where the record API listens, and what the people of its groups lend | a device's keys; each group's seed |

Held to `docs/principles/RUST_ARCH.md`: types encode the domain, ownership
encodes the architecture, I/O lives at the edge. Parse, don't validate; closed
sets are enums; one writer per journal, and no shared mutable state; a trait
only at a real boundary; `thiserror` in the crates, `anyhow` only in the
command and the app. Every decision branch traces at debug. New code is held to
these from its first line; forked code is held to them as it becomes ours.
Upstream's shape stays until a unit changes it: the node's map of open
journals sits behind a `Mutex`, which an actor per journal replaces.

| Invariant | Held by | State |
|---|---|---|
| Same acts, same state: a group's state is a pure function of its set of acts | svrngs-core is `no_std`, and `lean_check` keeps it so; an adapter reads a group only through the runtime's queries (`admission`, `standing`, `invite_for`), since the journal is crate-private to svrngs-runtime; and it reads an admission only as a `Projection` sealed in svrngs-core (`Admission::project`) or as `wire()`, the acts serialized for an app to fold, since the acts are private to the core | holds for every adapter; `wire()`'s JSON can be parsed back into acts, and no gate refuses that; inside the core, `membership` still sorts the acts admission hands it |
| No act, no effect: nothing that cannot be undone happens but from an act in the record — the group's state and this computer's; a lent service's own state is its own, and the member door only carries a request to it | a store appends only `Authored` lines, which only `commit` makes after a `Command` decided under the write lock; an effect runs from a `Due`, which only `Admission::react` makes, from a `ProcessManager` sealed in svrngs-core and only when every act it names as a cause stands in the record | every write goes through `commit`; compaction, the one effect, deletes only from its seals' `Due<Retire>`; the journal's `append` is a test fixture |
| Both | the core never depends on the node; the clock, randomness and locks live only where an adapter holds one (`clippy.toml` bans them; each site carries `#[expect(…, reason)]`, which fails once the site is gone) | holds |

One property test stands behind every fold: permuting or duplicating
`admit`'s input changes nothing. A decider's contract is given / when / then
rows.

**Two gates keep the apps out of the infra.** Every app lists its own nouns in
`apps/<app>/nouns.txt`, and `scripts/vocab_check.py` refuses any of them in
infra code — comments and tests apart, where an example is fine. So the infra
never learns what a film, an expense or a crush is. And the canaries of §8,
Expenses and Tap, run against the infra in `scripts/check.sh`: a change that
serves Films and breaks one fails the build.

**Lean, and kept lean.** `svrngs-core` stays under 3,000 lines of code and the
repository under 14,000, tests and `apps/` apart (`scripts/lean_check.py`);
a raise is the operator's, with the reason in the commit. `svrngs-core` stands
at 2,359: the operator raised its ceiling from 2,500 on 2026-10-05, for what U4
to U7 still add to the core, since REVIEW-audit-u3-auto-2 found no duplication
in it (ralph/REVIEW_FINDINGS.md #38), and the repository's from 12,000, at
9,601, for U4's page and its wasm and U6's handoff, about 2,100 lines between
them by the seat's estimate. `ARCHITECTURE.md` maps what
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
  list a door keeps. The seed is the founder's key alone, bound to the
  founder's name: `svrngs new` writes it into the journal's `roster.json`
  from the group's `seed.json` (`Seed::roster`, `crates/svrngs/src/group.rs`),
  and the issuer answers the seed to whoever proves a link's key (u2-8).
- **A grow-only set, linked like git.** Each act names its author's previous
  act (U2, inside the id and the signature, so the wire changes once, before
  Demo A writes a journal anyone keeps; landed: `SignedOp::prev`, and
  admission names an act whose previous act is not its author's act at
  seq − 1 a `WrongPrevious` gap, `crates/svrngs-core/src/admit.rs`); a copy
  holds an act only with its parent, so copies converge by union. Two acts on one parent are a fork, and
  forks void nothing (U7; operator, 2026-10-04, moved from U2). Today two acts
  on one seq exclude both (`crates/svrngs-core/src/admit.rs:467-492`), so
  whoever signed an `Admit` can take it back on every copy by signing a second
  act at its seq: a removal nobody sequenced. Until U7 that grants nothing a
  member lacks, since any standing key's `Remove` counts
  (`membership.rs:153-156`), so it lands with the removal it protects, and
  with the sync that carries both branches of a fork. U7 also closes two holes of the
  same family that only a member can open, each leaving a copy that never
  reads as complete while the acts themselves still apply: a forged line
  above a member's newest act moves their door's next seq past it
  (`crates/svrngs-node/src/journal.rs`, `Held::catch_up`; ralph/REVIEW_FINDINGS.md
  #16), and an act refused inside its author's chain is reported as a hole,
  since admission's seq audit counts admitted acts only (`admit.rs`, the
  per-actor sequence audit; 40b9466's body).
- **Removal has one decider** (U7). Anyone may ask to remove anyone; the group's
  keeper — one person's key, handed on by an act — sequences each request. A
  removed key cannot remove. There is no recovery inside a group.
- **Leaving is cheap.** A group's whole state is a directory every member's
  computer holds. A group that goes wrong refounds from any copy, within
  hours, without whoever it leaves out (operator: "with dotfile elegance").
- **A link admits each person once** (U2; operator, 2026-10-04, over one
  person per link, which neither demo nor a chat door could use: Sam sends one
  link to three friends, a door posts one per channel). A link carries a
  secret, and the key derived from it is the key its issuer's `Invite` act
  names, with the time it lapses. Redeeming it
  writes one `Admit` for one new key, signed by the issuer and co-signed by the
  new key. A link admits everyone who opens it until it lapses or its issuer
  closes it, and only the issuer's node redeems it (operator, 2026-10-04), so
  one node decides every use, and the fold counts a redeeming `Admit` only
  under an `Invite` that still counts. Landed in the record:
  `RailAct::Invite{key, until}`, and an `Admit` whose `invite` names the
  `Invite`'s id with the new key's signature over the namespace, that id,
  the person and the key (`Redemption`, `crates/svrngs-core/src/lib.rs`).
  `membership` counts it only when its signer signed the `Invite`, the
  `Invite` is held, counted earlier in the order and no counting `Correct`
  voids it, no newer `Invite` of the issuer's counted before the `Admit`, the
  `Admit` is timed before `until`, and the co-signature verifies; otherwise
  admission names a `RefusedRedemption` gap with its `InviteFault`. A link
  closes by its issuer's next `Invite`, from that `Invite`'s place in the
  order (u2-4b, landed: `InviteFault::Closed` names the newer `Invite`), so
  everyone the old link admitted keeps standing. A `Correct` of an `Invite`
  is not how a link closes: the void set is order-free, so it unseats every
  key the `Invite` admitted, as voiding each `Admit` would (`Voided`). Both
  kinds ship at line version 3. A redemption also names what holds the
  new key, a computer or a browser (u4-1, landed: `Device` in the
  `Redemption`, `crates/svrngs-core/src/lib.rs`); a browser's `Admit` ships
  at line version 4, a computer's keeps version 3's bytes, and one naming
  none, as every earlier line, is a computer's. Redeemed at the issuer's node (landed:
  `crates/svrngs-node/src/redeem.rs`): on the second protocol, the joiner,
  holding only the link, first asks, proving the link's key, and the issuer
  answers from its own `Invite` naming that key: the group's id, its seed
  and the `Invite`'s id, the seed believed only when it hashes to the id
  (u2-8; `group::answered`). It then
  presents the group's id, the `Invite`'s id, a person's name, its new key
  — the key it dials with — that key's consent and whether a computer or
  a browser holds it, all signed by the
  link's secret key; the node checks the name parses, the `Invite` is its
  own and names that key, then writes the `Admit` through its door only when the key does not
  stand already and `membership` over its journal with the `Admit` in it
  counts it, both under the write lock (`commit` with svrngs-core's `Redeem`), so the
  fold stays the one decider of a lapsed, closed or voided link. Every
  refusal is one sentence naming no group; a key that already stands is
  answered success and nothing is written.
- **A person's name is parsed once, in the core** (u2-9, landed): trimmed,
  1..=64 bytes, no control characters (`Name::parse`,
  `crates/svrngs-core/src/lib.rs`). A `Person` is made only from a `Name`,
  and deserialising one requires the name as parsed, so an `Admit` naming
  an empty person, 10 KB or a terminal's escape codes is a `MalformedLine`
  on any copy holding it, and its key gains no standing. The redeem route
  parses the name it is sent once the dialer and the link's key check out,
  before any group is looked at, writes nothing on a name that does not
  parse, and answers why it is not one, which names no group. The command's group and
  person names are the core's `Name`.
- **Ids cover everything signed** (U2, landed): the id is the whole hash of
  the namespace, actor, seq, previous act, timestamp and body, and the
  signature is over the id (`act_id` and `ring_op_message`,
  `crates/svrngs-core/src/sig.rs`). Every comparison of acts as identity —
  admission, membership, the view's heads, sync, and the journal's held set
  and ingest skip — uses the id the content derives (`derive_id`), never the
  `id` a line carries. Before, as forked, the id hashed the signed line,
  signature included, and kept 64 bits of it, a prefix `hash.rs` says is
  "never an identity", yet a peer's act was skipped as already held by the
  line's id: two acts one member ground to share a prefix, about 2^32
  hashes, were held one or the other by whichever a copy saw first, and
  copies diverged. A removal names the removed key's acts by hash,
  never by position or time (U7): today the fold ignores `through_seq` and
  cuts the key at the `Remove`'s place in the order
  (`crates/svrngs-core/src/membership.rs:153`), so a backdated act slips under
  the cut. The same order would put an act signed in the second of the
  `Admit` that gave its author standing before that `Admit` when the author's
  key sorts first (`membership.rs:124`), so the act would not count. The door
  signs at the later of now and one second past the newest act its journal
  holds (U2, landed; now `commit`'s, `crates/svrngs-core/src/command.rs`),
  upstream's measured fix (`commonwealth/crates/commonwealth-rail/examples/stage_member_act.rs:123-135`
  at `e1a43bcd2`), so a member's first act sorts after the `Admit` it has
  seen; `crates/svrngs-node/tests/group_not_mesh.rs` writes at once, under a
  new key that sorts first. Before, at the wall clock, that test failed 8
  runs in 10 without the wait it then carried. Writes faster than one a
  second run the stamps ahead of the clock, one second per write.
- **A signing key per person per group**, so a cut in one group touches no
  other; an endpoint key per device.

## 7. Packaging and reach

Computers first, running the client; phones through the browser for a group
that names a page origin: no install, no store, the same token. Computers
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
| the mesh: `commonwealth-core::mesh`, read by `RoundMembers::of` (`ring_sync/journal.rs:146`), `member_check` (`acceptor.rs:49`) and `MembershipRosterSource` (`rail.rs:113`, installed at `lib.rs:404`) | who the round dials; who may connect; every ring's default roster | **the register**: a group's members are the keys standing in `membership` over its seed and its `Admit` and `Remove` acts (`crates/svrngs-core/src/membership.rs`), read through `admit` over the group's `Admit`, `Remove` and `Correct` acts by `RingRail::standing` (`crates/svrngs-node/src/register.rs`). The round offers a group to its standing keys, the acceptor admits a key standing in any group the node holds, and the sync route asks the group the request names. No default roster is installed |
| `commonwealth-core::ids` | `NodeId`, the mesh's id for a machine; `NodePubkey`, a verified key | a member is the record's actor, its key's lowercase hex (`actor_of`), read from the key iroh verified; `NodeId` goes with the mesh |
| `commonwealth-transport::mesh_proof` | proves the mesh secret on a plain-IP hop | left behind: svrngs dials only over iroh, whose handshake proves the dialer's key |
| `commonwealth-transport::iroh` and `mesh-reach::guest` | dial by key: a loopback bridge per peer for the dialer (`IrohTransport`, `HttpBridge`, `pump`); for the dialled, an accept loop that copies each stream into the node's loopback listener (`IrohAcceptor`); the relayed endpoint and its discovery setting | the dialling half forked: one ALPN, no IP transport, no bench relay pin. The accept loop forked, and what it copies into replaced: hyper serves the node's router on each accepted stream, the key iroh verified reaching handlers as a typed request extension, so peer routes have no loopback port and no caller is served as local for lacking a header (`ring_routes.rs:65-72`). Upstream names this its upgrade path (`commonwealth/crates/commonwealth-transport/src/iroh.rs:23-24`) |
| `mesh-reach`'s port: `PeerTransport`, `PeerContact`, `PeerEndpoint`, `TrafficClass` | one seam over IP and iroh; a port per class of traffic | replaced: one transport, so a concrete type (RUST_ARCH: a trait only at a real boundary); a contact is a member's key; one ALPN constant, and an enum when a second protocol comes |
| `commonwealth-transport::iroh_identity_forward`, with `kernel-types::member`'s header scheme | stamps the dialer's verified key on every request head it parses and strips any the client typed | not forked: on the stream a handler reads the key from the connection, so no header can claim one, and only hyper parses a request. Upstream's rewrite passes the rest of a connection through untouched after a chunked body (`iroh_identity_forward.rs:419`, `iroh_routed_forward.rs:159`), so a member could name another member's key on the next request. U3's door sets the verified headers on the request it proxies, from the extension |
| `commonwealth-media::origins` (`OriginRegistry`, `admit_spliced_origin`) and `commonwealth-transport::iroh_routed_forward` | a table of many programs' origins by ALPN and path prefix | replaced: U1's node serves one router, its sync route, to members; U3's lent services are routes on it |
| `host-kit::shell::RouteBundle` | named route bundles a shell mounts | replaced by an `axum::Router` |
| `rail.rs`: `err` | the refusal body | forked with the route |
| `rail.rs`: `append`, `RailQuery`, `LiveBuffer`; the live route (`ring_routes.rs:163`) | the record API an app writes through; the nudge that wakes the round after a write; the live lane | U3, as the loopback record API: append, with the nudge, landed in u3-2 (`crates/svrngs-node/src/rail.rs`, `ring_sync::append`); the live lane in u3-3 (`LiveBuffer` and its doors in `rail.rs`, the live route in `ring_routes.rs`, the fan-out in `live.rs`) |
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
presence:* the round dials every standing member's computer, and one that
is offline costs a dial timeout each round; a browser's key is never
dialled (u4-1, §3 call 2). *The door asks the register too:*
`RingJournal::append` refused a signer the seed did not name
(now `commit`'s standing check, `crates/svrngs-core/src/command.rs`), which would leave two answers to
who is in a group (ARCH 8); in U1 it asks the same standing the round, the
acceptor and the sync route ask, so a member the founder admits can write.
`svrngs-node` gains tokio, axum,
reqwest and iroh, and iroh 1.0.2 raises the toolchain floor to Rust 1.91.

Until U2, a test admits members by writing `Admit` acts directly, as
upstream's `commonwealth/crates/commonwealth-rail/examples/stage_member_act.rs`
does, and runs two nodes in one process with no relay — the shape of upstream's
`commonwealth/crates/commonwealth-rails/tests/ring_round.rs`, which is built on
the mesh daemon and so is rewritten, not forked.

**U2's inventory.** What upstream at `e1a43bcd2` has for making a group and
joining one, and what each becomes. Upstream has no ring invite: nothing mints
a link for a ring, counts its uses or writes an `Admit` when someone joins.
What it has is a mesh invite, reusable until it lapses, and a guest grant,
reusable by design.

| Upstream | What it does there | Becomes |
|---|---|---|
| `mesh-join-vocab/src/deep_link.rs`: `build_https_guest_link` and `parse_https_guest_link` (`:244-300`), with `percent_encode` and `parse_query_params`; `build_join_link` (`:421-436`) | the-link's https form: every secret in the fragment, a parser that never reads left of the `#`, the dial as `iroh=`; and the client's form, `sovereign://join/<key>` | forked, then made short and opaque (u2-8; operator, 2026-10-04, §10): one base64url token carrying only what the joiner cannot ask the issuer for — a format byte (`Format`, a closed set), the issuer's key, which it dials, and 16 secret bytes the link's signing key is derived from (BLAKE3 `derive_key`, a named context), and the issuer's relay only when it runs without address lookup (§11, risk 5). A client's link with no relay is 80 characters, where the seven-field https link was 412, with the issuer's network addresses in it. The issuer answers the group's seed and its `Invite`'s id to the link's key (§6). Two wrappers carry the one token: the client's link, `svrngs://join/<token>`, upstream's `sovereign://` shape, which `new` and `link` always print; and `<origin>#<token>`, printed beside it only for a group that keeps a page origin, so nothing depends on a hosted page unless a group names one. `svrngs join` takes either, or the bare token. The origin is the group's setting on each computer, not a flag each call reads: `new --origin` keeps one, `join` keeps the one its https link was printed on and none from a client's link, and `link --origin` changes it. A computer never fetches the origin; it dials the issuer by key over iroh. A phone (U4) loads the page there, which dials over iroh in turn, so the origin is trusted with that page's code and never sees the fragment, which is why it can be anyone's. The https form with the key in its path (`:396-413`) stays behind: a host reads a path. A token whose random part spells a word §1 keeps from people is drawn again |
| `commonwealth/crates/commonwealth-rails/src/found.rs:46-114`, `join.rs:116-229`, `internal.rs:222-295`; `commonwealth-discovery/src/membership.rs:12-75` | the mesh invite: a 48-bit key, reusable until a 24-hour lapse (`membership.rs:53`), only its hash kept, in `mesh.json` outside any record. The joiner dials the founder by key and posts `/internal/join` with a proof that signs neither the key nor the mesh (`commonwealth-transport/src/identity.rs:130-141`); the founder checks and persists before answering | the handshake's shape forked: the joiner dials the issuer by key and proves it holds both the link's key and its own, over the group and the request; the issuer persists before answering. What it writes is an `Admit`, and every use of a link is in the record: the `Invite`, its lapse, its closing. The lapse stays upstream's day. The mesh snapshot, `MemberRecord` and `node_id` stay behind. Landed: `crates/svrngs-node/src/redeem.rs` |
| the acceptor U1 built (`crates/svrngs-node/src/node.rs`, `admit_dialer`) | closes every dialer standing in no group the node holds | kept for members. Redemption is served on a second ALPN, the one a key in no register may speak, with one route; `alpn.rs` becomes the enum it promised (landed: `Alpn`, and `Admit::Serve` names the router each protocol reaches) |
| `sovereign-grants`' guest grant (`guest_grant.rs`, `guest_session.rs`) | one QR code serves a room; its state is a map in memory | not forked: its state is lost on a restart. Its shape, one link for a room, is what U2 keeps, in the record |
| `commonwealth-transport/src/identity.rs:26-126` | the node key: a 32-byte seed at `<data_dir>/node_key`, written through a temporary file and a rename, mode 0600; an I/O error falls back to a key nobody keeps | forked with edits as the command's device key. An unreadable key refuses: a computer never silently becomes someone new (ARCH 6). The mesh proof and dial-info signing (`:128-184`) stay behind |
| `commonwealth-rails/src/cli.rs` and `main.rs` | four verbs, parsed by hand, no framework; exit codes 0, 1 and 3; a tokio runtime built by hand | the shape of `crates/svrngs`: `new <name>`, `join <link>`, `link <group>`, and the bare command, which runs the node for every group the computer holds. No parser dependency |
| `commonwealth/crates/commonwealth-rail/examples/stage_member_act.rs:94-157` | an `Admit` signed by a member, timed strictly after everything held | its signing steps are the redemption's write; its timing rule moves into the door (§6) |
| `sovereign/crates/sovereign-cli-mesh/src/ring_cmd/mod.rs:356-470`, `ring roster add` | joining a ring: two people copy a hex key by hand | replaced by the link; nothing forked |
| svrngs's own `valid_namespace` (`crates/svrngs-node/src/lib.rs:43-50`) | a group's id is a name the caller types, so two groups called House on one node collide | a group's id is the hash of its seed — the founder's key and the name they go by, the group's name and random bytes drawn at `new` — so a seed the issuer answers for a link proves the id. The name is what a person reads (landed: `crates/svrngs/src/group.rs`; the founder's name added, since the register binds every seed key to a person) |

Four facts follow. *Only the issuer redeems:* a link works while its
issuer's computer is reachable; Sam's always-on computer is, in both demos,
and any member redeeming waits in §9. *A new link closes the old one:* an
issuer has one open link per group, and minting another closes the last from
its place in the order: a redemption under the old link after the new one is
refused, and nobody the old link admitted loses standing (u2-4b, landed in
`membership`, `crates/svrngs-core/src/membership.rs`). So a leaked
link is closed by sending a new one, with no verb of its own. (Until
2026-10-04 this said the new link voids the old with a `Correct`; voids are
order-free, so that would have unseated everyone the old link admitted.) *No person's key seals yet:* `Admit.key` signs and dials, and
`svrngs-core` carries no encryption at all, so the key the Tap canary seals to
arrives with Tap, after U3. *A move is not a raise:* `scripts/lean_check.py`
counts the tests written inside `svrngs-core`'s source files, 798 of its 2,293
lines, though §5 sets tests apart; U2 moves them beside the files they test
before its rules land, and a raise stays the operator's.

**U3's inventory.** What upstream at `e1a43bcd2` has for lending a localhost
service and reaching the record from an app, and what each becomes. Upstream
has no `share` verb, no limit on how many people use a lend, no per-lend
refused paths, no busy refusal with a reason, no count of the people using a
lend, and nothing that measures a computer's upload: every counter in it and
in iroh 1.0.2 measures use, not capacity, and iroh's default congestion
controller (Cubic) reports no bandwidth estimate.

| Upstream | What it does there | Becomes |
|---|---|---|
| `sovereign/crates/sovereign-cli-mesh/src/publish_cmd.rs` (`svrn publish <name> <port>`) and `mesh_media/offer.rs` (`probe_origin`, Jellyfin's well-known ports) | writes a lend into `config.toml`; probes a port for a known service | the verb's shape and the probe, with edits: `svrngs share <port>` writes a lend declaration into the data dir, and works whether or not svrngs is already running (a running node reads the declarations; one per data dir, `crates/svrngs/src/cli.rs` `claim`). Landed (u3-5): `crates/svrngs/src/share.rs`; a port alone, on 127.0.0.1; the kind the given one, else the port's from a table kept as data (`well_known.txt`, 8096 alone: the door speaks plain HTTP, so upstream's TLS port 8920 is not shared by number), else refused; the name the given one, else the kind; the limit the given one, else 3, printed; the secret read from a prompt, unechoed on a terminal, sent as `authorization`; `share` runs the node unless svrngs already runs, and `svrngs unshare <name>` withdraws a lend |
| `commonwealth/crates/commonwealth-media/src/declared.rs:51-77, 169-229` | the lender's secret: a header value in a 0600 file under a 0700 dir, named by its header, never logged, never in anything shared | forked with edits: the lend's secret stays in the data dir and never enters the record. A lend's public part (name, kind, limit) is answered by its lender's node, never stored: the list of what members lend is computed by asking each standing member, as upstream's offers were (`commonwealth-media/src/reach.rs:131-168`), so an offline computer's lends are absent rather than stale. Landed (u3-4): `crates/svrngs-node/src/declared.rs`, the store under `<data dir>/secrets/lends/<lend>/`, a failed hardening failing the write; the declaration `<data dir>/lends/<lend>.json` (`lend.rs`), read per request. Landed (u3-5): the declaration holds the port, the kind, the limit and the refused paths, written and withdrawn only by `lend::declare` and `lend::withdraw`; `GET /lends` on the member ALPN answers a dialer standing in a group held what this computer lends now and how many people use each (the door's count of the people with a request in flight, `door::Using`; since u3-6, the people its limit counts); `Node::lends` asks every standing key at once and answers each lend with whose it is and an address on this computer's loopback, the dialler's bridge then `/lend/<name>/`, naming a computer that did not answer |
| `commonwealth-media/src/identity.rs:118-165` (`admit_spliced_origin`) and `commonwealth-transport/src/iroh_identity_forward.rs:297-328` (`rewrite_head`), with its tests at `:620-702` | refuse a non-member and a member outside the allow list; strip every client identity header and every header about to be declared, compared case-, padding- and control-byte-blind; then set identity, then the declared secret | the rules forked into the door's handler, the code not (§8, U1's inventory): the person and key come from the `Dialer` extension and the register, and one prefix of svrngs's own is stripped whole. The tests come over as tests. Landed (u3-4): `crates/svrngs-node/src/door.rs`, `/lend/<lend>/<path>` on the member ALPN; `x-svrngs-person` and `x-svrngs-key` set, every client header under `x-svrngs-` stripped; a client's padded or control-byte header name never reaches the door, since hyper refuses the request (400) |
| `sovereign/crates/sovereign-daemon/src/serve_client.rs:616-655` (`forward_stream`), the strip lists of `sovereign-pods/src/worker_inference_proxy.rs`, and `commonwealth-transport/src/iroh_routed_forward.rs:318` (`route_by_prefix`, its traversal guard) | a GET proxy that streams a response and passes `Range`; hop-by-hop strips; refusing `..`, `%2e`, `%2f` | with edits: every method, a streamed request body, `If-Range`, and the lend's refused paths matched as `route_by_prefix` matches a prefix. Upstream streams no request body anywhere. Landed (u3-4): one hyper connection per request, both bodies streamed; a refused path compared percent-decoded, case-blind and with repeated slashes as one; a response header holding the secret dropped |
| `commonwealth-transport/examples/media_bridge_bench.rs` (its Range-serving origin) and `commonwealth-rails/tests/two_daemons.rs:208-346` (`picky_origin`, 401 then 200) | prove Range sha-exact through the splice; prove a declared credential arrives | forked as test fixtures. Landed (u3-4): `crates/svrngs-node/tests/door/origins.rs` |
| `commonwealth/crates/commonwealth-rails/src/rail.rs`: `append_act` (`:244`), `log_answer`, `LiveBuffer` (`:781`), `err`; `api.rs:167` (`check_loopback`) | the record over loopback: append, log, and a live buffer drained by polling; loopback is the auth | forked with edits as the loopback record API: append, log and live only; the attestation branch goes with attest; the roster becomes the register. An app's act is signed by its member's node, with `on_behalf_of` naming the person the door verified when the app serves someone else. Landed (u3-2): append, log and the people with their keys (`GET /v1/rail/people`, from the register), on 127.0.0.1, each caller refused unless on loopback (`crates/svrngs-node/src/api.rs`); live in u3-3: `POST /v1/rail/live` offers a payload of at most 4096 bytes to the group's other standing keys, `GET` drains this computer's buffer for one group |
| `sovereign/crates/sovereign-daemon/src/routes_rail_live.rs:76-196` (`push_ephemeral`) and `ring_routes.rs:163` (the live route) | fans a live payload out to each online member | with edits: dialled over `IrohTransport` to the register's standing keys; `LiveEnvelope` (`crates/svrngs-node/src/peer_wire.rs:85`) gains its user (ralph/REVIEW_FINDINGS.md #11). Landed (u3-3): `crates/svrngs-node/src/live.rs`; the live route admits a dialer standing in the group its envelope names and refuses the rest, as the sync route does |
| `serving-policy-core/src/fair_sched.rs:163` (`fair_share_cap`) and `try_grant` | divide a budget evenly among the active; granted, queued or shed | the shape of the share and the limit, counted in people, not slots. Landed (u3-6): `crates/svrngs-node/src/door/limit.rs`, pure, the time an argument; a person counted while a request is in flight or within `WINDOW`, 30 s, of one; nothing queues: a newcomer past the limit is refused 503 with the reason and how many are using it; the share, the declared upload over the people counted, in `x-svrngs-declared-share-bps` on the request and its answer. The upload is the lend's `upload_bps`, from `svrngs share --upload <Mbit/s>`; a lend declaring none is told no share |
| `commonwealth-transport/src/iroh_path.rs` (`PeerPath`) and iroh's `Connection::paths()`, `Path::is_relay`, `PathStats` | whether a peer's path is direct or relayed; per-path bytes, RTT and congestion window | read for the trace a demo watches; never mistaken for capacity. Landed (u3-6): `iroh::Via`, the connection beside the `Dialer`, whose selected path's address, relay, RTT and bytes the door traces at info with each share |

Four facts follow. *The upload is declared:* the lender states it, measured by
hand as Demo A's run-of-show already does, and the share the door tells each
request says it is a declared figure (ARCH 6); measuring it waits in §9.
*Lends are asked, not stored.* *The local network is guessed:* iroh says
nothing about it, so a viewer whose path is a private, link-local or
unique-local address is taken to be in the lender's home and not counted,
and the trace says it guessed. *Localhost is the trust boundary:* any process
on the lender's computer can reach a lent port and type the identity header,
as it can reach any localhost service; upstream's per-process tie waits in
§9. Jellyfin is GPL-2.0-only, so svrngs talks to it at arm's length, over its
API (commonwealth-ai `quality/campaigns/ring-apps-shelf.md:187`). And
E0's Discord door, which called a host deleted with E0, went as the engine
did (u3-1); the chat doors start from tag `e0-bot` when their turn comes.

**U4's inventory.** What upstream at `e1a43bcd2` has for a phone's browser,
and what each becomes; written from the seat's survey
(`ralph/next/u4/UPSTREAM.md`), each path:line read again at `e1a43bcd2`. Upstream
has a wasm iroh dialler with a static page around it and the scripts that
build and deploy it, and nothing else U4 needs: no HTTP client in the browser
beyond one buffered GET, no fetch interception, no byte ranges, no request
bodies, no key kept in a browser, no redemption from one, and no measurement on
a phone. Paths under `ring-runtime/` are `sovereign/apps/ring-runtime/`;
sizes are lines of code, as the survey counted them.

| Upstream | What it does there | Becomes |
|---|---|---|
| `ring-runtime/src/lib.rs:1-203` (153) | one export, `dial_guest(dial, token, path)` (`:62-63`): parses `<hex>@<targets>` and refuses one with no http(s) relay URL (`:91-99`); binds an endpoint, `presets::Minimal`, a fresh `SecretKey::generate()` on every dial, `RelayMode::Default`, `GUEST_ALPN` (`:119-123`); waits for `online()` (`:139`), connects to the relay address (`:142-145`) and opens one bi-stream (`:159`); writes a hand-built `GET … Connection: close` (`:166-168`), reads `read_to_end(512 * 1024)` (`:178`) and finishes the send half only after it (`:185-187`); every layer pushed to `window.__trace` as it happens (`:27-35`) | forked into `web/` as svrngs-web (u4-3), upstream's file first with only `kernel-types` replaced by svrngs-core; then the person's key from the page's sealed store in place of `generate()` (call 1), the member ALPN, and a dial by key: the token's relay, else iroh's address lookup, which the N0 preset does in a browser over HTTPS (`PkarrResolver::n0_dns()`, `iroh-1.0.2/src/endpoint/presets.rs:122-128`), never refusing a bare key. The trace stays. The hand-built request and the buffered read go in u4-4: hyper's HTTP/1 client over each stream, any method, a streamed request body, the response body a `ReadableStream`, the send half open until the response ends, no cap. Landed (u4-4): `web/src/http.rs`; `connect(key, token)` answers the dialled computer, whose `fetch(method, path, headers, body)` answers a page's `Response`, its body handed over as each chunk arrives off the stream |
| `ring-runtime/Cargo.toml:1-34` (14) | a cdylib in its own `[workspace]` (`:32-34`), so the host workspace never builds it; "The LOCKED pin … measured WORKED at exactly these versions" (`:21-26`): iroh =1.0.2, wasm-bindgen =0.2.128, wasm-bindgen-futures =0.4.78, js-sys =0.3.105; `kernel-types` by path (`:30`) | forked with its pins and its own workspace (u4-3), svrngs-core in place of `kernel-types`. iroh 1.0.2 has no wasm feature: its build script turns the browser on by target (`iroh-1.0.2/build.rs:7`) |
| `ring-runtime/web/index.html:1-45` (27) | the static page: a CSP of `default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self' wss: https:; img-src 'none'; …; base-uri 'none'` (`:25`); `referrer` `no-referrer` (`:24`), so the fragment never leaves in a Referer; the script by absolute path, `/ring/app.js?v=__VERSION__` (`:43`) | forked (u4-3); its words become §1's; `no-referrer` and the strict CSP kept; the shell's `/ring/` paths follow wherever a group's origin serves the page |
| `ring-runtime/web/app.js:1-68` (47) | reads the fragment's `token`, `iroh`, `path` and `s` (`:25-29`); `init()`, then races `dial_guest` against 120 s (`:41-45`); pulls status and body out of the trace text (`:49-53`); registers the service worker after load, best effort (`:64-68`) | forked (u4-3), then the joining page (u4-5): the fragment is svrngs's token, parsed by svrngs-core (call 5); it asks the person's name once, makes and seals the key (call 1), asks and redeems at the issuer naming itself a browser (call 2), keeps the key and the group's name, and lists what the group's people lend. Landed (u4-5): `web/app.js` and `web/store.js`, `web/src/join.rs`, and the node's `GET /people/lends` |
| `ring-runtime/web/sw.js:1-34` (25) | a cache-first shell cache named by content hash (`:5-11`); same-origin GETs only (`:28-34`): "The dial itself is not HTTP to this origin — it is iroh — so requests to it never reach here" (`:26-27`) | forked (u4-3), the shell cache kept; then the runtime (u4-6, call 3): it opens the endpoint from the sealed key when it starts and again after the browser stops it, and serves every request under its scope through u4-4's client to the lender's door, `/lend/<name>/<path>`. Landed (u4-6): `web/sw.js`, a module, with `endpoint(key, token)` and its `dial(lender)` in `web/src/lib.rs` — one endpoint per key, since a relay sets an older endpoint with the same key aside when another binds (`iroh-relay-1.0.2/src/server/clients.rs:80-92`) |
| `scripts/build-ring-runtime.sh:1-119` (75) | checks for the wasm target (`:31-35`) and a `wasm-bindgen` CLI of exactly 0.2.128 (`:36-49`); on macOS uses Homebrew LLVM's clang and llvm-ar, since Apple's clang has no wasm32 backend (`:52-76`); `RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --release --target wasm32-unknown-unknown` (`:81`), the cfg for `kernel-types`' getrandom 0.3 (`:77-80`); `wasm-bindgen --target web` (`:85-86`); `wasm-opt -Oz` when installed (`:91-97`); a 12-hex content hash stamped into the three shell files (`:103-107`) | forked as `scripts/build-web.sh` (u4-3). Whether the getrandom cfg stays once `kernel-types` is gone is a change of meaning, its own commit |
| `landing/scripts/build-ring-runtime.sh:1-24`, `landing/scripts/verify-ring-deployed.sh:1-92`, `landing/vercel.json:1-73`, `landing/package.json:13` | build into `landing/ring/` and deploy one origin: "The trailing slash is LOAD-BEARING" (`build-ring-runtime.sh:7`), after "a phone hung on the wall link" (`:11-12`); a redirect `/ring` → `/ring/` (`vercel.json:4-10`); `sw.js` uncached and the wasm immutable (`:64-71`); a deploy checked by fetching the page's script as a browser resolves it | not forked: which origin serves the page, and how, is a group's. Their facts go into u4-8's package for the operator: a page served at a path needs its trailing slash, `sw.js` uncached, the wasm immutable |
| `landing/ring/` (generated, committed) | the built site; `wasm/ring_runtime_bg.wasm` is 4,295,444 bytes (`git ls-tree -l`); its glue reaches the relay with `new WebSocket` | not forked. Upstream's one figure for a page's weight. Ours, measured (u4-8, `demo/runs/u4/2026-10-05/`): the page fetches 5,103,571 bytes, 1,209,407 brotli, its wasm 5,027,239 without `wasm-opt`; a cold join took 15.35 MB from an origin sending no cache headers, the wasm fetched by the page, the worker's install and the worker's start (`ralph/REVIEW_FINDINGS.md` #51) |
| `kernel-types/src/alpn.rs:26` | `GUEST_ALPN = b"cwth/guest/0"`, the one spelling both ends import | not forked: svrngs's protocols are its `Alpn` (`crates/svrngs-node/src/alpn.rs:29-30`), spelled in svrngs-core from u4-2, so the page and the node spell each name once |
| `mesh-join-vocab/src/deep_link.rs:228-300` | the https link's build and parse, never reading left of the `#` | forked already (U2): `crates/svrngs/src/link.rs`; u4-2 moved the token's parse, now `crates/svrngs-core/src/link.rs`, and the signing of the ask and the redemption into svrngs-core (call 5), leaving the command `issuer.rs`: a fresh token's secret, and the issuer as iroh dials it |
| `sovereign/crates/sovereign-contracts/src/guest_pages/ring_shim.js:1-171` (105), with `shim.rs` and `guest_pages.rs` | `window.ring` over plain `fetch` to a LAN door: the bearer from `location.hash` (`:5`); a name claimed by `window.prompt` and `POST /v1/guest/session`, its handle kept in `localStorage` (`:30-61`) | not forked: a guest on a LAN door, which never dials iroh and for whom the host signs. svrngs's phone is a person with a key in the register. Both retire on svrngs's bar (`docs/RING_SPEC.md:26`) |
| `sovereign/crates/sovereign-daemon/src/guest_door.rs:617-729` (104) | the LAN door's proxy: the request body buffered to 16 MiB (`:619`, `:646`); headers forwarded but `host`, `connection`, `content-length` and `accept-encoding` (`:659-667`); the response buffered whole (`:689`) and only its status, `content-type` and body returned (`:681-696`) | not forked: it drops `Content-Range` and streams nothing. The other end of a phone's streams is U3's door (`crates/svrngs-node/src/door.rs`) |
| `sovereign-mobile/src-tauri/src/iroh_bridge.rs:14-17`; `scripts/ring-room-phone/phone.mjs` | a native Tauri app's iroh, its key "ephemeral (fresh key per app run)"; a Node stand-in for a phone browser, with a fake `location`, `localStorage` and `fetch` | not forked: a native phone app waits (§9). The browser tests drive real engines with Playwright, Chromium and WebKit with iPhone emulation (u4-7) |
| nothing | an HTTP client over iroh in wasm; fetch interception; byte ranges in a browser; request bodies; a key a browser keeps; redemption from a browser; a dial by bare key from wasm; any measurement on a phone (the survey searched every `.js`, `.ts`, `.html` and `.rs` file for each) | built new: the client (u4-4), the key and joining (u4-5), the runtime (u4-6), the measurement and the operator's package (u4-8) |

Seven facts follow. *The page holds the key's bytes* (call 1): iroh's
`SecretKey` is an ed25519-dalek key made only from 32 bytes or by `generate()`
(`iroh-base-1.2.0/src/key.rs:261`, `:318`, `:337`), and iroh signs its TLS
handshake with a synchronous call (`iroh-1.0.2/src/tls/resolver.rs:92-95`),
where `crypto.subtle.sign` is asynchronous. The issuer refuses a redeemer whose
dialling key is not the new key (`crates/svrngs-node/src/redeem.rs:383-387`,
`NotTheDialer`), and the member protocol admits only standing keys
(`crates/svrngs-node/src/node.rs:440-459`), so the endpoint key is the
person's key and exists as bytes in the page. Sealed, storage holds only
ciphertext; code running on the page can still use the key, which is why a
group trusts its origin with that code (§11, risk 5). This is read from
source: upstream never held a key in a browser. *A browser's key is never
dialled* (call 2): U1's "No presence" fact makes every round dial every
standing key, and a phone's page is closed most of the time. *The phone dials
the lender* (call 4), so a lent service's bytes cross no third computer;
upstream refused "the plaintext-terminating proxy" either way
(`docs/THE_LINK.md:241-242`), and a browser is relay-only for good
(`quality/campaigns/ring-apps-shelf.md:19-28`). *Measured once, never on a
phone:* browser-dial-2 ran Chrome for Testing 147.0.7727.15, headless, on
Linux x86_64, driven by `playwright-core` 1.59.1 (`ralph/DECISIONS.md:360`,
`:9948-9960`), and its one path fact is the send half: calling `finish()`
right after the write read `Ok(0 bytes)`, and holding it open until the
response read the 200 (`:9977-9983`). It recorded no dial time, first byte or
seek for any browser. The one contact with a real phone is a failure, the
trailing slash (`landing/scripts/build-ring-runtime.sh:11-12`). What a relay
gives natively bounds it: a 182 ms floor and 3.7 MB/s bulk through n0's relay
(`research/engram/LAB-RECORD.md:241`); a cold connection's setup, paid per
seek, p50 189 ms and p90 1,327 ms on a LAN (`sovereign/deploy/mesh/WORK_PLANE.md:577-579`).
*The build needs Homebrew's LLVM on a Mac:* browser-dial-1 failed there in
ring's build script, "No available targets are compatible with triple
"wasm32-unknown-unknown"" (`ralph/DECISIONS.md:9860`), and built on Linux with
clang 21. *Upstream kept browsers out of anyone else's group* — "A browser
holds no key in anyone else's ring" (`docs/RING_SPEC.md:205-209`;
`docs/RING_APP_LIBRARY.md:315`). §10's browser door, a group opening its own,
sets that aside: a phone's key stands in the register under a person's name
as a computer's does. *An origin is a place, not a copy:* browser storage is
per origin (`docs/RING_APP_LIBRARY.md:565-571`), so a group that changes its
page origin has its phones join again by link. Which codecs a phone plays is
the lent service's and its app's: Chrome plays no MKV and HEVC only on a GPU
(`quality/campaigns/ring-apps-shelf.md:193-194`).

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U1 a group is not a mesh | ring sync, its route and the dial-by-key tunnel forked into `svrngs-node`, the node's router served on each accepted stream; the register decides who may connect and who is synced, where upstream asked the mesh (`commonwealth/crates/commonwealth-rails/src/` at `e1a43bcd2`: `acceptor.rs:47-61`, `ring_sync/journal.rs:153-164`, the default roster at `lib.rs:404`) | a node in the group but no mesh syncs; a node outside the group is refused |
| U2 join by link | `svrngs new <name>`, `svrngs join <link>` and `svrngs link <group>` on a computer, and `svrngs` alone running the node; the link is one short token — the issuer's key and the link's secret, and the issuer's relay only when it runs without lookup — and the issuer answers the group's seed and its `Invite`'s id to the link's key; redeeming at the issuer's node writes one `Admit`, co-signed by the new key; ids over everything signed, the previous act among it; the door signing after everything it holds (§6) | three people open one link and each is admitted once; opening it again admits nobody twice; a lapsed or closed link admits no one; a dialer without the link's key is closed with no answer; nothing the command prints uses a word §1 keeps from people |
| U3 lend a service | the member door: a route that proxies to a lent service, the verified person set from the connection; `svrngs share <port>` with its lend declaration; streaming and byte ranges through the door; each request's share of the lender's upload; the people-at-once limit; the loopback record API (the rail routes and the live lane, forked); the list of what members lend | the app sees the verified person; a header the client typed is stripped; the lender's secret appears in no byte a member receives; with three streams, each is told a third of the declared upload, and a fourth person is refused with a reason |

**Demo A — Sam's films, three homes at once** (after U5; operator,
2026-10-04, once U3's dry run found no way into Jellyfin's own player that
keeps the bars — §11, risk 7). Sam lends Jellyfin with one command and sends a
link. Alex, Bo and Cy join from three homes and each plays a different film
from Sam's library in Films, in their own browser, at the same time, seeking
freely; none stalls another, and the door tells each its share of Sam's
upload, which Films turns into a picture. Sam's password never left Sam's
computer; nobody made an account.

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U4 the browser | a phone opens a group's https link, on the page origin the group named, and is in: its own key, a dial by key to the issuer to join and to each lender for what they lend (§3, "Joining"), and the runtime that renders a lent app through it — fetch, POST, streaming, byte ranges; the page is static files any origin can serve | a stock phone browser plays a lent film and seeks |
| U5 Films | `apps/films`: one shelf across every member's lent library, "from Sam's", search, play here; a device and play-session id per person and play; progress per person in the record; busy turned into a choice; the lend's credential made from the viewer's password, which the lender types once on their own computer (U3's dry run: today it is a whole request header the lender cannot make from what they are told) — on the §3 surface alone | a title two members lend shows once, with both; Bo's seek never restarts Cy's play; Films calls nothing outside §3 |
| U6 the screen handoff | the meeting point serves one stream to a TV on its network at an unguessable address; AirPlay from Safari, Cast from Chrome; the phone is the remote and may lock | a film plays on the TV from a phone browser; the address serves only the device it was handed to |

**Demo B — movie night** (after U6). Five friends at Sam's, phones only. Sam's
group names a page origin, so its link opens in a phone's browser. Sam
texts the link; each opens it, browses one shelf across three members'
libraries, and a film they pick plays on the TV. Nobody installs or signs up.
Gate: four of five newcomers get from the link to the film on the TV within
three minutes, unassisted.

**The canaries** run in `scripts/check.sh` from the moment their surface
exists, each a small app in `apps/` using only §3:
- **Expenses** (after U3) — no server, all state in the record: two nodes,
  everyone writes, a correction voids, and both fold to the same balances.
  Landed (u3-7): `apps/expenses/canary.mjs`, folding with `init` and
  `reduce` over Sam's and Alex's loopback record APIs, writes for Bo and Cy
  naming them in `on_behalf_of`, run by `crates/svrngs-node/tests/canaries.rs`,
  nodes in one process, no relay; a step of its own in `scripts/check.sh`.
- **Tap** (after U3) — private inputs and a ring born from a ring: two groups
  on one node; an app starts a group of two from inside a group; it seals its
  own payloads to members' public keys, which the register gains with it. (It
  said "after U2" until 2026-10-04; an app reaches the record through U3's
  loopback API, and no key in the register can seal, U2's inventory.)

**Then:** removal (U7, the keeper of §6, with forks that void nothing), before any group of strangers; then
the chat doors, Discord first — onboarding, last. After that the core grows one
class at a time, each chosen by the north star's tests.

## 9. Later, each with the trigger that brings it back

| Waits | Trigger |
|---|---|
| an app SDK, a sandboxed fold engine, signed manifests and install acts (QuickJS behind an `Engine` trait was `crates/svrngs-engine`, deleted 2026-10-04 with no user; it is at tag `e0-bot`) | an app needs its state offline on a device that is not its host, or strangers' code runs on members' devices |
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
| any member's node redeeming a link, once its `Invite` has synced | a newcomer's link fails because its issuer's computer was asleep, counted |
| a link's life set per link, other than a day | a newcomer meets a lapsed link, counted |
| measuring a lender's upload rather than taking the figure they declare (BBR3's pacing estimate, or a timed saturating pull) | a declared figure is wrong by enough to stall a viewer in a recorded run |
| a per-process tie between a node and the services it lends (upstream's `mint_tie`) | an app's act is written under a person by a process the lender did not run, or one computer is shared by people who do not trust each other |

## 10. Decisions

By the operator, 2026-10-04:
- computers run the client, and that is the way in; the browser is a door a
  group opens by naming a page origin, its own or ours (over "phones first,
  through the browser");
- a link is upstream's https form, everything it carries in its fragment;
  `https://svrnme.sh/ring/` is the default origin, and anyone may serve the
  page from their own (superseded by the next two, landed in u2-8);
- a link is short and opaque, for the ergonomics of sharing: one token
  carrying only what the joiner cannot ask the issuer for, and no word §1
  keeps from people anywhere in it (u2-8);
- nothing depends on a hosted URL by default: svrngs prints the client's
  link, `svrngs://join/<token>`, and an https link only for a group that
  names a page origin of its own choosing, svrnme.sh being one choice, not
  the default ("I don't want people to HAVE to depend on our hosted url").
  This supersedes the svrnme.sh default above;
- one link admits everyone who opens it, each once, until it lapses or is
  closed — not one person per link;
- only the issuer's node redeems a link;
- forks void nothing in U7, with removal, not in U2; U2 puts the previous act
  in the id.

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
   person's key with it, since its sealed bytes and the key that seals them
   are both in IndexedDB (§3, "Joining", call 1); a web app added to the home screen keeps it, and a
   lost key means joining again by link. And WebKit, every iOS browser's
   engine, seeks differently: in u4-8's dry run (`demo/runs/u4/2026-10-05/`)
   its first seek far ahead, asked a second after play, waited while the
   range it had open was read through, up to 29 s for a 45 MB film where
   Chromium's seeks all played within 2 s (`ralph/REVIEW_FINDINGS.md` #53).
5. **Members find each other through iroh's address lookup**, n0's DNS by
   default, as upstream's (§8, U1's inventory): it maps a key to its relay,
   never to a person, and a group is unreachable through it while it is down.
   Upstream's `discovery = "none"` with the group's own relay is the way out,
   kept by the fork. And n0's relays, as iroh spells them, are out of
   WebKit's reach: iroh writes each with a trailing dot
   (`iroh-1.0.2/src/defaults.rs:27-33`), WebKit refuses TLS to any host so
   written, and every iOS browser is WebKit. In u4-8's dry run no WebKit run
   through n0 got past opening the link (`ralph/REVIEW_FINDINGS.md` #52).
   Since u4-9 the command and the page write and read every relay without
   the dot (`svrngs_core::relay`, `crates/svrngs-node/src/relay.rs`): n0's
   relays, a link's, a relay a computer is told, and a lookup's answers, so
   a computer advertises its relay dotless and a phone reads an earlier
   build's dotless too. If iroh spells a relay some other way WebKit
   refuses, an iPhone fails again, and only a run through n0 in WebKit
   shows it. The page a phone loads from a link's origin is the same
   kind of default (U4): whoever serves it serves the code that holds the
   person's key, its bytes unsealed while the page runs, so a group that does not trust svrnme.sh serves the page from
   its own origin and prints its links there. And every lent service's
   pages run on that origin too (u4-6, call 3: a lend's address is under
   the worker's scope), so a script in a lent page can open the page's
   store and unseal the key of whoever opens it: in u4-6's test, a script
   on `/l/<ada>/page/` imported `store.js` and read bo's 32 bytes. So
   whoever lends a service, or a service compromised, can act as each
   person in the group who opens it. Putting the lends on an origin apart
   from the page's store is the way out, unbuilt.
6. **Two copies of the record's code.** A signing or canonical-form bug fixed on
   one side can be missed on the other. `scripts/ports.py` is the guard, and it
   is a listing, not a gate.
7. **Jellyfin's own player asks for a sign-in** — settled. The door injects a
   credential on requests, not a session in a browser, so either the browser
   is handed a token it then holds, which breaks "the lender's secret appears
   in no byte a member receives", or each person signs in to a viewer
   account. U3's dry run against Jellyfin 12.0.0
   (`demo/runs/a/dry-2026-10-04/`) found the second worse than feared: a
   member's sign-in through the door logs the lend's token out for everyone,
   and only a placeholder sign-in that no member can write gets past the
   page. The operator, 2026-10-04: Demo A plays in Films, after U5, which
   never needs that page; no unit builds a way into Jellyfin's own player.
