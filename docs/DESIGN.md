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
files, Actual for money, Mealie and Grocy for plans, Home Assistant. Run one
in a container on any member's computer, `svrngs share <port> --kind <name>`,
and everyone in the group can open it — no port opened, no tunnel, nothing
for them to install. An app that takes the signed-in person from a request
header lets each of them straight in once its lender names ours,
`X-Svrngs-Person`, in its settings (Navidrome's `ND_EXTAUTH_USERHEADER` and
`ND_EXTAUTH_TRUSTEDSOURCES`, Calibre-Web's "Allow Reverse Proxy
Authentication"); one that keeps its own accounts asks them once; Actual
asks for the one password its lender set. Each serves its pages from its
own root, so on a phone it needs a page host that gives every app an
address of its own (ours does); where every app shares one, none of them
opens. Stock Jellyfin plays and seeks through the door for two people at
once, from a computer and from a phone, each signing in once with an
account its lender made (`demo/runs/u5/2026-10-05-jellyfin-in/`), or with
nothing typed through a program its lender runs in front of it
(`apps/jellyfin/`, `demo/runs/mw/2026-10-08/`); Home
Assistant, Immich, Nextcloud, Actual and Paperless-ngx each do their first
real act from a phone, the rest by a census of their own docs
(`demo/runs/catalogue/2026-10-05/`, which says why Firefly III, BookStack,
Vikunja and Forgejo are not yet named).

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
- the person asking, verified, in a header the door sets, and the app
  asking in another (`x-svrngs-app`), read only from a pass (sup-1): on a
  phone the pass's `app`, the lend whose pages the group's page handed it
  to; on a computer the app its origin names from what the browser says
  of the page ("Lent apps", call 6), carried to another computer's door
  in a pass that computer signs for itself; none for a key of the group's
  asking with no pass;
- the people in the group, each with a public key the app may seal to;
- the record, over a loopback API on the member's own node: append, read, live;
- the services the group's members lend, each by name and an opaque kind the
  app interprets (`jellyfin`), and how many people are using each now;
- on each request through the door, the share of the lender's upload it may
  use right now;
- svrngs's word told from the lend's: every answer svrngs makes in a
  lend's place — the door's refusals, the lend's worker's and the
  computer's — carries `x-svrngs-refused`, naming which (`away`, `pass`,
  `closed`, `unlent`, `path`, `full`, `sign-in`, `fault`, `unasked`,
  `stopped`), and a lend's own answer never does (mnp-1, `door::LendRefusal`);
  on a computer a page asking another lend's origin by `fetch` reads
  svrngs's own answers there when it reads that lend — they allow its
  origin alone, never `*`, and expose `x-svrngs-refused` (lean-4 (b),
  `origin.rs` `readable`; sup-4's follow-up 1) — and a lend's own answer
  only as far as that answer lets its origin; a page that does not read
  the lend, and the home page, are allowed nothing;
- for a lend that asks (`svrngs share --ask "<what an app receives>"`, the
  lender's words, carried in `asks` on every list and never read by
  svrngs), only the app its person said yes to: the lend's door keeps each
  person's yes and stop per app, beside the lend's declaration and never
  in the record, and decides on them (sup-2, `door/choices.rs`). A lend
  asking nobody is carried as before; its own pages are carried whatever
  was said; an app with no yes, or no app named, is refused `unasked`, and
  one stopped `stopped`. Only the person says yes — `POST /choices/<lend>`
  on the members' protocol, `{app, said}` — and anything holding the
  access may say stop: with a pass, on either protocol, the body's type
  has no yes. `GET /choices/<lend>` answers the dialler's own. A stop
  refuses every request after it; one already carried, a live socket
  included, runs to its end. Each choice is keyed by the person the door
  resolves (`door::PersonKey`), today the one key they have, so a yes said
  on a computer holds there, and one said on a phone holds for that phone;
  `svrngs unshare` takes the choices with the lend;
- the home page, `http://svrngs.localhost:<port>/`, where a person says
  their yes on a computer, and sees their groups, the people in each,
  what each lends, opened at its origin, whose computer did not answer and
  what apps use, each with Stop — shown from the routes that answer an app
  (`/v1/groups`, `/v1/lends`, `/v1/rail/people`), groups that cannot be
  read said so (sup-2, `crates/svrngs-node/src/home.rs`). An app sends its
  person there: `/svrngs-reads` gives each lend that asks its words,
  `asks`, `ask`, to which it adds `&back=<its path>`, and `stop`, the
  address at its own origin it posts its own stop to (lean-4 (d)) — on a computer
  `…svrngs.localhost:<port>/ask?lend=<label>&app=<label>`, on a phone
  `<its origin>/svrngs-ask?lend=<label>&app=<label>`, which the lend's
  worker sends to the group's page at `#ask=<lend>&app=<label>&at=<path>`
  (sup-3, `web/lend.js` `asking`). Either page asks "Let Sam’s night use
  Ana’s taste?" around the lender's words, and lists what apps use, in
  one wording both load, a lend named as the bare command names it, with
  its kind when that differs from its name (lean-4 (c),
  `crates/svrngs-node/src/home/question.js`). On a computer a yes is kept by
  this computer's door for its own lend, else posted to the lender's door
  as this computer's key; on a phone the group's page posts it to the
  lender's door, dialled as the person; then the person goes back to the
  app at its path. An app stops itself, `POST` at `stop`,
  `/svrngs-reads/<label>/svrngs-stop` at its own origin; on a phone its
  worker sends that to the lend's door with the pass. The group's page
  lists what apps use too, asked of each lender when the person opens it,
  each yes with Stop. Neither page is ever framed, and neither runs an
  app's code; the home page answers only itself, and its only writes are
  the person's yes and stop; every other act stays in the command;
- reach for the devices in a room: a member's computer serves the lends its
  person names to the network it is on while the command runs, so a device
  there — a TV, through the app's own client for it — opens a lend at an
  address on its own network (below, "The room");
- for a lend whose lender declares a sign-in, each person signed in as
  themselves with nothing typed, by what a program of the lender's answers
  the door on their first request: a credential the door sets, a secret it
  keeps out of every answer, a script first in the lend's pages (below,
  "The room", call 6; mwp-1 and mwp-2 build it).

An app that needs anything else is a decision, not a convenience.

**What an app asks of its own computer** (u5-1), on the loopback API whose
address the node writes to `<data dir>/api` at each start
(`crates/svrngs-node/src/api.rs`, `API_FILE`), so an app it did not start
finds it: `GET /v1/groups` answers each group this computer holds — its id,
the `namespace` every route names, its name, and this computer's person in
it, `null` when its seed or its standing cannot be shown — and `GET
/v1/lends?namespace=<id>` what that group's people lend now, this
computer's own among them, each with its lender's person and key, its name,
kind, limit, how many use it and the upload its lender declares
(`upload_bps`, `null` when none), and the people whose computer did not
answer, named, their lends absent rather than stale
(`crates/svrngs-node/src/surface.rs`), and where this computer's browser
opens each, its `origin` (u5-2, `crates/svrngs-node/src/origin.rs`). Both
are asked now, nothing kept.

**What an app cannot yet ask**, each with the U5 row that adds it (the
seat's survey, `ralph/next/u5/UPSTREAM.md` §(c), every line read again at
u5-0):
- *The lender's own lend, opened* (u5-1 lists it; since u5-2 it opens at
  its origin on this computer through this computer's door's rules, its
  secret injected and its limit counted, one body with the door,
  `door::Who::Own`). Its secret lives only in the door's store, and nothing
  on loopback carries it to the lend.
- *A lend at an origin apart from the person's key* (u5-2, calls 1 and 6
  below). On a phone every lend was under the page's scope, the origin
  that holds the key (§11, risk 5), and on a computer a lender's lends
  shared the loopback port of the bridge to that lender. Since u5-2 each
  lend opens at an origin of its own: on a computer
  `http://<label>.localhost:<port>/` (`crates/svrngs-node/src/origin.rs`),
  which `/v1/lends` answers as each lend's `origin`; on a phone as §7 says,
  its worker (`web/lend.js`) dialling with a pass, never the key.
- *A live socket* (u5-2). The door stripped `Upgrade` as hop-by-hop, so
  Jellyfin's `/socket` answered 404 through it and 101 on the lender's
  computer (`demo/runs/a/dry-2026-10-04/README.md:69-73`); since u5-2 it
  asks the service again and splices the two once it answers 101
  (`door.rs`, `upgrade_of` and `splice`). A service worker sees no
  WebSocket at all, so on a phone a shim in the lend's pages carries it to
  the lend's worker, which speaks its frames over a stream the door
  switches (`web/shim.js`, `web/lend.js`).
- *A read from another origin* (none but svrngs's own refusals: u5-2
  makes the rest unneeded). The door sets no `access-control-*` header
  (`door.rs` has none), so a page on another origin can read neither a
  lend's JSON nor its share header; with each lend at an origin of its
  own, a lent page reads both from its own. A computer's origin allows
  the page of a lend that reads it to read svrngs's own refusals there,
  and nothing else (lean-4 (b), `origin.rs` `readable`).
  A page composing other lends ("Lent apps", call 6) reads them on a
  phone through its own worker, at addresses under its own origin
  (mnp-2), and on a computer, at each lend's own origin, by `fetch`, which
  says where it is from, only the lends it reads (sup-1), and only as far
  as the other app's own answer lets another origin: Jellyfin's did
  (`demo/runs/mn/2026-10-08/computer.json`, `handed`).
- *A secret kept out of response bodies* (u5-3 (c) measures; under sign in
  once, call 4, the lend carries none). The door drops a response header
  holding the lend's secret (`door.rs:438-459`) and reads no body; e0-bot's
  design held that Jellyfin writes its token into playback info and HLS
  playlists (`e0-bot:docs/DESIGN.md:583-586`), and only direct play was
  scanned (`demo/runs/a/dry-2026-10-04/README.md:81-88`).
- *A device per person* (u5-3 (a) and (d)). A lend's secret displaces the
  client's whole `authorization` (`door.rs:113-119`, `:386-412`), and with
  it the client's device: Jellyfin 12 reads its credential from that header
  alone (commonwealth-ai `.canon/sources/notes/invariant/35104a7e.md:3`),
  so through a lend with a secret every person is the lend's one device. A
  lend with no secret leaves each browser's own header.

