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
else: the person asking, verified, in request headers; a shared record that
outlives the computer it runs on; and reach to every member's computer. No
login, no SDK, no bundle. The headers are three, each with one use: who
asks is `X-Svrngs-Key`, their key; the name to show is `X-Svrngs-Person`,
which two people may share, so nothing is keyed on it; an account's name
is `X-Svrngs-User`, their name and the start of their key, so it never
collides. A key is who someone is and a name is a claim — canon's rule for
its own ledger too (`~/dev/canon` `a6960fc`, `docs/SIGNING_DESIGN.md`).

**And the apps already exist.** Everything people self-host is a web app on a
port: Jellyfin and Navidrome for the library, Immich for photos,
Audiobookshelf and Calibre-Web for books, Nextcloud and Paperless-ngx for
files, Actual for money, Mealie and Grocy for plans, Home Assistant, and
llama-server for a model. Run one in a container on any member's computer,
`svrngs share <port> --kind <name>`, and everyone in the group can open it —
no port opened, no tunnel, nothing for them to install. A friend's model is
lent as any app is: `svrngs share <port> --kind llama-server`, the
program's own name, and everyone opens its chat page or gives a program its
address, while svrngs keeps back, with nothing typed, what would show one
person's chats to the others (§3, "Lent apps, as they come"). An app that takes the signed-in person from a request
header lets each of them straight in once its lender names ours,
`X-Svrngs-User`, in its settings, and `svrngs share` knows two by their
ports and prints what to set: `svrngs share 4533` lends Navidrome, asks no
password and says `ND_EXTAUTH_USERHEADER=X-Svrngs-User` and
`ND_EXTAUTH_TRUSTEDSOURCES=127.0.0.1/32`; `svrngs share 8083`, Calibre-Web,
its "Allow Reverse Proxy Authentication" and header name
(`crates/svrngs-node/src/well_known.txt`; whether Calibre-Web signs in a name it
holds no account for is read from its settings, not run). That header is
an account's name unique by the person's key, so two people of one name
are two accounts and the lender's own are never theirs (bash-person; Demo
C's dry run signed a second `ana` in as Ana, Navidrome's admin, by the
name, `demo/runs/c/dry-2026-10-12/data/name.json`). One that keeps its own
accounts asks them once; Actual asks for the one password its lender set. Each serves its pages from its
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
for the next developer to copy and bend: Money, a group's shared book
(`apps/expenses/`); Events, a post and who came (`apps/events/`); and
the grapevine, an ask carried from group to group by the people in both,
one tap a group, its replies sealed to the two ends, where a developer
adds a kind of ask in one file (`apps/grapevine/`, its kind
`kinds/rooms.js`). Never the finished apps — the
community builds those (operator, 2026-10-03). "The iOS of groups" is an
analogy for being the thing others build on, not a plan to ship an iPhone app
first.

**Three rules for anything a person reads.** No universe to explain: a person
sees their group's name, people's names, apps' names, and *computer*, *phone*,
*the chat* — never ring, member, key, sync, journal, node or relay. Whoever
runs a relay is the one reader of that word: `svrngs relay` and its
flags speak to them of a name, ports and a certificate, and of the
computers and phones it lets through (§3, "A relay anyone runs"); so does
`new --relay`, which names one. And the
most general surface with the fewest features: the system learns by adding
apps, never by adding platform. And nothing imitates anything else (the
operator, 2026-10-14): a page brings what a browser does best — the
system's own fonts and colours, its own elements, plain rules — and its
warmth is colour, by the names the browser knows, never paper, a hand or
a stamp. Both pages wear one look, `crates/svrngs-node/src/home/look.css`,
which the group's page is given as it is given `question.js`.

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

**An app is a localhost service** — an *rngapp*. A member lends one from
their home page, under "Already running here", or with `svrngs share
<port>`. Every other member opens it through the **member door**,
tunnelled over iroh to the lender's node, which strips any identity header the
client sent and adds the verified one. A lent service — Jellyfin, a svrn
daemon's web UI — is an app like any other; what the group may do with it is a
lend declaration, kept as data: a secret the door injects and never lets out, the
paths it refuses, how many people may use it at once.

