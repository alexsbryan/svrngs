# svrngs — the design

> What svrngs is building, and the only design in force. It answers to
> `NORTH_STAR.md`; where they disagree, this file is rewritten. Where code
> and this file disagree, one of them is wrong: fix it in the same commit.
> What we build next, what waits and the operator's decisions are
> `docs/PLAN.md`; what exists is `ARCHITECTURE.md`.
>
> Distilled on 2026-10-17. The same design with each measurement, run and
> row beside its rule is this file at `6c7b8296`. A citation made before
> then of DESIGN §8 is now PLAN §1 or §4, or the paragraph §3 or §5 here
> that took it in; of §9 PLAN §2, of §10 PLAN §3, and of §11 §8 here. The
> design before the fork is this file at tag `e0-bot`. The record of why is
> commonwealth-ai's `docs/RING_APP_LIBRARY.md`, cited as LIB §n. A path in
> backticks that is not in this repository is in commonwealth-ai.

## 1. What svrngs is

**The architecture, in one breath:** a mesh of nodes addressed by their
keys, where any node can play any of a few roles: a replica of a group's
log, a lender of an endpoint, a client, or a relay. Only two things cross a
connection: entries in a group's log, gossiped among its replicas and sealed
to some keys when they're private, and requests to a node's endpoint with
the asker's key attached. Every shared state is each replica folding the same
log, and anything a node or code does for someone (read an entry, append
one, reach an endpoint) is a capability that someone's key granted.

Every rule below follows from that. An entry's author is its signer;
replicas only carry: a phone signs its own acts and a computer its own
person's, and no computer writes as anyone else (§6). What can act as a
person is what they or their group chose, said by what it can do: know who
they are, reach an endpoint as them, read or append entries, or decide for
them (svrn-docs `rings/current/WHAT_CAN_ACT_AS_YOU.md` §2), never by what
kind of app it is. Both pages list each under "What acts as you" (§3).
Sealing is not built yet (PLAN §2).

**For people:** *keep talking where you talk; keep what your group owns —
money, plans, files, the library — on your own computers, not a platform's.*
A group is a house, a team, a party: about twenty-five people who already
share a chat. Its story in 2026 is the browser on phones and the
computers that lend: *join once from the chat link; everything your group
lends has an address that never changes; bookmark it on your phone or your
laptop, and it opens* (svrn-docs `rings/current/IN_THE_BROWSER.md`; §3,
"Lent apps", call 1). Nothing is installed on a phone.

**For developers:** *anything you can run on localhost, you can share with
your group.* The group is the platform. An app gets three things and needs
nothing else: the person asking, verified, in request headers; a shared
record that outlives the computer it runs on; and reach to every member's
computer. No login, no SDK, no bundle. A key is who someone is and a name is
a claim — canon's rule for its own ledger too (`~/dev/canon` `a6960fc`,
`docs/SIGNING_DESIGN.md`) — so an app keys on `X-Svrngs-Key`, shows
`X-Svrngs-Person`, which two people may share, and names an account by
`X-Svrngs-User`, which never collides (§3, "What an app gets").