Three more the survey found were Films', and no U5 row adds them: a queue
for "you're next" (nothing queues, §8, U3's inventory); a live payload's
sender, which a drain does not name (`rail.rs:592-605`; §9); and a phone's
own writes, which reach the record through an app naming the person in
`on_behalf_of` (§8, U3's inventory).

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
copy of the record; it reads and writes through the nodes it dials. A
person admitted at one computer is known at another after a round of the
group's ring; a lender's door meeting their pass before its round asks
the ring once, for at most 10 s, before it answers, so their first
request is answered by every lender awake (mnp-1: 17 ms after joining,
where it was refused 58 times until the door's round, 58.5 s,
`crates/svrngs-node/tests/door/newcomer.rs`). The round dials every
member at once and the door answers as soon as one that knows them has
answered, so a computer asleep in the group holds only its own dial
(asleep-1: with one that never answers, 21 ms, where the round dialled
one by one and the door waited on its end, and refused once at 10.0 s,
as Dee's door refused Ben, `demo/runs/mnp/2026-10-09/`). A newcomer met
while a round already waits on an asleep dial waits for the next round,
up to that dial's ten seconds; read from the code, not run. U4 builds
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
   worker holds one endpoint per group, opened on the first request after it
   starts, and dials each lender from it. Its key is the one the page handed
   it in memory when it opened the group, else one it unseals itself; only
   the page makes the sealing key. A lend's address reached by a worker with
   neither is answered by a hand-over page, which unseals in the page, hands
   the key over and opens the address again (u4-10: Safari's worker on an
   iPhone read no sealing key where its page had kept one). u5-2 moved
   every lend off the page's origin, by the first and sixth of the calls
   below: the page keeps the key and its worker only the shell (`web/sw.js`);
   the endpoint is the lend's worker's, at the lend's own origin
   (`web/lend.js`), bound with the fresh key a pass names, and what was the
   hand-over page is the lend's start page (`web/open.js`), which reads the
   pass its worker cannot and hands it over.)
4. *The phone dials whoever lends a service*, directly by key, since it
   stands in the group. The issuer, the node the link names, only redeems and
   answers what the group's people lend, each with its lender's key. No node
   proxies another's lend.
5. *One body for the link.* The command and the page parse a token and sign
   the ask and the redemption with the same code, in svrngs-core: bytes and
   keys in, bytes out.

**Lent apps, as they come** (operator, 2026-10-05). North star 2 holds for
the apps people already self-host: `svrngs share 8096` lends Jellyfin as it
comes, and someone who never used it opens it from the list, plays and
seeks, on a computer or a phone. U5 builds that by seven calls; the evidence
is U5's inventory (§8).
1. *Lends off the key's origin, each at an origin of its own.* A computer
   serves every lend at an origin of its own on loopback, `<id>.localhost`,
   or a port per lend where a browser does not resolve `*.localhost`. On a
   phone each lend opens at `<id>.<page host>`, its origin's root and
   storage its own, at the price that the host and the phone's DNS see
   which lend is opened; `svrngs new` refuses a page whose host does not
   serve the page at every name under it (group.svrnme.sh does). One
   shared origin let any lent page unseal the key of whoever opened it
   (u4-6, §11 risk 5); no step is added for a person joining or a group
   starting; and a group that does not trust ours, or wants a stricter
   privacy policy, runs on any static host that answers every name under
   it (u5-2). A second way, every lend under `/l/<id>/` of one address
   apart from the page's (the floor), landed with u5-2 and was dropped
   (operator, 2026-10-06; §9, §10): it carried none of the five
   self-hosted apps the catalogue run lent, each asking for its own root.
2. *Jellyfin as it comes.* A lend serves stock jellyfin-web from the
   lender's Jellyfin; nothing in svrngs renders a library of its own. What
   U3's dry run found — a sign-in asked of everyone, and a member's sign-in
   logging out the lend's token (`demo/runs/a/dry-2026-10-04/README.md:35-50`)
   — is what u5-3 measures and u5-4 solves.
3. *The infra learns no Jellyfin.* Anything that knows Jellyfin's API lives
   in `apps/jellyfin/`, whose nouns `scripts/vocab_check.py` keeps out of
   the infra (§5), and what the door gains for it is general: any app.
4. *Sign in once, or none through a program the lender runs.* Lent as it
   comes, the lend carries no secret and each person signs in once with an
   account the lender made: **decided** (u5-3,
   `demo/runs/u5/2026-10-05-jellyfin/`; the operator, 2026-10-05) over zero
   sign-in in the infra, which took four rows, Jellyfin writing the
   person's token into the playback answers it sends, 43 times in each
   engine. Through a lend with no secret, stock jellyfin-web signed
   each person in with an account the lender made, played, and left the
   other alone in every pair; its socket was carried; no byte a person
   received held another's secret. One seek of 56 played again in 7,871 ms
   against Demo A's 5 s (WebKit as an iPhone over n0's relay): it stays a
   miss, and the seek bar is judged on the carriage, as a rate, in u5-9.
   (Landed, u5-4: `svrngs share` asks no password for a kind its
   well-known table says keeps its own accounts, keeps none though one is
   typed, and prints how to make each person one, the table's words for
   it (`crates/svrngs/src/well_known.txt`, `share::accounts_of`). Its
   check, `apps/jellyfin/check/`, has the lender make the accounts in
   Jellyfin's own pages by those words; then two people at once, from a
   computer and a phone in both engines, sign in, play and seek, each left
   alone, no secret in a byte either received
   (`demo/runs/u5/2026-10-05-jellyfin-in/`). One seek of a computer run
   under a load of 33 on 12 cores missed, 6,389 ms, and stays a miss.)
   Zero sign-in is the lender's to choose, outside the infra: a middleware
   ("The room", call 6) in `apps/jellyfin/mw/` makes each person who comes
   a Jellyfin user of their own with a code the lender gives it once
   (upstream's `provision`, forked), writes jellyfin-web's stored sign-in
   into the page it answers, and keeps each person's token out of what
   they receive, putting it back in what they ask. Lent as `svrngs share
   <its port>`, a newcomer played from the list with nothing typed, in
   both engines on a computer and a phone, two people apart and no byte
   holding the lender's key, with no line under `crates/` or `web/`
   changed (the middleware spike, kept, `demo/runs/mw/2026-10-08/`; how
   to run it, `apps/jellyfin/README.md`). Demo A's bar guards that nobody
   who came made an account (`demo/a.md`, `accounts_created`): accounts
   the lender makes before the run keep it, and so do the users the
   lender's middleware makes, one per person (the operator, 2026-10-08).
   (Judged on this host, u5-9: three at once for ten minutes, Alex on a
   computer, Bo and Cy on phones over n0's relay, in two paced runs. Of the
   phones' 88 seeks, 1 missed, 19,293 ms, its first part held about 18 s
   on the way to the door; the computer's 20 all played again. In a run
   before them, with seeks back to back, a WebKit phone stopped asking for
   its converted film after a seek, 21 of 22 seeks: not reproduced, not
   explained (`demo/runs/a/2026-10-05/`). The rate on real phones in three
   homes is the operator's run.)
5. *Measured before built.* What stock jellyfin-web does through the door —
   its sign-in, its live socket, two people's plays through one lend, and
   whether any byte carries a secret the lend holds — is measured (u5-3)
   before u5-4 builds on it.
6. *The person's key never enters a lend's origin.* The page signs a pass
   — a fresh key may act as this person toward these lends until this time
   — for the lend it opens and the lends its lender declared it reads
   (`svrngs share --reads <kind>`); the lend's origin dials with the pass's
   key, and the runtime keeps each lend's cookies apart, since a service
   worker sees no `Cookie`. A lent page can use only its own pass. Never
   the key (u5-2). So a lent page composes others by `reads` (the Movie
   Night spike, `demo/runs/mn/2026-10-08/` §8): a static page searched
   two Jellyfins as each person may see them and opened the chosen copy
   in its own, in 157 lines of its own and 39 of svrngs's. What `reads`
   gives, on a phone: the page's worker asks each lend of that kind as the
   person, and the door signs them in with nothing of the page's (12 asks
   at each door, no line of sign-in) — anything the person may, writes
   included, since a pass names its app, the lends it reads and a lapse,
   and no method or path (`crates/svrngs-core/src/pass.rs:139-156`). So an owner's operation
   beside their data is offered to one app for one person by the lend's
   own program (the scoped-use spike, `demo/runs/su/2026-10-09/` §9): Ana's
   taste ranked the films Tonight's Picks gave it and kept her history
   back, every bar passed in both engines, and the door carried two of its
   checks — the history refused by `--refuse`, the app named — while 46 of
   the taste's 98 lines decided the rest: that the asker is its owner on
   any of her phones, that she said yes to this app, that she has not
   stopped it; each one line from leaking, as its plants showed, and an
   app able to say her yes itself. Each became a row of scoped use made
   production (§8), and run again on them (sup-4,
   `demo/runs/sup/2026-10-10/`) the taste decides only that the asker is
   its owner and its own refusals' words, 11 lines, the app, the yes and
   the stop the door's; every bar passed in its spirit on Ana's computer
   from her home page, in Chromium and WebKit, and on her phones, and the
   other app's own yes was refused by the home page's rule. On a computer
   a page asking another lend's origin by `fetch` reads none of svrngs's
   own answers there, which carry no access-control header, so Tonight's
   Picks could read neither `unasked` nor `stopped` and said it could not
   read whether the taste lets it rank (that run's first follow-up,
   landed in lean-4 (b): svrngs's own answers there now allow a page that
   reads the lend). What
   `reads` did not give, Movie Night found: which
   lends those are, but in svrngs's own store, which the page read whole
   with the pass in it (28 lines; the handoff named no lender and kept
   back who did not answer); a lend opened at a path on a phone that
   never opened it (the page's host's 404, all four people); and on a
   computer, any lend at all. Since mnp-2 a lent page asks, at its own
   origin, `GET /svrngs-reads`: each lend it reads, its kind, name, lender
   by their name in the group and the address to ask it at, and the
   people whose computer did not answer; never the pass or its key. On a
   phone its worker answers from what the group's page handed, each
   address under `/svrngs-reads/<label>/` of the page's own origin; a
   page navigated to a path there is sent to the group's page, which
   signs that lend's pass, and that lend's start page goes to the path,
   so a phone that never opened it opens it at the path, and Back skips
   both (`web/lend.js`, `told` and `opening`). On a computer its origin
   answers from the record, each address the lend's own origin
   (`crates/svrngs-node/src/surface.rs`, `reads`). Since sup-1 the answer
   is also what a page reaches on a computer, one decider
   (`lend::Offered::reads_one`), and the app is named there as on a phone.
   A pass names its app apart from the lends it reads, `Pass { app, reads
   }`, signing the bytes `[app, ...reads]` signed before, and the door
   tells a service the app from it alone. A computer's origin names the
   app from what the browser says of the page asking, which no page's
   script can set (`crates/svrngs-node/src/origin.rs`, `app_of`): a
   navigation, a typed address (`Sec-Fetch-Site: none`) or a request from
   the lend's own origin is that lend's own page; a request from another
   lend's page is that page's, by its `Origin` at this computer's port;
   one from another page with no `Origin` (an `<img>`) is nobody's and
   refused `closed`, so a page wanting another lend's bytes asks by
   `fetch`; one with no `Sec-Fetch-*` is a program's, and the lend's own,
   as loopback is trusted (§9). A page reaches its own lend and those of
   the kinds its lender declared it reads, in a group both lenders and
   this computer stand in, else `closed`; a lender that did not answer is
   `away`, never a lend not read (`kept_to_reads`). The app is carried as
   the door's `Who::Own` to this computer's own lend, and to another's in
   a pass this computer signs for its page, its own key the person and
   the holder, five minutes long and signed for each request, so none is
   kept; that door reads the app from it and refuses one not the
   dialler's own `pass` (`door.rs`, `own_app`). Held by
   `crates/svrngs-node/tests/lend_origin.rs`, every row of the table at
   Alex's own lends and Sam's. Movie Night cut to it
   (`demo/runs/mnp/2026-10-09/`): its `find` 28 lines to 1, each library
   named by its lender and an asleep one's person said; four phones that
   never opened a Jellyfin opened the chosen copy at its item, signed in,
   in 15.8-17.2 s, one Back from it to the same results; the computer
   found both libraries with nothing handed. Two costs it measured
   waited on an asleep lender. The computer's answer asks every lender,
   so its first card waited 10.2 s on Cy's dial; since asleep-1 each
   lender is asked for at most 3 s (`node::ANSWER_WITHIN`) and one that
   has not answered by then is named as not answering, never dropped —
   the reads with a computer that never answers took 10.0 s, and take
   3.0 s (`crates/svrngs-node/tests/surface.rs`, one run each), the
   same bound on `/v1/lends` and a phone's list, one function. And a
   newcomer's first request at a second lender was refused once at about
   10 s, that lender's round dialling the same computer; since asleep-1
   the round dials its members at once and the door is answered by the
   first that knows them ("Joining"). Two others have landed (mnp-1): svrngs's
   refusals marked as its own, `x-svrngs-refused`, where Movie Night told
   them by status alone and said 19 of Dee's door's as the library's (11
   lines); and a newcomer let in by every lender at once ("Joining").
7. *A group's folder, synced both ways onto its members' disks*, beside
   lends and opt-in, a spike on the branch `folders`
   (`demo/runs/sy/2026-10-09/README.md`), a mesh Dropbox: `share --folder
   <dir> --group <group>` keeps a directory the same on the disk of each
   member of that one group (syp-4), and `join` keeps one at
   `~/svrngs/<the group's name>` for a person who joined, said where
   beside the join and on the home page (v1-onboarding (d); cli.rs
   `folder_place`), every joiner's name for it alike, so their folders
   are one; each scan's changes are one act in
   that group's record alone, each naming a path, its
   bytes by hash, and the entry it replaces; an entry no other replaces is a
   head, a path's second head written beside it named as a conflict with
   its writer's person. The bytes move by iroh-blobs: a member's node
   serves a hash only while a synced folder of a group the dialer stands
   in places it, and refuses every other, and fetches what it lacks by
   verified range from the group's members, a holder that does not answer
   within 3 s named and passed over. Who stands where is the round's
   (`Host::standings`), and a group whose record cannot be read stops its
   directory's step by the group's name, never read as placing nothing. The index is the record, not iroh-docs, whose
   namespace id would be a second decider of who reads. Run again on
   production (syp-5, `demo/runs/syp/2026-10-10/rerun/`, one host), all
   four of the spike's bars pass: an add, an edit and a delete by three
   members everywhere within 0.32 s (the spike's 1.63); a file reached Cy
   in 3.56 s from Ben with its author killed; two edits made apart both
   kept in every directory within 0.34-0.65 s, three runs of three (syp-6,
   `demo/runs/syp/2026-10-11/restart/`; the spike's 17.0 s); a node
   outside the group given no byte, refused at the accept and by the
   provider each. The edits had failed at syp-5, three of three, on two
   holes (that README §6), closed by syp-6: a node stopped by SIGTERM now
   closes its endpoint, where it had left each peer's iroh sending only to
   its dead address for up to 15 s, so its next process was not dialled;
   and a file no member gave is asked again with no event to wake it. A
   process killed outright still leaves that dead address for up to 15 s,
   the retry's cap. A thousand files reached two members in 0.92 s and one
   joining later in 0.53 s (the spike's 395.27 and 242.24), and a 1 MiB
   edit of a 100 MiB file still cost each member the whole file, in 7.04
   s. Off one host it is unrun. Its rows are the
   README's §9, made production by syp-4: the folder one group's (landed,
   Ana in two groups each with a folder `Shared`, the other's member
   refused her hash); a path parsed, since a member's act
   could write outside the folder on every other member's disk (landed: a
   `SyncPath` of named parts, an act naming `../outside` a malformed line
   on every copy); the
   store keeping what it held across a stop, reproduced first, since three
   times a store gave nothing of a file its directory held (landed: a
   fetched file's bytes lost at SIGTERM and `kill -9` alike, the store's
   writes committed in batches up to 500 ms apart, now made durable before
   the kept state names them); what a node keeps of its syncing, never an
   unreadable state read as empty (landed: the directories it syncs kept
   and resumed at each start, one directory each, and a kept state that
   cannot be read stopping its directory by name); the store keeping what
   the layout places and no more (landed: every other version dropped when
   what it keeps changes, M2's first version gone from each store 0.89 s
   after the edit); and a holder sending wrong bytes, planted (landed: a
   byte flipped on the wire refused by the fetch naming the holder, and
   who is taken as asleep decided once, the fetch's 60 s `ASLEEP_FOR`).
   Made fast by perf-2 (`demo/runs/perf/2026-10-10-make-it-fast/`): a
   file in one get, the record's layout read once a change, the apply woken
   by the record, files taken 8 at once and fetched 8 at once, and the step
   woken by the OS's file events (`notify`), the whole walk a sweep each
   ten minutes; a file written as it is taken left for its write's next
   event, found by that run. A file no member gave is asked again, a
   second after and doubling to 15 s, with no event to wake it, and
   nothing woken so while every file is written (syp-6 a). A folder streamed by
   range, group folders' second spike (`demo/runs/gf/2026-10-09/README.md`:
   a one-way share, one Jellyfin playing `.strm` stubs at a loopback
   endpoint on the meeting point, seven bars passed on one host), was
   dropped with its endpoint and its share at syp-1 (operator, 2026-10-10,
   "Drop streaming + share"): watching a member's films stays their lent
   Jellyfin's, and a folder fetched on open waits in §9.

**The room: the screen is the app's own client** (operator, 2026-10-05,
after five reads on a real TV, `demo/runs/u6/2026-10-05-screen/reads.json`:
"we're adding value, but we're not taking media on tvs and end user
devices into our scope"). A film from a library across town plays on the
TV in the room, and svrngs gives the room reach and nothing more. U6 builds
that by five calls, and a sixth waits; the evidence is U6's inventory (§8).
1. *The screen is the app's own client; svrngs gives it reach.* A TV plays
   a lend through the app's own client for it — "Jellyfin for WebOS" 1.2.2
   signed in and played by DirectPlay from a LAN address (the reads,
   `jellyfin-webos-app`) — pointed at an address on the TV's network; a
   phone as the remote is the app's own (Jellyfin's "Play on"). Nothing in
   svrngs knows AirPlay, Cast, DLNA, a TV, a player or what a lend serves.
   AirPlay from a phone's browser showed no video three ways — the page's
   worker-served address; jellyfin-web, whose hls.js sets
   `disableRemotePlayback`; a plain LAN file the TV never fetched — and old
   TV browsers are not supported: the LG's Chrome 87 could not open the
   page (operator: "a losing cause"). AirPlay and Cast from a page wait in
   §9.
2. *A computer in the room.* The address comes from a member's computer on
   the screen's network (operator, 2026-10-05): not from a phone, since a
   browser cannot listen, and not from a public address per lend (§9). Its
   hop to the library is computer to computer over iroh, which can go
   direct, where a browser's is relay-only (8.1 Mbit/s through n0's relay
   in the reads, `airplay-page`).
3. *The room door.* A member's computer serves the lends its person names
   to its own network while the command runs: each at a port of its own on
   this computer's private address, the lend's root its own as each lend's
   origin on loopback is (§7), in one body with those origins, not a
   second; reached as this computer's person — dialled under this
   computer's key, the way this computer's own browser reaches the lend.
   Call 4 of "Joining" holds: the screen is this person's own device, not
   another person's dial, and no node proxies a lend for anyone else. It
   carries what the origins carry: Range, both bodies streaming, a
   WebSocket — the app's remote rides its socket. The address is picked as
   upstream's `advertised_base` picks one: the private LAN before
   tailscale, never loopback, a wildcard or a public address, and absent,
   said so, when there is none. Anyone on that network reaches the named
   lends as this person while it runs, which the command says when it
   starts, in §1's words; a request from off this network is refused. The
   lender's door counts the room as one person and guesses its path as it
   does now.
4. *Measured, then told.* What Jellyfin's own client for a screen needs
   through the room door is measured (u6-2) before any of it is written:
   the server address typed once; sign-in without typing on a remote
   (Jellyfin's Quick Connect: the TV shows a code a signed-in phone
   approves); a newcomer's phone, signed in as another Jellyfin user,
   playing on the TV's session (a permission the lender sets); and whether
   the client keeps to the room address or moves to the address the
   lender's Jellyfin reports for itself (`/System/Info/Public`
   `LocalAddress`, which is across town). What the lender sets is written
   in `apps/jellyfin/`, never in the infra; anything the door would need is
   general — any app — or it is a finding for the operator.
   (Measured, u6-2, jellyfin-web 12.0.0 in a browser standing in for the
   TV, `demo/runs/u6/2026-10-06-jellyfin-screen/`: the room door needs
   nothing it does not do. Typed the room address once, the screen signed
   in, played by DirectPlay and seeked, nothing reaching Sam's Jellyfin
   but through the room door; Quick Connect, on as Jellyfin comes, signed
   it in with nothing typed; it kept to the room address, opened again and
   across the room door stopping, though Jellyfin reports itself at Sam's
   loopback. A newcomer's phone starts a film on it once the lender ticks
   "Allow remote control of other users" for them, then loses it — no
   pause, no seek — as it does with no svrngs anywhere, where the phone of
   the TV's own user keeps it. What the lender sets is
   `apps/jellyfin/ROOM.md`.)