**Or a folder the group carries** (v1ui-shelf (a); svrn-docs
`rings/current/COMMUNITY_APPS.md` §6-§7, as `V1_UI_BUILD.md` §3.4 narrows it).
A lend goes with its lender; a carried app stays with the group. It is a
folder `apps/<kind>/` holding an `index.html` in one of the group's synced
folders, which already puts its files on every member's computer by hash. Each
computer that keeps such a folder in step lends the app as its own, to that
group alone, named `<kind>-<the group id's first 16 letters>` so two groups'
apps of one name have two origins (`crates/svrngs-node/src/lend.rs`,
`Source::Carried`); so it opens on any of them, and on a phone through any of
them, while that one is on. Both pages show one copy of each
(`question.js`, `once`); every copy is listed, so a phone knows each computer
it can reach the group through. svrngs runs none of its code: the door answers
it from the store (`crates/svrngs-node/src/carried.rs`) — a file by its hash in
the version that runs as the person asking (below), `index.html` for a
directory's, fetched from a computer of the group's when this one never held
it — and from **its route**, `/svrngs/acts` at its own origin: `GET` answers its own
acts, each whose `kind` begins `<kind>/`, the prefix taken off, and each
correction of one, as `ops` in the shape `/v1/rail/log` gives them, voided ones
too, so an app's fold reads them as it reads the record (Events' decides whose
correction counts), with `you`, the group's id as `group`, its `people` now, and `spans`: each
person's time in the group, one `{person, joined, left}` for each stretch they
stood, so a house's rules count a past act with whoever stood when it was
written (canon-spans). The stretches come from svrngs-core's membership
(`Membership::spans`), the one decider of who stood when. `joined` is the time
on the act that admitted them, null for whoever started the group. `left` is
the time on the cut of their last key, null while one stands. Both are stamps
on acts, never a computer's clock, so every copy that holds the acts says the
same. A person who leaves and comes back does so by a new key, so they have two
stretches. `spans` is null while who stands is unsettled (#124), never empty;
`POST {op}` records one and `{corrects, op}` corrects one of its own, `op`
null to void it. svrngs puts the prefix on, and writes `on_behalf_of` as the
person the door resolved, never a word of the request's body; a correction of
another app's act is refused. Each act stays the writing computer's, signed by
its key, so its `person` says whose computer wrote it and `on_behalf_of` whom
it speaks for: Money's, Events' and grapevine's folds keep both, `by` and
`via`, and their pages say an entry another person's computer wrote as "bo,
through cy’s computer", the phone's own words (gv-via; §9). The route
answers every act each time, from no cursor, since a round may place an act
before one already read and a correction voids one read before. Every answer carries the containment floor's
headers (COMMUNITY_APPS §6.2): a policy that lets the page load and connect
only to its own origin, no camera, microphone or location, no DNS prefetch.
It comes to a group from the shelf (below, the home page's "brings things to
the table") or as any folder dropped into the group's folder under `apps/`.

**House rules** is the carried app where a house decides its rules
(canon-house; svrn-docs `rings/current/CANON_IN_SVRNGS.md`). It is
`apps/canon/`, brought as `House-rules`, and canon-core 0.5.0 built to plain
JavaScript decides every answer on it: what is a rule, what waits, who has a
say (`apps/canon/README.md`). svrngs carries its entries and vouches for who
wrote them; it neither reads nor enforces a rule. The house's people are the
house: from `spans`, every computer builds in memory canon's own grants of a
say, a `house:<group>` grant at each `joined` and a withdrawal at each
`left`, and records none of them. The first person to open it answers "How
does your house decide its rules?", and that answer is itself a rule the
house can change.

**What runs as you is what your group brought, and each new version is your
own yes** (canon-pin; svrn-docs `rings/current/WHAT_RUNS_AS_YOU.md` §3, §8).
Bringing pins it: the folder acts that place an app's files name their
hashes, and each path under `apps/<kind>/` as the record first placed it is
the app as it was brought (`svrngs_core::Runs`, folded in
`crates/svrngs-core/src/synced.rs` alike on every copy). A member's change to
its files is a new version, by its files' hashes, and runs as nobody until
they say yes. Both pages ask, under "Bring things to the table": "kim
changed Money's files on 16 Oct. Open the new version?" The yes (`POST
/runs`, on the home page and through the computer a phone reaches) records a
`RailAct::Runs` naming the app's files placed now, as the person's: its
`on_behalf_of`, else its writer's, so a yes through another member's
computer is that computer's word for the person, as every act it writes for
a phone is. Until a person says yes, the version they last said yes to, else
the one brought, keeps running as them, on their computers and phones
alike, and every computer keeps and serves each version someone runs
(`Runs::of`, the one decider). An app brought before the pin landed is
pinned at its first files, nothing migrated; a change made before then is a
new version like any later one. Whether a new version also waits for the
house's agreement is a house rule: canon may configure a house's
(WHAT_RUNS_AS_YOU §8). svrngs keeps the pin and each person's yes.
Not yet: an inline `<script>` (svrngs computes no hashes of them), bytes from
the route, and reading another app's acts.

**What an app gets — the whole surface, for our apps and a stranger's alike:**
- the person asking, verified, in four headers the door sets
  (`crates/svrngs-node/src/door.rs`, `Identity`): `x-svrngs-key`, who
  asks, the person the connection's key stands as, by their first key in
  the group — the same from every device of theirs (bash-devices), an id
  and not a key to seal to; `x-svrngs-person`, the name to
  show, their name in the register, never unique; `x-svrngs-user`, the
  name of their account in an app that makes accounts, their name kept to
  ASCII letters, digits, `-` and `.` and the first 16 hex of their person
  key (`user_name`; bash-person, so a second `ana` is never Ana's account,
  and Ana's devices are one account); `x-svrngs-device`, the device
  asking, by the key it stands in the group by — one of the person's keys,
  so an app that holds something to a device tells a person's devices
  apart under the one account (bash-two-at-once: Jellyfin listed Ana's
  laptop and phone as one device, its film overwritten, while each
  played its own, `demo/runs/two-at-once/2026-10-12/`) —
  and the app asking in another (`x-svrngs-app`), read only from a pass (sup-1): on a
  phone the pass's `app`, the lend whose pages the group's page handed it
  to; on a computer the app its origin names from what the browser says
  of the page ("Lent apps", call 6), carried to another computer's door
  in a pass that computer signs for itself; none for a key of the group's
  asking with no pass;
- where it is lent, in two headers more (events-links): `x-svrngs-label`,
  the lend's label, and `x-svrngs-pages`, the page of each group held on
  the lender's computer that the key asking stands in, `<group>=<page>`
  apart by a space, the page `none` when the group keeps none and `unread`
  when it cannot be read. Every such group, as no request says which it is
  for: a lend is lent to every group its lender's computer holds, and that
  computer's key stands in each, a phone's in one. From them an app writes
  a link into itself, `<page>#open=<label>&at=<path>`, which the group's
  page opens on a phone kept in a group of the lender's (`web/app.js`),
  and a computer's home page opens pasted (below); that it opens for the
  group's people alone is the app's check of who asks, as on any request;
- the people in the group, one row per key, each key one an app may seal
  to, with its person's first key, which every key of theirs shares;
- the record, over a loopback API on the member's own node: append, read,
  live — each live payload drained with who sent it, the key that dialled
  in and its person's first key, as the people above name them, so an app
  acts on who spoke without signing every payload (onair-live; svrn-docs
  `rings/current/ON_AIR.md` §4.3); still offered by the sender to each
  member online now, and kept nowhere;
- a signature for an app, checkable anywhere (onair-sign; svrn-docs
  `rings/current/ON_AIR.md` §4.2): `POST /v1/sign?app=<app>` on the same
  loopback API has the computer's own key, the one that signs its acts,
  sign the body's bytes, and answers `{key, sig}` in hex. Nothing says the
  key is a person's: to someone outside the group it is the computer's. What
  is signed is `"svrngs-app:" || app || ":" || bytes`, the app's name 1 to
  64 of `a`-`z`, `0`-`9` and `-`, so the `:` ends it
  (`crates/svrngs-core/src/app.rs`). That domain begins no other the core
  signs or hashes under (`sig.rs`), so an app's signature is never an act's,
  a link's, a pass's or another app's. A stranger checks one with the key,
  the app's name and the bytes alone, as the on-air page does
  (`apps/on-air/page/air.mjs`, `signedBytes` and `checkLine`, with
  `@noble/curves`):

  ```js
  const signed = concatBytes(utf8ToBytes(`svrngs-app:${app}:`), bytes);
  const ok = ed25519.verify(hexToBytes(sig), signed, hexToBytes(key));
  ```

  A page in a browser is refused it, as it is the record;
- bytes by hash, for an app, with a lifetime (onair-blob; svrn-docs
  `rings/current/ON_AIR.md` §4.1), on the same loopback API: `PUT
  /v1/blob?group=<id>&keep=<seconds>` puts the body in this computer's
  store and answers `{hash}`, its BLAKE3 in hex; `GET
  /v1/blob/<hash>?group=<id>` answers the bytes from that store, else
  fetched, verified, from a computer of the group holding them, and names
  the computer they came from in `x-svrngs-holder`. They move as a
  group's folder's do (below, the group's folder): one store, one
  provider whose one rule serves a hash to a dialer standing in a group a
  synced folder or an app placed it in, one verified fetch (`folder.rs`,
  `blob.rs`). Each hash lapses `keep` seconds after its put, on every
  computer holding it: a computer asked for one first asks the group's
  other computers what each holds of it and for how long yet, fetches it
  from those, holds it no longer than they do, and serves it to the group
  meanwhile, so each member that asked is a carrier for the rest. A get
  refused says which: `lapsed` (410) if any computer asked says so,
  `never_held` (404) only if every one does, else `unanswered` (502),
  with each computer asked and what it said — never empty bytes. Bytes an
  app places are not the record and not a folder (§6). Carrying spends
  the carrier's upload, uncapped: in the run, a station's computer sent
  about one and a half times its show for two listeners, and a listener
  up to half of it again (`demo/runs/onair/2026-10-13-blob/README.md`,
  "Carrying"); a cap, on the home page beside a lend's limit, is a row
  only when a run shows one needed;
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
  resolves (`door::PersonKey`), their first key in the group, so a yes
  said on one device of theirs holds on every one (bash-devices);
  `svrngs unshare` takes the choices with the lend;
- the home page, `http://svrngs.localhost:<port>/`, at the same address
  each time svrngs starts (§7, start-address): the person's own page on
  their computer, written for someone a bit afraid technically
  (start-home, `demo/runs/start/2026-10-13-home/`), in §1's words. At its
  top it says what it is for and what can be done on it. With no group
  yet, it says to make one there or join one from a link. Each group is
  shown as soon as its people are read, and what its people lend is added
  once each computer has answered or not, so a computer asleep holds back
  only that list (sup-2, `crates/svrngs-node/src/home.rs`; read from the
  routes that answer an app, `/v1/groups`, `/v1/lends`,
  `/v1/rail/people`). A group that cannot be read is said so, and so is
  whose computer did not answer. Each group is headed by its name, who
  made it and since when this computer has held it, so two of one name
  are told apart (#138), in a colour derived from its id, nothing to
  choose (look.css; the operator, "Derived from the group"). Under it
  (v1ui-door (c), V1_UI_BUILD §3.1):
  - **the door**, each person once, a cell in a grid: lit, "on now",
    while their computer answers; drawn off, "did not answer", when it
    did not, judged within the one round that asks what each lends and
    never lit before; and "in from a phone" when their only keys are a
    browser's (`device`, `/v1/rail/people`), whose presence nobody asks.
    Presence is only "on now", never when last seen;
  - **the index**, one table of what is in the group, on any port: each
    thing by its link; what it is, in its lender's word, its kind, never
    one of svrngs's own (`scripts/vocab_check.py` reads the pages'
    scripts too); whose; how it opens — in a tab, signing you in once
    when its lender's program does (`signs_in`) — its address for a
    program, and on the TV too while the room door serves it; and on or
    full now, how it opens and whether it is full judged once for both
    pages (`question.js`, `opens`; #158). Each computer that did not say what it lends is a row of
    its own, by how, so nothing is stale and nobody is left out: one
    running an older svrngs among them, whose answer carries an older
    version or none, its lends not read field by field
    (`crates/svrngs-node/src/version.rs`; arch-version).

  The group's page on a phone draws the same door and index from the same
  lines (`question.js`; v1ui-door (d)), judged by the answer of the
  computer it reached: `/people/lends` answers the people standing beside
  what they lend, and whose computer did not answer that one is drawn
  off. One version check, the node's `version.rs` compiled into the
  page, stands for the per-field fallbacks before it (#159, #162;
  arch-version): a computer the page reaches whose answer carries an
  older version, or none, is said once to run an older svrngs, and
  nothing of its answer is drawn. Each thing opens by its button, here, in the phone's tab; and the
  group is headed by who made it and since when the phone joined it,
  `maker` and `since`, kept at the join (`web/store.js`), nothing for a
  group kept before.

  On it a person:
  - opens each thing lent by a link that says what it is and whose ("Open
    Sam’s llama-server"), in a tab of its own beside the page;
  - copies the address to give a program for it, such as a chat program
    pointed at a friend's model, by a button that says so;
  - opens a link to a lent app someone sent in a group's chat,
    `<page>#open=<label>&at=<path>`, pasted under "Open a link from your
    group": the lend it names opens at that path in a tab of its own, as
    the group's page opens it on a phone (events-links (c)). The group's
    page itself, opened with such a link in a computer's browser that
    keeps no group, says the link opens in svrngs there and shows it to
    copy, since it cannot know the computer's port;
  - makes a group, by its name and theirs, made by the svrngs running
    there as `svrngs new` makes one (start-group, the node's `group.rs`
    `Asking::found`), so a computer whose svrngs runs from login, where
    `svrngs new` is refused, makes one with no terminal. A name one of
    their groups has already makes another group all the same, by the
    one rule `svrngs new` asks too (`group::namesakes`, #138; the
    operator, "Allow; tell them apart"): the page says so as the name is
    typed (`GET /group?name=`) and again once it is made, as the command
    says it when it makes one — "You’re already in Trip (Sam’s, since 3
    Oct). This makes a second group called Trip." — each held group named
    by who made it, "yours" when this computer did, and since when this
    computer has held it, the time its seed was written: the sentence
    `svrngs new` prints to the letter, its day as the command writes one
    (question.js `namesakes`, cli.rs `namesakes_said`; #157, pinned by
    crates/svrngs/tests/home_group.rs). The page then
    says it was made, and places the group's folder where a joiner's
    is (`Folders::place`, one home for `new`, `join` and the page); a
    place another group's folder holds already, one of the same name or,
    on a disk that ignores case, of a name differing only in case, stays
    its, and this group's goes beside it with its id's start after its
    name, so no two groups share one. The
    group keeps the page this computer's svrngs came from, the data dir's
    `origin`, which install.sh writes, so the links made for it are https
    links whose page gives the one line; a computer set up by hand keeps
    none, and its links are `svrngs://join/…` alone, said beside them. It
    keeps the relay its links carry, as a joiner keeps it
    (`Host::made_relay`, the one rule `svrngs new` with no `--relay` asks
    too, `crates/svrngs/tests/new_relay.rs`): where this computer is met now, so a group made
    while svrngs runs is met with no restart, and after one at the same
    relay (`cli::reach` reads what it keeps; one of n0's, at n0's, is
    kept as none, met where the build meets its groups). Measured by
    `crates/svrngs/tests/home_group.rs`: the bare `svrngs` running, the
    page in Chromium and WebKit makes a group, its folder placed, and a
    link the page made for it lets another computer in, neither
    restarted;
  - joins another group from a link someone sent, pasted under "Join a
    group from a link", by the svrngs running there as `svrngs join`
    joins (arch-command (c); the node's `join.rs` `Host::join`, the
    one join both ask), so a computer whose svrngs runs from login, where
    `svrngs join` is refused, joins with no terminal. Before the button
    is pressed, the page asks the computer that sent the link what it is
    for (`GET /join?link=`, `Host::asked`) and the button says it:
    "Join Trip as Ana", the name the one typed, or the person a link to
    add a computer of theirs names. Pressed (`POST /join`), the group's
    folder is placed as a joiner's, and the page says "You’re in Trip, as
    Ana." and draws the group with the others; a refusal is said below
    the button in the page's words (`join::NotJoined`). The group's page,
    opened from such a link on a computer, says it can be pasted there
    instead of its line. Measured by `crates/svrngs/tests/home_group.rs`
    in Chromium and WebKit: the bare `svrngs` running in Lab, Sam's Trip
    joined from its link and drawn beside Lab, neither computer
    restarted;
  - makes a link to invite someone, or to add their phone or another
    computer, made by the node as `svrngs link` makes it (`invite.rs`
    `Host::link`). The page shows the link, with a button to copy it, and
    says for how long it works and that it closes the last one;
  - opens the group's folder, one `/v1/groups` lists, in the computer's
    file browser (`open`, else `xdg-open`);
  - lends what runs on their computer, under "Already running here"
    (v1ui-lend (b), V1_UI_BUILD §3.2), only when they press "Look for
    programs running on this computer", as often as they like, never as
    the page loads (v1ui-lend-ask (a): a look is seconds of the
    computer's work): every port from 1024 to 49151 that
    answers on 127.0.0.1, probed as `svrngs share` probes one
    (`share::listening`), less svrngs's own and what it lends already,
    each by its address, and less what signs people in to a lend — a port a
    lend here names its sign-in, or a kind's helper the table names — which
    only svrngs here asks, the page saying once why (arch-helper (b),
    `share::signs_in`, REVIEW_FINDINGS #174; lending one, from the page or
    by `share`, is refused); when what it lends, or the look, cannot be
    read, the page says which, and the trace its error (#160). A kind the well-known table knows is named, with
    what it needs first — its own accounts and how to make each person
    one, the one setting that tells it who is who, or the paths svrngs
    keeps back of it — and lent by one button. Any other is lent only
    once the person gives it a name and says what it is, each one word,
    the kinds the table knows by name alone offered as words to say. The
    lend is decided as `share` decides one (`share::lend_of`), refused
    when nothing answers there or a lend of its name stands, each refusal
    of what the page sent and of the lend's own said apart and traced
    (#161), declared
    (`lend::declare`) through the page's `POST /running`, and the page
    then says what was lent and what it needs;
  - brings things to the table (v1ui-shelf (c), V1_UI_BUILD §3.4): in each
    group, the shelf — svrngs's own apps, Money, Events and House rules,
    listed as data in `crates/svrngs-node/src/shelf.txt`, each line naming
    the files it brings beside its page and script, and embedded at build
    (`crates/svrngs-node/build.rs`), so svrngs's code names none; a carried
    kind is one word, and the pages say its `-` as a space — each
    not in the group yet ticked, and one button, "Bring the ticked ones to
    <group>", writing each into the group's folder on this computer
    (`POST /shelf`, carried.rs `bring`), never over one there already,
    which then carries it as it carries any file; the page says what it
    brought. The group's page on a phone draws the same shelf
    (`question.js`, `table`) and brings through the computer it reaches,
    on the members' protocol, whose door asks that the phone stands in the
    group;
  - changes how many people at once may use what their computer lends, a
    number or everyone, or stops lending it (bash-limit, §10). This is
    written to the lend's declaration, which the door reads at its next
    request;
  - says yes to an app, or stops it; what apps use is listed only when
    there is something in it.

  What the page cannot do it names under "Done in a terminal", with the
  one line to type: removing someone (`svrngs remove`). Each group you
  are in ends with "Leave <group>", its words saying before what stays on
  this computer, what stops, that coming back needs a new invitation and
  that this computer cannot come back yet; whoever keeps the order of
  removals is shown the hand-on beside it, a choice of who keeps it next
  (`POST /v1/leaves` and `/v1/handovers` on the page's own origin, the
  node's `removal.rs`; groups-leave (c)). A group you left is listed under
  "Groups you left", with what stays of it here.
  The page answers only itself, from its own origin, and never runs an
  app's code: its only writes are the yes and stop, what runs here lent,
  the change to a lend, the link, the folder opened and the group made. An app sends its person there:
  `/svrngs-reads` gives each lend that asks its words,
  `asks`, `ask`, to which it adds `&back=<its path>`, and `stop`, the
  address at its own origin it posts its own stop to (lean-4 (d)) — on a computer
  `…svrngs.localhost:<port>/ask?lend=<label>&app=<label>`, on a phone
  `<its origin>/svrngs-ask?lend=<label>&app=<label>`, which the lend's
  worker sends to the group's page at `#ask=<lend>&app=<label>&at=<path>`
  (sup-3, `web/lend.js` `asking`). Either page asks "Let Sam’s night use
  Ana’s taste?" around the lender's words, and lists what apps use, in
  one wording both load, a lend named as the bare command names it, with
  its kind when that differs from its name (lean-4 (c),
  `crates/svrngs-node/src/home/question.js`); that file walks the lends
  that ask too, each page fetching their choices by its own transport, and
  says an ask naming what is not lent there in one line, "That app, or
  what it asks to use, is not lent here now." (onair-economize (d)). On a computer a yes is kept by
  this computer's door for its own lend, else posted to the lender's door
  as this computer's key; on a phone the group's page posts it to the
  lender's door, dialled as the person; then the person goes back to the
  app at its path. An app stops itself, `POST` at `stop`,
  `/svrngs-reads/<label>/svrngs-stop` at its own origin; on a phone its
  worker sends that to the lend's door with the pass. The group's page
  lists what apps use too, asked of each lender when the person opens it,
  each yes with Stop. Neither page is ever framed;
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
  the home page's port, kept as it is (§7), which `/v1/lends` answers as
  each lend's `origin`; on a phone as §7 says,
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
group's ring, which the admitting computer starts once it writes the
`Admit` (bash-first-minute (b): a newcomer's computer listed every lend
1.05 s after its join, where it had waited 57 s on the issuer's 60 s
round, `demo/runs/first-minute/2026-10-12/`). A computer that has not
heard of them yet closes their computer's dial as not admitted, and is
said so — "has not heard of this computer yet" — never as one that did
not answer ((c)). A lender's door meeting their pass before its round asks
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
up to that dial's two seconds (ten before events-asleep capped the
transport's dials at `dial::DIAL_WITHIN`); read from the code, not run.

*A name binds once where a stranger gives one* (bash-person; the
charter's "The name", `ralph/next/bash/CHARTER.md`): a key is who someone
is, a name is a claim, and a claim on a name someone already holds counts
only with that someone's key — canon's rule for its own ledger (`~/dev/canon`
`a6960fc`, `docs/SIGNING_DESIGN.md` §2, "A name binds once"). A link lets
no one in under a name a standing person of the group goes by, compared
case-blind: the `Redeem` command refuses it at the issuer's write, before
anything is written (`crates/svrngs-core/src/register.rs`, `NameTaken`),
and not the fold, where "first in the rail's order binds" would let any
member's computer evict a person by a backdated `Admit` under their name.
The joiner is told who goes by it, how to add this device if it is
theirs — `svrngs link <group> --device` on a computer of theirs — and how
to join under another name — `--as <your name>` after the link on a
computer, the page's line included; another name typed on a phone, which
the page asks again — and a removed person's name is free again.
Squatting a name before its person joins stays possible; the keeper can
remove the squatter.

*A person's other device joins by a link of theirs* (bash-devices;
svrn-docs `rings/current/ONE_PERSON_MANY_DEVICES.md` §3, phase 1): `svrngs
link <group> --device` on a computer makes a device link, an `Invite`
naming that computer's person (§6), and the issuer's answer to an ask
names them (`joining.rs`, `Asked::person`). A phone opening it reads
"Adding this phone to <group> as <name>’s…" and is asked no name; a
computer's `svrngs join`, the page's line included, joins as them. Whoever
opens it joins as that person, so the command says to send it to nobody
else. A computer or phone already in the group on its own stays as it
joined — the command says so once — and a phone that lost its key joins
again by a device link. Only a computer makes one, as only a computer
redeems a link (§9). Measured on this host (`demo/runs/devices/2026-10-12/`):
Ana's laptop in as Ana 1,015 ms after its `join`, her phone 5,184 ms after
opening the link, neither asked a name; every lend told one
`x-svrngs-key` and `x-svrngs-user` from her three devices, Jellyfin and
Navidrome each one account, the sign-in asked once; Sam's removal of Ana
refused all three within 1.23 s. That a lend's limit counts her once was
could-not-judge there — the door guessed every computer on one host in
Sam's home and counted only the phone — and stays the door test's. U4 builds
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
   pass its worker cannot and hands it over. One tab of the page at a time
   holds the person's endpoint for a group, by a lock named for it, since a
   second tab's endpoint set the first's aside at the relay, for the whole
   minute it was watched (groups-tabs; #176, `demo/runs/tabs/2026-10-16/`): a tab opened beside
   it says the group is open in another tab and offers to open it there.)
4. *The phone dials whoever lends a service*, directly by key, since it
   stands in the group. The issuer, the node the link names, only redeems and
   answers what the group's people lend, each with its lender's key and the
   relay its group's record names for it (`relay`, `null` when it names
   none; relay-in-record), and the page dials the lender through that relay,
   else the link's. No node proxies another's lend. The page keeps those
   lenders' computers, and dials the issuer and them at once, the issuer
   within 10 s (`dial::LINK_WITHIN`, three times the slowest working dial
   measured, `demo/runs/reach/2026-10-14/`), each lender within its 4 s:
   the first to answer is asked, the rest closed when they answer late, and
   the page says through whose computer it reached the group (v1ui-reach
   (c), v1ui-race (a)). When that answer fails, as a computer whose person
   left still answers the dial and no more, the next to answer of those not
   yet asked is asked; only once every one has been is the page stopped
   (canon-fix (d), #205). A member who lends nothing is not among them: the
   answer names no key of theirs.
5. *One body for the link.* The command and the page parse a token and sign
   the ask and the redemption with the same code, in svrngs-core: bytes and
   keys in, bytes out.

**Lent apps, as they come** (operator, 2026-10-05). North star 2 holds for
the apps people already self-host: `svrngs share 8096` lends Jellyfin as it
comes, and someone who never used it opens it from the list, plays and
seeks, on a computer or a phone. A friend's model is lent the same way
(start-llama): `svrngs share <port> --kind llama-server` lends llama.cpp's
own server as it comes, its chat page opened from the list and its
OpenAI-compatible API given to a program, answered whole and streamed 3 ms
from straight (`demo/runs/llama/2026-10-11/`, one host). Its kind is its
lender's word, never its port: 8080 is Open WebUI's and many dev servers'
too, so the well-known table gives it none, and `share` on a port it does
not know names the kinds it does. What a lend of it keeps back is the
table's, its lender typing none (`keeps:`, `share::kept_of`; `share` says
them and why): the routes that show one person's chats to the others or
change the server for everyone — `/slots`, `/v1/stream`, `/v1/streams`,
`/v1/chat/completions/control`, `/lora-adapters`, `/metrics`, `/tools`,
`/cors-proxy`, `/models` — as start-measure read llama.cpp and asked each
route, Bo reading Cy's whole answer by its id through a lend that kept
back only `/lora-adapters` (`demo/runs/start/2026-10-13/` §3 (b)). It
keeps back no `/props`, which its chat page reads, so everyone it is lent
to sees where its model's file is on the lender's disk: the door refuses
a path for every method alike, and never changes an answer. Held by
`crates/svrngs/tests/share.rs`, each route refused and the rest answered.
U5 builds the apps' part by seven calls; the evidence is U5's inventory
(§8).
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
   So for llama-server: its words, its paths and why, are the well-known
   table's alone (`crates/svrngs-node/src/well_known.txt`).
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
   it (`crates/svrngs-node/src/well_known.txt`, `share::accounts_of`; since
   bash-person (d) a row says `accounts:` or `header:`, the second for a
   kind that takes the person from `x-svrngs-user` — Navidrome on 4533,
   Calibre-Web on 8083 — whose lender is told what to set in it). Its
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
   (`surface.rs`, `read_by`, its kinds `lend::Offered::reads_one`), and
   the app is named there as on a phone.
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
   `fetch`; one with no `Sec-Fetch-Site`, which a browser sending any
   `Sec-Fetch-*` sends and Node's `fetch` leaves out (start-address,
   `demo/runs/start/2026-10-13/` §3 (a) 4), is a program's, and the lend's
   own, as loopback is trusted (§9) and as the record reads a page. A page reaches its own lend and those of
   the kinds its lender declared it reads, in a group both lenders and
   this computer stand in, else `closed`. A lender whose computer did not
   say what it lends is could-not-judge on both paths (onair-economize
   (a)): the page whose own lender is silent is refused its reads `away`,
   never "nothing is lent here"; a lend whose lender is silent is `away`
   where a page reaches it, never a lend not read; among the reads, its
   person is named `unanswered`. The app is carried as
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
   member of that one group (syp-4), and `new` and `join` keep one at
   `~/svrngs/<the group's name>` for its founder and each person who
   joined, said where beside the `new` or the join and on the home page
   (v1-onboarding (d); bash-first-minute (d); cli.rs `folder_place`),
   everyone's name for it alike, so their folders are one; each scan's changes are one act in
   that group's record alone, each naming a path, its
   bytes by hash, and the entry it replaces; an entry no other replaces is a
   head, a path's second head written beside it named as a conflict with
   its writer's person. The bytes move by iroh-blobs: a member's node
   serves a hash only while a synced folder of a group the dialer stands
   in places it, or an app placed it there and it has not lapsed (§3,
   "What an app gets"; onair-blob), and refuses every other, and fetches what it lacks by
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
   `x-svrngs-key` and `x-svrngs-person` named the person (since
   bash-person, `x-svrngs-user` names their account), the client's own
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
     `share` learning middleware. A kind whose sign-in helper svrngs
     knows (`well_known.txt`, a `helper <kind> <port> <steps>` row;
     Jellyfin's apps/jellyfin/mw on 8097) is lent signed in by it when it
     answers and its lender names none, from the home page and by `share`
     alike (`share.rs`, `helper_of`, in `lend_of`; v1ui-polish (d)): the
     page says so before it lends, and, the helper not answering, offers
     its steps, embedded at build and served at `/helper/<kind>`. svrngs
     builds, starts and stops it never, and never lends it: a port the
     table names a helper, or a lend names its sign-in, is refused as a
     lend's port (`share.rs`, `signs_in`; arch-helper (b)). A lend that declares no sign-in keeps
     its one secret (`declared.rs`; `door.rs:497-516`, `secret_headers`,
     displacing the client's header in `rewrite_headers`, `:468`) and its
     door byte for byte, held by an existing door test unchanged.
   - *What the door sends.* On a person's first request through the
     lend, before carrying it, the door asks the sign-in with that
     request's identity headers, `x-svrngs-key`, `x-svrngs-person`,
     `x-svrngs-user` and `x-svrngs-device` (`door.rs`, `Identity`), set as it sets them for the
     service, and
     `x-svrngs-stand-in`, the stand-in (below), so an answer's script
     stores it as the door names it and no app spells it (#90, si-1);
     nothing else: no client header, no body. One owner the door holds,
     as it holds `Using` (`door.rs:187`), asks for each device apart, so
     one device's slow sign-in holds nobody else's request, and a device's
     first requests at once wait on one ask (#88, si-1: until then it
     asked one person at a time, and one slow ask held every lend's;
     bash-devices kept one answer a person, bash-two-at-once one a device,
     as Jellyfin holds a token's session to the device it signed in).
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
     request asks again. Never forwarded unsigned. A sign-in that does
     not answer, or answers while the lend's own port takes no
     connection, is the lend not answering, not a no: `"kind": "away"`,
     "<lender>’s <lend> is not answering now: try again later", the
     lender named as this computer's person (v1ui-polish (a)). Any of
     svrngs's refusals to a browser's tab, which asks `Accept: text/html`,
     is drawn as a page of its words; a program is given the JSON.
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

**A relay anyone runs** (relay-verb; the operator, 2026-10-12, §10). A
phone's every byte crosses a relay, a browser's iroh being relay-only, so
a group need not lean on anyone's: `svrngs relay <name>` serves iroh's own
relay server, iroh-relay 1.0.2, from the svrngs binary — the one the
computers that dial it are built with — at `https://<name>/`, on the ports
every iroh dials a relay at: TCP 443 and 80, and QUIC address discovery on
UDP 7842 (`crates/svrngs/src/relay.rs`). Its certificate is Let's
Encrypt's, asked and renewed by the relay itself and kept in
`<data dir>/relay`; `--staging` asks Let's Encrypt's staging service
instead, to try a name first. A name, never a bare address: an address's
certificate needs Let's Encrypt's `shortlived` profile, which iroh-relay's
ACME client, tokio-rustls-acme 0.9.1, cannot ask — it orders DNS names
alone and names no profile (`acme.rs:129-130`, `:301-303`); the bare
address waits in §9. By default it lets through only the computers and
phones that stand in a group its computer holds — the register's one
answer, `standing_anywhere`, read from the record on disk as svrngs beside
it writes it, each connection asked again, each refusal traced; a computer
in no group is refused the verb, which would let nobody through, and one
whose groups cannot be read is refused as that, never as in none
(start-findings (c), #128). `--open`
lets everyone through, for a relay like svrngs's own, each computer or
phone sending it at most `OPEN_RATE`, set from a film played through a
relay at Demo A's bitrate: the most it drew in a second, 8,799,728 bytes
of three takes, rounded up to 8,800,000 (`demo/runs/relay-rate/2026-10-12/`).
A test starts it on 127.0.0.1 over a computer's data and dials it as
iroh's client does (`crates/svrngs/tests/relay.rs`). Its certificate is
not judged here, this host having no public address: the operator's run
is `svrngs relay <name> --staging` on a server `<name>` leads to, then
`curl -kv https://<name>/` naming Let's Encrypt's staging issuer. The one
who runs a relay reads its flags and its words; a person in a group never
sees the word.

**Where a computer meets its groups** (relay-default (a), relay-park (b);
§10). Where a group that keeps no relay of its own is met is a closed set,
`cli::MeetAt`: n0's relays with n0's address lookup, or svrngs's relay,
`https://relay.svrnme.sh/` (`cli::DEFAULT_RELAY`), with no lookup and n0
none of the path. The build ships one, `cli::SHIPPED`: n0's, chosen,
until the operator flips it for production, after V1 (§9; §10,
2026-10-13 and 2026-10-15). `cli::reach`, which `main.rs` calls with it, builds the computer's
`RelayConfig`: at svrngs's, `n0_services` off, so no lookup is built and a
key is dialled only at the relay its group's record names (§6); at n0's,
`n0_services` on, and n0's relays named beside the relays the computer's
groups keep whenever one of its groups keeps none, since a list replaces
n0's set (`crates/svrngs-node/src/guest.rs:86-94`). A group that runs its own relay is made with
`svrngs new <name> --relay <name or url>`: the computer keeps it beside the
group's seed (`group.rs`, `relay`), as `join` keeps the relay its link
carried, so every computer the group's links let in meets the group there,
and each says so in the record by its `Relay` act. A link carries its
issuer's home relay in either posture (`svrngs_node::invite::carried`,
relay-park (c); the command's until start-home), so a phone's page dials it and asks no lookup; a joiner at n0's
keeps none of n0's relays a link carries (`group::kept_relay`), which are
where its issuer's build met its groups, not the group's own, and kept
would outlast the switch. One gap is left open: a link minted before the
switch and opened by a computer after it, within the link's day, is kept,
since at svrngs's n0's relays are never listed to be told apart. A computer meets each
group it holds at the relay that group keeps, the build's where it keeps
none; iroh makes the nearest its home, so a computer in two groups with
two relays is reached at one, where the other group's computers dial it.
At n0's, a computer holding only a group with its own relay is met there,
n0's lookup on and no n0 relay named; holding a second group that keeps
none, it names the group's relay and n0's four, and iroh chose: in two runs on
this host, the group's relay on 127.0.0.1 over plain HTTP, which answers no
QUIC address discovery as `svrngs relay` does, gave way to n0's
`usw1-1` at once in one and after 25.7 s in the other (relay-park (b),
the commit's body). A
group's own relay lets a newcomer in before its `Admit` only with `--open`
(§9). A computer that reaches none of its relays in 10 s says which it
tried, and goes on trying. Tested at svrngs's by three computers made, joined and
syncing through one relay on 127.0.0.1, each built by `cli::reach`, none
with a lookup and none tracing an n0 host, planted with the old posture
(`crates/svrngs/tests/default_relay.rs`); as shipped, by a computer with
no group met at n0's relays and lookup, and one holding a group with its
own relay on 127.0.0.1 homed there, n0's named beside it once it holds a
group that keeps none, planted with `SHIPPED` at svrngs's
(`crates/svrngs/tests/shipped.rs`).

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
| `svrngs-node` | the adapters: sync and its route over iroh, redemption, the loopback record API with its live lane (landed, u3-2 and u3-3), the member door (landed, u3-4), the list of what members lend, asked of each (landed, u3-5), each for at most 3 s (landed, asleep-1, `node::ANSWER_WITHIN`), and answered to a member by the computer it dials, each with its lender's key (landed, u4-5), and the door's limit and shares, a pure decision the door's task owns (landed, u3-6); a WebSocket's upgrade carried, and the pass holders' protocol, which any key may speak, where a request presenting a person's pass for the lend it names, by name or label, is that person's (landed, u5-2), and its service told the pass's first lend, the app the page opened, as `x-svrngs-app`, none with no pass (the scoped-use spike's one header, su-1, `demo/runs/su/2026-10-09/` §5); every relay URL it makes or reads dotless where it enters — n0's relays, a relay it is told, a lookup's answers — in `relay.rs`, which the page compiles too (landed, u4-9); every dial the transport makes to a member's computer, and a phone's page's to a lender, given up when not answered within 4 s, the slowest working dial measured times three, rounded up (2 s from 347 ms, then 4 s from the capped take's 1,013 ms, events-wait), so a person is told an asleep lend's computer did not answer then and not after iroh's 30 s, in `dial.rs`, which the page compiles too; every request to a member's computer a bi-stream of the one connection kept to it, read by hyper's client as the phone's page reads one, the connection held as the page holds its dial, "not admitted" read from its close, a request read whole within 10 s, and no port on this computer's loopback that reaches another computer's door (landed, arch-peer, `iroh.rs` `IrohTransport::send`; through upstream's bridge, a port per member, any program on the computer and a page Chromium or WebKit let through reached a friend's lends and kept a yes as the person, `demo/runs/arch/2026-10-15-peer/`, `crates/svrngs-node/tests/peer.rs`); and a phone's page's dial to a lender held only while its computer sends something within that 4 s, a keep-alive each second, so a computer frozen as a lid closes is said away within about the cap, a request on the dial when it falls silent not dialled again (landed, events-wait, `dial::held`, `demo/runs/wait/2026-10-14/`); a joiner's dial to the computer its link names waits as iroh does, its first answered in 3.03 s on a computer just started (landed, events-asleep, `demo/runs/asleep/2026-10-14/`, `crates/svrngs/tests/home_group.rs`); the room door, the lends a person names served to this computer's network at its private address while the command runs, one body with the origins (landed, u6-1); a lend's sign-in, asked of the lender's program on a person's first request, its credential set on each of their requests, a stand-in swapped for its secret both ways (landed, mwp-1, `door/sign_in.rs`), its script first in the lend's pages (landed, mwp-2, the same file; §3, "The room", call 6); every answer it makes in a lend's place marked as its own, `x-svrngs-refused` and its kind, a service's own dropped (landed, mnp-1, `door::LendRefusal`), and a pass whose signer it does not know asked of the group's ring once, within 10 s, until a member that knows them answers, before it is refused (landed, mnp-1, `ring_sync::AskRound`; the members dialled at once, asleep-1); a lend's pages on this computer told, at its origin's `/svrngs-reads`, the lends they read, from the record (landed, mnp-2, `surface::reads`); each person's yes and stop to an app kept by the lend's door and decided there, by the person the door resolves (landed, sup-2, `door/choices.rs`, `door::PersonKey`); the home page at `svrngs.localhost`, answering only itself, never framed, its writes the person's yes and stop (landed, sup-2, `home.rs`) and the person's change to what this computer lends, how many at once or a stop (landed, bash-limit, `home.rs` `/lent`), and a link to a group made there as `svrngs link` makes it (landed, start-home, `invite.rs`), and the group's folder opened from there (landed, start-home, `home.rs` `/folder`), and a group made there as `svrngs new` makes one, its folder placed as a joiner's (landed, start-group, `group.rs`, `home.rs` `/group`), and another group joined there from its link as `svrngs join` joins (landed, arch-command (c), `join.rs`, `home.rs` `/join`), and what runs on the computer lent from there, probed and decided as `svrngs share` does (landed, v1ui-lend, `share.rs`, `home.rs` `/running`) | each lend's declaration and secret, and each person's choices for it; who is using each lend now; each person's sign-in answer, in memory |
| `svrngs-web` | the page's dial, wasm only, in `web/` with its static shell and a workspace of its own: a browser's endpoint, bound with the person's key, dialling a group's computer by key over a relay on the members' protocol (landed, u4-3); HTTP over the dial (landed, u4-4); joining — the ask and the redemption by svrngs-core's bodies, the key sealed in the page's store, and the list of what the group's people lend (landed, u4-5); the runtime, a service worker serving every request at a lend's address from the lender's door (landed, u4-6), a range that runs on — open-ended, or a `<video>`'s longer than a part — asked of the door in parts of 2 MiB (landed, u4-9), an open-ended one's first part at a new place 256 KiB and doubling as the reading runs on (u5-11), handed what it dials with in memory, and a page for a worker that holds nothing (landed, u4-10); since u5-2 at each lend's own origin, not the page's (`lend.js`, its start page `open.js`): the page signs a pass to a fresh key for the lend it opens and the lends it reads, the worker dials on the pass holders' protocol, keeps each lend's cookies, reads a redirect to a path under the lend, and with a shim in each lent page (`shim.js`) carries `WebSocket` and an app's own worker; its own answers in a lend's place marked as the door marks its own (mnp-1); its pages told the lends they read at `/svrngs-reads`, each at an address under their own origin that opens that lend at a path through the group's page (landed, mnp-2); a lend that asks told with its lender's words and an `ask` address that opens the group's page at `#ask=`, where the person's yes is posted to the lender's door as them, an app's stop sent by the worker with the pass, and what apps use listed there with Stop, the page never framed on its host (landed, sup-3); an adapter | none: the page's store holds the key, sealed; a lend's origin's store holds its pass and its cookies |
| `svrngs` | the command and the process: `new`, `join`, `link`, `share`, `unshare` and `room` (landed), and `share --sign-in` (landed, mwp-1), and `share --ask "<words>"`, its lender's words for what an app receives, carried in `asks` on every list of lends and never read by svrngs (landed, sup-2); the bare command prints where the record API listens, where the home page opens (sup-2), at the address it took at its first start (start-address, `api::kept_address`), and what the people of its groups lend | a device's keys; each group's seed |

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
| Same acts, same state: a group's state is a pure function of its set of acts, and of its seed, which its namespace hashes (`Seed::id`) | svrngs-core is `no_std`, and `lean_check` keeps it so; an adapter reads a group only through the runtime's queries (`admission`, `standing`, `invite_for`), since the journal is crate-private to svrngs-runtime; and it reads an admission only as a fold made in svrngs-core (`Admission::membership`, `Admission::invites`) or as `wire()`, the acts serialized for an app to fold, since the acts are private to the core | holds for every adapter; `wire()`'s JSON can be parsed back into acts, and no gate refuses that; inside the core, `membership` still sorts the acts admission hands it |
| No act, no effect: nothing that cannot be undone to a group happens but from an act in its record. What is one computer's own — its keys, its lends and their secrets, a person's yes and stop to them, its folders' kept state, an app's blob and its lifetime — is kept in that computer's files (the State column above) and never in the record; a lent service's own state is its own, and the member door only carries a request to it | a store appends only `Authored` lines, which only `commit` makes after a `Command` decided under the write lock; a peer's lines enter by `ingest_all` unjudged, and every fold judges them; nothing deletes a line: upstream's seal, its compaction and the effects that ran it (`process.rs`) went in arch-seal, and a `Seal` an older build wrote is held as an act that does nothing, said at debug (`admit`); what follows an act is written after it and driven by its writer: a cut told to the computers it cut, a leave told by the leaver's round until each computer answers (`leave_told`, #190) | every act this computer signs goes through `commit`, a peer's through `ingest_all`; the journal's `append` is a test fixture |
| Both | the core never depends on the node; the clock, randomness and locks live only where an adapter holds one (`clippy.toml` bans them; each site carries `#[expect(…, reason)]`, which fails once the site is gone) | holds |

One property test stands behind every fold: permuting or duplicating
`admit`'s input changes nothing. A decider's contract is given / when / then
rows.

**Two gates keep the apps out of the infra.** Every app lists its own nouns in
`apps/<app>/nouns.txt`, and `scripts/vocab_check.py` refuses any of them in
infra code — comments and tests apart, where an example is fine. So the infra
never learns what a film, an expense or a crush is. And the canaries of §8,
Expenses and Events, run against the infra in `scripts/check.sh`
(`crates/svrngs-node/tests/canaries.rs`): a change that serves Jellyfin and
breaks one fails the build. Tap, the third §8 names, is not built, and
nothing runs it (#196).

**Lean, and kept lean.** `svrngs-core` and the repository stay under the
caps in `scripts/lean_check.py`'s `BUDGET`, their one home: its comment says
each move since, and this paragraph the moves before it, which once
restated the caps here (#197; events-asleep raised it from 16,235 by 20, events-links by 33, events-principles by 17 and events-wait by 10, the operator's +80 for the events follow-ups all spent; v1ui-economize cut nothing, and of the operator's +650 for the v1 UI over 16,315, v1ui-reach raised it by 24, v1ui-door by 108, v1ui-lend by 121, v1ui-shelf by 286 and v1ui-findings by 17, and REVIEW-audit-v1ui-auto-2's cut lowered it by 1), the branch `folders`
too since main merged into it at syp-5, counting 13,778 and `svrngs-core`
2,719 there. A raise is the operator's, and is asked only after an
economizing pass (the operator, 2026-10-13: "Every raise needs a pass
first"): its commit gives the reason, names what the pass cut, and says why
the rest could not be found in the code there is; the pass's cuts lower the
cap to where they land, so room freed is never spent but by the operator's
choice. The budget counts the product:
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
merge (§10). And `svrngs-core`'s from 3,000, at 2,981, on 2026-10-12, for
one person, many devices (bash-devices; the operator, "yes to both";
svrn-docs `rings/current/ONE_PERSON_MANY_DEVICES.md` §6, "the core: about
+80"): a device link's person on an `Invite`, the fold's person by their
first key and `person_of`, removal and hand-on resolving a name to one
person — 3,051 with them, and no function of the core without a caller
outside the tests to make room from. And both on 2026-10-12, for svrngs's
own relay, n0 out of the path (§8; §10, 2026-10-12; `ralph/next/relay/`):
the repository's from 15,350, at 15,343, and `svrngs-core`'s from 3,080, at
3,072 — the operator, "I do think we need to host our own relay (can't
rely on n0)", the budget raised for that queue to
15,800 and `svrngs-core` 3,130: each computer's relay in its group's
record, folded in the core, and every dial of a member through it;
`svrngs relay`, a relay anyone runs; svrngs's relay the default. And the
repository's from 15,800, at 15,798, on 2026-10-13, `svrngs-core`'s kept
at 3,130, for svrngs easy to start with before v1 (`ralph/next/start/`):
the operator, "nail down the svrn / llama-server on svrngs story and make
it part of an early demo document for anyone getting started", "making
sure our localhost dashboard really makes the system easy to navigate for
someone who is a bit afraid technically", and "#126–#129 and #131", the
budget raised for that queue to 15,950: those five findings, an address
that stays put, llama-server lent with nothing to look up, and the home
page leading its reader; taken by start-findings, whose #127 was the
first change past 15,800. Then, from 15,950, at 15,921, the same day,
`svrngs-core`'s kept at 3,130: the operator, after start-measure's ranked
list (`demo/runs/start/2026-10-13/README.md` §5), +50 to 16,000 "so the
home page can make an invite link and open the group's folder"
(`ralph/next/start/CHARTER.md`); taken by start-home, whose link made on
the home page was the first change past 15,950. Then, from 16,000, at
15,994, the same day, `svrngs-core`'s kept at 3,130: the operator, after
start-home found that a computer set up by the one line cannot make a
group (`demo/runs/start/2026-10-13-home/NEEDS_HUMAN.md`), "+40 now, before
start-run", to 16,040, for making a group on the home page and a founder's
one line (`ralph/next/start/CHARTER.md`); taken by start-group.
Then down, the same day: the operator, "We keep just raising the bar
without considering how we could lower the floor", asked back under
15,800 and the caps lowered to where the cuts land. start-economize's
survey (`demo/runs/economize/2026-10-13/SURVEY.md`) found 50 lines the
product did not need, by the compiler (every `pub` demoted, the product
and the page rebuilt), the clone report and the open findings: five items
only tests reached and a script nothing ran. Cut, the repository went
from 16,036 to 15,986 and `svrngs-core` stayed at 3,121, so the caps were
15,987 and 3,122, one past each count. What the survey could not reach
without a change in behaviour is its `NEEDS_HUMAN.md`; the operator, "Accept
15,987; fold follow-ups into onair", and onair-economize made its four
follow-ups, each a change said and tested: one decider of what a lent page
reads, `svrngs`'s value flags read in one place, `remove` and `handover`
one `Verb`, and both pages' walk of what apps use in `question.js`. The
repository went to 15,950 and `svrngs-core` stayed at 3,121, so the caps
were 15,951 and 3,122; the onair queue's rows raise from there, each only
as far as it needs, within the operator's 16,287 and 3,182 for that queue
(`ralph/next/onair/CHARTER.md`). The repository's from 15,951, at 15,950,
the same day, `svrngs-core`'s kept at 3,122: to 15,962, for the live lane
naming who spoke (onair-live; svrn-docs `rings/current/ON_AIR.md` §4.3),
which counted 15,961 with it, after onair-economize's cuts above. Then
both, the same day: `svrngs-core`'s from 3,122, at 3,121, to 3,150, and the
repository's from 15,962, at 15,961, to 16,015, for a signature for an app
(onair-sign; ON_AIR §4.2), which counted 3,149 and 16,014 with it — 28
lines of the core's for the app's name and what is signed, 25 of the
node's for its route — after onair-economize's cuts above. Then the
repository's from 16,015, at 16,014, `svrngs-core`'s kept at 3,150: to
16,237, for bytes by hash, for an app, with a lifetime (onair-blob; ON_AIR
§4.1), which counted 16,236 with it — its first draft's 16,301, past the
operator's 16,287, cut by 65 in the row — after onair-economize's cuts
above. Then down, 2026-10-14, before the events queue's follow-ups raise:
events-economize's survey (`demo/runs/economize/2026-10-14/SURVEY.md`), by
the same instruments over onair's +286, found nothing unreached and
nothing written twice but synced.rs's own writer of a file kept whole,
beside declared.rs's. Cut, the repository went from 16,236 to 16,234 and
`svrngs-core` stayed at 3,149, so the caps are 16,235 and 3,150. The events
queue's rows raise from there, each only as far as it needs, within +80, to
16,315; `svrngs-core`'s stays (the operator, 2026-10-14, "+80 ceiling,
fixes included"; `ralph/next/events/CHARTER.md`).
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
  from the group's `seed.json` (`Seed::roster`, `crates/svrngs-node/src/group.rs`),
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
  `Remove` of a standing key not its own is the cut; so is a person's own,
  their leave (below, "Leaving a group"); anyone else's is an ask, counted
  as an act, cutting nobody. A cut key's
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
  landed is told by the first computer it asks to sync the group, which
  answers it the `Remove` naming it, with its signer's acts before it
  and every `HandOn` it lacks, so a keeper handed the order while it was
  off is read as the keeper and the cut as the cut (#184), and nothing
  else (`ring_routes.rs` `told_cut`, #179; served too where it stands in
  no group held there); every door refuses it all the same.
  Everyone in the group sees every removal and every ask to remove — who
  of whom, and when — on a computer's home page and a phone's group page,
  read from the register and written nowhere new (v1-keeper-office (b);
  §10, 2026-10-12): `Register::removals`
  (`crates/svrngs-runtime/src/register.rs`), carried by `/v1/groups` and
  `/people/lends`, in `question.js`'s one wording.
- **Leaving a group is a person's own act, and needs nobody's yes**
  (groups-leave; svrn-docs `rings/current/LEAVING.md`; the operator,
  2026-10-14: "Nobody's yes; keeper hands on first"). A `Remove` signed by
  a key standing as the person it removes is the cut, by them alone: no
  new kind of act, one more line of the fold (`membership.rs`, its module
  docs the rule). svrngs-core's `Leave` writes a `Remove` of every key
  standing as the person, each naming its newest acts, the signer's own
  last, since a leave is an act of the key it cuts and its cut covers it
  and what it descends from. The keeper's leave cuts nobody while anyone
  else stands, so `Leave` refuses it (`Keeps`) and writes nothing: they
  hand on first, by the `HandOn` that exists. The last person in a group
  leaves freely, and every copy stays with whoever held it. A `Removal`
  the fold would read as the signer's own leave is refused (`Own`), since
  its keys' order is not the leave's. `svrngs leave <group>` asks the node
  running here, as `svrngs remove` does (`POST /v1/leaves`, `removal.rs`
  `leave`): it commits the `Leave` and tells it at once to each computer
  the leave cut, the leaver's own others (#179), as a cut is told to the
  removed. Every other computer standing in the group is told it by the
  leaver's round, at once and each round after until it answers, took
  it or refused it (`ring_sync.rs` `leave_told`, #190): from then on
  the leaver's computer stands for nobody there, so no exchange carries
  the leave, and told once it was lost to a computer off as it landed.
  Either way the leave alone is sent, the computer's own `Remove`s
  (`RingRail::own_removes`), and nothing else (the operator, 2026-10-14:
  "Record only"). A computer off as it lands hears it once it and the
  leaver's are on together (`crates/svrngs/tests/leave_page.rs`,
  `a_leave_reaches_a_computer_off_as_it_landed`); one of the leaver's
  own is told by the first computer it asks to sync, as a removed
  computer is (`a_leave_reaches_the_leavers_other_computer`). From
  then on the leaver's computer holds the group and nobody stands in it
  there: `RingRail::standing` answers `Standing::Left`, with the keys its
  copy says stand, for any group whose register cut this computer's own
  key, left or removed (`crates/svrngs-runtime/src/register.rs`). Every
  gate that asks who stands — the round, the acceptor, the sync route,
  the door and the lends — finds nobody, so no round exchanges it, no
  sync serves it and nothing this computer lends reaches it; the record
  API still reads it. An address on that computer of a lend of someone
  still in the group says the cut, "You left Trip on 16 Oct" or a
  removal's "You are no longer in this group", never "nothing is lent
  here" (`origin.rs` `refused`, #182; `leave_page.rs`). Nothing on the leaver's disk is deleted and nothing
  is sent back: their copy of the record and the files its folder gave
  them stay (`crates/svrngs-node/tests/folders.rs`,
  `a_leave_stops_the_group_reaching_them_and_keeps_what_they_hold`).
  Everyone in the group sees a leave where removals show, "Ana left on
  <date>", in `question.js`'s one wording: `Register::removals` says a cut
  the fold kept as a leave (`Cut::left`, decided once by the fold's
  `leaves`, #180) `RemovalSaid::Left`, any other cut `Removed` (the
  operator, 2026-10-14: "Record only"). Leaving is
  final from that computer through V1: a computer signs every group with
  one key, and a cut key never stands again (the operator, 2026-10-14:
  "Final for now; per-group keys as a row"; on that row's survey,
  2026-10-16: "Final from this computer, through V1"; a key per group on
  computers waits in §9). So the issuer refuses to redeem a
  link for a key cut in its group (svrngs-core `RedeemRefusal::Cut`), where
  the fold alone would count the later `Admit`, and answers that refusal
  410, apart from every other, so the computer is told it as its cut
  (#181); and a computer that left, sent a new link, is told at once,
  before anything is redeemed, that it cannot join again from there yet
  (`join.rs`, `NotJoined::Left`), as the page and `svrngs join` say it,
  and told the same where its copy cannot say and the issuer refuses it
  (`join_by_link.rs`, `a_computer_the_group_cut_is_told_so_by_the_issuer`);
  a phone, with a key per group
  (`web/store.js`), can rejoin.
- **Leaving is cheap.** A group's whole state is a directory every member's
  computer holds. A group that goes wrong refounds from any copy, within
  hours, without whoever it leaves out (operator: "with dotfile elegance").
- **What an app places by hash is not kept** (onair-blob; §3, "What an
  app gets"). Its bytes are in no act and no folder; which hashes a
  computer carries for which group, and until when, is in its running
  process alone, each kept past its lapse so a get says `lapsed`, not
  `never_held`. The store's sweep, each second, drops a hash `keep`
  seconds after its put, on every computer that holds it. A computer
  stopped before then keeps the bytes on its disk until it starts again,
  and its first sweep drops them, since a restart carries nothing over:
  Ana's computer, stopped 25 s into a 60 s lapse, held its 10 pieces on
  disk and dropped all 10 in its first second started again
  (`demo/runs/onair/2026-10-13-blob/`, `ana-restart.txt`). A computer
  restarted says `never_held` of what it carried before. What lapses is
  the bytes; a fetched hash's tree of hashes can stay (§9).
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
  stand already, no standing person goes by the name in any case (bash-person, §3 "Joining"),
  and `membership` over its journal with the `Admit` in it
  counts it, both under the write lock (`commit` with svrngs-core's `Redeem`), so the
  fold stays the one decider of a lapsed, closed or voided link. Every
  refusal is one sentence naming no group, but a name that does not parse,
  said with why, and a name someone goes by, answered `409` with theirs; a
  key that already stands is answered success and nothing is written.
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
- **A key per device per group, bound to the person** (the operator,
  2026-10-12, "yes to both", amending "a signing key per person per group":
  a key shared by a person's devices would fork their chain, so each device
  keeps its own; `ralph/next/bash/CHARTER.md`, "Device links are this
  queue's"), so a cut in one group touches no other; an endpoint key per
  device. A person is their first key in the group — a seed row's first,
  or the key a plain `Admit` binds — so two people of one name are two
  people, never merged; a device link, an `Invite` naming the person
  (`device_of`), counts only from a key standing as them, and a key it
  admits is theirs, under their name, asked none (bash-devices, landed;
  `membership.rs`'s module docs; canon's rule that only a key bound to a
  name adds a key to it, `~/dev/canon` `docs/SIGNING_DESIGN.md` §2). The
  register answers each key's person (`Membership::person_of`); removal
  and hand-on take the one person a name names. Two people still come to
  one name — a race at two computers, a member's hand, a group from before
  device links where a person's second device joined under their own name
  — and of two, svrngs never guesses which is meant: the refusal says when
  each joined, the time on the `Admit` that made them a person or that they
  started the group (`Membership::joined`), `--joined` names one, the core
  deciding (`Membership::one_named`), and the home page lists each person
  once, two of one name each with when they joined (bash-same-name,
  landed; `crates/svrngs/tests/remove.rs`). That time is this computer's
  to the second with its offset, one form the refusal and the page write
  and `--joined` reads (home.rs `WHEN`), so the hour a fall-back night
  repeats reads back to one instant (start-findings (a),
  `crates/svrngs/tests/joined.rs`). Two let in the same second
  cannot be told apart this way; joining two such people into one waits
  in §9. Only a computer makes a device link; a phone making one, and one
  device removed apart from its person, wait in §9. A computer's one key
  for every group it holds (`crates/svrngs/src/identity.rs`) still departs
  from "per group" through V1; a key per group on computers waits in §9.
- **Each computer's relay is in the record** (relay-in-record (b), landed;
  §8's relay row): a `Relay{url}` act, signed by the key it speaks for, so
  nobody says where another is reached; its newest counting one wins, and
  one naming no relay withdraws it (line version 9). An act of its own
  rather than a field of the `Admit`: an `Admit` is its issuer's, written
  once, and a computer's relay moves with it; so the computer says it,
  and says it again when it moves. The fold keeps each key's
  (`Membership::relay`, read dotless by `relay::parse`, the one reading
  of a relay a link, the record, a person or a peer names, which
  svrngs-node's `relay::parse` wraps as iroh's URL; start-findings (e),
  #131), and only for a standing key;
  the register carries it beside each key (`Member::relay`). svrngs-core's
  `RelayAt` writes it only when the record says otherwise. Like the acts
  that move membership, a `Correct` voids none: a newer one replaces it.
  A computer says its home relay at each round's start, before it prints a
  link, and once a join's exchange brings its admission, taking it to the
  issuer in a second exchange — until one member holds it none can dial
  the joiner (`Host::say_relay`, node.rs); with relays and none reached yet
  it says nothing, so starting never withdraws it. A joiner waits for its
  home relay before that exchange (`ONLINE_WAIT`, cli.rs), and a push
  landing its asker's own relay wakes the round (ring_routes.rs), as the
  redemption's `Admit` does, so the group hears it at once
  (`demo/runs/relay-in-record/2026-10-12/`: without each, 60 s and 57 s). Every dial of a member
  carries the relay its group names (`Host::contact`, `Contact::named`):
  the round, the lends' list, the member door's asks, a folder's fetch,
  the lend door's dial and a removed computer told its cut (relay-in-record
  (c), landed). A key held in several groups is dialled, and told to a
  phone in the people answer, by the first group naming a relay for it,
  one rule (`node::relay_of`; start-findings (b), #127), so a group that
  has not yet heard the key's relay never stands for one that has; the
  live lane dials by the group it offers to. A key with none named is traced; with no address lookup
  the transport refuses it before iroh is asked, never dialling it blind.

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
(`demo/runs/u5/2026-10-05-localhost/`), and curl, Python's `openai` and
Node's `fetch` too, on macOS (`demo/runs/start/2026-10-13/` §3 (a) 2-4; on
Linux, where it rests on the system's resolver, could-not-judge). Its port
is the record API's, taken at the first start and kept in the data dir
(`<data dir>/api`, `svrngs_node::api::kept_address`), so a bookmark and a
program's address survive a restart, as a port svrngs named would not
across two data dirs on one host; another program holding it is said by
the command, exit 3, with what to do, never a new port taken in silence
(start-address). The command, install.sh and the home page say the page's
address in one sentence (`svrngs_node::home::address`). There the origin names the app
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
its place must send it too. Ours does.

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
joins the link as `svrngs join` does; keeps the page it came from in the
data dir (`<data dir>/origin`), which a group made on the home page keeps
for its https links (§3, "the home page"); keeps svrngs running at login as
the person (a LaunchAgent on macOS, a `systemd --user` unit on Linux);
and opens the home page. Run again, it updates in place and joins
nothing twice. **A founder's one line** (start-group): someone with no
link yet pastes `curl -fsSL <page>install.sh | sh -s -- '<page>'`, the
page's address where the link goes, which the group's page opened with
no link gives on a computer holding no group there, saying it gets svrngs
and opens the home page where they make their group (`web/app.js`
`offer`; `the_page_opened_with_no_link_offers_a_founder_s_line`); it does
all of the above but join, and the home page it
opens is where they make their group, with no terminal after
(`crates/svrngs/tests/install.rs`
`a_founder_s_line_installs_keeps_svrngs_at_login_and_opens_the_home_page`,
on macOS by its launchctl stand-in and on Linux by its systemctl one,
each starting svrngs as the system would). With the page's address in the link's place and `--relay
<name> [--open]` after it, it joins nothing and keeps `svrngs relay`
running instead (relay-verb (c); §3, "A relay anyone runs"): beside any
svrngs kept already, a LaunchAgent of its own on macOS, which since 10.14
lets a program take ports 80 and 443 without root (Apple's word, not run
here: no port is opened on this host); on Linux, where only the
system gives those, it prints a system unit — the person's user,
`CAP_NET_BIND_SERVICE` — for whoever runs the server to install as root,
and installs none, never running `sudo` (`crates/svrngs/tests/install.rs`,
each path by its stubs; a relay's agent there is recorded, never run). The line gives no name, so the person is named as their
computer shows them: the full name of the account `$USER` names (macOS's
`id -F`, Linux's GECOS), else `$USER` (bash-person (e), `cli.rs`
`user_name`; it was the login, "alexsbryan", onboarding's finding 1). Measured once, on this host over loopback, nobody reading the page:
from the paste to the home page listing the group and its folder, 6,029
ms, and to a member's photo in that folder, 6,489 ms; the four tarballs
built cold in 11:31; the Linux two never run, could-not-judge
(`demo/runs/onboarding/2026-10-11/`). Publishing the binaries with the
page is the operator's (`scripts/deploy-page.sh`, which builds both into
one site); until it runs, group.svrnme.sh serves no `install.sh` (404,
2026-10-11). A build meets its groups where `cli::SHIPPED` says (§3,
"Where a computer meets its groups"), and `deploy-page.sh` reads it: a
build at n0's relays, the chosen default until the switch, it publishes
and says so; a build at svrngs's it refuses to publish while the relay
`DEFAULT_RELAY` names does not answer (`scripts/relay-answers.sh`: its
`/ping` 200 and empty, as iroh-relay answers, never a page at the name —
relay.svrnme.sh answered Vercel's 404, 2026-10-12;
`crates/svrngs/tests/relay.rs`); a switch it cannot read it refuses. The
switch is the operator's, in order (§9; §10, 2026-10-13): flip
`SHIPPED` to `MeetAt::Svrngs`; run `scripts/deploy-relay.sh`, which puts
up `relay.svrnme.sh` — a DigitalOcean droplet in US West, a firewall
letting in TCP 80 and 443 and UDP 7842, `svrngs relay relay.svrnme.sh
--open` from this build as the system unit install.sh prints, the name's
records at Vercel, svrnme.sh's DNS — and waits until it answers; then
`deploy-page.sh`. None of it is run here (relay-default (c), relay-park
(d)).

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

Moved to `docs/PLAN.md`: the order of work is its §1, and what was built,
with each unit's inventory, its §4.

## 9. Later, each with the trigger that brings it back

Moved to `docs/PLAN.md` §2.

## 10. Decisions

Moved to `docs/PLAN.md` §3.

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
5. **Members find each other through n0** until the operator's switch,
   after V1 (§8, "our own relay" and "V1"; §9; §10, 2026-10-12, 2026-10-13
   and 2026-10-15): iroh's
   address lookup, n0's DNS by default, as upstream's (§8, U1's inventory),
   maps a key to its relay, never to a person, and a group is unreachable
   through it while it is down; and every byte a phone moves crosses a
   relay, a browser being relay-only, n0's under n0's terms. The relay
   queue built what closes both, and parked it at the gate (relay-park):
   each computer's relay kept in its group's record and dialled from there
   (relay-in-record); svrngs's own relay, with no lookup built (upstream's
   `discovery = "none"`, `n0_services` off), one switch away, and a group's
   own by `svrngs new --relay`; the page dialling only the relays its link
   and its group name, a link naming none refused in a person's words
   (`web/src/lib.rs` `bind_for`, web/app.js `through`), with every address
   off this host refused it in both engines
   (`crates/svrngs-node/tests/browser/main.rs`, `away`). So the risk is
   open again until the switch, by the operator's choice: a computer meets
   a group that keeps no relay at n0's relays and asks n0's lookup, while a
   phone's page asks it nothing, its link carrying its issuer's relay. n0
   is the chosen default, never a fallback when svrngs's relay or a group's
   fails. Run end to end on one host with svrngs's posture, Demo C's steps
   opened no address of n0's (`demo/runs/c/no-n0-2026-10-13/`). A relay that admits only its group
   is not yet one a phone can use: the page opens each lend from a fresh
   key holding the person's pass, not their key, so that no lend's origin
   holds it (§3, "Lent apps", call 6), and the relay refuses that key as standing
   in no group — the phone in, its lends unplayable, the join page in the
   film's place. And a newcomer it refuses is told the computer that sent
   the link did not answer. After the switch, what stays is ours: svrngs's relay is then the one most
   groups share, so while it is down a group with no relay of its own
   reaches no phone and no member it cannot reach direct — the same
   centre, ours rather than n0's, with `svrngs relay` the way out anyone
   can take. And n0's relays, as iroh spells them, are out of
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