**And the apps already exist.** Everything people self-host is a web app on
a port: Jellyfin and Navidrome for the library, Immich for photos,
Audiobookshelf and Calibre-Web for books, Nextcloud and Paperless-ngx for
files, Actual for money, Mealie and Grocy for plans, Home Assistant, and
llama-server for a model. Run one on any member's computer, `svrngs share
<port> --kind <name>`, and everyone in the group can open it: no port
opened, no tunnel, nothing for them to install. An app that takes the
signed-in person from a request header lets each person straight in once
its lender names `X-Svrngs-User` in its settings; `svrngs share` knows
Navidrome (4533) and Calibre-Web (8083) by their ports and prints what to
set (`crates/svrngs-node/src/well_known.txt`). An app that keeps its own
accounts asks each person once. Each serves its pages from its own root, so
on a phone it needs a page host that gives every app an address of its own,
as ours does. Which apps are named here, and why others are not yet, is the
catalogue run's (`demo/runs/catalogue/2026-10-05/`).

**What we build:** the platform, and one small reference app per class of
app, for the next developer to copy and bend: Money, a group's shared book
(`apps/expenses/`); Events, a post and who came (`apps/events/`); the
grapevine, an ask carried from group to group by the people in both, where
a developer adds a kind of ask in one file (`apps/grapevine/`); House
rules, where a house decides its rules (`apps/canon/`); and Give, an app
the group keeps on one member's computer at a time, a carried page and a
program of its own beside svrngs (`apps/give/`). Never the finished
apps: the community builds those. "The iOS of groups" is an analogy for
being the thing others build on. The iPhone app is a door beside the
group's page, never the way in: it adds only what a page can't (§7).

**Three rules for anything a person reads.**
1. *No universe to explain.* A person sees their group's name, people's
   names, apps' names, and *computer*, *phone*, *the chat* — never ring,
   member, key, sync, journal, node or relay. Whoever runs a relay is the
   one reader of that word: `svrngs relay` and `new --relay` speak to them
   of a name, ports and a certificate.
2. *The most general surface with the fewest features.* The system learns
   by adding apps, never by adding platform.
3. *Nothing imitates anything else.* A page brings what a browser does best
   — the system's own fonts and colours, its own elements, plain rules —
   and its warmth is colour, by the names the browser knows, never paper, a
   hand or a stamp. Both pages wear one look,
   `crates/svrngs-node/src/home/look.css`.

## 2. How we build — and the missteps that taught it

Each rule below cost us a day once. With `docs/principles/`, they are how a
choice gets made here.

1. **Take what exists; fork it and own it — never rewrite it.** We wrote a
   record from scratch while commonwealth-ai's `rail-core` already did the
   job, with a hundred tests and a measured browser dial; the rewrite is
   deleted. Before a new type or module, look here, then in commonwealth-ai's
   ring code.
2. **Take the slice you need, not the system around it.** We nearly shipped
   commonwealth's `cw-rails` as a sidecar: a 30k-line mesh daemon over nine
   crates, to use its 5k-line ring slice, with every fix waiting on a
   commonwealth release. We fork the slice instead (§4).
3. **Build what the next demo needs; everything else waits for its
   trigger.** We designed an app SDK, a wasm sandbox, signed manifests, TV
   casting and group compute in detail before one group had used anything,
   and the first change of direction made most of it moot. Design ahead
   only as far as it takes to keep a door open, and write the rest as a row
   of PLAN §2.
4. **A change of direction rewrites the design.** For a day this file
   carried a section that "overrides everything below it", over a thousand
   lines it contradicted. A design describes one thing; the old one lives
   in git, at a commit or tag this file names.
5. **Draw the line by what each side owns.** A group is not a mesh: the
   people in a group decide who is in it, not whose machines they are.
   App-specific code lives in the app. A chat is a door, not the core. Our
   library holds no integration code.
6. **Demos keep us honest.** No more than three units without a scene a
   future user would want to watch, recorded, with its run-of-show written
   before its first run (`demo/README.md`). People bars are judged by
   strangers.
7. **Ship code, not prose.** A record is written when something lands,
   never for an attempt.
8. **The design says what is; the evidence lives with the run.** This file
   grew to 47,000 words as each landing wrote its measurements, rows and
   dates beside the rule they proved, until the rules could not be found.
   A rule here names the code that holds it. How it was measured is the
   run's README under `demo/runs/` and the commit that landed it; what was
   built when is PLAN §4.

## 3. The shape

**A group** is a record and the computers and phones of the people in it.
Each member's computer runs a node that holds a full copy of the group's
journal and syncs it with the other nodes over iroh. There is no server; a
member's always-on computer, when there is one, keeps the group reachable
while the rest sleep, and is the meeting point a phone dials.

**An app is a localhost service** — an *rngapp*. A member lends one from
their home page, under "Already running here", or with `svrngs share
<port>`. Every other member opens it through the **member door**, tunnelled
over iroh to the lender's node, which strips any identity header the client
sent and adds the verified ones. A lent service — Jellyfin, a svrn daemon's
web UI — is an app like any other. What the group may do with it is its
*lend declaration*, kept as data on the lender's computer: a secret the
door injects and never lets out, in place of the client's own header of its
name (a lend with no secret leaves each browser's own), the paths it
refuses, how many people may use it at once. A lend is lent to every group
its lender's computer holds. *Lends are asked, not stored:* each computer
answers what it lends when asked, so an asleep computer's lends are absent,
never stale. The lender's own lend, opened on their own computer, goes
through their own door's rules too, its secret injected and its limit
counted (`door::Who::Own`).

**A port, not a page.** A program that speaks UDP or TCP is lent with
`svrngs share <port> --udp --kind <kind>`, or `--tcp`: its declaration says
`"raw": "udp"` or `"raw": "tcp"`, and the door carries it on a protocol of
its own (`Alpn::Port`), never as a page. A member's computer opens one
connection for one lend, naming the lend and how it travels. Each use is
let in by its person's standing and the lend's limit, which counts people,
not flows or connections, and a person past the limit is told so in the
door's words. While the connection is open the lender asks the dialer's
standing again every second, and once they no longer stand it closes the
connection, every flow and stream on it: a removed member is cut within
seconds, whatever their own computer does (`door::udp::serve`).

- **UDP.** Each program sending to the member's socket is a flow. The
  lender gives each flow a socket of its own on 127.0.0.1:0, connected to
  the lent port alone; it closes after a minute idle. Packets cross as QUIC
  datagrams. One larger than a datagram is cut and made whole on the far
  side, and a fragment lost loses that packet alone, as on the open
  internet (`crates/svrngs-node/src/door/udp.rs`).
- **TCP.** Each program's connection to the member's listener is a QUIC
  stream of its own, and the lender connects it to 127.0.0.1:<port>. A
  stream has flow control of its own, so one connection stalled holds back
  no other, nor the record's sync. A side's close is its stream's finish,
  so a half-close crosses (`crates/svrngs-node/src/door/tcp.rs`). A TCP
  port is lent only when something answers on it, as a page's is.
- **Play.** On a member's home page a port's row offers Play, not a tab.
  Pressed, the member's computer listens on `127.0.0.1` at the lend's own
  port (`room::port_of`, by its label, the same each time), UDP or TCP as
  the lender's computer says the lend travels, and the row says "Connect
  to 127.0.0.1:<port>." with the address copied. Which lends have Play on
  is kept in `play.json`, so they open again as svrngs starts. Each is
  open only while the member and its lender stand in a group held there
  together, read every 5 s, and nothing listens while it is off
  (`crates/svrngs-node/src/play.rs`, its owner on the room's model). A raw
  port carries no token, so any program on that computer reaches the lend
  as its person while Play is on: each Play on is a line under "What acts
  as you", "Open on this computer: <name> (<lender>’s), while Play is
  on.", whose Stop is the row's own.

**The person's own programs are the trust boundary.** A program on this
computer asks the record API showing the token in the data dir's `token`
file, beside `api`, owner-only, as `Authorization: Bearer <token>`;
without it every record route refuses, 401 `no_token`, naming the file
(`crates/svrngs-node/src/api.rs`, `holds_the_token`). Loopback alone was
the boundary until give-token, and it is not one: a container on a Mac
running podman reaches the Mac's `127.0.0.1` (svrn-docs
`rings/current/GIVE.md` §5). A program the person runs reads the token as
it could read their key, so it speaks as its person, and only as its
person: the record API writes a body naming this computer's person as its
own act, and refuses one naming anyone else (`not_yours`), whose entries
are signed on their own devices. Sign-in's routes keep their client's own
secret. Any process on the lender's computer can still reach a lent port
and type an identity header, as it can reach any localhost service. A tie
between the node and each process it lends to waits in PLAN §2.

**A browser comes in through an open.** The home page and each lend's
origin answer a browser only with this computer's session for that host,
and a program only with the token (`crates/svrngs-node/src/session.rs`).
An open is an address with a code, single-use and gone in a minute, that
becomes the host's session where it is redeemed (`/svrngs-open`): `svrngs
open` mints one for the home page, from the token, and hands it to the
browser, and so do install.sh and the opener it puts in the person's
Applications, or on Linux their applications menu; the home page mints
one for each lend it opens (`/to`). A session is the token's HMAC over the
host's first name, kept nowhere, so a restart keeps every bookmark working
and one host's session opens no other. Anything else is answered "Open
svrngs from your computer", naming that opener, with no terminal needed.
Neither the session nor the token reaches a lend's service. A container
on the Mac is refused at both (`demo/runs/give/2026-10-26-container/`).

**A lent app writes as whoever asks by naming the act.** Its answer names,
in `x-svrngs-append`, a JSON array of acts, each `{group, op}` or `{group,
corrects, op}` as a carried app's route takes one. The asker's own side
reads it before the page sees the answer, shapes each under the kind the
lend declares (`grant::app_act`), signs it as the person and carries it to
the lender (`POST /carry`), so the app's next read holds it. A computer's
origin (`crates/svrngs-node/src/origin.rs`, `appended`) signs as its own
person, in a group both it and the lender stand in. A phone's worker
(`web/lend.js`, `appended`) signs with the pass's holder, in the group it
was opened from, under a second pass the group page signs at the open,
whose app is that kind's part of the group (`Label::carried`), so the
fold's `Grant` lets the holder write that part alone. A correction of an
act outside that kind, or a group the two do not share, withholds the
answer and says why; nothing of it is written. The header never reaches
the page.

**Or a folder the group carries.** A lend goes with its lender; a carried
app stays with the group. It is a folder `apps/<kind>/` holding an
`index.html` in one of the group's synced folders, so its files are already
on every member's computer by hash. Each computer that keeps the folder in
step lends the app as its own, to that group alone, named `<kind>-<the
group id's first 16 letters>` so two groups' apps of one name have two
origins (`crates/svrngs-node/src/lend.rs`, `Source::Carried`). It opens on
any of them while that one is on, and on a phone through any of them; both
pages show one copy of each, and list every computer that carries it. svrngs
runs none of its code: the door answers it from the store
(`crates/svrngs-node/src/carried.rs`) — each file by its hash in the version
that runs as the person asking (below, "What runs as you"), fetched from
another of the group's computers when this one never held it — and from
**its route**,
`/svrngs/acts` at its own origin:
- `GET` answers the app's own acts — each whose `kind` begins `<kind>/`,
  the prefix taken off — and each correction of one, voided ones too, in
  the shape `/v1/rail/log` gives, so an app's fold reads them as it reads
  the record. Beside them: `you`; the group's id as `group`; `page`, the
  page its https links print on, as `svrngs link` prints them, null where
  none is kept (Give's install line names it); its `people`
  now; and `spans`, each person's stretches in the group, `{person, joined,
  left}`, so a house's rules count a past act with whoever stood when it
  was written (`Membership::spans`, the one decider of who stood when).
  Their times are stamps on acts, never a computer's clock, so every copy
  says the same; `joined` is null for whoever started the group, `left`
  while they stand, and one who leaves and comes back does so by a new key,
  with two stretches. `spans` is null while who stands is unsettled, never
  empty. The route answers every act each time, from no cursor, since a
  round may place an act before one already read.
- `POST {op}` records one act; `{corrects, op}` corrects one of its own,
  `op` null to void it. svrngs puts the prefix on (`grant::app_act`, one
  copy for the door and the phone) and signs it on the person's own
  device: a computer's own person's by that computer, and a phone's by the
  phone's worker, with the pass's holder under the pass, carried to the
  lend's computer unchanged (`POST /carry`). A computer writes nothing as
  anyone another device holds. A correction of another app's act is
  refused. A line the phone signed and no computer took yet is kept beside
  the pass and goes, the same bytes, with the next write.

Each act stays its signer's: its `person` says whose device wrote it, and
its `person_key` whose person, the same from each of their devices; an
older act's `on_behalf_of` says whom its computer spoke for (§6), and the
node's answer adds `on_behalf_of_key`, that person's first key, where one
person of that name had joined by the act's time (`ops_json`,
`crates/svrngs-node/src/rail.rs`; admission carries the name and
interprets nothing). Money's,
Events' and the grapevine's folds keep both, `by` and `via`, and their pages
say an entry an older svrngs wrote for another person as "bo, through cy’s
computer". Money and Events keep each person by that first key, `key`, and
say them by name: two people of one name are two balances and two who are
coming, told apart by when each joined, and one person's computer, phone
and older entries are one (`who`, `apps/expenses/expenses.js`,
`apps/events/events.js`); an answer from an older svrngs, which names no
key, is kept by name. A correction counts only from the person whose entry
it changes, by key where both acts carry one, so two people of one name
stay two (`voids`). Every answer carries the containment floor's headers: the page
may load from and connect to its own origin only; no camera, microphone or
location; no DNS prefetch (`scripts/trust_floor.txt`). The floor keeps out
code from anywhere but the app's own files, and reach nobody chose: a CDN,
a tracker, a fetch elsewhere. It does not stop an app the person chose
from sending out what it reads, since a page that navigates carries what
it holds, and a browser has other ways out. What protects what a carried
app reads is that its people chose it: its files are pinned, each version
runs as a person only on their yes, the record says who brought or
changed it, and it reads its own kind alone, the trust a person already
gives the programs on their computer (svrn-docs
`rings/current/WHAT_CAN_ACT_AS_YOU.md` §5). A sandbox earns its cost once
code arrives from outside a person's rings (PLAN §2). A carried app comes to a group from the shelf
(below, "The home page") or as any folder dropped into the group's folder
under `apps/`.

**House rules** is the carried app where a house decides its rules
(`apps/canon/`, brought as `House-rules`; `apps/canon/README.md`).
canon-core, built to plain JavaScript, decides every answer on it: what is
a rule, what waits, who has a say. svrngs carries its entries and vouches
for who wrote them; it neither reads nor enforces a rule. The house's people
are the house: from `spans`, every computer builds canon's own grants of a
say in memory — a `house:<group>` grant at each `joined`, a withdrawal at
each `left` — and records none of them. The first person to open it answers
"How does your house decide its rules?", and that answer is itself a rule
the house can change.

**Give** is the app a group keeps: an app whose state is one directory runs
on one member's computer at a time, in rootless Podman from the app's own
published image, and copies of its state reach every member through the
group's folder (`apps/give/README.md`; svrn-docs `rings/current/GIVE.md`).
It is its own package on this surface alone — the record API, `svrngs
share` and `svrngs unshare`, the synced folder and its own carried page —
and links no svrngs crate (`apps/give/tests/surface.rs`). Who keeps what is
Give's fact, not the group's: one fold (`apps/give/fold/`), which its
program links and its page runs built to JavaScript, as House rules runs
canon-core. svrngs learns no keeping; it lends what Give's program shares,
a page or a port. An app is a card, its own `compose.yaml` with an
`x-svrngs` block of five fields: `title`, `about`, `door`, `state` and
`asks`. Every move and every takeover is a person's yes, and a keeper that
comes back after another took over keeps its later state as a copy anyone
may start instead, never merged. Any member whose computer has Podman and
Give's program may keep (the operator, 2026-10-26).

**What runs as you is what your group brought, and each new version is your
own yes.** Bringing an app pins it: the folder acts that place its files
name their hashes, and each path under `apps/<kind>/` as the record first
placed it is the app as it was brought (`svrngs_core::Runs`, folded in
`crates/svrngs-core/src/synced.rs` alike on every copy). A member's change
to its files is a new version, by its files' hashes, and runs as nobody
until they say yes. Both pages ask without interrupting, under "What acts
as you": one line, "1 app changed. See what’s new", opening to "Money: kim
changed its files on 16 Oct." and "Open the new version". The yes (`POST
/runs`) records a `RailAct::Runs` naming the app's files, signed by the
person's own device: the home page's by this computer, and a phone's by
the phone's own key, the computer it reaches naming the act and carrying
the signed line back unchanged (`POST /carry`). A yes counts only from its
person's own device and only naming bytes a folder act placed at those
paths, so no other member's computer, and no file a member supplied
outside the folder, decides what runs as anyone. An app brought before the
pin is pinned at its first files, nothing migrated.
Until a person says yes, the version they last said yes to, else the one
brought, keeps running as them on their computers and phones alike, and
every computer keeps and serves each version someone runs (`Runs::of`, the
one decider). Whether a new version also waits for the house's agreement is
a house rule canon may configure; svrngs keeps the pin and each person's
yes. Not yet pinned: an inline `<script>`, bytes from the route, and reading
another app's acts.

**What an app gets — the whole surface, for our apps and a stranger's
alike:**
- *Who asks*, in headers the door sets (`crates/svrngs-node/src/door.rs`,
  `Identity`):
  - `x-svrngs-key`: the person, by their first key in the group, the same
    from every device of theirs — an id, not a key to seal to;
  - `x-svrngs-person`: the name to show, their name in the register, never
    unique;
  - `x-svrngs-user`: the name of their account in an app that makes
    accounts — their name kept to ASCII letters, digits, `-` and `.`, and
    the first 16 hex of their person key (`user_name`), so a second `ana`
    is never Ana's account, and Ana's devices are one account;
  - `x-svrngs-device`: the device asking, by the key it stands in the group
    by, so an app that holds something to a device tells a person's devices
    apart under one account;
  - a guest (below, "A visit is folded beside membership") is told as their
    host's: `x-svrngs-person` `Sam (Kim's guest)`, `x-svrngs-key` and
    `x-svrngs-device` their visit's key, and `x-svrngs-user` `.guest` after
    `user_name`'s, which no member's account ends in; never as their host;
  - `x-svrngs-app`: when one app's page asks another lend, the app asking,
    read only from a pass, or on a computer from what the browser says of the
    page ("Lent apps", call 6); none for a key asking with no pass;
  - where it is lent, `x-svrngs-label`, the lend's label, and
    `x-svrngs-pages`, the group page of each group the lender's computer
    holds that the asker stands in, `<group>=<page>` apart by a space (the
    page `none` when the group keeps none, `unread` when it cannot be read).
    From them an app writes a link into itself, `<page>#open=<label>&at=<path>`,
    which the group's page opens on a phone and a computer's home page opens
    pasted. That it opens for the group's people alone is the app's check of
    who asks, as on any request.
- *The people in the group*, one row per key, each key one an app may seal
  to, with its person's first key.
- *The record*, over a loopback API on the member's own node: append, read,
  live. Each live payload is drained with who sent it, the key that dialled
  in and its person's first key, so an app acts on who spoke without signing
  every payload. A live payload is offered to each member online now and
  kept nowhere.
- *A signature for an app*, checkable anywhere: `POST /v1/sign?app=<app>` on
  the same API has the computer's own key sign the body's bytes, and answers
  `{key, sig}` in hex. What is signed is `"svrngs-app:" || app || ":" ||
  bytes`, the app's name 1 to 64 of `a`-`z`, `0`-`9` and `-`
  (`crates/svrngs-core/src/app.rs`). That domain begins no other the core
  signs or hashes under (`sig.rs`), so an app's signature is never an
  act's, a link's, a pass's or another app's. Nothing says the key is a
  person's: to someone outside the group it is the computer's. A stranger
  checks one with the key, the app's name and the bytes alone
  (`apps/on-air/page/air.mjs`, with `@noble/curves`):

  ```js
  const signed = concatBytes(utf8ToBytes(`svrngs-app:${app}:`), bytes);
  const ok = ed25519.verify(hexToBytes(sig), signed, hexToBytes(key));
  ```

  A page in a browser is refused it, as it is the record.
- *Bytes by hash, with a lifetime*: `PUT /v1/blob?group=<id>&keep=<seconds>`
  puts the body in this computer's store and answers `{hash}`, its BLAKE3;
  `GET /v1/blob/<hash>?group=<id>` answers the bytes from that store, else
  fetched and verified from a computer of the group that holds them, named
  in `x-svrngs-holder` (`folder.rs`, `blob.rs`, the store and provider a
  group's folder uses). Each hash lapses `keep` seconds after its put on
  every computer that holds it: a computer asked for one asks the others
  what each holds and for how long, holds it no longer than they do, and
  serves it to the group meanwhile, so each member that asked carries it for
  the rest. A refused get says which: `lapsed` (410) if any computer asked
  says so, `never_held` (404) only if every one does, else `unanswered`
  (502) — never empty bytes. These bytes are neither the record nor a
  folder (§6). Carrying spends the carrier's upload, uncapped.
- *The services the group's members lend*, each by name and an opaque kind
  the app interprets (`jellyfin`), and how many people use each now.
- *The share of the lender's upload* each request through the door may use
  right now ("Many people, one library").
- *svrngs's word told from the lend's*: every answer svrngs makes in a
  lend's place — the door's refusals, the lend's worker's and the
  computer's — carries `x-svrngs-refused`, naming which (`away`, `pass`,
  `closed`, `unlent`, `path`, `full`, `fault`, `unasked`,
  `stopped`, `removed`, `ended`), and a lend's own answer never does (`door::LendRefusal`). Any
  of svrngs's refusals to a browser's tab, which asks `Accept: text/html`, is
  drawn as a page of its words; a program is given the JSON. A lend whose
  computer did not answer, `away`, is drawn as its person's window on the
  door, dark (`home/away.html`, which a phone's lend worker shows too,
  naming the lend): that their computer did not answer, never that it
  sleeps, nor since when. On a
  computer a page that reads a lend may read svrngs's answers at that lend's
  origin, which allow its origin alone, never `*`, and the lend's own answer
  only as far as that answer allows; any other page is allowed nothing
  (`origin.rs`, `readable`).
- *Only the apps its person said yes to*, for a lend that asks (`svrngs
  share --ask "<what an app receives>"`, the lender's words, carried in
  `asks` on every list and never read by svrngs). The lend's door keeps each
  person's yes and stop per app, beside the lend's declaration and never in
  the record, and decides on them (`door/choices.rs`). A lend asking nobody
  is carried as before, and its own pages are carried whatever was said; an
  app with no yes, or no app named, is refused `unasked`, and one stopped
  `stopped`. Only the person says
  yes (`POST /choices/<lend>` on the members' protocol); anything holding
  the access may say stop. A stop refuses every request after it; one
  already carried runs to its end. Each choice is keyed by the person the
  door resolves (`door::PersonKey`), so a yes on one device holds on every
  one; `svrngs unshare` takes the choices with the lend.
- *Reach for the devices in a room* ("The room").
- *Each person signed in as themselves with nothing typed*, for a lend
  signed in with svrngs ("The room", call 6).

An app that needs anything else is a decision, not a convenience.

**What an app asks of its own computer.** The node writes the loopback API's
address to `<data dir>/api` at each start (`crates/svrngs-node/src/api.rs`,
`API_FILE`), so an app it did not start finds it. `GET /v1/groups` answers
each group this computer holds: its id, the `namespace` every route names,
its name, and this computer's person in it, `null` when its seed or its
standing cannot be shown. `GET /v1/lends?namespace=<id>` answers what that
group's people lend now, this computer's own among them: each with its
lender's person and key, its name, kind, limit, how many use it, the upload
its lender declares (`upload_bps`, `null` when none), and its `origin`,
where this computer's browser opens it — and the people whose computer did
not answer, named, their lends absent rather than stale
(`crates/svrngs-node/src/surface.rs`). Both are asked each time, nothing
kept. Each lender is asked for at most 3 s (`node::ANSWER_WITHIN`), and one
that has not answered by then is named as not answering, never dropped: the
same bound, in one function, on `/v1/lends`, a lent page's reads and a
phone's list.

**The home page**, `http://svrngs.localhost:<port>/`, at the same address each
time svrngs starts (§7): the person's own page on their computer, written for
someone a bit afraid technically, in §1's words
(`crates/svrngs-node/src/home.rs`). It says at its top whose it is and who
sees it, and no more: what a link lasts is said where one is made. With no
group yet it shows joining from a link first, then making a group, and
nothing else: what acts as you, a link from a group, lending and the
terminal's line wait for a group (the fifth xenia set: June read 80-119
words of her 40 there, and pasted Kim's link into the other box,
`demo/xenia/findings/wordy-home.md`). Each group is shown as soon as its people are read, and what they lend
is added once each computer has answered or not, so a computer asleep holds
back only that list. Each group is headed by its name, who made it and since
when this computer has held it, so two of one name are told apart, in a colour
derived from its id, nothing to choose. Under it, what is in the group
first, then who is in it (the operator, 2026-10-26: Money found first):
- **the door**: each person once, a cell in a grid, in the order they
  joined — lit, "on now", while their computer answers; drawn off, "did not
  answer", when it did not; "in from a phone" when their only keys are a
  browser's. Presence is only "on now", never when last seen.
- **the index**: one table of what is in the group, on any port — each thing
  by a link of its name, its kind in its lender's word when it differs,
  never one of svrngs's own (`scripts/vocab_check.py` reads the pages'
  scripts too), and beside it what opening it agrees to; whose; how it
  opens (in a tab, signing you in once when its lender's program does); and
  on or full now. At the group's foot, off the first screen, what is for
  other devices and programs: each can be opened to this WiFi, or closed
  (the operator, 2026-10-26), and its address copied for a program; then
  the group on a screen. In the row and the page's top they were words
  read before Money (the fifth xenia set, Kim's Home 90 of her 30). A computer that did not say what it lends is a row of its
  own, by why, so nothing is stale and nobody is left out — among them one
  running an older svrngs, whose answer is not read field by field
  (`crates/svrngs-node/src/version.rs`).

On it a person can:
- open each thing lent ("Sam’s llama-server"), in a tab of its own, or
  copy its address for a program, such as a chat program pointed at a
  friend's model;
- open a link to a lent app someone sent in the chat, pasted under "Open a
  link from your group", once in a group;
- join a group from a link someone sent, make one, and make a link to invite
  someone or to add their phone or another computer;
- open the group's folder in the computer's file browser;
- once in a group, lend what runs on their computer, under "Already running
  here": only when
  they press "Look for programs running on this computer", never as the
  page loads, since a look is seconds of the computer's work: every port
  from 1024 to 49151 that answers on 127.0.0.1, less svrngs's own, what it
  lends already and what signs people in to a lend. A kind the
  well-known table knows is named, with what it needs first, and lent by one
  button; any other once the person gives it a name and says what it is;
- bring things to the table: the shelf, svrngs's own apps (Money, Events,
  House rules), listed as data in `crates/svrngs-node/src/shelf.txt` and
  embedded at build so svrngs's code names none, each brought into the
  group's folder by one button, never over one there already. Its boxes
  start unticked, and for a group just joined it is folded as "Bring more
  to <group>": a newcomer read the ticked boxes as a grant asked of them
  (the operator, 2026-10-26; `demo/xenia/findings/
  after-join-reads-as-permissions.md`);
- change how many people at once may use what their computer lends, a number
  or everyone, or stop lending it;
- say yes to an app, or stop it;
- show a group on a screen — a TV, a spare laptop, a projector — at
  `/?screen=<its id>`, opened in a tab of its own from "Show <group> on a
  screen" at the group's foot: its door alone, big, each person's lends in their window, read
  again a second after each read began, since back to back a 25-computer
  group's founder spent 180% of a core
  (`demo/runs/twentyfive/2026-10-17/README.md`); and beside it who joined
  and what was lent since the screen opened, the newest marked, kept by the
  page alone. Who went off is never a line: the door says only who is on now;
- leave a group: "Leave <group>" says first what stays on this computer,
  what stops, that coming back needs a new invitation and that this
  computer cannot come back yet; the keeper is offered the hand-on beside it.
  A group they left is listed under "Groups you left".

Each is the command's own act, by the same code — a group made as `svrngs
new` makes one (the node's `group.rs`), joined as `svrngs join` joins
(`join.rs`), a link as `svrngs link` (`invite.rs`), a lend as `svrngs share`
decides it (`share.rs`) — so a computer whose svrngs runs from login, where
`svrngs new` and `svrngs join` are refused while it runs, needs no terminal.
A group's folder goes where a joiner's does (`Folders::place`, one home for
`new`, `join` and the page); a place another group's folder holds already,
by its name or, on a disk that ignores case, a name differing only in case,
stays its, and this group's goes beside it with its id's start after its
name. A name one of their groups already has makes another
group all the same, said as the command says it — "You’re already in Trip
(Sam’s, since 3 Oct). This makes a second group called Trip." — by one rule
(`group::namesakes`). What the page cannot do it names under "Done in a
terminal", with the one line to type: removing someone (`svrngs remove`).

The page answers only itself, from its own origin (§7), is never framed, and
never runs an app's code. Its only writes are the person's: a yes or a stop,
what runs here lent, a change to a lend, a link, the folder opened, a group
made or joined, what it brings to the table, a new version said yes to, a
leave or a hand-on.

**The group's page on a phone** leads with "You’re in <group>" and the
index, then draws the same door and shelf, folded as the home page folds
it for someone who has just joined, from
the same lines (`question.js`), judged by the answer of the computer it
reached, and brings through that computer, whose door asks that the phone
stands in the group. One version check, the node's `version.rs` compiled into
the page, says a computer the page reaches runs an older svrngs, and draws
nothing of its answer. Its heading says only "You’re in <group>, as
<name>"; who made the group and since when the phone joined it heads its
people, so two of one name are still told apart. Its index is a line each,
the open and what opening it does, and "Full now." when it is: whose it is
the open names, and everything opens in that tab, so no column says
either. The maker in the heading and a head of four columns were words the
fifth xenia set's Ana read before her first tap, 60-70 of her 20
(`demo/xenia/findings/wordy-the-group-s-page.md`).

**Asking for a yes.** An app sends its person to a page no app runs at: on a
computer the home page, `…svrngs.localhost:<port>/ask?lend=<label>&app=<label>`,
and on a phone the group's page at `#ask=<lend>&app=<label>&at=<path>`.
`/svrngs-reads` at the app's own origin gives each lend that asks its
words, `asks`, an `ask` address, to which the app adds `&back=<its path>`,
and `stop`, where it posts its own stop. Either page asks "Let Sam’s night
use Ana’s taste?" around the lender's words, in one wording both load
(`question.js`), posts the yes to the lender's door as the person, and sends
them back to the app at its path. Both pages list each yes under "What
acts as you", with its Stop ("Lent apps", call 6).

**Joining: the client first, the browser a door a group opens.** `svrngs new
<name>` makes a group and prints its link, `svrngs://join/<token>`. On a
computer `svrngs join <link>` redeems it, and the computer is a member:
running the client is the way in, and nothing depends on a hosted URL. A
group that wants phones without an install names a page origin, one it
hosts or ours, and its links carry the same token as `<origin>#<token>`. On
a phone that link opens in the browser and the person is in: the page makes
the person's own key and reaches the group by dialling over iroh in wasm,
relay-only. A phone holds no copy of the record; it reads and writes through
the computers it dials.

A person admitted at one computer is known at another after a round of the
group's sync, which the admitting computer starts as soon as it writes the
`Admit`. A computer that has not heard of them yet says so — "has not heard
of this computer yet" — never that it did not answer. A lender's door
meeting a pass it does not know asks the group once, for at most 10 s,
before it answers; the round dials every member at once, and the door
answers as soon as one that knows them has.

A phone joins by five calls:
1. *The page holds the key's bytes, sealed.* The person's key is 32 bytes,
   sealed by a non-extractable WebCrypto AES-GCM key, both kept in
   IndexedDB. It cannot be a non-extractable signing key: iroh signs its
   handshake synchronously, and the issuer redeems only for the key that
   dials it.
2. *A browser's key dials and is never dialled.* The redemption names
   whether its new key is a computer's or a browser's (`Device`), and the
   round, the live fan-out and the list of what members lend skip a
   browser's key, since a phone's endpoint lives only while its page runs.
3. *The page holds the person's key; each lend's worker holds a pass.* The
   page's service worker keeps only the shell (`web/sw.js`). A lend opens at
   an origin of its own, where its worker (`web/lend.js`) dials with the
   fresh key a pass names ("Lent apps", call 6), and the lend's start page
   (`web/open.js`) hands the pass over. One tab of the page at a time holds
   the person's endpoint for a group, by a lock named for it, since a second
   endpoint of one key sets the first aside at the relay; a tab opened beside
   it says the group is open in another tab.
4. *The phone dials whoever lends a service*, directly by key, since it stands
   in the group. The issuer, the computer the link names, only redeems and
   answers what the group's people lend, each with its lender's key and the
   relay its group's record names for it, and the page dials the lender
   through that relay, else the link's. It dials only the relays its link and
   its group name, and a link naming none is refused in a person's words
   (`web/src/lib.rs`, `bind_for`). No computer proxies another's lend. The
   page dials the issuer and those lenders at once — the issuer within 10 s
   (`dial::LINK_WITHIN`), each lender within 4 s — asks the first to answer,
   and says through whose computer it reached the group. When that answer
   fails, it asks the next to answer, and stops only once every one has been
   asked. A member who lends nothing is not among them.
5. *One body for the link.* The command and the page parse a token and sign
   the ask and the redemption with the same code, in svrngs-core: bytes and
   keys in, bytes out.

*On an iPhone, join from the Home Screen.* A web app added to the Home
Screen keeps a storage of its own, so the person's key kept by Safari is
not there: opened from the Home Screen after joining in Safari, the page
joins again, as a second person of the same name, and a phone cannot make
the device link that would join as the first (measured on the operator's
iPhone, iOS 18.7, 2026-10-30). So before the name is asked, in iPhone
Safari alone, the page says to add it to the Home Screen first and join
from there; it knows Safari by `navigator.standalone`, false there, true
in a Home Screen web app and undefined in every other browser, never by
its user agent (`web/app.js`, `askName`). The link's token stays in the
page's address, which the Home Screen copy keeps.

*A name binds once where a stranger gives one.* A key is who someone is, a
name is a claim, and a claim on a name someone already holds counts only
with that someone's key. A link lets no one in under a name a standing
person of the group goes by, compared case-blind: the issuer refuses it
before anything is written (`crates/svrngs-core/src/register.rs`,
`NameTaken`), not the fold, where "first in order binds" would let any
member's computer evict a person by a backdated `Admit` under their name.
The joiner is told who goes by it, how to add this device if it is theirs,
and how to join under another name. A removed person's name is free again.
Squatting a name before its person joins stays possible; the keeper can
remove the squatter.

*A person's other device joins by a link of theirs.* `svrngs link <group>
--device` on a computer makes a device link, an `Invite` naming that
computer's person (§6). Whoever opens it joins as that person, asked no
name, so the command says to send it to nobody else. A computer or phone
already in the group on its own stays as it joined, and a phone that lost
its key joins again by a device link. Only a computer makes one, as only a
computer redeems a link.

*The phone's app joins by the same link* (designed, not built; §7). On our
page host the chat link is a universal link, so with the app installed it
opens in the app in place of the page, by the
`apple-app-site-association` file the host serves beside it. A group on a page host of its own
gets none: its link opens the page, and the app's "Paste a link" takes it.
The app asks the name as the page asks it, in the page's words
(`web/app.js`, `askName`), and makes the same two calls to the issuer,
`/ask` and `/redeem` (`join`), over svrngs-core's `joining`, with a key of
its own kept in the Keychain. Its redemption names `Device::Browser`
(call 2). A device link opened in the app joins as that link's person,
asked no name, as on the page. A person already in through the page who
joins from the app is in twice, told apart by when each joined. Joining as
their page's self waits for a phone that makes a device link (PLAN §2).

**Lent apps, as they come.** North star 2 holds for the apps people already
self-host: `svrngs share 8096` lends Jellyfin as it comes, and someone who
never used it opens it from the list, plays and seeks, on a computer or a
phone. A friend's model is lent the same way: `svrngs share <port> --kind
llama-server`, its chat page opened from the list and its OpenAI-compatible
API given to a program. A kind is its lender's word, never guessed from a
port the well-known table does not name — 8080 is many programs' — and
`share` on a port it does not know names the kinds it does. What a lend keeps
back is the table's, its lender typing none (`keeps:`, `share::kept_of`):
for llama-server, the routes that would show one person's chats to the others
or change the server for everyone (`/slots`, `/v1/stream`, `/v1/streams`,
`/v1/chat/completions/control`, `/lora-adapters`, `/metrics`, `/tools`,
`/cors-proxy`, `/models`). The door refuses a path for every method alike,
and never changes an answer: llama-server's `/props`, which its chat page
reads, shows everyone where the model's file is on the lender's disk.

**A model or a tool is lent like any app.** A server that answers OpenAI's
`/v1/models` is a model, and one that answers MCP's `initialize` over
Streamable HTTP is a tool. `apps/pantry/`, an app on the person's own
computer, asks each lend its protocol, lists them with whose computer lends
them and the address to paste, and gives one OpenAI-compatible address that
routes each request by its `model` to the friend who lends it; nothing is
registered, and the pantry keeps no kind list. Behaviour is another app on
the consumer's computer — an OpenAI-in, OpenAI-out adapter that calls the
pantry — so every lend it reaches is reached as that person; the vision
bridge in `apps/pantry/examples/` is the worked example
(`demo/runs/models/2026-10-22-run/`).

Seven calls:
1. *Each lend at an origin of its own.* A computer serves every lend at
   `http://<label>.localhost:<port>/`; a phone opens it at
   `https://<label>.<page host>/`. Each lend's root and storage are its own,
   at the price that the page's host and the phone's name lookups see which
   lend is opened. One shared origin let any lent page unseal the key of
   whoever opened it, and a shared path prefix carried none of five
   self-hosted apps, each asking for its own root. A group that does not
   trust our host runs on any static host that answers every name under it
   (§7). The service is still asked at its own address
   (`Host: localhost:<port>`), and the door tells it the browser's origin
   as a reverse proxy does: `X-Forwarded-Host` the host the browser asked,
   `X-Forwarded-Proto` `http` for a `*.localhost` host or an address (the
   room's, "The room", call 3) and `https` for a page host's name, and any
   of either the client typed displaced (`door::tell_origin`). An app that trusts its proxy builds its pages on
   the lend's own origin, as Jellyfin's SSO plugin does once the lender
   sets its stock `KnownProxies` (`demo/runs/oidc/2026-10-21-jellyfin-2/`).

   A lend's address is its bookmark, and never changes: its label is its
   lender's key and its name. Opened where its worker is not, a browser
   that never opened it or cleared it, the page's host answers with the
   group's page, which sends the browser on to the group's page at
   `#open=<label>&at=<path>&cold`: holding the group it signs a pass and
   comes back at the path, and holding none it says this browser hasn't
   joined. A tab is sent on at most once a minute, so a worker that never
   takes its pass is said, not bounced again (`web/app.js`, `cold`). A
   path of the app's own, such as Jellyfin's `/web/`, is no file of the
   page's host, which answers it with the site's `404.html`, as Vercel,
   Netlify, Cloudflare Pages and GitHub Pages serve one for a missing path:
   `scripts/build-web.sh` makes it from `index.html`, its paths from the
   root and its policy the same, so at a lend's origin it sends the browser
   on as above, and at the group's own address it says there is no page
   there.

   On an iPhone each web app added to the Home Screen keeps a storage of
   its own, apart from Safari's. A lend's own address added there is a
   browser that has not joined: its first open lands on the group's page
   holding no group, which says so. The way to keep the house on a phone is
   the group's page added to the Home Screen and joined from there
   ("Joining").

   A lend also has an alias, its name at whose it is, opened on the group's
   page at `#open=films@Sam`, read as `URLSearchParams` reads a fragment
   (a space as `+` or `%20`). Whose is said as the page names people: two
   of one name told apart by when each joined, `films@Sam (joined 24 Oct)`,
   and two computers of one person lending one name by when each computer
   joined, `films@Sam (computer joined 25 Oct)`; an app the group carries
   is its kind, `#open=Money`, any copy. One decider for both pages
   (`question.js`, `alias`): an alias names one of what the group lends now,
   or it is said that it names none or more than one, never a nearest
   match. Every page reads the same record, so it names the same lend on
   each, and its day is UTC's, as two browsers' own days near midnight
   differ: the alias one person sends opens for another anywhere, and the
   name the page shows beside a lend says the same day. It is unique within a group
   only: across groups, or as a name of its own, it would need a centre.
   In a computer's browser each row of the group's page gives the address
   to keep, its alias, or its label where two lends share one, with one
   line in the same words as the home page gives a lend's
   `<label>.localhost` address (`question.js`, `keeping`): "Bookmark
   this." A phone's rows say none, as a Home Screen copy of the page made
   after joining joins again, as someone new. An address typed or tapped on the group's page is
   a new fragment the browser does not load, so the page loads again to
   open it. On a computer the group's page also says how it reaches you,
   from the chat link to the computer lending a thing, each link in the
   page's words (svrn-docs `rings/current/IN_THE_BROWSER.md` §3); on a
   phone it is left out of the first look, which is the group's things'.
2. *Jellyfin as it comes.* A lend serves stock jellyfin-web from the
   lender's Jellyfin; nothing in svrngs renders a library of its own.
3. *The infra learns no Jellyfin.* Anything that knows an app's API lives
   in `apps/<app>/`, whose nouns `scripts/vocab_check.py` keeps out of the
   infra (§5), and what the door gains for it is general: any app. An app's
   words and paths are the well-known table's alone
   (`crates/svrngs-node/src/well_known.txt`).
4. *Sign in once, or none.* Lent as it comes, a lend carries no secret and
   each person signs in once with an account the lender made: `svrngs share`
   asks no password for a kind its table says keeps its own accounts, and
   prints how to make each person one (`share::accounts_of`); for a kind
   that takes the person from `x-svrngs-user`, it prints what to set. Zero
   sign-in is the lender's to choose, outside the infra: an app that speaks
   OpenID Connect names svrngs once in its own settings, and the door signs
   each person in as themselves ("The room", call 6).
5. *Measured before built.* What a stock app does through the door — its
   sign-in, its live socket, two people's plays through one lend, whether
   any byte carries a secret the lend holds — is measured before anything is
   built on it.
6. *The person's key never enters a lend's origin.* The page signs a *pass*
   — a fresh key may act as this person toward these lends until this time —
   for the lend it opens and the lends its lender declared it reads (`svrngs
   share --reads <kind>`). The lend's origin dials with the pass's key, and
   its worker keeps each lend's cookies apart, since a service worker sees no
   `Cookie`, and asks for gzip but for a range, unpacking it, since it sees no
   `Accept-Encoding` either and llama-server's chat page comes only in gzip. A lent page can use only its own pass. A pass names its app
   apart from the lends it reads, `Pass { app, reads }`, and the door tells a
   service the app from it alone (`crates/svrngs-core/src/pass.rs`). A pass
   names no method or path: through it the page may do anything the person
   may, writes included.

   A phone's holder is one key per lend's origin, kept there as a WebCrypto
   key that cannot be copied: the group's page hands its bytes only when
   the origin keeps none, `open.html` imports them as a key no script can
   export and drops them (`web/open.js`), and every pass after names the
   same holder, which the group's page keeps by the lend's label
   (`web/store.js`, `holder`), so the phone's acts in an app are one chain.
   Its pages can use the key while open, as they use what the lend reads,
   and are trusted the same way: the person chose the app, and its code is
   pinned to what they said yes to. They cannot carry it off. iroh's
   handshake needs an endpoint key as bytes, so each worker run makes its
   own, and the holder names it in a link, a short pass the door checks
   beside the pass (`x-svrngs-link`; `Pass::carries`). An origin holding no
   key its pass names asks the group's page for a new one (`#open=<label>
   &anew`), which a window another page opened is never given. Where the
   browser loses that key on its way to the origin's worker — Safari on an
   iPhone does, any CryptoKey, `messageerror` in its place
   (`demo/runs/ios/2026-10-30-observe/keyprobe/`) — the start page hands the
   worker the bytes it was given, in memory alone, and the worker imports
   them; they are kept nowhere at the origin, so a worker the browser
   stopped holds no key, its start page asks for a new holder, and a request
   in flight as it stopped fails once (held by `web/tests/handover.mjs`,
   `KEYS_LOST`). A lend at a
   computer on an older svrngs, whose door takes no link, is opened as
   before, its holder's bytes kept at the origin for the worker to dial as,
   and the writes the worker signs wait until a computer on the new svrngs
   takes them, the page saying so (version 2, `version.rs` `links`).
   Measured: `demo/runs/capability/2026-10-23-pass-key/` (`probe.mjs`,
   `idle.mjs`); held by `web/tests/ask.mjs` step 5.

   So a lent page composes others: at its own origin, `GET /svrngs-reads`
   answers each lend it reads — its kind, name, lender and the address to
   ask it at — and the people whose computer did not answer, never the pass
   or its key (`surface.rs`, `read_by`, one decider for a phone and a
   computer). On a phone, a page sent to such an address goes through the
   group's page, which signs that lend's pass, so a lend opens at a path on
   a phone that never opened it.

   On a computer, the lend's origin names the app asking from what the
   browser says of the page, which no script can set
   (`crates/svrngs-node/src/origin.rs`, `app_of`): a typed address or a
   request from the lend's own origin is the lend's own page, and so is the
   whole tab going there, a link the person follows, unless its `Origin`
   names another lend; a request from another lend's page is that page's,
   by its `Origin`, a frame's by its `Referer`, so a page's frame or form is
   held to what its fetch is; one from another page naming no lend, an
   `<img>`, is refused `closed`; and a WebSocket's handshake, which sends
   no `Sec-Fetch-Site`, is judged by its `Origin`. A program showing the
   token is the lend's own. A page's fetch of a lend it reads carries no
   cookie, as it is another origin, so `/svrngs-reads` answers that lend's
   address with a capability in its path,
   `http://<read>.localhost:<port>/svrngs-as/<cap>/`: the token's HMAC over
   the pair, the page's lend and the read lend. The door takes the prefix
   off before anything is decided, and lets the request in only when `app_of`
   names another lend whose capability it is; a link the person follows
   under it goes on to the home page's `/to`, so the home page's session
   decides. A capability is never a session: whoever holds one is that page
   toward that one lend, and the person's yes is still judged on every
   request. A page reaches its own lend and the kinds its lender
   declared it reads, in a group both lenders and this computer stand in,
   else `closed`. Another computer's app reaches them as this computer's
   person only on that person's yes, else `unasked`, "Say yes on your home
   page: open this app from there.": per person and per app, never per
   version, since live code can't be pinned, kept beside the lends' yeses
   (`door/choices.rs`, `reaching`) and judged where the reach is
   (`surface.rs`, `reaches`). Opening the app from the home page is the yes:
   its button names the app, "Sam’s notes"; the line beside it says what
   it reads and writes, "It writes what you say there into House’s record,
   as you. It also opens your friends’ films as you."; and pressing the
   button is the consent, posted before the app opens. On a phone the
   group page's open, with the same line beside it, is the yes, as the
   pass it signs names what the app reads. An app this computer lends
   reads what its person declared. Beside the open of an app the group
   carries is first what it is for, the shelf's words or "its own part of
   the group’s record", then "Opening it lets it read and write that, as
   you.", so its first version runs as a person once they open it so named
   (`question.js`, `named` and `agreed`). Led by "It reads and writes", a
   newcomer read it as done on her behalf without her yes (the fifth xenia
   set, `demo/xenia/findings/said-too-much-the-group-s-page.md`). Each yes is
   listed under "What acts as you", folded under its heading, a list to
   look at, never in the way,
   on both pages, with since when and a Stop (`question.js`, `acts`, which
   groups what acts as the person by what it can do: reads and writes,
   opens friends' things, knows who you are, open to this WiFi, open to
   programs on this computer (Play, "A port, not a page"), can decide
   for you, the last only the person's own devices, by key). On a computer
   the Stop is kept as the yes is, and holds from the app's next request; on
   a phone it is kept with the group, and the next pass the page signs for
   the app names nothing it reads, so it holds within 12 hours, as the
   last pass lapses. A lender whose computer did not say what it lends is
   could-not-judge: `away`, or `unanswered` among the reads, never "nothing
   is lent here". The app is carried to this computer's own lend as the
   door's `Who::Own`, and to another computer's in a pass this computer signs
   for its page, its own key the person and the holder, five minutes long and
   signed for each request; that door refuses one not the dialler's own
   (`door.rs`, `own_app`).
7. *A group's folder, synced both ways onto its members' disks*, a mesh
   Dropbox. `new` and `join` keep one at `~/svrngs/<the group's name>`, said
   where beside the command and on the home page; `share --folder <dir>
   --group <group>` keeps another directory the same on every member's disk.
   Each scan's changes are one act in that group's record alone, each naming
   a path, its bytes by hash, and the entry it replaces. An entry no other
   replaces is a head; a path's second head is written beside it, named as a
   conflict with its writer's person. A path is parsed (`SyncPath`, of named
   parts), so no act can write outside the folder on another member's disk.
   The bytes move by iroh-blobs: a computer serves a hash only to a dialer
   standing in a group whose folder, or an app, placed it there, and fetches
   what it lacks by verified range from the group's members, passing over a
   holder that does not answer within 3 s; who is taken as asleep is decided
   once (`ASLEEP_FOR`), and who stands where is the round's
   (`Host::standings`). The index is the record, not
   iroh-docs, whose namespace would be a second decider of who reads. A
   group whose record cannot be read stops its folder by the group's name,
   never read as placing nothing; a kept state that cannot be read stops its
   directory the same. The store keeps what the folder's layout places and
   no more, made durable before the kept state names it. Watching a member's
   films stays their lent Jellyfin's: a folder is synced, not streamed.

**Many people, one library — first class, not an edge case.** Three friends
in three homes on Sam's library at once is the normal evening.
- *The upload is declared.* The lender states it (`svrngs share --upload
  <Mbit/s>`); the door divides it evenly among the people a lend counts,
  and tells each request its share in `x-svrngs-declared-share-bps`, as a
  declared figure. A lend that declares none is told no share.
- *The local network is guessed.* iroh says nothing of it, so a viewer whose
  path is a private, link-local or unique-local address is taken to be in
  the lender's home and not counted, and the trace says it guessed.
- *Transcoding is the app's.* Nothing in svrngs asks a service for less.
- *Too many at once.* How many a lend lets in at once is its lender's
  choice, and none chosen lets in everyone in its groups: no number stands in
  for one nobody chose. Its lender sets, changes or clears it on their home
  page or by `svrngs share --limit`, and the door takes it at its next
  request, with no restart. A person counts while a request of theirs is in
  flight or within 30 s of one (`door/limit.rs`, `WINDOW`); nothing queues.
  Past the limit the door refuses with its reason in a person's words, the
  figures, and that the number is the lender's to change.
- *The same room* counts as one person, whoever holds the remote ("The
  room").

**The room: a lend opened to this WiFi, for any device's own app.** A film
from a library across town plays on the TV in the room, a phone there opens
a friend's llama-server, and svrngs gives the devices on this WiFi reach and
nothing more. Every lend can be opened to it, any app on a port; its words
assume no screen, a TV one device among others.
1. *A device's own app is the client; svrngs gives it reach.* A TV plays a
   lend through the app's own client for it — Jellyfin's own TV app —
   pointed at an address on this WiFi; a phone as the remote is the app's
   own, and a browser on any device is a client too. Nothing in svrngs knows
   AirPlay, Cast, DLNA, a TV, a player or what a lend serves. Old TV
   browsers are not supported.
2. *A computer in the room.* The address comes from a member's computer on
   the devices' network: not from a phone, since a browser cannot listen,
   and not from a public address per lend. Its hop to the library is
   computer to computer over iroh, which can go direct, where a browser's is
   relay-only.
3. *The room is the running node's, turned on per lend.* Its person opens
   a lend to this WiFi or closes it with the button in that lend's line
   below its group's things on the home page ("Open it to this WiFi",
   "Close it to this WiFi"; "Open to this WiFi", under "What acts as you",
   lists what is open), or opens it with `svrngs room <name>...`, which asks the svrngs running
   here, else the one it starts, at the same loopback route (`/v1/room`):
   the room has one decider (`crates/svrngs-node/src/room.rs`). Each lend on
   is served at a port of its own on this computer's private address,
   decided by its label so it is the same every run, its root its own, in
   one body with the lends' origins on loopback, not a second. It reaches
   each lend as this computer's person, as this computer's own browser
   does: a device on this WiFi is this person's own, and no computer proxies a
   lend for anyone else. It carries what the origins carry: Range, both
   bodies streaming, a WebSocket. The address is the private one the
   outward route leaves from, else the one private address listed, else
   none, said so — never loopback, a wildcard, tailscale's or a public
   address. A lend is kept on across restarts, by its lender's key and its
   name, with the network it was turned on at, told apart by that
   network's router's hardware address; it is served only while this
   computer is on that network, read again every few seconds. On another
   network, or one that cannot be read, it stays closed and says why, and
   opens again on the network it was turned on at. Its listener is bound
   by its label alone, so the room opens as svrngs starts, before any
   lender answers; a lend whose lender is away keeps its address, answered
   "did not answer" within the dial's cap, and serves once they are back.
   What is on, when it cannot be read, is said, never taken for nothing:
   the room serves nothing and the page says why until a lend is turned on
   again, which writes it over.
   Anyone on that network reaches the lends on as this person, which the
   page says beside its button and the command when it turns one on; a
   request from off the network is refused. `/v1/room` answers each lend
   on, where it is served or why not, in the one wording the page and the
   command show.
4. *Measured, then told.* What an app's own client for a screen needs
   through the room door is measured before any of it is written. What the
   lender sets lives in `apps/<app>/` (`apps/jellyfin/ROOM.md`), never in
   the infra; anything the door would need is general, or a finding for the
   operator.
5. *The lender's door counts the room as one person*, and guesses its path
   as it does any other. Who in the room asked is the app's to know, by its
   own sign-in.
6. *Sign-in with svrngs; the door's own.* An app that speaks OpenID
   Connect names the lender's svrngs once in its own settings, and the door
   answers it with the person it already tells the app in its headers — no
   program per app, and no consent screen. Nothing is typed, because the
   claims are the door's own: `sub` the person's key, `preferred_username`
   their account, `name` their name, and `groups` `["member"]` or
   `["guest"]`. There is no `email` until mail is real (§ "Sign in once").
   - *One issuer string, exact*: `http://127.0.0.1:<api port>/oidc/<lend>`,
     with no trailing slash, byte-equal in discovery and in every ID
     token's `iss`. It is plain http on the lender's loopback: the traffic
     never leaves the computer, and a certificate for a loopback name would
     need a centre (north star 3).
   - *The front channel* is the lend's own origin,
     `http://localhost:<lend port>/.svrngs/oidc/authorize`. The door
     answers it itself, after the asker is known and before the lend's
     refusals and its limit, and never carries it to the app. So a
     stranger or a removed member is refused before any code. It checks
     `response_type=code`, the lend's client, the app's own redirect — on
     its own address, `http://localhost:<lend port>` (or `127.0.0.1` or
     `[::1]`, the hosts by which the door also turns a service's redirect
     into a path), when it does not trust a proxy's headers, or on the
     lend's own origin when it does trust the door's `X-Forwarded-Host` and
     `-Proto` — the scope, `state`, `nonce` and PKCE, and mints a one-time
     code bound to that person, the client, the redirect, the challenge and
     the nonce. A guest's code names the guest, never their host. A
     redirect a browser would read as another host's, `//` or a `\` after
     the origin, or one holding a control character, is refused, as the
     door never makes such a service's redirect a path either: both read
     it through one type (`door::LendPath`).
   - *The back channel* is the record API's listener, which answers only
     this computer: discovery, the keys, the token and userinfo, behind the
     guard that lets no page ask it. No listener is added and the trust
     floor is unchanged. `client_secret_basic`, `client_secret_post` and a
     public client with no secret are accepted; PKCE S256 is checked when
     sent and not required; a nonce is echoed when sent and omitted
     otherwise; no PAR endpoint is listed.
   - *What is signed* is ES256 through `ring`, already in the lock: one key
     per lender's computer at `<data dir>/oidc/key.pk8`, 0600, never in the
     record and never synced, the same across a restart so the apps' cached
     keys hold. The JWKS answers the same `kid` that signed, so the key a
     client verifies is the key that signed.
   - *Nothing leaves the lender's computer.* The key stays in the data dir,
     each code in this run's memory and used once, and a token is on no
     wire but the app's and in no record.
   - *Turning it on* is the lender's: the home page, or `svrngs share`,
     prints the issuer, a client id and a secret for the app's form, and
     the deviations in the lender's words (plain http on this computer,
     ES256, no email yet). A lend declares its one secret or its sign-in
     with svrngs, never two; a declaration's unknown field is refused,
     never ignored. The client id and secret are made from
     `[A-Za-z0-9-._~]` only, the declaration holding the secret is written
     0600, and the page shows it once (`lend::declare`, `oidc::client`).
   - An app whose page script navigates to the authorize address itself is
     not served (SIGN_IN_WITH_SVRNGS §8 10): the address would be the
     viewer's own machine. No program per app stands beside a lend: the one
     the well-known table named retired with its row (`oidc-retire`), so a
     kind is a port or its lender's word alone.

**A chat's door is the group's link.** A person posts the link where they
talk, as they post any link, and it opens the same from Discord, iMessage,
WhatsApp, Slack or Signal. svrngs's part is the link previewing as an
invitation — a title and a description on the group's page — with the token
in the link's `#` part, which no chat's preview fetches, so the preview
names no group and carries no token. No package per chat, no bot, no
commands; the core never knows a chat exists.

**A relay anyone runs.** A phone's every byte crosses a relay, a browser's
iroh being relay-only, so a group need not lean on anyone's: `svrngs relay
<name>` serves iroh's own relay server from the svrngs binary — the one the
computers that dial it are built with — at `https://<name>/`, on TCP 443 and
80 and QUIC address discovery on UDP 7842 (`crates/svrngs/src/relay.rs`). Its
certificate is Let's Encrypt's, asked and renewed by the relay itself and
kept in `<data dir>/relay`; `--staging` tries a name first. A name, never a
bare address: iroh-relay's ACME client orders DNS names alone. By default it
lets through only the computers and phones that stand in a group its
computer holds, read from the record on disk and asked again on each
connection, each refusal traced; a computer in no group is refused the
verb, and one whose groups cannot be read is refused as that. `--open` lets
everyone through, each at most `OPEN_RATE`, 8,800,000 bytes a second, the
most a film at Demo A's bitrate drew.

**Where a computer meets its groups.** A group that keeps no relay of its
own is met at one of a closed set, `cli::MeetAt`: n0's relays with n0's
address lookup, or svrngs's relay at `https://relay.svrnme.sh/`
(`cli::DEFAULT_RELAY`) with no lookup and n0 none of the path. The build
ships one, `cli::SHIPPED`: n0's for now, chosen, never a fallback when
another relay fails, until the operator's switch (PLAN §1). A group that runs
its own relay is made with `svrngs new <name> --relay <name or url>`: every
computer its links let in meets it there, and says so in the record by its
`Relay` act (§6). A link carries its issuer's home relay, so a phone's page
dials it and asks no lookup; a joiner keeps none of n0's relays a link
carries, since they are the issuer's build's, not the group's. A computer
meets each group it holds at that group's relay, else the build's, and iroh
makes the nearest its home. A computer that reaches none of its relays in
10 s says which it tried, and goes on trying.

## 4. Domains, and the fork

| Domain | What it owns | Meets svrngs by |
|---|---|---|
| **svrngs** | the record, the node, the doors, the command, plain words, reference apps, demos | — |
| **chats** | nothing of ours: a person posts the group's link where they talk (§3, "A chat's door is the group's link") | the link |
| **apps (rngapps)** | anything served on localhost; app-specific code lives only in `apps/<app>/` | the surface in §3, and nothing else |
| **sovereign (svrn)** | local AI on a member's computer | its OpenAI-compatible endpoint on loopback |
| **commonwealth-ai** | the mesh for one owner's machines, and the ring code svrngs forked | nothing at runtime; `scripts/ports.py` pairs each forked file with its original |

**The fork.** svrngs owns the group's code, forked from commonwealth-ai's
ring slice at `e1a43bcd2`: the record (`commonwealth-rail-core`), the
journal (`commonwealth-rail` and `oplog`), ring sync and its route
(`commonwealth/crates/commonwealth-rails/src/ring_sync`, `ring_routes.rs`),
the sync bodies (`sovereign/crates/sovereign-peer-wire`), the dial-by-key
tunnel and the accept loop
(`commonwealth/crates/commonwealth-transport/src/iroh.rs`,
`mesh-reach/src/guest.rs`), and the rail routes an app writes through
(`rail.rs`). What each upstream piece became — forked, replaced, or left
behind — is each unit's inventory, PLAN §4. How a piece comes over:

- **Behaviour kept first, then changed.** A piece lands with only the edits
  it needs to build here — paths rewritten, a small dependency inlined, a
  test re-asserted without a crate we do not carry — each named in its
  commit and in `ARCHITECTURE.md`. Its edition, rustfmt policy and clippy
  allowances stay upstream's, and every source file carries `ported from
  commonwealth-ai <path>@<commit>`. Behaviour changes only in later commits
  (ARCH 2), so a diff against upstream shows what svrngs changed. Where
  upstream code is tied to something we do not fork — the mesh — the tie is
  cut in the commit that replaces it, which says which lines changed
  meaning.
- **A fix shows its twin.** commonwealth-ai keeps `rail-core` for its work
  plane and the door for its mesh. The two systems share no wire, so the
  copies are a fork and may diverge; `scripts/ports.py` lists each pair and
  whether the original changed since — "unchanged" means upstream has not
  moved, not that the fork still matches it — so a signing fix on one side
  is seen on the other.
- **The arrow points one way.** svrngs links no commonwealth-ai or sovereign
  crate (`scripts/lean_check.py` refuses them from `Cargo.lock`).
  commonwealth-ai's ring-facing code — `ring_cmd`, ring-doc, ring-runtime,
  `ring_shim.js`, the phone's ring shell — retires as svrngs replaces each
  piece (its `docs/RING_SPEC.md`, retirement list).

## 5. Code

| Crate | What it is | State |
|---|---|---|
| `svrngs-core` | the record: acts, admission into one order, membership, the sync digest; and everything a browser's page must spell as the command and the node do — a link's token, the ask and the redemption, the protocols' names, a relay's URL, a lend's label, the pass, and a sign-in token's claims and signing input. `no_std`, so the compiler refuses filesystem, clock, network and threads; time and keys are arguments | none |
| `svrngs-runtime` | the only owner of each group's journal on disk, its projections and its writes: the journal and its write index, the open-journal map, the register's standing | the journals |
| `svrngs-node` | the adapters: sync and its route over iroh; redemption; the loopback record API and its live lane; the member door, its limit and shares, its sign-in with svrngs and each person's choices; the list of what members lend; each lend's origin on this computer; where this computer is, and on which network, by its router's hardware address; the room, each lend on it served on the network it was turned on at; the home page; the lender's sign-in key and its JWKS | each lend's declaration and secret, and each person's choices for it; who is using each lend now; each lender's sign-in key; the lends on for the room, and the network each was turned on at |
| `svrngs-web` | the page's dial, wasm only, in `web/` with its static shell and a workspace of its own: a browser's endpoint dialling a group's computer by key over a relay, HTTP over the dial, joining, and the worker that serves each lend at its own origin with a pass | none: the page's store holds the key, sealed; a lend's origin's store holds its pass and its cookies |
| `svrngs` | the command and the process: `new`, `join`, `link`, `share`, `unshare`, `room`, `remove`, `handover`, `leave` and `relay`; the bare command runs the node for every group this computer holds and prints where the home page opens | a device's keys; each group's seed |

Every dial the transport makes to a member's computer, and a phone's page's
to a lender, is given up when not answered within 4 s, the slowest working
dial measured times three (`dial.rs`, which the page compiles too), so a
person is told an asleep computer did not answer then, not after iroh's 30 s.
A joiner's dial to the computer its link names waits as iroh does. A phone's
dial is held only while the computer sends something within that time, a
keep-alive each second, and a request on a dial that falls silent is not
dialled again. A dial a request or a socket failed on is let go, so the
next one dials the computer afresh once it answers again (V1.1.3,
`demo/runs/doc/2026-10-20/found/`). Every request to a member's computer is a stream of the one
connection kept to it, read whole within 10 s, and no port on this
computer's loopback reaches another computer's door.

Held to `docs/principles/RUST_ARCH.md`: types encode the domain, ownership
encodes the architecture, I/O lives at the edge. Parse, don't validate;
closed sets are enums; one writer per journal, and no shared mutable state;
a trait only at a real boundary; `thiserror` in the crates, `anyhow` only in
the command and the app. Every decision branch traces at debug. New code is
held to these from its first line; forked code is held to them as it becomes
ours. Upstream's shape stays until a change replaces it: the node's map of
open journals sits behind a `Mutex`, which an actor per journal replaces.

| Invariant | Held by | State |
|---|---|---|
| Same acts, same state: a group's state is a pure function of its set of acts, and of its seed, which its namespace hashes (`Seed::id`) | svrngs-core is `no_std`, and `lean_check` keeps it so; an adapter reads a group only through the runtime's queries (`admission`, `standing`, `invite_for`), since the journal is crate-private to svrngs-runtime; and it reads an admission only as a fold made in svrngs-core (`Admission::membership`, `Admission::invites`) or as `wire()`, the acts serialized for an app to fold, since the acts are private to the core | holds for every adapter; `wire()`'s JSON can be parsed back into acts, and no gate refuses that; inside the core, `membership` still sorts the acts admission hands it |
| No act, no effect: nothing that cannot be undone to a group happens but from an act in its record. What is one computer's own — its keys, its lends and their secrets, a person's yes and stop to them, its folders' kept state, an app's blob and its lifetime — is kept in that computer's files (the State column above) and never in the record; a lent service's own state is its own, and the member door only carries a request to it | a store appends only `Authored` lines, which only `commit` makes after a `Command` decided under the write lock; a peer's lines enter by `ingest_all` unjudged, and every fold judges them; nothing deletes a line, and a `Seal` an older build wrote is held as an act that does nothing; what follows an act is written after it and driven by its writer: a cut told to the computers it cut, a leave told by the leaver's round until each computer answers | every act this computer signs goes through `commit`, a peer's through `ingest_all`; the journal's `append` is a test fixture |
| Both | the core never depends on the node; the clock, randomness and locks live only where an adapter holds one (`clippy.toml` bans them; each site carries `#[expect(…, reason)]`, which fails once the site is gone) | holds |

One property test stands behind every fold: permuting or duplicating
`admit`'s input changes nothing. A decider's contract is given / when / then
rows.

**Two gates keep the apps out of the infra.** Every app lists its own nouns
in `apps/<app>/nouns.txt`, and `scripts/vocab_check.py` refuses any of them
in infra code — comments and tests apart, where an example is fine. So the
infra never learns what a film, an expense or a crush is. And the canaries
run against the infra in `scripts/check.sh`: a change that serves Jellyfin
and breaks one fails the build.

**The canaries**, each a small app in `apps/` using only §3, and one
number:
- *Money* (`apps/expenses/canary.mjs`, run by
  `crates/svrngs-node/tests/canaries.rs`): no server, all state in the
  record. One computer per person, each writing as its own, a correction
  voids, and every copy folds to the same balances.
- *Events* (`apps/events/canary.mjs`, the same harness): an event posted to
  two groups, another's correction counting for nothing; every copy folds
  to the same event, and the second group's record holds none of the
  first's answers.
- *Conformance* (`apps/web-conformance/`, run by
  `crates/svrngs-node/tests/conformance.rs`): the web carried whole. One
  route per thing a self-hosted app leans on — a page asking for
  `/`-absolute paths, a WebSocket echo, a 64 MiB file read by Range and
  uploaded streaming, its own service worker, `SharedArrayBuffer` under
  COOP/COEP, a Host check with an absolute redirect, a cookie sign-in, local
  storage, a script policy by nonce, a post it takes only from its own
  address, a film sought twenty times — lent by two computers at once and
  opened from a computer and a phone, in Chromium and in WebKit as an iPhone.
  Neither lend's cookie reaches the other's requests, no byte holds either
  secret, and a lent page finds no key it could unseal.
- *The ceiling*, a Rust test (`an_idle_round_is_under_the_ceiling`): at 25
  members and 10,000 acts an idle round costs under 0.1 s of processor. The
  round's digest, its signatures and the register are each folded from the
  journal's tail, never rebuilt from the whole journal on each exchange
  (`crates/svrngs-node/tests/scale.rs` is the instrument).

WebKit's seeks back to back through a lend are a rate, printed beside the
same seeks straight at the canary, not a gate. Chromium's are a gate, but on
a host whose load is past 3 a core (`crates/svrngs-node/tests/browser/load.rs`,
`BUSY`) a miss is could-not-judge and fails nothing. Tap, a third canary once
named, is not built, and nothing runs it.

**Lean, and kept lean.** `svrngs-core`, the repository and its Swift stay
under the caps in `scripts/lean_check.py`'s `BUDGET`, their one home. Swift
is first-party code kept whole, on a rung of its own between the core's and
the repository's, never counted in either; its comment says
each move, and PLAN §4 the moves before it. A raise is the operator's, and
is asked only after an economizing pass: its commit gives the reason, names
what the pass cut, and says why the rest could not be found in the code
there is. The pass's cuts lower the cap to where they land, so room freed is
never spent but by the operator's choice. The budget counts the product:
tests, `apps/` and `demo/runs/` apart. A recorded run's driver is what ran,
kept byte for byte so its data reads against it, and no check runs one; a
helper a check runs lives outside `demo/runs/` (`web/tests/serve.mjs`,
`web/tests/control.mjs`), and code elsewhere under `demo/` counts.
`ARCHITECTURE.md` maps what exists and changes in the commit that changes
it.

**Words.** The *record* is a group's signed acts; its *journal* is the file
that holds them on one computer; the *register* is the part of the record
that says who is in the group — its `Admit` and `Remove` acts and their
fold. Forked code keeps upstream's words until a change renames them: *ring*
and *namespace* for a group, *roster* for a seed list of members, and names
such as `commonwealth-rail` and `MeshStore` in its comments. Upstream
comments cite ARCH principles by old section number (ARCH §18.1); the table
at the end of `docs/principles/ARCH_PRINCIPLES.md` maps them.

## 6. The record

Signed acts admitted into one total order by `(ts_unix, actor, seq, id)`:
`Record`, `Correct`, `Admit`, `Remove`, `HandOn`, `Invite`, `Relay`,
`Synced` (a folder's changes), `Runs`, `Visit` and `Signers`, the keeper's
cutover (`RailAct`,
`crates/svrngs-core/src/lib.rs`). `Correct` voids without erasing. A
per-actor digest lets sync send only what is missing. Upstream's `Seal`
retired everything its signer wrote before it, on every copy it reached;
here it does nothing. The variant stays so an older journal still parses,
no command writes one, and an app's is refused (`NotAnAppAct`). Forgetting
from the journal waits in PLAN §2.

- **The group decides who is in it.** Who may connect and who is synced come
  only from the group's register — the key that seeds it, and its `Admit`
  and `Remove` acts as `membership` folds them — never a mesh or a list a
  door keeps. The seed is the founder's key alone, bound to the founder's
  name; a group's id is the hash of its seed, so a seed the issuer answers
  for a link proves the id (`Seed::id`). The round, the acceptor, the sync
  route and the door all ask the one standing, so there is one answer to who
  is in a group (ARCH 8).
- **A grow-only set, linked like git.** Each act names its author's previous
  act, inside its id and its signature (`SignedOp::prev`), and one whose
  previous act is not its author's at seq − 1 is a `WrongPrevious` gap; a
  copy holds an
  act only with its parent, so copies converge by union. Two acts on one
  parent are a fork, and forks void nothing: both branches are held and each
  act weighed as itself, and the fork is named, a `SequenceFork` beside a
  complete answer (`crates/svrngs-core/src/admit.rs`). Only a line whose
  signature verifies moves a member's next seq, and the seq audit counts
  every act of a bound signer, so a member cannot leave another copy
  reading as incomplete.
- **Ids cover everything signed.** The id is the whole hash of the
  namespace, actor, seq, previous act, timestamp and body, and the
  signature is over the id (`act_id`, `crates/svrngs-core/src/sig.rs`).
  Every comparison of acts as identity uses the id the content derives
  (`derive_id`), never the `id` a line carries. A computer signs at the
  later of now and one second past the newest act its journal holds
  (`crates/svrngs-core/src/command.rs`), so a member's first act sorts after
  the `Admit` it has seen; writes faster than one a second run the stamps
  ahead of the clock.
- **A holder writes only what its pass grants.** A line may carry, inside
  its id, the pass that lets its signer, a holder, act as the person who
  signed the pass: a phone's act in an app it opened. It counts as that
  person's, under their standing, only appending its app's own `<kind>/`
  part, the kind the pass's label names (`Label::carried`), only before the
  pass lapses, and never deciding: a holder's `Runs`, or any act of the
  register, counts nowhere. Anything else is `RailGap::Ungranted`, a
  contradiction, or an absence while what it corrects is not held
  (`crates/svrngs-core/src/grant.rs`, judged in `admit`, each pass
  verified once). A cut covers its key's holders as it covers the key, and
  its `through` names their newest acts. A holder's line ships at line
  version 12, which an older build names `NewerVersionLine`.
- **An entry's author is its signer.** A line counts as its signer's
  person, or a holder's as the person its pass names. A line naming
  another person, as an older svrngs wrote a phone's act (`on_behalf_of`),
  counts as before only where the keeper's cutover covers it: `Signers
  {through}`, each key's newest acts the keeper's computer held, written
  once per group on its first round at the new version (`Cutover`; node.rs
  `cut_over`), the keeper's office as a cut is, and covering by ancestry
  as a cut does. Past it such a line is `RailGap::CarriedForAnother`, a
  contradiction, and counts as nobody's (`admit.rs`). A yes, a `Runs`,
  counts only from its person's own device and only naming bytes a folder
  act placed at those paths (`synced.rs`, `written`), whenever it was
  written, so a phone person's yes that went through another computer
  falls back to their last own yes, else the app as brought, and their page
  asks once (§3, "What runs as you"). A yes is kept by the person's first
  key (`Membership::person_of`), so one person's yes never runs as another
  of the same name; names are for display, and two of one name are told
  apart by when each joined, "ada (joined 24 Oct)" (`question.js`,
  `called`). Signed before and after: `demo/runs/authorship/2026-10-24/`,
  its design and what a fold costs.
- **Removal has one decider.** Anyone may ask to remove anyone; the group's
  keeper — one person's key, handed on by an act — sequences each request.
  A removed key cannot remove. There is no recovery inside a group
  (`crates/svrngs-core/src/membership.rs`, its module docs the rule):
  - The keeper at the seed is the maker's key, moved only by the keeper's own
    `HandOn` to a standing key. An act that follows its author's own `HandOn`
    by `prev` carries no keeper's authority, so an ex-keeper's backdated
    second hand-on moves nothing. The office isn't the person: a hand-on that
    made a keeper survives its giver's removal.
  - The keeper's `Remove` of a standing key not its own is the cut; so is a
    person's own, their leave (below); anyone else's is an ask, counted as an
    act, cutting nobody.
  - A removal names the removed key's acts by hash, never by position or
    time: `Remove{key, through}` names the removed key's newest acts its
    signer held, and the cut covers them and every act they descend from.
    Any other act of the key counts nowhere (`RailGap::Removed`), wherever
    its timestamp puts it, so it cannot remove, ask or hand on. Which cuts
    count turns on the walk, so the walk runs again until the cuts settle; a
    cut naming an act a copy lacks is `UnheldCut`, an absence; a cut that
    cycles is said unsettled (`Membership::unsettled`), and nothing is
    decided on it. The register folds standing from the acts that move
    membership, every act of a key a `Remove` names, and every `Correct`,
    since one may be a visit's Stop (`svrngs_core::standing_reads`).
  - A `Correct` voids nothing that moves membership.
  - The keeper can equivocate, signing two acts on one parent; that is the
    trust risk 2 names (§8).

  A person removes someone, or asks it, in the command: `svrngs remove
  <name> --group <group>` and `svrngs handover <name> --group <group>` ask
  the node running here (`crates/svrngs-node/src/removal.rs`), which commits
  svrngs-core's `Removal`, a `Remove` of each key standing as that person,
  or `HandOnTo`: removal and hand-on take the one person a name names. From
  anyone but the keeper it is an ask, and the words name who decides. A
  `Removal` the fold would read as the signer's own leave is refused
  (`Own`). The cut is
  told at once to the removed person's computers with what it covers, and
  each removal or hand-on wakes the round, so every other computer holds it
  at once. A cut is told with one selection, wherever it is told
  (`ring_routes.rs` `told_with`): of what the computer lacks, the signer's
  acts through its highest `Remove`, and every `HandOn`, how the fold reads
  a keeper the seed did not name. So no copy holds a cut ahead of an act it
  covers, and copies that hold the same cut agree (#211). From then on each computer answers a removed key only with
  `removed`, "You are no longer in this group", and refuses a pass it
  signed the same; their pages say they are no longer in the group, and a
  synced folder serves them nothing new. **What they already hold stays
  theirs**: removal deletes nothing on their disk and sends nothing back. A
  removed computer that was off as the cut landed is told by the first
  computer it asks to sync, by that selection, and nothing else.
  Everyone in the group sees every removal and every ask to remove — who of
  whom, and when — on both pages, read from the register and written nowhere
  new (`Register::removals`).
- **Leaving a group is a person's own act, and needs nobody's yes.** A
  `Remove` signed by a key standing as the person it removes is the cut, by
  them alone: no new kind of act, one more line of the fold. svrngs-core's
  `Leave` writes a `Remove` of every key standing as the person, the
  signer's own last. The keeper hands on first: their leave is refused while
  anyone else stands (`Keeps`). The last person in a group leaves freely,
  and every copy stays with whoever held it. `svrngs leave <group>`, or the
  home page, commits the leave and tells it at once to the leaver's other
  computers; every other computer standing in the group is told it by the
  leaver's round, each round until it answers. A leave goes with the
  leaver's own acts before it that the computer lacks, and nobody else's
  (the operator, 2026-10-21): each computer is asked its digest, the acts
  its answer carries never taken, then sent what it lacks, one budget at a
  time, until its digest covers the leave; one whose digest stops moving is
  asked again next round, never sent the leave alone.
  From then on, on the leaver's computer, nobody stands in the group
  (`Standing::Left`, for any group whose register cut this computer's own
  key, left or removed): no round, sync or lend reaches it, and the record
  API still reads it. An address there of someone's lend says "You left Trip on
  16 Oct", never "nothing is lent here". Nothing on the leaver's disk is
  deleted. Everyone in the group sees "Ana left on <date>" where removals
  show; whether a cut was a leave is decided once, by the fold (`Cut::left`,
  said as `RemovalSaid::Left` or `Removed`).

  Leaving is final from that computer for now: a computer signs every group
  with one key, and a cut key never stands again. So the issuer refuses to
  redeem a link for a key cut in its group (`RedeemRefusal::Cut`, answered
  410), and a computer that left is told, before anything is redeemed, that
  it cannot join again from there yet (`NotJoined::Left`). A phone, with a
  key per group (`web/store.js`), can rejoin. A key per group on a computer
  waits in PLAN §2.
- **Leaving is cheap.** A group's whole state is a directory every member's
  computer holds. A group that goes wrong refounds from any copy, within
  hours, without whoever it leaves out.
- **A link admits each person once.** A link carries a secret, and the key
  derived from it is the key its issuer's `Invite{key, until}` names, with
  the time it lapses, a day on. Redeeming it writes one `Admit` for one new
  key, signed by the issuer and co-signed by the new key (`Redemption`,
  `crates/svrngs-core/src/lib.rs`). A link admits everyone who opens it,
  each once, until it lapses or its issuer closes it.
  - *Only the issuer redeems*, so one computer decides every use, and a link
    works while its issuer's computer is reachable. The fold counts a
    redeeming `Admit` only under an `Invite` its signer signed, held,
    counted earlier in the order, with no newer `Invite` of the issuer's
    counted before the `Admit`, the `Admit` timed before `until`, and the
    co-signature verifying; otherwise admission names a `RefusedRedemption`
    gap with its `InviteFault`.
  - *A new link closes the old one.* An issuer has one open link per group;
    minting another closes the last from its place in the order, and
    everyone the old link admitted keeps standing. A leaked link is closed
    by sending a new one, with no verb of its own.
  - *The redemption* (`crates/svrngs-node/src/redeem.rs`), on a protocol of
    its own: the joiner, holding only the link, first asks, proving the
    link's key, and the issuer answers the group's id, its seed and the
    `Invite`'s id, the seed believed only when it hashes to the id. The
    joiner then presents a person's name, its new key — the key it dials
    with — that key's consent, and whether a computer or a browser holds
    it, all signed by the link's key; a browser's `Admit` ships at line
    version 4, a computer's keeps version 3's bytes, and one naming neither
    is a computer's. The issuer writes the `Admit` under
    the write lock only when the key does not stand already and no standing
    person goes by the name, so the fold stays the one decider of a lapsed or
    closed link. Every refusal is one sentence naming no group, but a name
    that does not parse, said with why, and a name someone goes by, answered
    `409` with theirs; a key that already stands is answered success and
    nothing is written.
- **A person's name is parsed once, in the core.** Trimmed, 1 to 64 bytes,
  no control characters (`Name::parse`). A `Person` is made only from a
  `Name`, so an `Admit` naming an empty person, 10 KB or a terminal's escape
  codes is a malformed line on any copy, and its key gains no standing. The
  redeem route parses the name before any group is looked at, and the
  command's group and person names are the core's `Name` too.
- **A key per device per group, bound to the person.** Each device keeps its
  own key, since a key shared by a person's devices would fork their chain,
  so a cut in one group touches no other. A person is their first key in the
  group — a seed row's first, or the key a plain `Admit` binds — so two
  people of one name are two people, never merged. A device link, an
  `Invite` naming the person, counts only from a key standing as them, and a
  key it admits is theirs, asked no name (`Membership::person_of`). Of two
  people of one name, svrngs never guesses which is meant: the refusal says
  when each joined (`Membership::joined`, to the second with its offset),
  `--joined` names one, and the home page lists each with when they joined.
  A computer's one key for every group it holds
  (`crates/svrngs/src/identity.rs`) still departs from "per group".
- **Each computer's relay is in the record.** A `Relay{url}` act, signed by
  the key it speaks for, so nobody says where another is reached. Its newest
  counting one wins, one naming no relay withdraws it, and a `Correct` voids
  none (line version 9). The fold keeps each standing key's relay
  (`Membership::relay`), read by the one parse of a relay a link, the record,
  a person or a peer names (`relay::parse`), without the trailing dot iroh
  writes and WebKit refuses. A computer says its home relay at each round's
  start and once a join brings its admission; with relays and none reached yet
  it says nothing, so starting never withdraws it, and a joiner waits for its
  home relay before that exchange (`ONLINE_WAIT`). Every dial of a member
  carries the relay its group names (`Host::contact`); a key held in several
  groups is dialled by the first group naming a relay for it
  (`node::relay_of`). A key with none named is traced, and with no address
  lookup the transport refuses it before iroh is asked, never dialling it
  blind.
- **A visit is folded beside membership** (svrn-docs `GUESTS.md` §3, §5).
  A `Visit{guest, key, opens, until}`, signed by the host's computer, names
  the guest, a key fresh for the visit and the lends it opens, by each
  lender's key (line version 11). It is a visit only when, at its place in
  the walk, its key was never a member's or another visit's, `until` is
  within 48 hours of the act's own time, and with it the group's guests on
  then stay fewer than the people standing and the host's at most two:
  `VISIT_LONGEST` and `GUESTS_PER_HOST`, one constant each in
  `crates/svrngs-core/src/membership.rs`. Otherwise it is no visit on any
  copy, and nothing refuses it. A visit binds nothing, so a guest's key
  never stands. It ends at its `until`, at the host's `Correct` of it,
  their Stop, which another's `Correct` is not, or when the host's key is
  cut; the act stays counted, seen (`Guest::live`). The lender's
  door, on the pass holders' protocol, lets in a pass a visit's key signed
  as the guest, only while the fold says the visit is live and it names
  this lend, which is the host's own, this computer's and never a carried
  app (`door.rs`, `guest_of`; `RingRail::visit_anywhere`). It counts the
  guest as one person against the lend's limit. Each refusal says which of
  those refused it: `ended`, with the end, its time, the host's Stop or the
  host no longer a member; else `closed`. A visit is made by a link in any
  chat: the host's home page asks how long, 48 hours chosen to start and
  nothing longer offered, and makes `<page>#visit=<token>`, a join link's
  token with its secret in the fragment. It is in no `Invite`, since
  redeeming one writes an `Admit`: the host's computer keeps it a day
  (`invite.rs`, `VISIT_LINKS`), and `/visit` on the redeem protocol claims
  it once, judged as an ask is, and writes the `Visit` for the name the
  guest gave and the key their page made, `until` the length from then.
  The guest's page keeps the visit apart from every group (`web/store.js`,
  `keepVisit`), so the link opened again shows it again or says it has
  ended. Every member's home page, and the phone's page through whichever
  computer it reaches (`/people/lends`), lists who is visiting now, by
  host, with until when, from the fold alone (`rail.rs`, `visiting`). The
  host's home page lists their own first, each with Stop, which writes the
  `Correct` as an app's is (`home.rs`, `POST /visits`), only of a visit
  this computer hosts, and draws the page again at the act's own time; the
  phone's page says a guest's Stop is there.
- **What an app places by hash is not kept.** Its bytes are in no act and no
  folder; which hashes a computer carries for which group, and until when,
  is in its running process alone, each kept past its lapse so a get says
  `lapsed`, not `never_held`. A computer stopped before a lapse keeps the
  bytes on disk until it starts again, and its first sweep drops them, since
  a restart carries nothing over: it then says `never_held` of what it
  carried. What lapses is the bytes; a fetched hash's tree of hashes can stay
  on disk, named by the hash (PLAN §2).

## 7. Packaging and reach

Computers first, running the client; phones through the browser for a group
that names a page origin: no install, no store, the same token. Computers
run the node — the `svrngs` process on macOS and Linux — and an always-on
one is the group's meeting point, its port mapped by iroh; relays carry what
cannot go direct. A TV is reached through a member's computer on its network,
never by a page (§3, "The room"). Windows waits in PLAN §2. A svrn daemon on the same computer is an optional neighbour for
local AI; it holds no group state.

**A phone's apps open apart from its page.** The page a group's link opens
holds each person's key, so no lent app's page runs at its origin. On a
phone each app opens at an address of its own, `https://<label>.<page
host>/`, so the page's host must serve the page at every name under it.
`svrngs new --origin <page>` and `svrngs link --origin <page>` check that by
asking a name nobody made, over https, for the page at its root, then print
where the apps open (`crates/svrngs/src/cli.rs`, `checked_page`). A page at
an address no name falls under, an IP address, or on a host that fails is
refused, the reason saying what such a host must do; a page
address that does not give the page is not judged (exit 3). group.svrnme.sh
does (`scripts/deploy-page.sh`), so a group that names ours types nothing
more. The cost: the page's host and the phone's name lookups see which app
is opened, since its label names its lender's key and the app. The host
sends `frame-ancestors 'none'` on every answer (`web/vercel.json`), since a
lent app is a name under its host and could frame the page where a person
says yes; a host a group names in its place must send it too.

**The phone's app adds what the page can't; the page stays the way in.**
Designed, not built: it waits in PLAN §2 until it is resourced, not
before 2027 (the operator, 2026-10-30). The chat link
opens the page in the browser and the person is in, so the app is never
the first ask and never required. It earns its place by four things a page
can't do (svrn-docs `rings/current/ROAD_TO_IOS_GA.md` §2.1):
- it opens a lend in the phone's own app, Jellyfin in Jellyfin, at an
  address a page can't give another app;
- it remembers the person: its key is in the Keychain, where Safari may
  clear the page's (§8, risk 4);
- it goes direct over cellular, where the page is relay-only;
- it keeps a lend playing with the phone locked, where the page's stops at
  the lock (`demo/runs/ios/2026-10-30-observe/README.md` §3).

The app's network extension, an iOS packet tunnel, holds an iroh endpoint
with the person's own key. That key stands as a browser's (`Device::Browser`,
§3 "Joining", call 2): it dials and is never dialled. So the app is no new
kind of device and adds no line to svrngs-core. A tunnel's key admitted as
a computer made its person "did not answer" on the page, since the list of
what members lend dialled it (the same README, "Found on the way").

Each lend the group's people lend is at `127.0.0.1:<port>` on the phone, a
port of its own each, kept so an address saved in another app survives. A
port another app holds is said in the app's words, never moved in silence.
The tunnel stays up by iOS's on-demand rule, so there is nothing to turn on.
The endpoint starts at the first connection to a lend and stops when idle.
Starting it took under 1 s of the 10 s from the switch to a film's first
frame (that README §1), so it is not kept warm. When the phone's link
changes, the door reopens at once, before its silence limit runs out: a
walk off WiFi lost 5.6 s, 4 of them `dial::held()`'s limit (that README §2).

The app's build alone patches iroh and netwatch. It takes the exact crates
from the registry by their checksums and applies the patches at build, so
no vendored iroh is in the tree, and the computers' build stays stock. The
patches:
- netwatch ignores AirDrop's interfaces;
- iroh takes no net report on a link change that keeps the route and
  public address, and reports on the ladder's periodic step, which
  refreshes a direct path every 300 s
  (`demo/runs/phone-replica/2026-10-27/`, its patches and ladder).

The app lives in `phone/`, its own Cargo workspace as `web/` is, since it
holds libc at 0.2.189 for iOS; the Xcode project is under `phone/ios/`.

Its words follow §1's three rules: never VPN, tunnel, key, relay or node,
iOS's own prompt the one place "VPN" appears. It has two screens and a
sheet in iOS's own controls (ROAD_TO_IOS_GA §2.3):
- **House**, the lends;
- **This phone**, what acts as you, with the page's words and Stop;
- **the sheet** before iOS's prompt, the first time a lend is opened or
  copied.

It asks a name and iOS's prompt, nothing else: no account, notifications,
location or contacts. No lend opens inside it (App Review guideline 4.7).
An asleep lender, a lend that asks a yes, a refused link and a taken port
are each said in its words and traced. The house gets it by TestFlight's
internal testers or an ad hoc build, since a VPN app on the store needs an
organization's account (guideline 5.4); the store waits in PLAN §2.

**On a computer** each app opens at `http://<label>.localhost:<port>/` on its
own loopback (`crates/svrngs-node/src/origin.rs`). Its port is the record
API's, taken at the first start and kept in the data dir (`<data dir>/api`,
`svrngs_node::api::kept_address`), so a bookmark and a program's address
survive a restart; another program holding it is said by the command, exit
3, with what to do, never a new port taken in silence. The command,
`install.sh` and the home page say the page's address in one sentence
(`svrngs_node::home::address`). The home page, at
`http://svrngs.localhost:<port>/`, answers only itself — reads from its own
origin, a typed address or a navigation, and writes only with its own exact
`Origin` — so a lent page, same-site but never same-origin, can neither read
it nor say a yes; it sends `frame-ancestors 'none'`. A lend's address in a
browser that holds no session there, a bookmark of a lend it never opened,
sends the whole tab, a `GET` navigation of a document and nothing else, to
the home page's `/to`, which opens the lend as its own links do, by a code
for that origin alone, only with the home page's session, and back at the
path (`origin.rs`, `homeward`). It is the navigation any page could already
make to `/to`, so it grants nothing new; a frame, a fetch or a form there
is refused as before, and a browser svrngs never opened is refused by the
home page in the same words.