5. *Demo B as it now runs* (operator, 2026-10-05: the library is across
   town; the people in the room are on the TV's WiFi): five friends at
   Dee's, Sam's library across town, Dee's computer running the room door
   and Dee's TV Jellyfin's own app, each set up once by Dee before the
   evening (§8). The gate stands as it was. (Dry run, u6-9, on one
   computer, `demo/runs/b/2026-10-06/`: five browser profiles as phones,
   one at a time, from the link to their film on a TV stand-in in
   25–39 s, the TV reaching Sam's Jellyfin only through the room door;
   whether five people find the steps is the operator's run.)
6. *Middleware* (operator, 2026-10-06; its spike kept, 2026-10-08): a
   lent app that forwards to another — an ordinary program on the lender's
   computer that listens on a port, takes the door's verified request and
   passes it to the service behind it, so `svrngs share <its port>` lends
   the pair. Only the lender runs one, on their own computer; svrngs ships
   no runtime or SDK for it, and never starts or supervises one. The
   door's guarantees stay outside it (identity set, the limit and
   refused paths checked, an echoed secret stripped); its contract is
   HTTP, the door's `x-svrngs-*` headers and the loopback record API; an
   app's nouns live in middleware, never the infra. Its spike, after Demo
   B, did Jellyfin's zero sign-in so ("Lent apps", call 4) with no diff
   under `crates/` or `web/` and no header the door does not already set:
   `x-svrngs-key` and `x-svrngs-person` name the person, the client's own
   `authorization` its device (`demo/runs/mw/2026-10-08/`, its rule
   judged by its spirit, §6, and its follow-ups, §7). Past the door, the
   middleware's port believes loopback: a process on the lender's computer
   naming a person's key there is answered as them, and is the lender's
   own (§9's per-process tie keeps its trigger).
   *Made production* (operator, 2026-10-08): the spike's middleware was
   1,039 lines of code (`apps/jellyfin/mw/src` at `3c5dbae`, counted as
   bar 2 below counts; b764b6a's count was 1,043), about 600 of them any
   app's — a second proxy beside the door, streaming and a WebSocket,
   whose list of hop headers already disagreed with the door's
   (`ralph/REVIEW_FINDINGS.md` #82); a credential per person, made on
   their first request; a secret swapped for a stand-in in what the
   service answers and back in what it is asked; a script put first in a
   page — and the library carried none of it. So the door carries those
   three, general, as one hook a lend declares over HTTP, its *sign-in*,
   and an app's middleware keeps only its own nouns, as the sign-in's
   answerer beside the service rather than a forward in front of it: for
   Jellyfin, making a user, its credential's form and what its page
   stores. The contract (mwp-0; `ralph/next/mwp/STATE.md`'s rows build
   it), each path:line in it as at `252bf22`, which wrote it — mwp-1 moved
   `door.rs`'s lines and mwp-3 deleted `swap.rs` and `forward.rs`:
   - *Declared by `share`.* `svrngs share <port> --sign-in <its port>`
     names, beside `--kind` (`crates/svrngs/src/cli.rs:110-125`), a
     program on this computer that answers for the lend who each person
     is to it. A port alone, as the lend is (`share.rs:12-14`): the door
     asks `POST /` at `127.0.0.1:<its port>`, and `share` refuses it when
     nothing listens there, as it refuses a lend's own port. It is kept in
     the lend's declaration (`lend.rs:250`, `LendFile`), not with its
     secret, since a port is none; a node that does not know the field
     refuses the declaration (`deny_unknown_fields`), never ignores it. A
     lend declares its one secret or a sign-in, never both: given
     `--sign-in`, `share` asks no password, says each person is signed in
     by that port's answer with nothing typed, and prints no kind's words
     for making accounts — the spike's follow-up 1 (its README §7),
     answered without
     `share` learning middleware. A lend that declares no sign-in keeps
     its one secret (`declared.rs`; `door.rs:497-516`, `secret_headers`,
     displacing the client's header in `rewrite_headers`, `:468`) and its
     door byte for byte, held by an existing door test unchanged.
   - *What the door sends.* On a person's first request through the
     lend, before carrying it, the door asks the sign-in with that
     request's identity headers, `x-svrngs-person` and `x-svrngs-key`
     (`door.rs:103-111`), set as it sets them for the service, and
     `x-svrngs-stand-in`, the stand-in (below), so an answer's script
     stores it as the door names it and no app spells it (#90, si-1);
     nothing else: no client header, no body. One owner the door holds,
     as it holds `Using` (`door.rs:187`), asks for each person apart, so
     one person's slow sign-in holds nobody else's request, and a person's
     first requests at once wait on one ask (#88, si-1: until then it
     asked one person at a time, and one slow ask held every lend's).
   - *What comes back.* `200` and a JSON object, every field required and
     an unknown one refused: `secret`, the person's secret, 16 to 512
     bytes of letters, digits, `-`, `_`, `.` and `~` — what a query, a
     header and a JSON string carry unescaped, so one search finds it in
     each; `headers`, headers set on each of the person's requests,
     displacing the client's of the same name as the one secret does,
     none under `x-svrngs-`, none hop-by-hop, no `host`, `{}` for none;
     and `script`, JavaScript put first in the lend's pages to that person
     (below), or `null`. Anything else — another status, no answer in
     30 s, a field missing or malformed, a script holding the secret or
     `</script` — refuses the person's request, `502`, `{"error": "<lend>
     could not sign you in", "kind": "sign-in"}`, the reason traced at
     warn on the lender's side, never sent to the person; the next
     request asks again. Never forwarded unsigned.
   - *What it keeps, and when it asks again.* Each person's answer, by
     key, in the door's memory for this run of the node: never logged,
     never in the record, never on the wire but to the service, as
     `declared.rs` keeps the one secret; not on disk, since asking again
     after a restart costs one ask a person, and a file would hold every
     person's credential at rest. It asks again after a restart, and when
     the service answers `401` to a request signed in with a kept answer:
     the door forgets it and passes the `401` on, and the next request
     asks. Measured: Jellyfin answers 401 to a token a reset of its
     user's password revoked (`demo/runs/mw/2026-10-08/mw-1/README.md:28-33`),
     which an answerer restarting does (that run's §7, follow-up 4).
   - *The swap.* One stand-in for every lend and person,
     `svrngs-stand-in`, the spike's (`apps/jellyfin/mw/src/swap.rs:16`),
     spelled once, in the door (`door/sign_in.rs`, `STAND_IN`), and named
     to the answerer in each ask:
     it names nobody, since the door puts in the secret of whoever it let
     in, never by the stand-in; one per lend would hide nothing and add
     a store; one an answer chose could be ordinary text or hold the
     secret, checks no app has asked for (the trigger: an app whose
     client reads its token's form, as a JWT's expiry). Inward, the
     stand-in in a request's query and in each header's value becomes the
     asker's secret — the spike set the two query names it knew
     (`swap.rs:21`), the door knows none — and a path and a request's
     body go as sent. Outward, every secret the door holds for the lend,
     the asker's and any other person's the service writes, becomes the
     stand-in in every textual answer, header and body:
     `drop_echoed_secret` (`door.rs:574-595`) swapping where it dropped,
     a header that cannot hold the stand-in still dropped; a body
     streamed, a secret split across two chunks found by holding back one
     byte fewer than the longest secret; the answer's length dropped,
     since the stand-in's is not the secret's. Textual by `Content-Type`:
     `text/*`, JSON, XML, a playlist (`*mpegurl`), JavaScript, and none
     named, read rather than presumed bytes (the spike skipped scripts and
     styles, `swap.rs:63-76`, knowing Jellyfin's). Every request through a
     lend with a sign-in goes without its `Accept-Encoding`, as the
     spike's did (`forward.rs:188`), and a textual answer compressed
     anyway is refused, `502`, never sent on unread. A socket's frames
     are carried as the door carries them: mw-2 found no token in one
     (`demo/runs/mw/2026-10-08/README.md` §6, bar 2).
   - *The script*, put by the door first in `<head>` of every `200`
     `text/html` answer to a `GET` of that person's, as the spike did
     (`forward.rs:252`), since a `204` or `304` has no page to put it in — after the first `<head…>`, else
     `<html…>`, else `<!doctype…>`, else at the start, as `web/lend.js:338-340`
     puts its shim, read in the answer's first 64 KiB and the rest
     streamed — so it runs before every script of the page's own. In the
     door, not at the lend's origin: the door is the one place a
     computer's way (`origin.rs`, through `door::lend_for` as `Who::Own`
     or across the dial), a phone's (the pass holders' protocol) and the
     room's (`origin.rs`, `reach`) all pass. On a phone the lend's worker
     then puts its shim after `<head…>` too, so the shim runs first and
     the script second, both before the page's own — mw-2's order, the
     middleware's script where the door's will be. Under a
     `Content-Security-Policy` naming a nonce the script carries it, as
     lend.js's shim does (`:250-257`), which the conformance canary's page
     asks (`apps/web-conformance/serve.mjs:150-154`, `script-src
     'nonce-…' 'strict-dynamic'`); under a policy that runs no inline
     script, by nonce or `'unsafe-inline'`, it does not run, traced at
     warn and not worked around, and a policy in a `<meta>` is not read.
     Such a page goes with `Cache-Control: no-store`, its `ETag` and
     `Last-Modified` dropped, and a request whose `Accept` names
     `text/html` goes without `If-None-Match` and `If-Modified-Since`, so
     no copy kept from before the script, or for another, is answered
     (the spike's, `forward.rs:189-192`, `:280-284`).
   - *What Jellyfin's middleware becomes* (mwp-3): the sign-in's answerer
     and nothing else. Asked, it finds or makes the person's user
     (`apps/jellyfin/mw/src/viewer.rs:220`, `provision`, kept), signs it
     in on a device of the person's own, named from their key (`:353`,
     `sign_in`), and answers the token as `secret`; `authorization`,
     `MediaBrowser Client=…, Device=…, DeviceId=…, Version=…,
     Token="<token>"`, in `headers`; and page.rs's script (`:60-71`),
     writing jellyfin-web's stored sign-in with the stand-in for its
     token. The lend is Jellyfin's own port: `svrngs share 8096 --sign-in
     <the answerer's port>`. Deleted: forward.rs, its proxy and its hop
     list (#82 closes with it); person.rs's identity parse and its device
     read from the client, but for reading the two headers; swap.rs;
     page.rs's `written`; people.rs's `On::Last`, which the stand-in in
     the socket's query now does. The cost: one person in two browsers is
     one device to Jellyfin, where the spike kept each client's, since a
     token's session stays the device it was signed in under
     (`mw-1/README.md:14-26`). Landed (mwp-3): the script also names that
     device as jellyfin-web's own (`_deviceId2`), so its socket names it;
     mw-2's four pairs and three plants passed and were caught as before,
     and the middleware is 669 lines by bar 2's count, past its prediction
     — none of them the door's pieces (`demo/runs/mwp/2026-10-08/README.md`).
   - *The inventory* (AGENTS.md 11), upstream read at commonwealth-ai
     `e93aa6881` (HEAD, 2026-10-08); `git diff d4356181c e93aa6881` over
     each file below prints nothing, so the spike's inventory stands
     (`demo/runs/mw/2026-10-08/README.md` §1). `mesh_media/viewer.rs`'s
     `provision`: forked already, the answerer's. host-kit's
     `shell/files.rs:50-58` puts a `<script src>` before `</head>`, last:
     not forked, since the door's must run ahead of a page's own scripts
     that are not `defer`, so its rule is lend.js's, and mwp-2's test
     gives the door lend.js's cases. `iroh_identity_forward.rs`: the door
     already (§8, U1's inventory), its rewrite carrying a person's headers
     where it carries the lend's one secret. `iroh_routed_forward.rs:247`
     (`mint_tie`) and `:260` (`tied_pubkey`): not forked; the answerer
     believes loopback as the middleware did, so a process on the lender's
     computer asking it gets a person's credential, and is the lender's
     own (§9 keeps the tie's trigger). Nothing upstream swaps a secret in
     a body (`git grep -n -i "mask\|redact" e93aa6881` over
     `commonwealth-transport`, `commonwealth-media` and
     `sovereign-cli-mesh` finds only `iroh.rs:58`, a proxy's address
     redacted for a log) or asks a program for a person's credential:
     both are written new.
   - *The bars*, each with its prediction, written before any product
     commit of the queue; one host, one run of each, not a rate
     (AGENTS.md 7), and a run that could not finish is could-not-judge:
     1. *Jellyfin through the sign-in, the proxy gone*: mw-2's four pairs,
        a computer and a phone each in Chromium and WebKit, and its three
        plants (`demo/runs/mw/2026-10-08/mw-2/README.md`): nothing typed,
        every seek playing again within 5 s, no byte holding the lender's
        key, either person's password or the other's token, the two
        apart; each plant — no stored sign-in, the lender's key for the
        page's token, one account on one device — caught. Predicted: all
        pass, one phone seek in WebKit perhaps missing, as the spike
        predicted (its README §4); and each person's own token in no byte
        either.
     2. *The middleware is Jellyfin's part*: its code lines, `awk
        'FNR==1{t=0} /^[[:space:]]*#\[cfg\(test\)\]/{t=1} !t &&
        !/^[[:space:]]*($|\/\/|\/\*|\*)/' apps/jellyfin/mw/src/*.rs | wc
        -l`, 1,039 at `3c5dbae`. Predicted: about 430, between 380 and 480
        — viewer.rs's 241, the answerer's listener and answer about 90,
        the script about 45, each person's user kept about 45, the two
        headers read about 10. Over 480, the door carries less than this
        contract says.
     3. *svrngs's new lines inside the raise*: `scripts/lean_check.py`,
        13,970 of 14,400 before the queue. Predicted: about 320, mwp-1
        about 250 and mwp-2 about 70, between the seat's 270 and 400;
        past 430 is the operator's.
     4. *A second app on the same sign-in* (mwp-4): the conformance
        canary, given accounts of its own — a token per person, read from
        a bearer `authorization` and written back in a body, a redirect
        and a query it answers — and an answerer of its own beside it,
        run by `scripts/check.sh`, so every build keeps the sign-in
        general rather than one run, with no service this host lacks. A
        stranger's app from the catalogue run is the harder proof; its
        trigger is a sign-in the canary passes and a stranger's app
        cannot use. Predicted: its answerer about 60 lines, and `git diff`
        of `crates/` and `web/` in mwp-4 empty.
     5. *No app noun in the infra*: `scripts/vocab_check.py` at every
        commit of the queue. Predicted: passes; the door's new words are
        sign-in, secret, stand-in, script and header.
   - *Landed* (mwp-1 to mwp-4, 2026-10-08), judged by each bar's spirit;
     one host, one run of each. Bar 1: mw-2's four pairs passed and its
     three plants were caught, the proxy gone, the slowest seek 4,008 ms,
     each person's own token in no byte either
     (`demo/runs/mwp/2026-10-08/README.md`). Bar 2: 669 lines, past 480
     by 189, none of them the door's pieces — the checks' five plants
     (about 55), the credential's `Device` form (about 35, not
     predicted), and refusals and traces as rustfmt breaks them (the
     same README). Bar 3: 14,398 of 14,400, 428 new — mwp-1 375 (its
     refusals and traces), mwp-2 51, mwp-3's `run.sh` 2 — past the 320
     predicted by 108, inside the raise. Bar 4: the canary's answerer
     (`apps/web-conformance/answer.mjs`) is 30 lines and its accounts 32
     more, 22 in `serve.mjs` and the page's check 10 in `checks.js`
     (`git diff 673db84 edb4072`, lines of code); Alex's phone and Bo's
     computer, each in Chromium and WebKit, are each signed in as
     themselves with nothing typed — by the token the sign-in's script stored, sent as a query,
     by the door's bearer and through a redirect — and no byte either
     receives holds either token or the lender's key; its plant, one
     person's account answered for both, caught by both of its asserts
     (`a9ac3e1`). No line of `crates/`'s sources or of `web/` changed in
     mwp-4; its test is beside the canary's others, in
     `crates/svrngs-node/tests/conformance.rs`, the one harness that runs
     it on this host alone, so the bar's letter (`crates/` empty) missed
     by that test (`30dac6a`). Bar 5: clean at every commit. The catalogue
     app, the harder proof, keeps its trigger above.

**A chat's door is the group's link** (operator, 2026-10-10: "Hopefully not
more the 50 lines in our own source and a pretty commodity integration"). A
person posts the link where they talk, as they post any link, and it opens
the same from Discord, iMessage, WhatsApp, Slack or Signal. svrngs's part is
the link previewing as an invitation — a title and a description on the
group's page — with the token in the link's `#` part, which no chat's
preview fetches, so the preview names no group and carries no token. No
package per chat, no bot, no commands: E0's Discord door, `discord.js` on
Discord's gateway with a bot token, slash commands and a guest book keyed by
Discord user id, 517 code lines (`bridge/*.mjs` and `crates/svrngs/src/bridge.rs`
at tag `e0-bot`), is the failure this answers; a chat's own door waits in
§9. The core never knows a chat exists.

## 4. Domains, and the fork

| Domain | What it owns | Meets svrngs by |
|---|---|---|
| **svrngs** | the record, the node, the doors, the command, plain words, reference apps, demos | — |
| **chats** | nothing of ours: a person posts the group's link where they talk (§3, "A chat's door is the group's link") | the link |
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
| `svrngs-core` | the record: acts, admission into one order, membership, the sync digest; and what a browser's page spells as the command and the node do (u4-2): a link's token, the ask and the redemption a joiner signs with it, the issuer's answer and the seed it is believed by (u4-5), the protocols' names, and a relay's URL without the DNS root's trailing dot, which WebKit refuses (u4-9); a lend's label, one DNS label naming its lender's key and its name, and the pass a person signs for another key toward named lends until a time, with its check (u5-2). `no_std`, so the compiler refuses filesystem, clock, network and threads; time and keys are arguments | none |
| `svrngs-runtime` | the only owner of each group's journal on disk, its projections and its writes: the journal and its write index, the open-journal map, the register's standing | the journals |
| `svrngs-node` | the adapters: sync and its route over iroh, redemption, the loopback record API with its live lane (landed, u3-2 and u3-3), the member door (landed, u3-4), the list of what members lend, asked of each (landed, u3-5), each for at most 3 s (landed, asleep-1, `node::ANSWER_WITHIN`), and answered to a member by the computer it dials, each with its lender's key (landed, u4-5), and the door's limit and shares, a pure decision the door's task owns (landed, u3-6); a WebSocket's upgrade carried, and the pass holders' protocol, which any key may speak, where a request presenting a person's pass for the lend it names, by name or label, is that person's (landed, u5-2), and its service told the pass's first lend, the app the page opened, as `x-svrngs-app`, none with no pass (the scoped-use spike's one header, su-1, `demo/runs/su/2026-10-09/` §5); every relay URL it makes or reads dotless where it enters — n0's relays, a relay it is told, a lookup's answers — in `relay.rs`, which the page compiles too (landed, u4-9); the room door, the lends a person names served to this computer's network at its private address while the command runs, one body with the origins (landed, u6-1); a lend's sign-in, asked of the lender's program on a person's first request, its credential set on each of their requests, a stand-in swapped for its secret both ways (landed, mwp-1, `door/sign_in.rs`), its script first in the lend's pages (landed, mwp-2, the same file; §3, "The room", call 6); every answer it makes in a lend's place marked as its own, `x-svrngs-refused` and its kind, a service's own dropped (landed, mnp-1, `door::LendRefusal`), and a pass whose signer it does not know asked of the group's ring once, within 10 s, until a member that knows them answers, before it is refused (landed, mnp-1, `ring_sync::AskRound`; the members dialled at once, asleep-1); a lend's pages on this computer told, at its origin's `/svrngs-reads`, the lends they read, from the record (landed, mnp-2, `surface::reads`); each person's yes and stop to an app kept by the lend's door and decided there, by the person the door resolves (landed, sup-2, `door/choices.rs`, `door::PersonKey`); the home page at `svrngs.localhost`, answering only itself, never framed, its writes the person's yes and stop alone (landed, sup-2, `home.rs`) | each lend's declaration and secret, and each person's choices for it; who is using each lend now; each person's sign-in answer, in memory |
| `svrngs-web` | the page's dial, wasm only, in `web/` with its static shell and a workspace of its own: a browser's endpoint, bound with the person's key, dialling a group's computer by key over a relay on the members' protocol (landed, u4-3); HTTP over the dial (landed, u4-4); joining — the ask and the redemption by svrngs-core's bodies, the key sealed in the page's store, and the list of what the group's people lend (landed, u4-5); the runtime, a service worker serving every request at a lend's address from the lender's door (landed, u4-6), a range that runs on — open-ended, or a `<video>`'s longer than a part — asked of the door in parts of 2 MiB (landed, u4-9), an open-ended one's first part at a new place 256 KiB and doubling as the reading runs on (u5-11), handed what it dials with in memory, and a page for a worker that holds nothing (landed, u4-10); since u5-2 at each lend's own origin, not the page's (`lend.js`, its start page `open.js`): the page signs a pass to a fresh key for the lend it opens and the lends it reads, the worker dials on the pass holders' protocol, keeps each lend's cookies, reads a redirect to a path under the lend, and with a shim in each lent page (`shim.js`) carries `WebSocket` and an app's own worker; its own answers in a lend's place marked as the door marks its own (mnp-1); its pages told the lends they read at `/svrngs-reads`, each at an address under their own origin that opens that lend at a path through the group's page (landed, mnp-2); a lend that asks told with its lender's words and an `ask` address that opens the group's page at `#ask=`, where the person's yes is posted to the lender's door as them, an app's stop sent by the worker with the pass, and what apps use listed there with Stop, the page never framed on its host (landed, sup-3); an adapter | none: the page's store holds the key, sealed; a lend's origin's store holds its pass and its cookies |
| `svrngs` | the command and the process: `new`, `join`, `link`, `share`, `unshare` and `room` (landed), and `share --sign-in` (landed, mwp-1), and `share --ask "<words>"`, its lender's words for what an app receives, carried in `asks` on every list of lends and never read by svrngs (landed, sup-2); the bare command prints where the record API listens, where the home page opens (sup-2), and what the people of its groups lend | a device's keys; each group's seed |

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
serves Jellyfin and breaks one fails the build.

**Lean, and kept lean.** `svrngs-core` stays under 3,000 lines of code and the
repository under 15,350 (`scripts/lean_check.py`), the branch `folders`
too since main merged into it at syp-5, counting 13,778 and `svrngs-core`
2,719 there; a raise is the operator's, with the reason in the commit. The budget counts the product:
tests, `apps/` and `demo/runs/` apart. A recorded run's driver is what ran,
kept byte for byte so its data reads against it (`ralph/REVIEW_FINDINGS.md`
#61), and no check runs one; a helper a check runs lives outside it
(`web/tests/serve.mjs`, `web/tests/control.mjs`), and code elsewhere under
`demo/` counts. The operator set `demo/runs/` apart on 2026-10-10 (lean-1,
"Budget counts product only"): the repository went from 15,284 to 13,428,
the cap kept; main merged into the branch `folders` at syp-3 counted
13,372, `svrngs-core` 2,729, inside both caps. Every figure below counted
`demo/runs/`, as the rule then did.
`svrngs-core` stood at 2,359 when the operator raised its ceiling from 2,500
on 2026-10-05, for what U4
to U7 still add to the core, since REVIEW-audit-u3-auto-2 found no duplication
in it (ralph/REVIEW_FINDINGS.md #38), and the repository's from 12,000, at
9,601, for U4's page and its wasm and U6's handoff, about 2,100 lines between
them by the seat's estimate; and the repository's from 14,000, at 13,970,
on 2026-10-08, for the door to carry what the middleware spike wrote twice
(§3, "The room", call 6): of its 1,050 lines of code, about 600 are any
app's, less the 330 the door and the command already have, about 270
new, and 400 with the door's traces and refusals (the seat's estimate);
and from 14,400, at 14,398, on 2026-10-08, the sign-in having taken 426
against that 400: for its decision in one place and one person's sign-in
never holding another's (`ralph/REVIEW_FINDINGS.md` #87, #88), and the
one header the scoped-use spike may add (§8, "Then"); and from 14,500,
at 14,466, on 2026-10-08, for Movie Night made production (§8, "Then";
`demo/runs/mn/2026-10-08/README.md` §9): svrngs's refusals named as its
own (~30), a newcomer let in by every lender at once (~50), a lent page
told the lends it reads on a phone and a computer (~75), a lend opened
at a path (~25), and ~70 for the traces and refusals they carry, 250
by the seat's estimate; and from 14,750, at 14,644, on 2026-10-09, for
scoped use made production, the computer first (§8, "Then"; svrn-docs
`rings/current/SCOPED_USE_MADE_PRODUCTION.md` §8): the door's choices,
their check and routes (~180), the computer's origin naming the app,
keeping a page to its reads and carrying the app by its own pass (~120),
the computer's page at `svrngs.localhost` (~100), the phone's page and
worker (~110), `share --ask` (~25) and the pass's app (~6), about 540
against the 106 left, 450 by the seat's estimate. Then core-0 made the
room (operator, 2026-10-09; svrn-docs `rings/current/ONE_PERSON_MANY_DEVICES.md`
§6): upstream's guest attestation and roster warrant, which no code of
svrngs used (`attest.rs`, `introduce.rs`, `Roster.vouches`), deleted —
svrngs-core from 2,957 to 2,628, the repository from 14,644 to 14,315.
And from 15,200, at 15,162, on 2026-10-09, after sup-2: the repository
had grown 847 since core-0, against the 541 the design estimated for
asleep-1, sup-1 and sup-2, and the phone's page and worker (~110, sup-3)
had 38 left; 150 by the operator's choice. And on the branch `folders`
alone, from 15,200, at 15,163, on 2026-10-09, for the mesh-Dropbox spike
(`ralph/next/sync/`): about 500 lines for the folder's index, its watcher
and the apply of others' changes with their conflicts, the gate that asks
the record, and the command, by the seat's estimate; the spike counted
543, its index the record's, not iroh-docs
(`demo/runs/sy/2026-10-09/README.md` §1, §7). The branch keeps its cap,
16,500 at most while it merges main in, and takes main's with the last
merge (§10).
`ARCHITECTURE.md` maps what exists and changes in the commit that changes it.

**Words.** The *record* is a group's signed acts; its *journal* is the file
that holds them on one computer; the *register* is the part of the record that
says who is in the group — its `Admit` and `Remove` acts and their fold.
Forked code keeps upstream's words until a unit renames them: *ring* and
*namespace* for a group, *roster* for a seed list of members, and names such as
`commonwealth-rail` and `MeshStore` in its comments. Upstream comments cite
ARCH principles by old section number (ARCH §18.1); the table at the end of
`docs/principles/ARCH_PRINCIPLES.md` maps them.

## 6. The record

**As forked**: signed acts — `Record`, `Correct`, `Seal`,
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
  forks void nothing (U7; operator, 2026-10-04, moved from U2). Landed,
  u7-removal (b): both branches are held and each act weighed as itself, so
  every act on either branch counts by the rules any act does, and the fork
  is named, a `SequenceFork` beside a complete answer (`GapClass::Contradiction`,
  `crates/svrngs-core/src/admit.rs`, the per-actor audit). Before, two acts
  on one seq were excluded from what an app folds and a folder's changes,
  so an author could take back their own act on every copy by signing a
  second at its seq, and every copy read as incomplete; standing never read
  the fork, since `membership` folds before the audit. Sync already carries
  both branches (`two_forked_nodes_exchange_until_both_hold_the_fork`,
  svrngs-runtime). U7 also closed two holes of the
  same family that only a member can open, each leaving a copy that never
  read as complete while the acts themselves still applied (landed,
  u7-removal (d)): a forged line above a member's newest act moved their
  door's next seq past it, and now only a line whose signature verifies
  moves it (`crates/svrngs-runtime/src/journal.rs`, `Held::catch_up`;
  ralph/REVIEW_FINDINGS.md #16); and an act refused inside its author's
  chain was reported as a hole, since admission's seq audit counted
  admitted acts only, and now it counts every act of a bound signer
  (`admit.rs`, the per-actor sequence audit; 40b9466's body).
- **Removal has one decider** (U7). Anyone may ask to remove anyone; the group's
  keeper — one person's key, handed on by an act — sequences each request. A
  removed key cannot remove. There is no recovery inside a group. Landed in
  the core, u7-removal (c) (`crates/svrngs-core/src/membership.rs`, its
  module docs the rule): the keeper at the seed is the maker's key (for a
  seed of many keys, upstream's shape, the first key of the person whose
  name sorts first), moved only by the keeper's own `HandOn` to a standing
  key; an act that follows its author's own `HandOn` by `prev` carries no
  keeper's authority, so an ex-keeper's backdated second hand-on moves
  nothing. The office isn't the person: a hand-on that made a keeper
  survives its giver's removal, since a cut covers each of the cut key's
  hand-ons that moved the keeper before it in the walk as though its
  `through` named it (v1-keeper-office (a); §10, 2026-10-12). Before, a
  keeper's cut of whoever handed on to them that named acts only before
  the hand-on undid itself on the next walk and that walk redid it, and
  the fold said it did not settle (#124). A cut whose own signer stands by
  an act of the cut key it does not cover still cycles; the fold says so
  (`Membership::unsettled`), and nothing is decided on it. The keeper's
  `Remove` of a standing key not its own is the cut;
  anyone else's is an ask, counted as an act, cutting nobody. A cut key's
  later acts count nowhere (`RailGap::Removed`), so it cannot remove, ask
  or hand on. A `Correct` voids nothing that moves membership — before,
  any member's `Correct` of an `Admit` unseated its key and a removed key's
  `Correct` of its cut restored it (`membership.rs`'s void layer, red at
  `f2b9e03`) — so a link closes only by its issuer's next `Invite`, and
  `InviteFault::Voided` is gone. The keeper can equivocate, signing two
  acts on one parent; that is the trust risk 2 names. A person removes
  someone, or asks it, in the command (u7-removal (e); §10, 2026-10-09):
  `svrngs remove <name> --group <group>` and `svrngs handover <name>
  --group <group>` ask the node running here (`crates/svrngs-node/src/removal.rs`),
  which commits svrngs-core's `Removal` (a `Remove` of each key standing
  as that person, naming each key's newest acts held) or `HandOnTo`; from
  anyone but the keeper it is an ask, and the words name who decides. The
  cut is told once to the removed person's computers, the cut alone, and
  each removal or hand-on written wakes the round, so every other computer
  holds it at once, not at the next round a minute on (take 1 of
  `demo/runs/u7/2026-10-11/`: a lender not the keeper served the removed
  person 43.3 s after the cut). From
  then on each computer of the group answers a removed key on the
  members' protocol only with svrngs's refusal `removed`, "You are no
  longer in this group", and refuses a pass it signed the same, so every
  lender's door refuses them by the register (`node.rs`,
  `removed_anywhere`); their home page and the group's page on their
  phone say they are no longer in the group; a synced folder serves them
  nothing new, since its provider and the acceptor ask standing. **What
  they already hold stays theirs**: removal sends nothing back and
  deletes nothing on their disk — the files a folder gave them, the copy
  of the record they hold. A removed computer that was off as the cut
  landed is not told by svrngs; every door refuses it all the same.
  Everyone in the group sees every removal and every ask to remove — who
  of whom, and when — on a computer's home page and a phone's group page,
  read from the register and written nowhere new (v1-keeper-office (b);
  §10, 2026-10-12): `Register::removals`
  (`crates/svrngs-runtime/src/register.rs`), carried by `/v1/groups` and
  `/people/lends`, in `question.js`'s one wording.
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
  voids nothing (U7, below "Removal has one decider"); before, it unseated
  every key the `Invite` admitted, a removal no keeper sequenced. Both
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
  never by position or time (U7; landed, u7-removal (c)): `Remove{key,
  through}` names the removed key's newest acts its signer held, by id, and
  the cut covers them and every act they descend from by `prev`; any other
  act of the key counts nowhere, wherever its timestamp puts it. Before, the
  fold ignored `through_seq` and cut the key at the `Remove`'s place in the
  order, so a backdated act slipped under the cut. Which cuts count turns
  on the walk and the walk on what they cover, so the walk runs again until
  the cuts settle; a cut naming an act a copy lacks is `UnheldCut`, an
  absence. The node's register folds standing from the acts that move
  membership and every act of a key a `Remove` names
  (`svrngs_core::standing_reads`), so it covers what admission covers. The same order would put an act signed in the second of the
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
the group's meeting point, its port mapped by iroh; iroh's relays carry what
cannot go direct. A TV is reached through a member's computer on its
network, never by a page (below). A native phone app and
Windows wait in §9. A svrn daemon on the same computer is an optional neighbour
for local AI; it holds no group state.

**A phone's apps open apart from its page** (u5-2; §3, "Lent apps", calls
1 and 6). The page a group's link opens holds each person's key, so no lent
app's page runs at its origin: a lent page there could unseal the key of
whoever opened it (§11, risk 5). On a phone each app opens at an address of
its own, `https://<label>.<page host>/`, so the page's host must serve the
page at every name under it. `svrngs new --origin <page>` and `svrngs link
--origin <page>` check that by asking a name nobody made, over https, for
the page at its root and comparing it with the page's own (u5-10:
www.svrnme.sh/g/ resolved every name and served the page at none but its
own), then print where the apps open (`crates/svrngs/src/cli.rs`,
`checked_page`). group.svrnme.sh does (the Vercel project svrngs-page,
deployed by `scripts/deploy-page.sh`), so a group that names ours types
nothing more. Each app's root and storage are its own, and its pages hold a
pass naming it as the app and the lends it reads, no other. The cost: the page's host and
the phone's name lookups see which app is opened, since its label names its
lender's key and the app. A page at an address no name falls under — an IP
address — or on a host whose name nobody made does not serve the page is
refused, the reason saying what such a host must do; a page address that
does not give the page — its host unreachable, or no page there — is not
judged (exit 3). On a computer each app opens at
`http://<label>.localhost:<port>/` on its own loopback
(`crates/svrngs-node/src/origin.rs`), which every browser measured resolves
(`demo/runs/u5/2026-10-05-localhost/`); there the origin names the app
asking from what the browser says of its page, and keeps the page to its
own lend and the lends it reads (sup-1; §3, "Lent apps", call 6). Beside
them, at `http://svrngs.localhost:<port>/`, the home page: the person's
page on their computer, where they say yes to an app and stop it (sup-2;
§3, "What an app gets"). It answers only itself — reads from its own
origin, a typed address or a navigation, writes only with its own exact
`Origin` — so a lent page, same-site but never same-origin, can neither
read it nor say a yes, and it sends `frame-ancestors 'none'`, which
Chromium was watched refusing a lent page's frame of it. On a phone the
group's page is the page no app runs at, where a person says yes (sup-3;
§3, "What an app gets"): a lent app is a name under the page's host and
could frame it at `#ask=`, so the host sends `frame-ancestors 'none'` on
every answer (`web/vercel.json`; a `<meta>` policy cannot carry it), which
Chromium and WebKit were watched refusing, and a host a group names in
its place must send it too. Ours sends it once the operator deploys the
page again (`scripts/deploy-page.sh`).

**A computer gets svrngs from the group's link** (v1-onboarding; the
operator, 2026-10-11, §10). On a computer the group's page gives one line
to paste in a terminal, `curl -fsSL <page>install.sh | sh -s --
'<link>'`, composed in the page from the link's `#` part, so the token
reaches no server. The host that serves the page serves the rest beside
it, so any host a group names in ours' place serves it too: `install.sh`
at the page's address, and under `get/` a tarball of `svrngs` for each
platform built — macOS on Apple silicon and on Intel, Linux on x86_64 and
aarch64 as static musl binaries, cross-built here by zig
(cargo-zigbuild) — beside one `SHA256SUMS`. Nothing is fetched from
anywhere else: the repository is private and has no releases.
`install.sh`, POSIX `sh`, names the platform and refuses any other;
fetches the tarball and the sums from the address the link names and
refuses a mismatch; installs into `~/.local/bin`, never with `sudo`;
joins the link as `svrngs join` does; keeps svrngs running at login as
the person (a LaunchAgent on macOS, a `systemd --user` unit on Linux);
and opens the home page. Run again, it updates in place and joins
nothing twice. Publishing the binaries with the page is the operator's
(`scripts/deploy-page.sh`).

**A room's devices reach a lend through a member's computer there** (u6-1
landed it; §3, "The room", calls 2 and 3). A TV takes no link and runs no
page: the app's own client for it is typed an address on its network
once. So `svrngs room <name>...` on a member's computer serves the lends
its person names at `http://<private address>:<port>/` while it runs, a
port of its own per lend that its label decides, the same every run, as
that person, and `/v1/lends` answers a served lend's room address beside
its `origin`, as `room` (`crates/svrngs-node/src/room.rs`). The address is
the private one the outward route leaves from, else the one private
address listed, else none, said so; a caller whose address the door
would not guess home is refused. Its hop to the lender is computer to
computer over iroh, never through a page.

## 8. Order of work

**One experience, done delightfully: the group's films, in the Jellyfin a
member already runs, on any phone and the TV in the room.** Sam lends
Jellyfin as it comes; everyone in the group opens it from the list and
plays it here or on the TV; no port forwarding; the lender's password never
leaves their computer. Nothing else is built until Demo B passes, except the
two canaries and U5's catalogue run, which keep Jellyfin from bending the
infra into a media platform. (Until 2026-10-05 this was Films, an app of
our own with one shelf across every member's library; the operator dropped
it for Jellyfin as it comes, §3 "Lent apps, as they come", §10.)

**Many people, one library — first class, not an edge case.** Three friends in
three homes on Sam's library at once is the normal evening. What breaks, and
who fixes it:
- *Sam's upload is shared* — infra. The door measures Sam's upload and each
  viewer's path, divides the upload evenly among the people watching, and tells
  each request its share; shares change as people start and stop. Sam's own
  home, on the local network, is never counted against it.
- *Sam's computer has one processor* — Jellyfin's own. Its player plays the
  file as it is when the browser can, which costs Sam nothing
  (`stream.mp4?Static=true`, `demo/runs/a/dry-2026-10-04/README.md:52-61`),
  and transcodes when it cannot: WebKit remuxed an MKV through Jellyfin
  (svrn-docs `rings/runs/strm-2026-10-05/README.md`). Nothing in svrngs asks
  it for less; the share the door tells each request is in Sam's log.
- *Each person is their own to Jellyfin* — u5-3 and u5-4. Through a lend
  with a secret every request carries the lend's one device, so one
  person's sign-in logged the lend out for everyone (U3's dry run); a lend
  with no secret leaves each browser its own sign-in and device, which is
  how `svrngs share 8096` lends it since u5-4, each person with an account
  the lender made (§3, call 4). Jellyfin kills
  running transcodes by device id and play session
  (`DynamicHlsController.cs:1504`, in a source read once and held in
  neither repository), so whether Bo seeking leaves Cy's play alone is
  u5-3's to measure, never assumed.
- *Too many at once* — the door refuses past the lend's limit with its
  reason in a person's words and the figures (`door.rs:571-585`); nothing
  turns it into a choice of another lender.
- *The same room* — the TV plays through the app's own client, which
  reaches the lend through a member's computer in the room (§3, "The
  room"); the remote is the app's own, and the lender's door counts the
  room as one person, whoever holds the remote.

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
presence:* the round dials every standing member's computer, all at once
(asleep-1), and one that is offline costs its own dial's timeout each
round, the others' exchanges not waiting on it; a browser's key is never
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
| `landing/ring/` (generated, committed) | the built site; `wasm/ring_runtime_bg.wasm` is 4,295,444 bytes (`git ls-tree -l`); its glue reaches the relay with `new WebSocket` | not forked. Upstream's one figure for a page's weight. Ours, measured (u4-8, `demo/runs/u4/2026-10-05/`): the page fetches 5,103,571 bytes, 1,209,407 brotli, its wasm 5,027,239 without `wasm-opt`; a cold join took 15.35 MB from an origin sending no cache headers, the wasm fetched by the page, the worker's install and the worker's start (`ralph/REVIEW_FINDINGS.md` #51); since u4-9 the wasm is fetched once, by one URL the page and the worker share, and a cold join took 5.34 MB (`demo/runs/u4/2026-10-05-reach/`) |
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
(`crates/svrngs-node/src/node.rs:648-667`), so the endpoint key is the
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

**U5's inventory.** What upstream at `e1a43bcd2`, and svrngs's own runs,
have for lending an app as it comes, Jellyfin first, and what each becomes;
written from the seat's survey (`ralph/next/u5/UPSTREAM.md`, made for Films
and kept here only where a U5 row uses it), each path:line read again at
u5-0, upstream's at `e1a43bcd2` and svrngs's at that commit. Upstream has no
lend at an origin of its own, no pass, no WebSocket across a door, and no
stock web app measured through one; what an app cannot yet ask of svrngs is
§3's list. Paths under `mesh_media/` are
`sovereign/crates/sovereign-cli-mesh/src/mesh_media/`.

| Upstream | What it does there | Becomes |
|---|---|---|
| `mesh_media/viewer.rs:304-463` (`provision`, 323 lines without its tests), with `:143-196` and `:216-296` | finds or makes one Jellyfin user, sets its password, writes its policy over the origin's own — `POST /Users/{id}/Policy` replaces rather than patches, and needs `AuthenticationProviderId` and `PasswordResetProviderId` (`:21-27`, `:143-175`) — reads the policy back (`:177-196`), signs in by `AuthenticateByName` (`:443`) and keeps `MediaBrowser Token="…"` (`:461`); a token already held is proved by one `GET /Users/Me` (`:216-296`). An `/Auth/Keys` key is admin-equivalent (`:6-12`); a non-admin sets no password (403, `:205-208`) | zero sign-in's shape (§3, call 4): a Jellyfin user per person, made by the lender's computer with a key the lender gives once. Forked into `apps/jellyfin/mw/src/viewer.rs`, behaviour kept first, then a user per person in place of one viewer, as the middleware's first module (mw-1, `62560f0`, `717f433`); never into the infra (call 3) |
| `docs/RING_APP_LIBRARY.md:802-816` | three ways a lent service keeps its users, cheapest first: the app reads the member header; the lender's credential injected, one shared viewer; an account per member, made on `Admit` and disabled on `Remove`. Mapping the member header onto an app's trusted proxy header is "From memory — verify per app" (`:810-811`) | the first is §1's "an app that takes the signed-in person from a request header", which u5-catalogue-dry's census checks per app against each app's own docs; the second is what U3's dry run broke on; the third is zero sign-in |
| tag `e0-bot`, `docs/DESIGN.md:580-587` and `:878` | the door masked the secret in response bodies because "Jellyfin writes its token into playback info and into the HLS playlists' subtitle, trickplay and variant URLs"; a lend refused `/Sessions/Logout`, "which would sign the whole group out"; `Remote-User` and per-person accounts waited for "a named service that needs them" | not forked. The body claim was never measured: u5-3 (c) asks whether any byte a person receives carries a secret the lend holds, the transcode included. Jellyfin is now the named service |
| `.canon/sources/notes/invariant/35104a7e.md:3-9`; `scripts/cw-media-demo.sh:31-40`, `:54-56` | Jellyfin 12.0.0 takes its credential only from `Authorization: MediaBrowser … Token="…"`: `X-Emby-Token`, `X-MediaBrowser-Token` and `?api_key=` each answer 401, and the inner space and quotes are load-bearing | a fact `apps/jellyfin/` and u5-3 build on: a lend's secret displaces the client's whole `authorization` (`crates/svrngs-node/src/door.rs:113-119`, `:386-412`), the client's device with it. Nothing forked |
| `mesh_media/offer.rs:94-97` (`WELL_KNOWN_ORIGINS`) | 8096, then 8920 | forked already (U3): `crates/svrngs/src/well_known.txt`, 8096 alone, so `svrngs share 8096` names Jellyfin |
| `commonwealth/crates/commonwealth-media/src/reach.rs:94-95` | refuses a member's own node: "its {} is already local" | left behind: u5-1 lists this computer's own lends, where `Node::lends` skips them (`crates/svrngs-node/src/node.rs:475-477`), and u5-2 serves them through this computer's door's rules |
| `quality/campaigns/ring-apps-shelf.md:187` | Jellyfin is GPL-2.0-only: "API interop at arm's length only" | held: svrngs talks to Jellyfin over HTTP and its API and copies none of its code |
| svrngs `demo/runs/a/dry-2026-10-04/` (`dry.sh`, `drive.mjs`, `README.md`) | Jellyfin 12.0.0 through the door on one computer, the macOS app's server with data dirs of its own: the wizard over the API (`dry.sh:60-76`) and the lend's secret minted (`:88-95`); opened with nothing typed, a sign-in (`README.md:35-37`); a member's sign-in logs the lend's token out for everyone, the door's header carrying the lend's one `DeviceId` (`:39-50`); with a placeholder sign-in written into the page's local storage (`drive.mjs:41-47`), it plays in 1,137 ms and each seek plays again within 123 ms (`README.md:52-61`); an API key ruled out (`:63-67`); `/socket` 404 through the door, 101 direct (`:69-73`); no secret in a received byte but the token Jellyfin handed the browser signing in (`:81-88`); three at once and a transcode not measured (`:90-93`) | u5-3's base: its scratch Jellyfin is `dry.sh`'s, and what the run found is what u5-3 measures and u5-4 solves (call 2). The placeholder sign-in stays a driver's trick: no person can write it |
| svrn-docs `rings/runs/strm-2026-10-05/` (its commit `457f120`) | the group-folder experiment's first run: Jellyfin 12.1.0 over two `.strm` stubs; every browser played the holder's MP4 from the stub's URL; WebKit, alone and as an iPhone, remuxed the MKV through Jellyfin's own HLS, reading the whole file unpaced; FAIL on its pre-registered byte-share bar (93.0% and 85.8%) | §9's group folders, after U5; no U5 row builds on it |

Four facts follow. *Lends shared the key's origin until u5-2:* on a phone
every lend was under the page's scope, where u4-6's test had a lent page's
script unseal the person's key (§11, risk 5), and on a computer a lender's
lends shared the bridge to that lender, its loopback port beside the record
API's; since u5-2 each opens at an origin of its own (§7). *Two nodes on one host dial on loopback only:* iroh moves a
connection dialled on loopback onto this computer's private address, which
the door guesses home and tells no share (`ralph/REVIEW_FINDINGS.md` #33),
so U5's two-node tests build every node with `RelayConfig::on_loopback_only`,
as the door's do (`crates/svrngs-node/tests/door/main.rs:56`). *A sign-in is
the app's:* the door sets the person on every request, and an app that
reads no such header asks its own sign-in; Jellyfin is one (U3's dry run),
and which of §1's apps read one is the census's to find. *Jellyfin's own
cross-origin rules were never measured* in either repository; with each lend
at an origin of its own (call 1), stock jellyfin-web asks none.

**U6's inventory.** What upstream at `e1a43bcd2`, and svrngs at `d95039c`,
have for the room door, and what each becomes; written from the seat's
survey (`ralph/next/u6/UPSTREAM.md`), made for a handoff to a TV that the
operator has since taken out of scope (§3, "The room"), and kept here only
where the room door uses it, each path:line read again at u6-0. Upstream
has a LAN door that lives only while it is wanted and an address picker,
and nothing measured on a TV; its door's proxy buffers, so U4's inventory
left it. svrngs serves a lend only on an iroh stream or on loopback.

| Upstream or svrngs | What it does there | Becomes |
|---|---|---|
| `sovereign/crates/sovereign-daemon/src/guest_door.rs:412-499` (`serve`, `wait_for`) | the LAN guest door's listener: `0.0.0.0:9744` when no bind is named (`:434-443`), bound only while a grant is live, the grant store polled (`:455-456`, `:491-499`), shut down gracefully at the last expiry (`:477-486`) — "safe by LIFETIME, not by address" (`:429-431`); `ConnectInfo` on its service (`:472-476`) | its shape, not its code: the room door's listeners live while the command runs, bound on the private address alone, so safe by both. Not forked: it waits on a grant store svrngs does not have (`:454`) and serves the daemon's router and auth (`:475`) |
| `sovereign/crates/sovereign-cli-mesh/src/mesh_guest_link.rs:37-55` (`advertised_base`), `:63-96` (`guest_bind_url`) | "the door names the PORT, this machine names the ADDRESS" (`:10-12`): a wildcard or loopback bind is never advertised (`:45-53`), no candidate is absence (`:34-36`), and the LAN before tailscale, measured: "the default picked 100.104.36.28 while the room's phones needed 192.168.1.7" (`:67-73`) | its rules, written smaller (the next two rows). A device on the room's WiFi cannot reach tailscale's range, so here "before" is "instead of": the private LAN address or absence, never tailscale's. Not forked: it reads a config's bind and `relay_candidates` |
| `commonwealth/crates/commonwealth-discovery/src/mesh_discovery.rs:56-77` (`classify_ip`), `:98-124` (`relay_candidates`), `:237-313` (`local_ip_candidates`) | every IPv4 address outside 100.64/10 is "lan", a public one included (`:58-66`); the interfaces from the `if-addrs` crate, which svrngs does not carry, minus loopback and link-local (`:259-283`); a container bridge listed before tailscale (`:21-32`); the outward route's address by a UDP connect that sends nothing, its fallback (`:290-298`) | not forked: its "lan" would pick a public address, which call 3 refuses, and iroh lists the interfaces already (next row). The classifier is std's `Ipv4Addr::is_private`, which `guess` uses already (`crates/svrngs-node/src/door/limit.rs:54-65`). The UDP connect is kept, as the tie-break: on this host it answers `192.168.1.4`, the address the TV reached in the reads |
| iroh 1.0.2 `Endpoint::addr()` (`src/endpoint.rs:1196`, its addresses gathered at `src/socket.rs:1814-1955`), netwatch 0.19.1 `LocalAddresses::new` (`src/ip.rs:34-93`) | with the wildcard bound, every interface that is up, paired with the bound port (`socket.rs:1926-1953`); loopback only when there is nothing else (`:1937-1940`), link-local IPv4 likewise (`ip.rs:63-66`, `:86-93`); the port-mapped and reflexive addresses its reports find (`socket.rs:1814-1875`); no interface's name. Measured: a node here logged `dial=Some("…@192.168.1.4:49713")` (`demo/runs/a/2026-10-05/data/alex-run.log:3`) | **the picker's source, written smaller** (u6-1's row leaves the choice here), with no new crate: a pure function over the addresses `ip_addrs()` lists and the outward route's — the private IPv4 address the route leaves from, when it is among them; else the one private IPv4 address listed, when there is one; else absence, said so. With no name kept, a bridge in a private range (a container's, `172.17.0.0/16`) is told from the LAN only by the route, which does not leave through it. A node built `on_loopback_only` lists loopback alone (`crates/svrngs-node/src/guest.rs:215-219`), so the picker's tests plant the lists |
| svrngs `crates/svrngs-node/src/origin.rs:59-107` (`lent`), `:119-158` (`carried`) | each lend's origin on loopback: the lend its `Host` names, this computer's own through the door's one body as `Who::Own` (`:68-79`), another's dialled as this computer's key through the bridge to its lender's door (`:90-106`), a WebSocket spliced once both sides switch (`:145-153`) | **the room door's body**, one with it (call 3): the lend named by the listener's port in place of the `Host`, which on a room's device is the private address |
| svrngs `crates/svrngs-node/src/api.rs:64-85` (`router`), `:132-160` (`from_this_computer`), `:165` (`check_loopback`) | the loopback API's listener, where each lend's origin is served beside the record's routes behind one check that the caller is on this computer | not reused: the room door is a listener of its own that answers no route of the record's. Its caller check has `from_this_computer`'s shape and `guess`'s home as its predicate — private, link-local or unique-local — so "this network" has one decider |
| svrngs `crates/svrngs-node/src/door.rs:736-805` (`lend_for`), with `guess` | the lender's door: the person from the dialer's key and the register (`:736-765`); the path's address guessed home or away (`:805`) | unchanged (call 3): the lender's door sees the room door's computer as its person, counted once however many devices in the room ask, its path guessed as now — from across town, away and counted |
| svrngs `crates/svrngs-node/src/node.rs:300` (`Node::lends`), `:367-370` (`origin`) | `/v1/lends` answers where this computer's browser opens each lend | gains a served lend's room address (u6-1, §7) |
| svrngs `apps/web-conformance`, `crates/svrngs-node/tests/conformance.rs` | the conformance canary (§8) | the room door's check: its routes at the private address, from a client not on loopback (u6-1) |

Six facts follow. *Two of the survey's gaps are U6's:* no listener for a
room's devices (`api.rs:165` refuses any bind off loopback, and the door
answers only a verified dialer, `door.rs:718-731`) and no address picker;
its other thirteen went with the handoff, each named in u6-0's commit.
*The room is one person to the lender:* the room door dials as its
computer's key, so a room of five is one person to Sam's limit and share,
and who in the room asked is the app's to know, by its own sign-in.
*Typed once needs an address that stays:* the screen's client is typed the
address once (call 4), so the room door keeps each lend's port from one
run to the next; the private address is the router's to hand out.
*A port per lend parts roots and storage, not cookies:* a browser keeps
cookies per host, not per port (RFC 6265 §8.5, not measured here), so on a
device that opens two lends at the room address, each lend's cookies reach
the other's requests. Every lend there is reached as the one person, and
the screen's client is one app's. *Plain HTTP on the room's network:* as
upstream's LAN door is by design, "the room's WiFi is the trust
environment a room accepts" (`docs/THE_LINK.md:217-222`); a lend's secret
never crosses it, staying in the lender's door, but what the app's own
client sends does, its sign-in included, as in the reads, where the TV's
app reached Jellyfin over plain HTTP (`jellyfin-conns.log:9`).
*Unmeasured here:* whether macOS lets a device on the network reach a
`svrngs` process's listener. In the reads the TV reached the Jellyfin
app's server on this computer, never `svrngs`; a client on this host
never crosses that gate, so u6-9's package is its first. And the
repository stands at 13,826 of 14,000 lines (`scripts/lean_check.py` at
`d95039c`), so the room door has 174; a raise is the operator's.

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U1 a group is not a mesh | ring sync, its route and the dial-by-key tunnel forked into `svrngs-node`, the node's router served on each accepted stream; the register decides who may connect and who is synced, where upstream asked the mesh (`commonwealth/crates/commonwealth-rails/src/` at `e1a43bcd2`: `acceptor.rs:47-61`, `ring_sync/journal.rs:153-164`, the default roster at `lib.rs:404`) | a node in the group but no mesh syncs; a node outside the group is refused |
| U2 join by link | `svrngs new <name>`, `svrngs join <link>` and `svrngs link <group>` on a computer, and `svrngs` alone running the node; the link is one short token — the issuer's key and the link's secret, and the issuer's relay only when it runs without lookup — and the issuer answers the group's seed and its `Invite`'s id to the link's key; redeeming at the issuer's node writes one `Admit`, co-signed by the new key; ids over everything signed, the previous act among it; the door signing after everything it holds (§6) | three people open one link and each is admitted once; opening it again admits nobody twice; a lapsed or closed link admits no one; a dialer without the link's key is closed with no answer; nothing the command prints uses a word §1 keeps from people |
| U3 lend a service | the member door: a route that proxies to a lent service, the verified person set from the connection; `svrngs share <port>` with its lend declaration; streaming and byte ranges through the door; each request's share of the lender's upload; the people-at-once limit; the loopback record API (the rail routes and the live lane, forked); the list of what members lend | the app sees the verified person; a header the client typed is stripped; the lender's secret appears in no byte a member receives; with three streams, each is told a third of the declared upload, and a fourth person is refused with a reason |

**Demo A — Sam's films, three homes at once** (after U5; operator,
2026-10-05, in Jellyfin as it comes, where on 2026-10-04 it was put in an
app of our own once U3's dry run found no way into Jellyfin's own player
that kept the bars — §11, risk 7). Sam lends Jellyfin with one command and
sends a link. Alex, Bo and Cy join from three homes and each plays a
different film from Sam's library in Jellyfin's own player through the
lend, in their own browser, at the same time, seeking freely; none stalls
another, and the door tells each its share of Sam's upload, in Sam's log.
Sam's password never left Sam's computer; nobody made an account (§3, "Lent
apps", call 4, on how that clause meets a lender's middleware).

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U4 the browser | a phone opens a group's https link, on the page origin the group named, and is in: its own key, a dial by key to the issuer to join and to each lender for what they lend (§3, "Joining"), and the runtime that renders a lent app through it — fetch, POST, streaming, byte ranges; the page is static files any origin can serve | a stock phone browser plays a lent film and seeks |
| U5 any app on a port, Jellyfin as it comes | what an app cannot yet ask (§3): the groups this computer holds and what their people lend, on the record API (u5-1); every lend at an origin of its own, a pass in place of the person's key, the web carried whole, its live socket included (u5-2); stock Jellyfin measured through a lend and the way past its sign-in decided (u5-3), then built (u5-4); §1's list of apps measured on five of them (u5-catalogue-dry); Demo A's dry run (u5-9) — on the §3 surface alone, and nothing that knows Jellyfin outside `apps/jellyfin/` | after `svrngs share 8096`, someone who never used Jellyfin opens it from the list, from a computer and from a phone, in Chromium and WebKit, plays, and seeks three times, each playing again within five seconds; two people at once, one seeking five times, never restart the other's play; no byte a person receives holds a secret the lend keeps; the conformance canary's routes all pass on a computer and with the wildcard, and on the floor (dropped since, u6-00) all but the two it cannot give, each named; §1 names only apps the census supports |
| U6 reach for the room | the room door: a member's computer serves the lends its person names to its own network while the command runs, each at a port of its own on its private address, as that person (u6-1); Jellyfin's own client for a screen measured through it, and what its lender sets written in `apps/jellyfin/` (u6-2); Demo B's dry run (u6-9) — and no screen, player or media word in svrngs (§3, "The room") | a client on the room's network, not on loopback, passes the conformance canary's routes through the room door; a lend not named is not served, nothing is served once the command stops, and a request from off the network is refused; the address picked is the private LAN's, or absent; Jellyfin's own client for a screen, stood in for by jellyfin-web, signs in, plays and seeks three times, every request through the room door |

**Demo B — movie night** (after U6; operator, 2026-10-05: the library is
across town, and the people in the room are on the TV's WiFi). Five friends
at Dee's, phones only; Sam's library is across town. Before the evening
Dee sets up two things, once: Dee's computer runs the room door for Sam's
library, and Dee's TV has Jellyfin's own app (§3, "The room"). Sam's
group names a page origin, so its link opens in a phone's browser. Sam
texts the link; each opens it, opens Sam's library, and puts a film they
pick on the TV. Nobody installs or signs up. The run-of-show is
`demo/b.md`.
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
- **Conformance** (u5-2) — the web carried whole: a localhost service with
  one route per thing a self-hosted app leans on — a page asking for
  `/`-absolute paths, a WebSocket echo, a 64 MiB file read by Range and
  uploaded streaming, its own service worker, `SharedArrayBuffer` under
  COOP/COEP, a Host check with an absolute redirect, a cookie sign-in,
  local storage; and, from u5-catalogue-dry's misses, its page under a
  script policy by nonce, its socket's state read against the socket's own
  constants, its sign-in's cookie read by the page before any reload, a
  post it takes only from its own address, named, by fetch and by a form, and
  the upload told its length; and, from u5-9's stall, the file played as
  sound and sought twenty times 250 ms apart, the last playing again within
  5 s, over a phone's link from a home (u5-11). Landed (u5-2): `apps/web-conformance`, lent by Sam's and
  Bo's computers at once and opened by Alex from a computer and from a phone,
  each app at an address of its own (§7), in Chromium and in WebKit as an
  iPhone, by `crates/svrngs-node/tests/conformance.rs`; a step of its own in
  `scripts/check.sh`, printing each route's verdict per way. Every route
  passes on a computer and with the wildcard; the floor's mode went with
  the floor (u6-00). WebKit's seeks back to back are a rate,
  not a gate (operator, 2026-10-06): each run prints and keeps their verdict
  beside the same seeks straight at a canary over the same link
  (`web/tests/control.mjs`) and the host's load, and `scripts/check.sh`
  counts every run's misses on this host (§9 has the trigger).
  Chromium's stay a gate, but on a host whose load at the run's start is
  past 3 a core (`BUSY`) a miss is could-not-judge, with that load, and
  fails nothing; a pass is passed at any load, and `scripts/check.sh`
  prints each could-not-judge line apart (chk-1; operator, 2026-10-10).
  Neither lend's cookie reaches the other's requests, Bo's door names
  Alex's key, no byte Alex receives holds either secret, a lent page finds
  no group whose key it could unseal, and a lent page's post to the record
  API writes nothing. Since mwp-4 it is the sign-in's second app too: given
  the lender's key it keeps accounts, a token each, and its answerer
  (`answer.mjs`) is a lend's sign-in, so `two_people_signed_in` has Alex's
  phone and Bo's computer each signed in as themselves, in both engines
  (§3, "The room", call 6, bar 4).
- **Tap** (after U3) — private inputs and a ring born from a ring: two groups
  on one node; an app starts a group of two from inside a group; it seals its
  own payloads to members' public keys, which the register gains with it. (It
  said "after U2" until 2026-10-04; an app reaches the record through U3's
  loopback API, and no key in the register can seal, U2's inventory.)

**The ceiling** (after Demo B; operator, 2026-10-05). The scale instrument
(`crates/svrngs-node/tests/scale.rs`) found where a group outgrows its round:
an idle round passes its sixty seconds at about 7,900 acts for twenty-five
members and about 860 for a hundred, because the digest, every signature and
the register are rebuilt from the whole journal on every exchange and every
connection, and an asleep member costs a ten-second timeout each round. Each
fix is the tail fold `Held` already keeps for writes
(`crates/svrngs-runtime/src/journal.rs:36-58`); prototyped behind switches,
they took that round from 73 s to 10 ms (svrn-docs
`rings/current/THE_CEILING.md`, its runs under `rings/runs/`). Dialling at once
stays §9's row; the view each act carries goes with U7.

| What lands | Bar, watched failing first |
|---|---|
| first the round's own account of what it cost — members dialled, reached and asleep, acts moved, processor and wall time, each fold kept or rebuilt — then the digest folded from the journal's tail, with an equal digest answered unread; each act's signature checked once, as it enters; the register folded from the tail | at 25 members and 10,000 acts an idle round costs under 0.1 s of processor in the instrument (73 s at `15a9de7`, where it was measured), held by a canary in `scripts/check.sh` whose plant is today's code; the cost line predicts a point it was not fitted on |

(Landed, ceiling-1 to ceiling-4: the account, then the digest, the
signatures and the register each caught up from the tail; the canary,
`an_idle_round_is_under_the_ceiling`, a step of `scripts/check.sh`, failed
on the register read whole at 5.65 s and reads about 0.01 s. The line
first fitted, its exchange linear in *N*, was refuted at 100 members; refitted
with a term in *N*² that nothing yet explains, it predicted 75 members within
8%. The runs and predictions are in `scale.rs`'s module docs.)

**Then**, in this order (operator, 2026-10-07): the ceiling, above; the
middleware spike (§3, "The room", call 6; kept, 2026-10-08), then made
production, the door carrying its general pieces (operator, 2026-10-08;
landed, mwp-0 to mwp-4: Jellyfin's middleware from 1,039 lines to 669,
svrngs 426 new, a second app signed in through the same sign-in in 30;
call 6, "Landed"); then the sign-in decided in one place, its stand-in
named once, and one person's sign-in never holding another's
(`ralph/REVIEW_FINDINGS.md` #87, #90, #88; operator, 2026-10-08;
landed, si-1: `forward` 242 lines to 171, the stand-in named in the
door's ask; its Jellyfin run in `demo/runs/si/2026-10-08/`); then
two spikes before folders (operator, 2026-10-08): Movie Night, a thin app
composing the group's Jellyfins — discover, request as the person, open
— with every line it writes that is not its own counted (svrn-docs
`rings/current/THIN_APP_COMPOSITION.md`; run, 2026-10-08: 157 lines its
own, 39 svrngs's, request carried whole, discovery and opening not, §3,
"Lent apps", call 6), then scoped use, an operation
its owner offers beside their data, to one app acting for one person
(`rings/current/ACL_AND_SCOPED_USE.md` §4, §9; run, 2026-10-09: every
bar passed, held by 46 of the service's 98 lines and two of the door's checks,
§3, "Lent apps", call 6), each made production
from its data, as the middleware was — Movie Night by four rows
(`demo/runs/mn/2026-10-08/README.md` §9): a lent page told the lends it
reads, whose and where and who did not answer, on a phone and a
computer alike, with no pass in its hands; svrngs's refusals named as
its own; a newcomer let in by every lender at once, not at each one's
next round; a lend opened at a path from a page that reads it (landed,
mnp-1 and mnp-2: Movie Night's plumbing 39 lines to 12, `find` 28 to 1,
bar 2 passed for a phone that never opened that Jellyfin, bar 4 asleep
before the list and bar 5's letter passed, `demo/runs/mnp/2026-10-09/`;
§3, "Lent apps", call 6) — and
scoped use by four (`demo/runs/su/2026-10-09/README.md` §10), three
of them next, the computer first (operator, 2026-10-09; svrn-docs
`rings/current/SCOPED_USE_MADE_PRODUCTION.md`): the app named on the
pass, and by a computer's origin, which keeps a page to its reads
(landed, sup-1; §3, "Lent apps", call 6); a
person's yes and stop kept by the lend's door, said on a page no app
runs at — on a computer the home page, `svrngs.localhost` (landed,
sup-2; §3, "What an app gets"), and on a phone the group's page (landed,
sup-3) — never by the app; then the spike run again,
on a computer first (landed, sup-4: the taste's enforcing 46 lines to 11,
Tonight's Picks' plumbing 39 to 16, every bar passed in its spirit on a
computer from its home page in Chromium and WebKit and on the phones, the
unasked yes refused by svrngs on a computer, `demo/runs/sup/2026-10-10/`;
its three follow-ups there landed in lean-4 (b) to (d) (operator,
2026-10-10; `ralph/next/lean/`): svrngs's answers at
a lend's origin readable by the page that reads it, one wording of
svrngs's question, a stop address beside `ask` — and its computer
journeys run again on them, Tonight's Picks saying `refused-unasked`
before the yes and `refused-stopped` after a stop in Chromium and WebKit,
`demo/runs/supf/2026-10-10/`); after a row that keeps an
asleep computer from holding anyone's answer
(`demo/runs/mnp/2026-10-09/README.md`, "What no prediction named" 1 and
2). The fourth, a lend its lender's person
alone reaches on any device of theirs, waits on another device for the
same person (§9; `rings/current/ONE_PERSON_MANY_DEVICES.md`, deferred);
group folders (§3, call 7 of "Lent apps"; §9), synced onto each member's
disk and not streamed (operator, 2026-10-10, after the mesh-Dropbox
spike's four bars on one host, `demo/runs/sy/2026-10-09/`; §10), made
production on the branch `folders` by its queue `syp`, main merged in as
the lean rows cut what the product does not carry — recorded runs'
drivers, tests in counted files, code nothing reaches, code written twice
(`ralph/next/lean/SURVEY.md`; lean-1, lean-2 and lean-3's first cut in at
syp-3): the rows the spike's README's §9 lists (landed, syp-4: §3, call 7
of "Lent apps"), then made fast by perf-1's candidates 1-7 (landed,
perf-2: on loopback, one host, a small change at another member's disk in
164 ms, 1,000 files in 373 ms, a change in a folder of 100,000 found
within 432 ms with an idle node at 0.33% of a core, ten members each
holding one change within 515 ms, a member joining 10,000 files in 1.7 s
once it stands; a 100 MiB file at 83 MB/s to each of two members at once
and 139 MB/s to one, a bare iroh-blobs get's own rate; a 1 MiB edit still
moving the whole file, `demo/runs/perf/2026-10-10-make-it-fast/README.md`),
then the spike's run again (landed, syp-5: three of its four bars
passed on production, `demo/runs/syp/2026-10-10/rerun/README.md`), main
merged in again; then the fourth's two holes closed (landed, syp-6: a
node stopped by SIGTERM closing its endpoint, a file no member gave asked
again; S3 passed three runs of three, `demo/runs/syp/2026-10-11/restart/`),
merged into main by the operator (2026-10-11, `4475a89`; §10); then
Demo C's units, below (operator, 2026-10-10): removal (U7, the keeper of
§6, with forks that void nothing; landed, u7-removal: Demo C's removal
check passed on one host, every lender refusing the removed person 0.048 s
after the keeper's command, `demo/runs/u7/2026-10-11/README.md`), before
any group of strangers;
onboarding — a computer's svrngs got from the group's link, the link handed
to it, the group's folder placed for the person, the link previewing as an
invitation in any chat (§3, "A chat's door is the group's link") — with
the middleware's P1 follow-ups as its parts (operator, 2026-10-08;
`demo/runs/mw/2026-10-08/README.md` §7, items 5 and 7; its item 1,
`share`'s words, answered by the sign-in's declaration, mwp-1): a lender
keeps a library back from the people the middleware makes accounts for;
Demo A's steps with nothing typed, and its dry run (its other four, P2 and
below, wait in that §7); then the library, the folder and a join measured
between computers in two homes, through a relay; then Demo C. The phone
replica spike waits in §9 (operator, 2026-10-10). After that the core grows
one class at a time, each chosen by the north star's tests.

**Demo C — the group's whole life** (after U7, onboarding and the two homes;
operator, 2026-10-10; its run-of-show `demo/c.md`, its bars the seat's
proposal and the operator's to set). Friends who already share a Discord
channel, each in their own home: Sam makes the group and posts its link
there; Ana puts a trip's photos in the group's folder; four newcomers on
computers and one on a phone open the link and play a film of Sam's, an
album of Ana's — Navidrome taking the person from svrngs's header — and, on
a computer, open a photo from their own disk; Sam removes Bo; Ana's laptop
closes, and the last newcomer still gets the trip from the others. Gate:
four of five within five minutes on a computer, svrngs got from the link
included — one line the group's page gives, pasted in a terminal (operator,
2026-10-11), and two on a phone, the phone in the group within ten seconds of
the link; no account but one a lender's program makes, no key asked for.

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U7 removal | landed, u7-removal: one keeper, the group's maker at first, sequences removals, handed on only by its own act; anyone's `svrngs remove <name> --group <group>` is an ask until the keeper's is the cut, by hash; a removed key cannot remove, ask or hand on; forks void nothing; a removal or hand-on wakes the round (§6, §10) | within 30 s of the keeper's removal every lender refuses the removed person in a person's words, and nothing new in a folder reaches them; their own `Remove` counts nowhere; what they held stays theirs. Watched failing: take 1, a lender not the keeper refused at 43.3 s, at its next round; passed at take 2, 0.048 s, on one host (`demo/runs/u7/2026-10-11/`) |
| onboarding | on a computer, svrngs got from the group's link, the link handed to it, the home page opened and the group's folder on the person's disk; the link previewing as an invitation in any chat; the middleware's P1 follow-ups, above | a newcomer's computer from the link to the group's three things within five minutes, unassisted, one line pasted in a terminal and nothing else typed |
| two homes | the library, the folder and a join measured between computers on two networks, through a relay, each path as iroh selected it | measured, not judged: the numbers Demo C's bars are read against |

## 9. Later, each with the trigger that brings it back

| Waits | Trigger |
|---|---|
| an app SDK, a sandboxed fold engine, signed manifests and install acts (QuickJS behind an `Engine` trait was `crates/svrngs-engine`, deleted 2026-10-04 with no user; it is at tag `e0-bot`); and the runner that would carry an app's function to a person's data, its yes to that code at a version — scoped use's spike served its operation from a program its owner runs, as the survey orders (`rings/current/ACL_AND_SCOPED_USE.md` §4: "a subsequent convenience to earn with a real distributed app"; `demo/runs/su/2026-10-09/` §7) | an app needs its state offline on a device that is not its host, or strangers' code runs on members' devices, or an app sends its function to run beside a person's data |
| a native phone app, and groups of phones only; first, phones as replicas of the group (operator, 2026-10-07): a member's phone holds the group's record and the files it shares, and syncs with whoever it reaches through a relay, so the group stays reachable while every computer sleeps (§11, risk 1) — a replica, not a relay in iroh's sense, since a phone on cellular sits behind its carrier's NAT. The phone's OS decides it: a page cannot listen and a browser's iroh is relay-only (§3, "The room", call 2), iOS suspends an app soon after it leaves the screen, and a network extension (the VPN slot) runs in the background under a memory cap. The spike, on iOS first and no larger than proves it: an iPhone app whose network extension holds an iroh endpoint with a member's key and answers a computer's dial by key every few minutes for 24 hours, the phone locked in a pocket; recorded, answered of dialled, memory against the cap, battery and data, against bars the operator sets before it runs. The record on the phone follows only if the endpoint stays reachable; Android after iOS | after Demo C, or the operator moves it in (operator, 2026-10-10: "I don't want to distract us from the core right now, but wanna keep it on the back burner"; its design, deferred, svrn-docs `rings/current/PHONE_REPLICA.md`); or the browser cannot keep what a group needs — its key, or a copy — measured |
| a chat's own door: a package that posts a group's link for a person, names a group for its channel, or follows a channel's members (E0's, at tag `e0-bot`, §3) | a group asks for one that a pasted link cannot answer, counted |
| Windows | a group asks |
| AirPlay and Cast from a page, and DLNA: a screen handed a lend's address by a phone, for an app with no client of its own on it. In the reads (`demo/runs/u6/2026-10-05-screen/reads.json`) AirPlay from Safari on an iPhone showed the LG no video three ways — the page's address, which only the phone's worker answers; jellyfin-web, whose hls.js sets `disableRemotePlayback`; a plain LAN file, which the TV never fetched — and iOS has no Cast sender. Until 2026-10-05 this was U6 (§3, "The room", call 1) | an app with no client of its own for a screen is asked for on one, counted |
| a public address per lend, for a room with no member's computer in it: a relay that forwards bytes it cannot read to the lender, who holds a certificate per lend — a new service, and Let's Encrypt's per-domain limit (§3, "The room", call 2) | a group with no computer in the room asks, counted |
| watching together from different homes: one transcode, shared, in step | a group asks to watch the same film apart at the same time |
| sealing in the infra | two apps need the same seal |
| group compute: asks leased by members' computers and answered as acts; and peer inference, a source's excerpts released to another member's computer for a job (`rings/current/ACL_AND_SCOPED_USE.md` §5), out of scoped use's spike (`demo/runs/su/2026-10-09/` §7) | Ask is next, or an owner's operation needs inference their own computer cannot run |
| files beside the record; the live channel beyond what U3 carries | the reference app that needs each |
| a second chat adapter, and the trait it earns | a group on another chat asks |
| compacting the journal | a journal grows past a size measured to matter |
| a keeper the group chooses, or one that rotates (Ostrom's third principle, collective choice; §11 risk 2): today the keeper is the maker's key, moved only by the keeper's own hand-on (§6, "Removal has one decider"), and a hand-on survives its giver's removal (§10, 2026-10-12); a group's choice would be acts its people sign that the fold counts into one hand-on, or a turn the fold moves on its own — a new rule of §6, the operator's | a group asks to choose its keeper, or to take turns, counted |
| a ladder of sanctions people can see, removal its last rung (Ostrom's fifth principle, graduated sanctions): the gentler steps are each owner's already and need no keeper — a yes stopped (`question.js`'s Stop, on both pages), a share stopped (`svrngs unshare`), a lend kept back from one person (v1-onboarding (e), queued) — so the ladder is only those steps shown to the group beside its removals (§6, v1-keeper-office (b)) | a group removes someone for what a lesser step would have met, counted |
| exporting a group as one document (upstream's checkpoint route), checking the asker is in that group | a group refounds from a copy, or someone asks to export one |
| the endpoint's self-heal (upstream's `self_heal.rs`, `iroh_watchdog.rs`) | a node's endpoint wedges in a recorded run |
| skipping a round's members known offline. Dialling them at once landed (asleep-1, `ring_sync::run_one_round`), so an asleep one holds only its own dial; the round still ends when that dial gives up, about 10 s, and a door's ask begun meanwhile waits for the next round (§3, "Joining") | a round's measured time nears its interval, or a newcomer is refused while a round waits on an asleep computer, counted |
| another device for the same person. Scoped use asked (`demo/runs/su/2026-10-09/` §9): its run listed each of Ana's phones' keys in her taste's owners by hand (`apps/taste/check/check.sh:173`, `OWN`; `:137` at `305efde`), each phone joining as a member of its own, and a lend its lender's person alone reaches needs it (§8, "Then"); made production, Ana was asked once on her computer and once on each phone, and a stop on one phone left the other's yes, each choice its own key's (sup-4, `demo/runs/sup/2026-10-10/`) | that row is queued |
| the wire published for a second implementation | someone sets out to write one |
| local discovery on one WiFi | Apple's multicast entitlement is granted |
| any member's node redeeming a link, once its `Invite` has synced | a newcomer's link fails because its issuer's computer was asleep, counted |
| a link's life set per link, other than a day | a newcomer meets a lapsed link, counted |
| measuring a lender's upload rather than taking the figure they declare (BBR3's pacing estimate, or a timed saturating pull) | a declared figure is wrong by enough to stall a viewer in a recorded run |
| a per-process tie between a node and the services it lends (upstream's `mint_tie`). A middleware believes loopback: a process on the lender's computer naming a person's key at its port was answered as them (`demo/runs/mw/2026-10-08/mw-1/`), which is the lender's own (§3, "The room", call 6); and a request at a lend's origin with no `Sec-Fetch-*` is taken for that lend's own page (sup-1, `origin.rs` `app_of`), which neither widens nor narrows what a process could reach before | an app's act is written under a person by a process the lender did not run, or one computer is shared by people who do not trust each other |
| a synced folder's later parts, each with its trigger. A mounted folder: every file of a group's folder listed from the record's layout and fetched on open, by verified range, as group folders' streamed spike fetched (`demo/runs/gf/2026-10-09/`, its endpoint dropped at syp-1; `demo/runs/sy/2026-10-09/README.md` §9, "A mounted folder") — a file system of svrngs's (macFUSE, or Apple's File Provider), a dependency and a process apart. Its streamed spike's rows that only a folder not on the viewer's disk needs — a stub that reaches its viewer, who the endpoint answers, stubs in step with the record (that README's §9, rows 1, 3 and 4) — wait with it. A version as its blocks: a file a hash sequence of its 1 MiB blocks (iroh-blobs' `HashSeq`), the record's entry naming the sequence, so an edit moves only the blocks it changed — a 1 MiB edit inside 100 MiB moved all 104,857,600 bytes to each member on every path, before perf-2 and after it, in 1.9 s on loopback, and again on production in 7.04 s through n0's (syp-5, `demo/runs/syp/2026-10-10/rerun/README.md` §5), and what an entry's hash names would change in the record and in what a store keeps (perf-1, `demo/runs/perf/2026-10-10/README.md` §9 T4, §10 candidate 8; perf-2, `demo/runs/perf/2026-10-10-make-it-fast/README.md` §3 T4; the operator, "Wait, revisit after the rerun", §10). A file past one holder's upload: a 100 MiB file reached each of two members at 83 MB/s at once, about 167 MB/s from its one holder, where one member alone took it at 139 MB/s, a bare iroh-blobs get's rate (that README §4) — a member fetching from another that holds the file is unmeasured. A joining member's wait to stand: its share refused until the round admits it, 54-58 s of the round's 60 s, where its every file then arrived in 1.7 s of 10,000 and 61.5 s of 100,000 (that README §5) — perf-2's harness's, members started apart; a member joining by `join` stood before its share, and held 1,000 files 0.53 s after it (syp-5's rerun, §5). Dialling direct before a relay: no gain measured, iroh having moved every connection on all three paths direct before its first fetch, in perf-1 and perf-2 alike (perf-1's README §9; perf-2's §3) | the mounted folder: a member whose disk cannot hold the group's folder, counted. The blocks: a member waiting on a large file moved whole for a small edit, counted, or the operator queues T4. A file past one holder: a member waiting on a large file several fetch at once, counted between computers. The wait to stand: a newcomer's folder waiting on the round past a bar the operator sets. The dial: a fetch measured through a relay between two members a direct path reaches |
| a person who only searches counted apart from one who watches: the door counts each on a lend for `WINDOW`, 30 s, after their last request (`crates/svrngs-node/src/door/limit.rs:17-19`), a search as a film — Movie Night's searcher was counted on both Jellyfins 26.3 and 26.5 s after one search and not at 36.2 and 36.4 s, the most at once 2, nobody refused (`demo/runs/mn/2026-10-08/doors.json`, `limit`) | a lend at its limit refuses someone while those it counts only searched, counted |
| budgets on an owner's operation: how often an app may ask it, so asks cannot rebuild what it keeps back (`rings/current/ACL_AND_SCOPED_USE.md` §4, §6). Ana's taste ranks whatever films it is given, and ranked an app's two that sent her yes itself (`demo/runs/su/2026-10-09/verdicts.json`, "the unasked yes") | an operation's answers rebuild what its owner kept back, shown in a run, or an owner asks for a limit on one |
| a live payload naming its sender | an app needs to know who offered a live payload, and cannot take it from the payload |
| WebKit's closed-range seeking through a lend: its seeks back to back, taken in serial 64 KiB reads of each short part, a round trip each, missed 5 s in 1 of 8 tries after u5-11, where straight at the canary over the same link they missed in none (`demo/runs/u5/2026-10-06-seek-stall/`; §11 risk 4); a rate in `scripts/check.sh`, not a gate. And Chromium's on a loaded host: its seeks are a gate, but sup-4's check, sy-1's three on the branch `folders` and REVIEW-audit-sup-auto-2's each missed only them, at load averages of about 40-130 on this host's 12 cores, with nothing near them changed, and passed alone; since chk-1 a miss past 3 a core (`conformance.rs` `BUSY`) is could-not-judge, never passed (operator, 2026-10-10) | Demo A's real-phone sessions miss the seek bar in Safari more often than WebKit straight at the server on the same link; or Chromium's seeks are could-not-judge in more `scripts/check.sh` runs than they are judged, counted |
| our own relay as the default: `iroh-relay` (Let's Encrypt built in) on a small VPS, the operator inclined to DigitalOcean, US West, at `relay.svrnme.sh` (TCP 80 and 443, UDP 7842), carried in every link's token so a phone never needs n0's; `svrngs new --relay <url>` for a group that runs its own; n0's terms for production use read first. Open then: whether ours also drops n0's address lookup, computers learning each other's relay from the group's record. Until then n0's public relays and lookup, `RelayConfig::default()` (`crates/svrngs-node/src/guest.rs:66-76`), nothing paid for (operator, 2026-10-05). Every byte a phone sends crosses a relay (a browser is relay-only), so a host's bandwidth price decides it | svrngs goes to production, or a group beyond the operator's uses it |
| the floor: every app under `/l/<label>/` of one address apart from the page's, on any static host, the link carrying that address after its token (a second address given to `svrngs new`, u5-2; dropped by u6-00). Its cost was one origin for every app, so a lent page could read what another kept and use another's pass while it lasted, and its every app lost its root: the catalogue run lent five self-hosted apps on it in Chromium and WebKit and none got past its sign-in, each asking for `/`-absolute paths the floor's host answered 404, and Paperless's post refused by its own origin check (`demo/runs/catalogue/2026-10-05/RESULTS.md`, "The floor"; `003e0d5`) | a group whose host answers no name nobody made asks for an app that keeps to relative paths, counted |

## 10. Decisions

By the operator, 2026-10-10 and 2026-10-11, on the scene v1 answers to
(`demo/c.md`):
- "Yeah write the scene that we can fulfill and the Demo C run of show":
  Demo C, the group's whole life, after U7, onboarding and the two homes
  (§8); its bars the seat's proposal, the operator's to set.
- on a chat, "Hopefully not more the 50 lines in our own source and a pretty
  commodity integration", then "Rewrite that way": a chat's door is the
  group's link (§3, §4); a chat's own door waits (§9).
- the phone replica: "I don't want to distract us from the core right now,
  but wanna keep it on the back burner" (§9).
- S3's two holes, "Queue it now", fixed before the merge (syp-6); the
  branch `folders` merged into main by the operator, 2026-10-11 (`4475a89`).
- onboarding by a download script: "it's the most cross platform and
  requires the least machinery. We're a lazy but brilliant dev team; we
  want the most general purpose solutions possible"; and of Demo C's gate,
  "Pasting one line in the terminal is fine" — a computer's newcomer pastes
  one line the group's page gives, and types nothing else (§8, "Demo C");
  the route, the page's host serving the script and the binaries beside
  the page, is §7's ("A computer gets svrngs from the group's link").

By the operator, 2026-10-12, "Yes that sounds good", on the seat's reading
of Ostrom: the hand-on is a collective-choice act, a removal operational
enforcement, and enforcement never unmakes an office already given — "the
office isn't the person". So a hand-on survives the removal of the one who
handed on (§6, "Removal has one decider"); every removal and every ask to
remove is shown to everyone in the group (§8); and a keeper the group
chooses, and a ladder of sanctions, are written to wait (§9).

By the operator, 2026-10-10, on perf-1's candidates for a fast synced
folder (`demo/runs/perf/2026-10-10/README.md` §10):
- "One row after syp-4 (a)-(e)" for candidates 1-6: one get a file, the
  apply's match by path, the record read once a change, the apply woken by
  the record, files taken at once, fetches at once — perf-2-make-it-fast,
  fetches at once moved into it from syp-4 (f), since before the record is
  read once a change they queue at the holder's one task.
- "Yes, in the same row" for candidate 7: the step woken by the file
  system's events, through the dependency `notify` (8.2.0); any other new
  crate is still the operator's.
- "Wait, revisit after the rerun" for candidate 8, a version as its blocks
  (§9 keeps it waiting, with its trigger).

By the operator, 2026-10-10, before the synced folder made production
(`ralph/next/lean/SURVEY.md`):
- "Queue them increase the budget, but let's also see if there are
  opportunities to economize": the lean rows cut what the product does not
  carry; main's cap stays 15,350, and the branch `folders`' may rise to
  16,500 at most while it merges main in (syp-3), coming back to main's
  with the last merge.
- the budget counts the product: `demo/runs/` apart, as `apps/` and tests
  are, a recorded run's driver kept as it ran (lean-1, §5).
- a group's folder is synced onto its members' disks, not streamed: group
  folders' loopback endpoint and its one-way share are dropped on the branch
  `folders` (syp-1), and watching a member's films stays their lent
  Jellyfin's. A folder fetched on open, by verified range, waits for a member
  whose disk cannot hold the group's folder (§9).

By the operator, 2026-10-09, during scoped use made production:
- group folders' second spike runs now, in a worktree beside scoped use,
  not after it (§8, §9): a spike's code lives on its own branch, and how
  folders become production is decided from its verdict.
- a group's folder is more than its films in one Jellyfin: a folder synced
  both ways on its members' disks, written by any of them, is spiked next
  on the same branch, before group folders' production rows (§8, §9).

By the operator, 2026-10-09, before scoped use made production:
- a computer is the main way in, and a phone its teaser and onboarding,
  for now: a piece is built for a computer first, a phone beside it.
- a person's yes to an app is said on a page no app runs at, never by
  the app. On a computer that page is the home page,
  `http://svrngs.localhost:<port>/`, served by the person's own node:
  their groups, what each lends, opened from there, and what apps use of
  theirs. It answers only itself and is never framed. Its writes are the
  person's own choices; sharing, links and removing someone stay in the
  command until a row moves each. The repository's budget rose from
  14,750 to 15,200 for it (§5).

By the operator, 2026-10-08, after the middleware spike:
- the library carries the weight. The door takes the middleware's general
  pieces as hooks a lend declares over HTTP — a credential per person,
  that credential kept out of answers, a script first in the lend's pages
  — and an app's middleware keeps only its own nouns; before group folders
  (§8, "Then"; §3, "The room", call 6). The repository's budget rose
  from 14,000 to 14,400 for it (§5).

By the operator, 2026-10-06:
- a phone opens each app at an address of its own under the page's host,
  and only so: the floor, every app under one address apart from the
  page's, is dropped (u6-00), since the catalogue run found it carried none
  of five self-hosted apps, each asking for its own root
  (`demo/runs/catalogue/2026-10-05/`), and the repository stood at 13,952
  of 14,000 lines with the room door to come. `svrngs new` refuses a page
  whose host does not serve the page at a name nobody made, saying what
  such a host must do (§3, "Lent apps", call 1; §7; §9 has the trigger);
- middleware is a spike after Demo B, and no U6 row builds it: a lent app
  that forwards to another, run only by its lender, with svrngs shipping no
  runtime or SDK for it (§3, "The room", call 6, with its invariants;
  "or plugin format" dropped by the operator, 2026-10-08, above). Its
  spike, 2026-10-08, was kept by its bars read for their
  spirit, as the operator read them that day ("the bars are guidance to
  align us at the start, the spirit of them are what guides how we
  iterate"): Jellyfin's zero sign-in is a middleware in `apps/jellyfin/`
  (§3, "Lent apps", call 4; `demo/runs/mw/2026-10-08/`).

By the operator, 2026-10-05, after five reads on a real TV
(`demo/runs/u6/2026-10-05-screen/reads.json`):
- the screen is the app's own client, and svrngs gives the devices in a
  room reach to a lend through a member's computer on that network ("we're
  adding value, but we're not taking media on tvs and end user devices
  into our scope"); nothing in svrngs knows AirPlay, Cast, DLNA, a TV or a
  player, and AirPlay and Cast from a page wait in §9 (§3, "The room",
  calls 1 to 4). This replaces the screen handoff of 2026-10-03, below;
- old TV browsers are not supported: the LG's Chrome 87 could not open the
  page ("a losing cause");
- the room's address comes from a member's computer on the screen's
  network, not from a phone and not from a public address per lend (§9);
- Demo B: five friends at Dee's, Sam's library across town, the people in
  the room on the TV's WiFi; its gate unchanged (§3, "The room", call 5;
  §8).

By the operator, 2026-10-05:
- north star 2 holds for the apps people already self-host: `svrngs share
  8096` lends Jellyfin as it comes, and someone who never used it opens it
  from the list, plays and seeks, on a computer or a phone. Films is dropped,
  and Demo A plays in stock Jellyfin; group folders are an experiment after
  U5, settled in the wild. The seven calls that build it are §3's "Lent
  apps, as they come" (over "Demo A plays in Films", 2026-10-04, §11
  risk 7);
- the ceiling's fixes 1–3 — the digest, signatures checked once, the
  register, each kept by a tail fold — after Films (read here as after
  Demo B, §8's rule); dialling at once stays a §9 row;
- the scale instrument is promoted as an ignored test
  (`crates/svrngs-node/tests/scale.rs`);
- scale is settled from first principles and the instrument on one
  computer, with no other machines;
- production gets logging and telemetry enough to see whether a group
  degrades as feared (a shape is proposed in svrn-docs
  `rings/current/THE_CEILING.md`, "Seeing it in production").

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
  first, through the browser; chat doors are onboarding, built last. (The
  experience is the group's films in stock Jellyfin since 2026-10-05,
  above; Films is dropped. The screen handoff is reach for a room's
  devices since 2026-10-05, above. A chat's door is the group's link since
  2026-10-10, above.)

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
   That range was closed, to the film's end (`bytes=66394-45245597`):
   WebKit probes `bytes=0-1` and then asks on to the end, where Chromium
   asks `bytes=N-` and cuts it. Since u4-9 the worker asks the door for
   either as a part of 2 MiB (`web/sw.js` `PART`), so a seek waits at most
   for the part being read: in the dry run again
   (`demo/runs/u4/2026-10-05-reach/`), every WebKit seek played on within
   3.4 s through n0 and 0.8 s locally, the first asked within 0.9 s of
   play. The worker's trace (u5-11) shows WebKit's cancel reaching the
   worker's stream, and with parts no longer bounded WebKit read its ranges
   through again (`demo/runs/u5/2026-10-06-seek-stall/`,
   `tried-closed-whole`). Seeking back to back on a phone's link — 300 ms
   round trip, a third of a 50 Mbit/s uplink, a second queued — the last
   of twenty seeks in WebKit through a lend played again past 5 s in 1 of
   8 tries after u5-11 (4 of 8 before), and straight at the same canary
   over the same link in none of 8 (1.9–4.2 s): through a lend WebKit
   answers each short part with serial 64 KiB reads, one round trip each.
   Chromium's played again there in 0.7–0.8 s (4.0–5.0 s before), a gate
   in `scripts/check.sh` save on a host past 3 a core's load, where a miss
   is could-not-judge (chk-1); WebKit's is a rate there, beside the same seeks
   straight at the canary each run, and waits in §9.
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
   build's dotless too. In the dry run again
   (`demo/runs/u4/2026-10-05-reach/`), WebKit joined through n0 in all
   three runs. If iroh spells a relay some other way WebKit refuses, an
   iPhone fails again, and only a run through n0 in WebKit shows it. The
   page a phone loads from a link's origin is the same
   kind of default (U4): whoever serves it serves the code that holds the
   person's key, its bytes unsealed while the page runs, so a group that does not trust svrnme.sh serves the page from
   its own origin and prints its links there. And every lent service's
   pages run on that origin too (u4-6, call 3: a lend's address is under
   the worker's scope), so a script in a lent page can open the page's
   store and unseal the key of whoever opens it: in u4-6's test, a script
   on `/l/<ada>/page/` imported `store.js` and read bo's 32 bytes. So
   whoever lent a service, or a service compromised, could act as each
   person in the group who opened it. Since u5-2 each lend opens at an
   origin apart from the page's store, with a pass for that lend and the
   lends it reads, never the key (§3, "Lent apps", calls 1 and 6; §7), and
   u4-6's measurement is kept as a test of the conformance canary
   (`crates/svrngs-node/tests/conformance.rs`). What is left: the page's
   host still serves the code that holds the key.
6. **Two copies of the record's code.** A signing or canonical-form bug fixed on
   one side can be missed on the other. `scripts/ports.py` is the guard, and it
   is a listing, not a gate.
7. **Jellyfin's own player asks for a sign-in** — reopened 2026-10-05. The door injects a
   credential on requests, not a session in a browser, so either the browser
   is handed a token it then holds, which breaks "the lender's secret appears
   in no byte a member receives", or each person signs in to a viewer
   account. U3's dry run against Jellyfin 12.0.0
   (`demo/runs/a/dry-2026-10-04/`) found the second worse than feared: a
   member's sign-in through the door logs the lend's token out for everyone,
   and only a placeholder sign-in that no member can write gets past the
   page. The operator, 2026-10-04, put Demo A in Films, after U5, which
   never needed that page; on 2026-10-05 dropped Films for Jellyfin's own
   player through the lend (§3, "Lent apps, as they come"). u5-3 decided
   sign in once (§3, "Lent apps", call 4): a person signs in once with an
   account the lender made, and nothing the lend holds reaches them. u5-4
   built it, which closes this risk: the lend keeps no secret, and with
   one planted each way the check failed, then passed in both engines
   (`demo/runs/u5/2026-10-05-jellyfin-in/`).