**A computer gets svrngs from the group's link.** On a computer the group's
page gives one line to paste in a terminal, `curl -fsSL <page>install.sh |
sh -s -- '<link>'`, composed in the page from the link's `#` part, so the
token reaches no server. The host that serves the page serves the rest
beside it, so any host a group names serves it too: `install.sh` at the
page's address, and under `get/` a tarball of `svrngs` for each platform
built — macOS on Apple silicon and on Intel, Linux on x86_64 and aarch64 as
static musl binaries, cross-built by zig (cargo-zigbuild) on mimalloc, since
the malloc of zig's musl kept what svrngs freed
(`demo/runs/leak/2026-10-17/`) — beside one `SHA256SUMS`. Nothing is
fetched from anywhere else. `install.sh`, POSIX `sh`:
- names the platform and refuses any other;
- fetches the tarball and the sums from the address the link names, and
  refuses a mismatch;
- installs into `~/.local/bin`, never with `sudo`, and puts it on the PATH
  of the person's login shell when that is zsh or bash, once, in the
  profile a new terminal window reads, and says so;
- joins the link as `svrngs join` does, and keeps the page it came from in
  `<data dir>/origin`, so a group made on the home page gets https links;
- keeps svrngs running at login as the person (a LaunchAgent on macOS, a
  `systemd --user` unit on Linux), and opens the home page.

Run again, it updates in place and joins nothing twice. The person is named
as their computer shows them: the full name of the account `$USER` names,
else `$USER`. *A founder's line* has the page's address where the link goes:
it does all of the above but join, and the home page it opens is where they
make their group. With `--relay <name> [--open]` after it, it keeps `svrngs
relay` running instead: a LaunchAgent of its own on macOS; on Linux, where
only the system gives ports 80 and 443, it prints a system unit for whoever
runs the server to install as root, and installs none.

*A computer that keeps apps* gets Give's program from the same line, with
`--give` after the link or the page's address. The page host serves it
beside svrngs, a tarball per platform under `get/` listed in the same
`SHA256SUMS`, built with svrngs's release (`scripts/build-release.sh`), so
the two ship together, a coupling of distribution and not of code.
`install.sh` checks both against the sums before it installs either, puts
`give` beside `svrngs` and keeps `give keep` running at login, waited for
as it stops, since it copies each app it keeps to the group then. Give's
page gives that line, from the page its group's https links are on, ours
where the computer keeps none, to a computer that cannot keep yet, with
the one other install it needs, Podman (`apps/give/page/index.js`).

Publishing the binaries with the page is the operator's
(`scripts/deploy-page.sh`). It reads `cli::SHIPPED` (§3, "Where a computer
meets its groups"): a build at n0's relays it publishes and says so; a build
at svrngs's it refuses to publish while the relay `DEFAULT_RELAY` names does
not answer (`scripts/relay-answers.sh`); a switch it cannot read it refuses.

## 8. Risks

1. **The always-on computer is the group's availability.** When it sleeps
   and no other member is online, the group is unreachable.
2. **The keeper is trusted to sequence removals** and can delay or refuse
   one; the exit is refounding.
3. **A media flagship pulls the infra toward media.** The nouns gate and the
   canaries are the guard. Members lend their own servers to their own group;
   svrngs hosts nothing, and its relays carry only ciphertext.
4. **Phone browsers forget, and seek their own way.** Safari may clear a
   site's storage, and the person's key with it, since its sealed bytes and
   the key that seals them are both in IndexedDB; a web app added to the Home
   Screen keeps its own, apart from Safari's, so the person joins inside it
   (§3, "Joining"), and a lost key means joining again by link. Whether a
   lend opened from a Home Screen web app stays in it, or opens in Safari
   with Safari's storage, is not observed yet. WebKit, every
   iOS browser's engine, asks a closed range to a film's end where Chromium
   asks an open one, so the worker asks the door for either in parts of
   2 MiB (`web/lend.js`, `PART`), and a seek waits at most for the part being
   read. Through a lend, WebKit still reads each short part in serial 64 KiB
   reads, a round trip each, so its seeks back to back on a phone's link can
   miss 5 s where Chromium's do not.
5. **Members find each other through n0, until the operator's switch.**
   iroh's address lookup, n0's by default, maps a key to its relay, never to
   a person, and a group is unreachable through it while it is down; every
   byte a phone moves crosses a relay, n0's under n0's terms. What closes
   both is built and parked: each computer's relay in its group's record,
   svrngs's own relay with no lookup, and a group's own by `svrngs new
   --relay`. Until the switch a computer meets a group that keeps no relay at
   n0's relays and asks n0's lookup, while a phone's page asks it nothing.
   After it, svrngs's relay is the one most groups share — the same centre,
   ours rather than n0's — with `svrngs relay` the way out anyone can take.
   A relay that admits only its group cannot yet carry a phone's lends: the
   page opens each lend from a fresh key holding a pass, which stands in no
   group. And whoever serves a group's page serves the code that holds the
   person's key, its bytes unsealed while the page runs, so a group that does
   not trust svrnme.sh serves the page from its own origin. Likewise a
   lend's own pages can read the pass its page was handed and the pass's
   key, good as the person toward that lend and the ones it reads until the
   pass lapses: a lend is trusted as its pinned code (§3, "Lent apps", call
   6).
6. **Two copies of the record's code.** A signing or canonical-form bug fixed
   on one side can be missed on the other. `scripts/ports.py` is the guard,
   and it is a listing, not a gate.
