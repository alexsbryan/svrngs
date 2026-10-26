# svrngs — the plan

> What we build next and in what order, what waits for its trigger, the
> operator's decisions, and the record of what was built. The design is
> `docs/DESIGN.md`, what exists is `ARCHITECTURE.md`, and a queue's rows are
> `ralph/next/<queue>/STATE.md`. Work is the next unit in §1: each unit moves
> the demo that follows it, and work that moves no demo waits. A change to
> the order rewrites §1 in place, never a section over it. A bare § names a
> section of this file; DESIGN's are named so. Until 2026-10-17 these
> sections were DESIGN §8–§10 (`docs/DESIGN.md` at `6c7b8296`).

## 1. Order of work

**V1 — the cut line** (after events; operator, 2026-10-15, of the
grapevine demo app: "And then I think that's our V1 cut line"; asked, "arch,
groups, grapevine; V1 on n0" and "arch → groups → grapevine"; svrn-docs
`rings/current/GRAPEVINE_DEMO.md` §12; §3, 2026-10-15). After events the
v1ui queue made the home page and the phone's page what V1 shows (landed,
`ralph/next/v1ui/`). Then, in this order, and nothing else until V1:
- the arch queue, the architectural audit's cuts (`ralph/next/arch/`,
  its `AUDIT.md`; landed, and audited by REVIEW-audit-arch-auto-1);
- the groups queue (`ralph/next/groups/`): its economizing pass first, then
  the node's kept connections, leaving a group with nobody's yes, a key
  per group on a computer surveyed and left for after V1 (§2), a leave
  told to the leaver's other computers, then their run;
- the grapevine demo app (`ralph/next/grapevine/`, in `apps/grapevine/`):
  an ask crossing groups through people, taps only, nothing changed in
  svrngs but the leave's last fixes first (gv-leave, +30 lines); landed,
  its scene run under svrngs on the identity model groups-keys leaves,
  three computers on one host, every person a program
  (`demo/runs/grapevine/2026-10-16/`): Sam's ask to the ask gone from
  Kim's page in 20.4 s, every wait over a second a turn of Jo's 5 s loop,
  and no sealed word in any file but a person's own. Its run-of-show is
  `demo/grapevine.md`; GRAPEVINE §7's product bars and kill bar, runs
  with real neighbours, are the operator's and never-ran.

That is V1, released 16 October 2026 (`RELEASES.md`; the page published at
`de8923f7`, commit `5bc0db8e`), and it meets its groups at n0's relays and
lookup, as `cli::SHIPPED` names now. V1.1, released 17 October 2026, is the
page and downloads published at `a555a225`: V1 with the north-star audit's
fixes, the loopback patch, the trust floor and pinned carried code, and none
of House rules, which waits for its run and this queue's closing audit.
V1.1.1, released 18 October 2026, is the page and downloads published at
`6db0cccf` (tag `v1.1.1`, on no branch): V1.1 with the Linux build's
allocator fix (`61a06614` here) and nothing else, House rules still not
in it (`demo/runs/v1.1.1/2026-10-18/`). V1.1.2, released 18 October 2026,
is the page and downloads published now, at `aacb3fe5` (tag `v1.1.2`, on
no branch): V1.1.1 with a group on a screen (`7a385472` and `2bebba22`
here) and nothing else (`demo/runs/v1.1.2/2026-10-18/`).

**After V1**, in this order (the operator, 2026-10-16: "Ok so the queue
next is: loopback, canon, then guest"; "Yes, floor and pin first"):
- the loopback patch (`ralph/next/v1patch/`): the tests' allowance of
  `ws://127.0.0.1:* http://127.0.0.1:*` out of the published group page and
  into the page the tests serve (svrn-docs `rings/current/WHAT_RUNS_AS_YOU.md`
  §2, §8); landed, `c739605b`, checked by
  `the_published_page_allows_no_loopback` (`crates/svrngs-node/tests/browser/main.rs`);
- the canon queue (`ralph/next/canon/`, its `CHARTER.md`), what runs as you
  pinned before a house's rules are carried (WHAT_RUNS_AS_YOU §1, §7;
  svrn-docs `rings/current/CANON_IN_SVRNGS.md`): its economizing pass first
  (landed, `088d6f32`), then the trust floor, the one file that pins every
  page's policy and every listener, with a gate in `scripts/check.sh`
  (landed, `1467d286`); then a carried app pinned at the files its group
  brought, each new version run for a person only after their own yes
  (landed, `40295803`; DESIGN §3, "What runs as you"); then canon-core built to plain
  JavaScript through `wasm2js`, so carried pages get no `'wasm-unsafe-eval'`,
  and carried as House rules on the shelf, membership its standing; then a
  house's run;
- then GUESTS's first build (svrn-docs `rings/current/GUESTS.md` §10,
  2026-10-18; `ralph/next/guests/`), cut by a lean pass. The operator chose
  "A link in any chat", "+150 repo, +60 core" and "At the run". A member
  has a guest by a link sent in any chat. The visit is folded with its
  bounds held at two guests per host and 48 hours. The door lets a guest in
  at their host's own lends, and every home page shows who's visiting.
  Then a run on one host, and the queue stops for a visit with real people,
  whose bars the operator sets before it. GUESTS's later parts wait in §2.
- then the models queue (`ralph/next/models/`, written and held for the
  operator's launch, 2026-10-22: "still held"): the pantry's minimum, from
  svrn-docs `rings/current/MODELS_ON_THE_RING.md` and far smaller, an app
  in `apps/pantry/` and no platform line. The operator: "What is the
  absolute minimum we can offer that makes it near obvious for the
  community to extend and add what they want near trivially to the
  architecture? That's the main point for this." A lend that speaks OpenAI's
  API is a model and one that speaks MCP is a tool. `pantry list` shows them,
  and `pantry serve` gives one local address that routes each request by
  its `model` to the friend who lends it. Behaviour beyond that is an
  OpenAI-in, OpenAI-out adapter; the vision bridge is the worked example.
  Its first row measures the story's open servers here and stops for the
  operator to choose. Its story, "Your friends' local AI, in the app you
  already use", is a screenshot fixed by a coder that cannot see, voice in
  and out, and a friend's tool, run on one host against bars C1 to C7. No
  Ollama ("Fully open and high integrity software only"). The queue landed:
  `apps/pantry/` lists every lend that answers `/v1/models` or MCP's
  `initialize`, and `pantry serve` gives one OpenAI-compatible address
  routed by `model`, with the vision bridge as the worked example of an
  adapter. Its run (`demo/runs/models/2026-10-22-run/`) met C1 to C7 on one
  host; C7's limit is could-not-judge there because every path ends in the
  lender's home, and the door's own tests prove it. What the run taught
  waits in §2. The queue now stops for a run with real people, whose bars
  the operator sets before it.
- the stall queue (`ralph/next/stall/`), run alone (the operator,
  2026-10-23: "Now, alone"), closed on evidence the same day: the streaming
  test's 9 MB range held still after its 206 in all six of models-fresh's
  closing gates, and made no red in a battery of 13 runs nor in the gate's own
  command, under loads to 132, so load does not make it. The cause is
  could-not-judge; a step that times out now says what its body had drained
  (54844461), and the row reopens at the next red in a gate;
- the capability queue (`ralph/next/capability/`), one deliverable and one
  release (the operator, 2026-10-23: "I'd like to bundle it all in with"
  svrn-docs `rings/current/WHAT_CAN_ACT_AS_YOU.md`), built 2026-10-23 to
  2026-10-24. The security fixes that note found: a navigation judged as a
  fetch (c08db75d), Money's corrections only from their author (fd3347ac),
  and a phone's holder a key its pages cannot copy (0c9032ed). Consent you
  can see: the open says what an app does and is the yes (d55b5506), and
  "What acts as you" (36aa173f). Authorship by signature (capability-signed,
  ab98bda1), which built what §2 held as "a phone's write signed by the
  phone's own key", and keys everywhere (capability-keys, c7e81354). Its
  release, with House rules, is V1.2, published 25 October 2026 from
  `0c9dc094`, tagged `v1.2` (the operator: "Ok release it";
  `demo/runs/capability/2026-10-24-release/README.md`); what it left
  waits in §2, each with its trigger;
- then what the capability queue left keyed by name, the first row after
  its release (the operator, 2026-10-25: "Record now, fix after release";
  "Just knock it out here"), landed 2026-10-25 (keys-names, 4a723d38 to
  249823cb): Money's balances and Events' who is coming keep each person
  by first key and say them by name, "sam (joined 3 Oct)"; an act an older
  computer wrote for someone is theirs by key where one person of that
  name had joined by then, from the node's answer (`rail.rs` `ops_json`;
  the operator chose the node over the core and the apps, "Node"); and
  the door's "did not answer" rows, the removals and guest hosts name each
  by key beside the name. The grapevine still judges a correction, a
  person's own records and who carried an ask by name
  (`apps/grapevine/grapevine.js:237`, `:248`, `:482`): a row of its own;
- the give queue (`ralph/next/give/`; the operator, 2026-10-25; svrn-docs
  `rings/current/GIVE.md`): any port through the door, raw UDP and TCP, so
  "if you can run it on localhost, you can share it" holds past HTTP; a
  caller check on the record API, since a container on a Mac reaches its
  loopback (GIVE.md §5); then Give, in `apps/give/`, a group keeping an
  app in rootless podman, on svrngs's public surface alone. Its
  economizing pass cut 13 lines (3acd44aa, 842f9da3, 36f58a12; the cap
  18,228, 85a1817b) and estimated the svrngs rows at +2 to +5 core and
  +515 to +1,000 library (`demo/runs/economize/2026-10-25-give/SURVEY.md`
  §6). The operator answered its `NEEDS_HUMAN.md` (a7865e6e): the ceiling
  +1,005 and +5, at most 19,233 and 3,244. Landed, give-token: every record
  route asks a program for the token in the data dir's `token` file and
  refuses one without it, 401 `no_token` naming the file; the command,
  install.sh and every app that asks the record show it; +48 library, 0
  core, the cap 18,276 (662803be; `crates/svrngs-node/tests/record_api.rs`,
  `a_program_without_the_token_is_refused_on_every_record_route`).
  Landed, give-session: the home page and each lend's origin answer a
  browser only with a session an open made (`svrngs open`, install.sh's
  opener, the home page's `/to`) and a program only with the token; a
  lend's page reads another at a capability in its read address, the
  director's decision (`demo/runs/give/2026-10-26-session/DECISION.md`); a
  container on a Mac is refused (`demo/runs/give/2026-10-26-container/`);
  +216 library after its own cut of 15, 0 core, the cap 18,492.
  Landed, give-udp: `svrngs share <port> --udp` lends a UDP port, its
  packets carried as QUIC datagrams on `Alpn::Port` and reassembled whole
  past `max_datagram_size`, a stranger and a person past the limit refused
  (`crates/svrngs/tests/udp.rs`, `crates/svrngs-node/src/door/udp/tests.rs`);
  +249 library, +2 core, 0 `apps/`, +1 surface (a lend may be UDP), the
  caps 18,742 and 3,242.
  Landed, give-tcp: `svrngs share <port> --tcp` lends a TCP port, each
  connection a QUIC stream of its own on the lend's one connection, one
  stalled connection holding back neither another nor the record, a
  half-close carried, a stranger and a person past the limit refused
  (`crates/svrngs/tests/tcp.rs`); +126 library, 0 core, 0 `apps/`, 0
  surface (the same item, a lend may be TCP), the cap 18,868, its raise
  landed for the operator (c0d3b829; "Go for both", give-play's too).
  Landed, give-play: a member presses Play on a raw lend's row, and the
  lend opens on `127.0.0.1` at its kept port, UDP or TCP as it is, the same
  after a restart, gone on Stop and when the member stops standing, and
  listed under "What acts as you" with its Stop
  (`crates/svrngs/tests/play.rs`, `crates/svrngs-node/tests/browser/play.rs`);
  +226 library, 0 core, 0 `apps/`, +2 surface (a lend's `raw`, and
  `/v1/play` answering a program holding the token), the cap 19,095, its
  raise landed for the operator (2f454259).
  Landed, give-play-scene: the gate's "scenes" step plays two lent ports,
  a UDP and a TCP echo program, each person pressing Play on the other's
  row and their bytes crossing (`crates/svrngs/tests/scenes/ports.rs`,
  b9fbd681), red on a listener unbound behind its row; the run-of-shows
  `demo/factorio.md` and `demo/minecraft.md`, written before any run; 0
  in each ledger. The two-home sessions they describe wait for the
  operator, who set no bars ("we can do the two-home session another
  time", 2026-10-26), and are never-ran until played. Beside them stands a
  rehearsal on one host, Factorio 2.1.23 through a UDP port lent and Play:
  9 minutes, no desync, no drop, its path neither of the two the bars
  would name (`demo/runs/factorio/2026-10-26-rehearsal.json`, d5f65cdb).
  Not carried: the jellyfin, movie-night and taste `check.sh` runs,
  outside the gate, which open a lend's origin bare. The operator, "Let's do
  half the rows": the loop stopped after the svrngs half, at
  give-play-scene, until the operator said Give's half goes, after
  give-cut (HUMAN-give-half, 2026-10-26). Landed, give-cut: the lender
  asks a raw lend's dialer's standing again every sweep, 1 s, and closes
  the connection once they no longer stand, so a removed member's open
  TCP connection or UDP flow ends on the lender's side, not their own
  (`crates/svrngs/tests/tcp.rs`, `udp.rs`,
  `a_member_no_longer_standing_is_cut`; 662e30be); +16 library, 0 core, 0
  `apps/`, 0 surface, the cap 19,111, its raise landed for the operator
  (6019879b). Landed, give-app-runtime: `apps/give`, a crate linking no
  svrngs crate (`tests/surface.rs`), with a card read once and refused by
  field, a runtime naming no svrngs type that starts, checks, snapshots,
  restores and stops a card in rootless podman, and `give check`; the
  Minecraft and Factorio cards pass it on this Mac and on Linux
  (`demo/runs/give/2026-10-26-cards/`), a plain copy and a Mac's new
  machine never-ran there; 0 core, 0 library, `apps/` about 1,430 lines
  of Rust by lean_check's rule and 68 of YAML, past GIVE.md §9's 460 to
  860 for these pieces, and Give's surface its five `x-svrngs` fields and
  `give check`. Landed, give-app-keep: the keeper fact's one fold
  (`apps/give/fold`), linked by Give's program and built to JavaScript for
  its page (`apps/give/build.sh`), both held to `fold/cases.json`; `give
  keep`, which lends an app only while the record names its computer,
  copies it to the group's folder every 5 minutes and as it stops, each
  copy naming its card version, and stops it when another computer keeps
  it; `give page` and the carried page, with "Start it for the group" and
  "kept by"; and scene 1 with the stand-in, in the gate's scenes
  (`crates/svrngs/tests/scenes/give.rs`). 0 core, 0 library, `apps/`
  about 1,160 lines by lean_check's rule (the fold 249, the program's
  record, lending, shelf, page and keeping 619, its commands +116, the
  runtime and card +24, the page 93, the build 50), against GIVE.md §9's
  80 to 150 for the fold and 500 to 950 for supervision and snapshots;
  Give's surface gains `give keep` and `give page`. Who may keep is asked
  of the operator, any member until then
  (`demo/runs/give/2026-10-26-keep/QUESTION.md`);
- sealing, at its §2 trigger.

The operator's switch stands apart from that order: when they flip
`SHIPPED` to svrngs's relay for production (§2; DESIGN §7), #134 and #135
(`ralph/REVIEW_FINDINGS.md`), whose moment is that switch, land with it.
The grapevine's standing carries are a row after V1 too (GRAPEVINE_DEMO
§12.1).

## 2. Later, each with the trigger that brings it back

| Waits | Trigger |
|---|---|
| an app SDK, a sandboxed fold engine, signed manifests and install acts (QuickJS behind an `Engine` trait was `crates/svrngs-engine`, deleted 2026-10-04 with no user; it is at tag `e0-bot`); and the runner that would carry an app's function to a person's data, its yes to that code at a version — scoped use's spike served its operation from a program its owner runs, as the survey orders (`rings/current/ACL_AND_SCOPED_USE.md` §4: "a subsequent convenience to earn with a real distributed app"; `demo/runs/su/2026-10-09/` §7) | an app needs its state offline on a device that is not its host, or strangers' code runs on members' devices, or an app sends its function to run beside a person's data |
| a native phone app, and groups of phones only; first, phones as replicas of the group (operator, 2026-10-07): a member's phone holds the group's record and the files it shares, and syncs with whoever it reaches through a relay, so the group stays reachable while every computer sleeps (DESIGN §8, risk 1) — a replica, not a relay in iroh's sense, since a phone on cellular sits behind its carrier's NAT. The phone's OS decides it: a page cannot listen and a browser's iroh is relay-only (DESIGN §3, "The room", call 2), iOS suspends an app soon after it leaves the screen, and a network extension (the VPN slot) runs in the background under a memory cap. The spike, on iOS first and no larger than proves it: an iPhone app whose network extension holds an iroh endpoint with a member's key and answers a computer's dial by key every few minutes for 24 hours, the phone locked in a pocket; recorded, answered of dialled, memory against the cap, battery and data, against bars the operator sets before it runs. The record on the phone follows only if the endpoint stays reachable; Android after iOS | after Demo C, or the operator moves it in (operator, 2026-10-10: "I don't want to distract us from the core right now, but wanna keep it on the back burner"; its design, deferred, svrn-docs `rings/current/PHONE_REPLICA.md`); or the browser cannot keep what a group needs — its key, or a copy — measured |
| the phone's door, a spike: an iOS packet tunnel extension holding the node's endpoint with a member's key, serving a group's raw TCP lend on the phone's 127.0.0.1 to the phone's own apps, from anywhere, the app closed: Jellyfin's app playing a film over WiFi and cellular, music through a locked phone, memory against the tunnel's cap, a stranger refused and a leak's kill seen first. The node already builds for iOS with libc held at 0.2.189. Its design, bars proposed and cost (about $99, one row, three and a half to four and a half hours of the operator's), is `demo/runs/phone-tunnel/2026-10-26/DESIGN.md`; its case is svrn-docs `rings/current/WITHOUT_BEING_WATCHED.md` §8. It dials out where the row above answers, and shares that row's first half: an endpoint alive in a tunnel under its cap | the operator's Apple developer account is settled and the operator sets the bars (operator, 2026-10-26: "I'll get the developer account settled before we start") |
| a chat's own door: a package that posts a group's link for a person, names a group for its channel, or follows a channel's members (E0's, at tag `e0-bot`, DESIGN §3) | a group asks for one that a pasted link cannot answer, counted |
| Windows | a group asks |
| the two-home game sessions of `demo/factorio.md` and `demo/minecraft.md`: each game 30 minutes between two homes, once on a direct path and once with the relay forced, desyncs, drops and the game's ping counted per path. Never-ran: no bars are set, and the one rehearsal (`demo/runs/factorio/2026-10-26-rehearsal.json`) ran both computers on one host, so neither path | the operator sets the bars and plays them (2026-10-26: "we can do the two-home session another time"), or a group playing through a lent port reports a desync or a drop |
| the switch to svrngs's relay, parked at the gate (relay-park; §3, 2026-10-13): the build's one constant flipped from n0's to svrngs's at `relay.svrnme.sh`, then `scripts/deploy-relay.sh`, then `scripts/deploy-page.sh`, in that order (DESIGN §7), each the operator's; n0's relays and lookup the command's default until then, and nothing else changed by it; with it, #134 and #135 (`ralph/REVIEW_FINDINGS.md`), whose moment is the switch | V1 released on n0's relays (§1, "V1"; §3, 2026-10-15), and the operator starts paying for svrngs's relay (the operator, 2026-10-13: "until I'm actually ready to start paying / flip the switch for production") |
| AirPlay and Cast from a page, and DLNA: a screen handed a lend's address by a phone, for an app with no client of its own on it. In the reads (`demo/runs/u6/2026-10-05-screen/reads.json`) AirPlay from Safari on an iPhone showed the LG no video three ways — the page's address, which only the phone's worker answers; jellyfin-web, whose hls.js sets `disableRemotePlayback`; a plain LAN file, which the TV never fetched — and iOS has no Cast sender. Until 2026-10-05 this was U6 (DESIGN §3, "The room", call 1) | an app with no client of its own for a screen is asked for on one, counted |
| a public address per lend, for a room with no member's computer in it: a relay that forwards bytes it cannot read to the lender, who holds a certificate per lend — a new service, and Let's Encrypt's per-domain limit (DESIGN §3, "The room", call 2) | a group with no computer in the room asks, counted |
| a relay at a bare address, no name needed (svrn-docs `rings/current/GROUP_RELAYS.md` §4.4): a six-day Let's Encrypt certificate for the address, its `shortlived` profile, which tokio-rustls-acme 0.9.1, iroh-relay's ACME client, cannot ask — its `Identifier` is `Dns` alone and its order names no profile (`acme.rs:129-130`, `:301-303`); no second ACME client beside it (`ralph/next/relay/CHARTER.md`) | iroh-relay's ACME client asks for an address, or a member who would relay has no name to give it, counted |
| a relay for a group's own refusing a newcomer whose key does not yet stand (`GROUP_RELAYS.md` §4.3): a link's issuer whose relay is `svrngs relay` without `--open` would refuse the joiner dialling it there before its `Admit` is written — read from `relay.rs`'s admission, not run; since relay-default `new --relay` names a group's own in its links, so that relay runs `--open` until this lands (`svrngs help`) | a group runs its own relay and names it in its links, counted |
| watching together from different homes: one transcode, shared, in step | a group asks to watch the same film apart at the same time |
| sealing in the infra: an entry sealed to some keys, shown beside it before it is written, "Only Priya sees who chipped in" (svrn-docs `rings/current/WHAT_CAN_ACT_AS_YOU.md` §9, §10 row 4: +130 to +260 repository, +60 to +120 svrngs-core). Open before it can be sized firmly: a phone's lend origin holds only a holder key (DESIGN §3, "Lent apps", call 6), so a carried app on a phone cannot yet open a sealed entry. Its queue is written at its trigger, from that table and what the capability queue found | two apps need the same seal (lending, Tap) |
| an app declaring what it reaches, so lending it says "Reel opens your friends' Reel as whoever opens it" and the lender only agrees, in place of typing `svrngs share --reads <kind>` (WHAT_CAN_ACT_AS_YOU §11): today the lender declares it, and the open says it from what they declared (DESIGN §3, "Lent apps", call 6) | the first app outside svrngs's own that needs `--reads` (the operator, 2026-10-23: "Wait, with a trigger") |
| a sandboxed carried page: a framed page with its links brokered, so an app cannot send out what it reads. Today the floor keeps out foreign code and reach nobody chose, and a carried app is trusted with what it reads because its people chose it, its files pinned and each version their yes (DESIGN §3, "Or a folder the group carries"; `scripts/trust_floor.txt`'s header; WHAT_CAN_ACT_AS_YOU §5) | code arriving from outside a person's rings: apps carried beyond the people who know who wrote them |
| the capability core as a move of its own: one `may` in svrngs-core deciding every capability, Know, Reach, Read, Append and Decide, over `Grant`s and a `Standing` (WHAT_CAN_ACT_AS_YOU §10, its target shape). Sized 2026-10-22 at +160 to +250 svrngs-core and +140 to +270 repository, removing 15 to 25 lines, since the deciders are mostly single-sited already (`Pass::check`, `Append::decide`, `Runs`, `Guest::live`, `reaches`); the log side's `Grant` and `Capability` came with authorship (`crates/svrngs-core/src/grant.rs`), and reach keeps `Pass::check` | a third change touching those sites (WHAT_CAN_ACT_AS_YOU §10, 2) |
| group compute: asks leased by members' computers and answered as acts; and peer inference, a source's excerpts released to another member's computer for a job (`rings/current/ACL_AND_SCOPED_USE.md` §5), out of scoped use's spike (`demo/runs/su/2026-10-09/` §7) | Ask is next, or an owner's operation needs inference their own computer cannot run |
| files beside the record; the live channel beyond what U3 carries | the reference app that needs each |
| a second chat adapter, and the trait it earns | a group on another chat asks |
| forgetting from the journal, by meaning (svrn-docs `rings/current/A_RECORD_FOR_YEARS.md` §4). Upstream's seal forgot by author and sequence: it retired everything its signer wrote before it, on every copy it reached, so a member who sealed took their expenses out of everyone's book, and any app could set it off (that draft's §2). It went in arch-seal (the operator, 2026-10-15: "Remove it"): an app's seal is refused, and a seal an older build wrote retires nothing here, its acts below it still counting (`ring_sync/tests.rs` `a_seal_another_member_signs_retires_nothing_here`). A copy an older build pruned says what it misses, as holes; this build keeps to the run such a copy claims, so it is not sent what it pruned every round (`sync.rs` `ops_missing_from_within`) | a journal grows past a size measured to matter (that draft's §6, question 3) |
| a key per group on a computer, so a person who left a group, or was removed from it, can rejoin it from that computer as someone new under the same name. A computer signs and dials every group with one key (`crates/svrngs/src/identity.rs`; `node.rs`, "One key signs and dials"), and a cut key never stands again (DESIGN §6), so today that person rejoins only from a phone, which keeps a key per group (`web/store.js`). groups-keys' survey (`demo/runs/keys/2026-10-16/SURVEY.md`, 64ed421b) laid out five shapes: A, a signing key derived per group, which derives the cut key again unless an epoch is kept per group; B, an `Admit` naming a signing key and the dialled key, 30-45 svrngs-core lines and 150-225 in the repository; C, an endpoint per key the computer holds, as a phone has, 0 svrngs-core lines and 220-330 in the repository, every existing group's keys unchanged, one more relay connection per rejoined group; D, the cut key admitted again, which makes no one new and undoes a keeper's removal; E, a second svrngs on the computer, two home pages. C is the smallest safe shape; it did not fit the 41 lines the groups queue had left (`NEEDS_HUMAN.md` beside it), and the operator kept leaving final from the computer through V1 (2026-10-16: "Final from this computer, through V1"). The estimates are from the sites, not a spike; a spike of C's signer per group is the first step | after V1, or a person who left or was removed asks to rejoin from the same computer, counted |
| a keeper the group chooses, or one that rotates (Ostrom's third principle, collective choice; DESIGN §8, risk 2): today the keeper is the maker's key, moved only by the keeper's own hand-on (DESIGN §6, "Removal has one decider"), and a hand-on survives its giver's removal (§3, 2026-10-12); a group's choice would be acts its people sign that the fold counts into one hand-on, or a turn the fold moves on its own — a new rule of DESIGN §6, the operator's | a group asks to choose its keeper, or to take turns, counted |
| a ladder of sanctions people can see, removal its last rung (Ostrom's fifth principle, graduated sanctions): the gentler steps are each owner's already and need no keeper — a yes stopped (`question.js`'s Stop, on both pages), a share stopped (`svrngs unshare`), a library kept back from everyone the middleware signs in (v1-onboarding (e), landed, `jellyfin-mw --keep-back`; from one person, not yet) — so the ladder is only those steps shown to the group beside its removals (DESIGN §6, v1-keeper-office (b)) | a group removes someone for what a lesser step would have met, counted |
| GUESTS's later parts (svrn-docs `rings/current/GUESTS.md` §8): a visit made through the grapevine thread's page; the welcome, a housemate's lend opened to a guest by its lender's yes; the house's two settings, `GuestRules`; those settings decided in House rules (CANON_IN_SVRNGS §4); and the words for people about guests | each its own, in GUESTS §8; the words for people, a visit with real people |
| exporting a group as one document (upstream's checkpoint route), checking the asker is in that group | a group refounds from a copy, or someone asks to export one |
| the endpoint's self-heal (upstream's `self_heal.rs`, `iroh_watchdog.rs`) | a node's endpoint wedges in a recorded run |
| skipping a round's members known offline. Dialling them at once landed (asleep-1, `ring_sync::run_one_round`), so an asleep one holds only its own dial; the round still ends when that dial gives up, about 4 s since events-wait (`dial::DIAL_WITHIN`; 2 s at events-asleep, 10 s before), and a door's ask begun meanwhile waits for the next round (DESIGN §3, "Joining") | a round's measured time nears its interval, or a newcomer is refused while a round waits on an asleep computer, counted |
| a phone making a device link (phase 2 of svrn-docs `rings/current/ONE_PERSON_MANY_DEVICES.md` §3). Since bash-devices a computer makes one (`svrngs link <group> --device`), and a phone or another computer opening it joins as that computer's person, asked no name (DESIGN §3, "Joining"); a phone cannot make one, since only a computer redeems a link (the row "any member's node redeeming a link", below), so a person whose only device in a group is a phone adds no second one as themselves | a person whose devices in a group are phones alone asks to add another as themselves, counted; or any member's node redeeming a link lands |
| a device link apart from its issuer's one open link: a computer has one open link (DESIGN §6, "A link admits each person once"), so making a link for its person's devices closes the link they sent a friend, and the other way about; the command says so as it does for any link (bash-devices; `demo/runs/devices/2026-10-12/README.md`, finding 3, not measured) | a person's link to a friend is closed by their device link, or the other way, counted |
| joining two people of one name into one person: a group from before bash-devices where a person's second device joined under their own name holds them as two people, not migrated (svrn-docs `rings/current/ONE_PERSON_MANY_DEVICES.md` §3), and a race or a member's hand can give two people one name; since bash-same-name the keeper removes either by when they joined (DESIGN §6), but making one a device of the other is a change to who a person is beyond phase 1, the operator's (ralph/next/bash/CHARTER.md, "Leave for the operator"), never built by that row | a person standing as two people of their own name in a group asks to be one, counted |
| removing one lost device of a person: a `Remove` is per key already, and the person's other devices keep standing (svrn-docs `rings/current/ONE_PERSON_MANY_DEVICES.md` §3, "Removal"), but `svrngs remove` names a person and takes every device of theirs (bash-devices); naming one device — by the person from another of theirs, or by the keeper — is a change to who a person is beyond phase 1, the operator's (ralph/next/bash/CHARTER.md). Until then a lost phone's key stands as its person until the keeper removes the person | a person loses a device that stands in a group, counted |
| the wire published for a second implementation | someone sets out to write one |
| local discovery on one WiFi | Apple's multicast entitlement is granted |
| any member's node redeeming a link, once its `Invite` has synced | a newcomer's link fails because its issuer's computer was asleep, counted |
| a link's life set per link, other than a day | a newcomer meets a lapsed link, counted |
| measuring a lender's upload rather than taking the figure they declare (BBR3's pacing estimate, or a timed saturating pull) | a declared figure is wrong by enough to stall a viewer in a recorded run |
| a per-process tie between a node and the services it lends (upstream's `mint_tie`). A middleware believes loopback: a process on the lender's computer naming a person's key at its port was answered as them (`demo/runs/mw/2026-10-08/mw-1/`), which is the lender's own (DESIGN §3, "The room", call 6); and a request at a lend's origin with no `Sec-Fetch-Site` is taken for that lend's own page (sup-1, `origin.rs` `app_of`), which neither widens nor narrows what a process could reach before | an app's act is written under a person by a process the lender did not run, or one computer is shared by people who do not trust each other |
| svrngs as an OpenID Connect provider for its lends, in place of a sign-in program per app: a lender's app with a stock "Sign in with OpenID" setting (Immich, Audiobookshelf, Grafana, Forgejo, Jellyfin's SSO plugin) names svrngs as its issuer, and the lender's door completes the login for whoever opened the lend, asked no password. Then `apps/jellyfin/mw/` and the well-known table's `helper` line retire (`crates/svrngs-node/src/well_known.txt`). Its design began 2026-10-20, svrn-docs `rings/current/SIGN_IN_WITH_SVRNGS.md`. The operator, of `jellyfin-mw`: "If it's not something anyone in the room can run off the shelf I'm pretty turned off by idiosyncratic tweaks for specific apps"; then "if there's something that solves a general svrngs problem and the n+1 use case for it is now easier then we can do it" | the demo evening is over (operator, 2026-10-20: "let's add the OIDC trial"); the trial is two apps, Immich or Audiobookshelf first, then Jellyfin's SSO plugin |
| getting the latest svrngs from the home page (svrn-docs `rings/current/UPDATES.md`): a version svrngs knows, and the page's Update, which takes `install.sh`'s steps and check — fetch, `SHA256SUMS`, swap, restart — so nobody pastes the line again; the Linux unit restarting always, as macOS's does. Today an update is `install.sh` run again (`web/install.sh:67-69`). The operator: "We should have a better upgrade story than just running an install script again", then of a first draft with a release key, the group carrying versions and versioned phone pages: "Feels a little overcomplicated on the release work; people just need to be able to get the latest code simply" | the demo evening is over |
| a synced folder's later parts, each with its trigger. A mounted folder: every file of a group's folder listed from the record's layout and fetched on open, by verified range, as group folders' streamed spike fetched (`demo/runs/gf/2026-10-09/`, its endpoint dropped at syp-1; `demo/runs/sy/2026-10-09/README.md` §9, "A mounted folder") — a file system of svrngs's (macFUSE, or Apple's File Provider), a dependency and a process apart. Its streamed spike's rows that only a folder not on the viewer's disk needs — a stub that reaches its viewer, who the endpoint answers, stubs in step with the record (that README's §9, rows 1, 3 and 4) — wait with it. A version as its blocks: a file a hash sequence of its 1 MiB blocks (iroh-blobs' `HashSeq`), the record's entry naming the sequence, so an edit moves only the blocks it changed — a 1 MiB edit inside 100 MiB moved all 104,857,600 bytes to each member on every path, before perf-2 and after it, in 1.9 s on loopback, and again on production in 7.04 s through n0's (syp-5, `demo/runs/syp/2026-10-10/rerun/README.md` §5), and what an entry's hash names would change in the record and in what a store keeps (perf-1, `demo/runs/perf/2026-10-10/README.md` §9 T4, §10 candidate 8; perf-2, `demo/runs/perf/2026-10-10-make-it-fast/README.md` §3 T4; the operator, "Wait, revisit after the rerun", §3). A file past one holder's upload: a 100 MiB file reached each of two members at 83 MB/s at once, about 167 MB/s from its one holder, where one member alone took it at 139 MB/s, a bare iroh-blobs get's rate (that README §4) — a member fetching from another that holds the file is unmeasured. A joining member's wait to stand: its share refused until the round admits it, 54-58 s of the round's 60 s, where its every file then arrived in 1.7 s of 10,000 and 61.5 s of 100,000 (that README §5) — perf-2's harness's, members started apart; a member joining by `join` stood before its share, and held 1,000 files 0.53 s after it (syp-5's rerun, §5). Dialling direct before a relay: no gain measured, iroh having moved every connection on all three paths direct before its first fetch, in perf-1 and perf-2 alike (perf-1's README §9; perf-2's §3) | the mounted folder: a member whose disk cannot hold the group's folder, counted. The blocks: a member waiting on a large file moved whole for a small edit, counted, or the operator queues T4. A file past one holder: a member waiting on a large file several fetch at once, counted between computers. The wait to stand: a newcomer's folder waiting on the round past a bar the operator sets. The dial: a fetch measured through a relay between two members a direct path reaches |
| a fetched hash's tree of hashes left on disk after its bytes are dropped: iroh-blobs 0.103 writes a fetched blob's tree (`<hash>.obao4`) while the blob is partial, and when it completes with a tree small enough to keep inline (a blob over 16 KiB and up to about 4 MiB) copies it into its database and leaves the file (`store/fs/bao_file.rs:215-224`); a drop of the entry then removes the database's row alone (`store/fs/meta.rs:594-596`). No byte of the content stays, but the file's name is the hash: each of the 10 pieces Bo and Cy carried left 192 bytes after its lapse (`demo/runs/onair/2026-10-13-blob/`, `stores-after.txt`), and a group's folder's fetches leave the same. Not fixed beside the store: the file is the store's, and only iroh-blobs knows when it is free | a person asks that what lapsed or was dropped leave no name on their disk, or a store's directory is counted to grow by it past what matters, or iroh-blobs removes it upstream |
| a person who only searches counted apart from one who watches: the door counts each on a lend for `WINDOW`, 30 s, after their last request (`crates/svrngs-node/src/door/limit.rs:17-19`), a search as a film — Movie Night's searcher was counted on both Jellyfins 26.3 and 26.5 s after one search and not at 36.2 and 36.4 s, the most at once 2, nobody refused (`demo/runs/mn/2026-10-08/doors.json`, `limit`) | a lend at its limit refuses someone while those it counts only searched, counted |
| budgets on an owner's operation: how often an app may ask it, so asks cannot rebuild what it keeps back (`rings/current/ACL_AND_SCOPED_USE.md` §4, §6). Ana's taste ranks whatever films it is given, and ranked an app's two that sent her yes itself (`demo/runs/su/2026-10-09/verdicts.json`, "the unasked yes") | an operation's answers rebuild what its owner kept back, shown in a run, or an owner asks for a limit on one |
| WebKit's closed-range seeking through a lend: its seeks back to back, taken in serial 64 KiB reads of each short part, a round trip each, missed 5 s in 1 of 8 tries after u5-11, where straight at the canary over the same link they missed in none (`demo/runs/u5/2026-10-06-seek-stall/`; DESIGN §8, risk 4); a rate in `scripts/check.sh`, not a gate. And Chromium's on a loaded host: its seeks are a gate, but sup-4's check, sy-1's three on the branch `folders` and REVIEW-audit-sup-auto-2's each missed only them, at load averages of about 40-130 on this host's 12 cores, with nothing near them changed, and passed alone; since chk-1 a miss past 3 a core is could-not-judge, never passed (operator, 2026-10-10), the bar's one home `crates/svrngs-node/tests/browser/load.rs` (`BUSY`), which the browser tests' film reads too since bash-film-load — its Chromium seek, after the worker's stop, missed at 11.4 s in one check and stayed under 3 s in 29 runs at loads to 19 a core (`demo/runs/film-load/2026-10-12/`) | Demo A's real-phone sessions miss the seek bar in Safari more often than WebKit straight at the server on the same link; or Chromium's seeks are could-not-judge in more `scripts/check.sh` runs than they are judged, counted |
| a page whose film never starts once its relay socket breaks: in bash-devices-1's `scripts/check.sh` WebKit's film played nothing in 45 s in its first browser (`crates/svrngs-node/tests/browser/main.rs`, "the film must play"), its group's page having logged "WebSocket connection to … /relay failed … Socket is not connected" before it opened the lend, and played in 0.6 s once the browser was opened again; the film's worker, whose console Playwright cannot read, logged nothing. Not seen in 29 runs an engine at loads to 19 a core (`demo/runs/film-load/2026-10-12/`), so whether it was the harness or a worker that stops for good is not known; every browser run keeps the relay's and the doors' lines in `target/traces/browser.log` since bash-film-load | the film fails to play again in a `scripts/check.sh` run, read with that trace; or a phone's page stops answering after its relay socket breaks in a recorded run |
| the floor: every app under `/l/<label>/` of one address apart from the page's, on any static host, the link carrying that address after its token (a second address given to `svrngs new`, u5-2; dropped by u6-00). Its cost was one origin for every app, so a lent page could read what another kept and use another's pass while it lasted, and its every app lost its root: the catalogue run lent five self-hosted apps on it in Chromium and WebKit and none got past its sign-in, each asking for `/`-absolute paths the floor's host answered 404, and Paperless's post refused by its own origin check (`demo/runs/catalogue/2026-10-05/RESULTS.md`, "The floor"; `003e0d5`) | a group whose host answers no name nobody made asks for an app that keeps to relative paths, counted |
| nextest for `scripts/check.sh`'s `tests` step, each test its own process and the binaries at once, as commonwealth-ai runs its suite (its `scripts/sovereign-test.sh:52-62`, `.config/nextest.toml`; test execution 90.5 s to about 19 s over 178 binaries, its `ci.yml:465`). Here the step is about 200 s of serial running and 70 s of compiling (`8142dc03`); 40-70 s of running is an estimate, door's 34 s binary its floor. What it needs, read and not run (the survey of 2026-10-22): browser, conformance and join_page stay on `cargo test`, as their own steps (browser's `JELLYFIN` lock on port 8096 and its `PATH_SET`, `crates/svrngs-node/tests/browser/main.rs:3364`, `:3608-3651`; the harness's `site()` would rebuild `target/web/site` once per test; `traced()` truncates its log once per test); a test group of one thread for peer, leave_page and home_group, which drive Playwright through `web/tests/*.mjs`, as commonwealth-ai's `daemon-boot` group; home.rs's `/running` test alone, since `share::listening()` connects to every port from 1024 to 49151 (`crates/svrngs-node/src/share.rs:196`); a step of its own for the doctests, which nextest does not run and svrngs-core's `compile_fail` proofs need (`admit.rs:365`, `command.rs:28`); `--no-tests=fail`, and never-ran when nextest is missing, never `cargo test` in its place; a `slow-timeout` from the slowest test measured, so a hang is a named failure. On Linux the room's ports (40000-48999, `room.rs` `port_of`) sit inside the ephemeral range, where another test's socket can take one. No retries: a retry that passes is a substituted verdict. Two timed runs before and two after, so the gain is a measured one | `scripts/check.sh` is the longest wait in a row's close, or a row skips it for its time, counted; or the operator moves it in (2026-10-22: "We can put the nextest ideas in some TODO doc") |

| `pantry up`, one command to serve and lend | a provider asks for one after the run, counted |
| the design's card, facts and matcher (MODELS_ON_THE_RING §3, §6) | a person cannot choose among their friends' lends by the model picker alone, counted |
| need acts, standing in the record (§2 of that design) | a person asks the group for a model nobody lends, counted |
| one MCP address gathering every friend's tools | a person adds more than a handful of tool lends and asks for one, counted |
| several models behind one lend through llama-swap (MIT; measured clean at models-scope, but it writes a file path as `model` and adds a supervisor) | a provider wants several models behind one address, counted |
| a stdio MCP server that knows who is calling: `mcp-proxy` passes none of the door's headers, so a bridge that maps them is needed (`demo/runs/models/2026-10-22-scope/data/responses/mcp/stdio-through-door.txt`) | a lent stdio tool must act per person, counted |
| per-person permission on a tool beyond the door's who and how many | a lender asks to let one friend use a tool that writes and not another, counted |
| a pipeline lent to others, reaching lends on someone else's behalf | the operator, since it crosses DESIGN §3, "The room" |
| phones: the pantry's address is on the person's loopback; a phone reaches it through the room or a carried page | a person asks, counted |
| a pantry page on the home page | a provider or consumer who does not use a terminal asks, counted |
| the deterministic spread by the person's key (MODELS_ON_THE_RING §3, 5) | two consumers' loads collide on one provider while another sits idle, counted |
| a wire other than OpenAI's or MCP's (ComfyUI's, SearXNG's own): reachable today at its lend's own address | through the pantry when one is asked for, counted |
| weights over the ring, budgets, A2A, card signatures, and the rest of MODELS_ON_THE_RING §12 | each with its trigger there |
| a view of what is lent as last seen, with when, so freshness has one decider (lends are live-only, so a routing app pays a dial per friend) | a second app must show a lend that is asleep without dialling every friend, counted |
| a readable handle that tells two people of one name apart, so apps stop inventing one (the pantry's `handles` is private to `apps/pantry/`) | a second app must name a person whose name is not unique, counted |
| a server app carried from group to group as a pinned recipe, run on a person's computer only after their yes, never supervised by svrngs ("how will we solve the distribution problem if say someone writes a cool app for their neighborhood"; v1 is to clone and run it) | a group asks for a second app that runs as each of its people, counted |

## 3. Decisions

By the operator, 2026-10-10 and 2026-10-11, on the scene v1 answers to
(`demo/c.md`):
- "Yeah write the scene that we can fulfill and the Demo C run of show":
  Demo C, the group's whole life, after U7, onboarding and the two homes
  (§4); its bars the seat's proposal, the operator's to set.
- on a chat, "Hopefully not more the 50 lines in our own source and a pretty
  commodity integration", then "Rewrite that way": a chat's door is the
  group's link (DESIGN §3, DESIGN §4); a chat's own door waits (§2).
- the phone replica: "I don't want to distract us from the core right now,
  but wanna keep it on the back burner" (§2).
- S3's two holes, "Queue it now", fixed before the merge (syp-6); the
  branch `folders` merged into main by the operator, 2026-10-11 (`4475a89`).
- onboarding by a download script: "it's the most cross platform and
  requires the least machinery. We're a lazy but brilliant dev team; we
  want the most general purpose solutions possible"; and of Demo C's gate,
  "Pasting one line in the terminal is fine" — a computer's newcomer pastes
  one line the group's page gives, and types nothing else (§4, "Demo C");
  the route, the page's host serving the script and the binaries beside
  the page, is DESIGN §7's ("A computer gets svrngs from the group's link").

By the operator, 2026-10-12: "I do think we need to host our own relay
(can't rely on n0) before our v1 ship. And ideally however we do that is
something packageable to make it easy for anyone who wants to run it
themselves"; then, asked, a `svrngs relay` verb in the svrngs binary rather
than n0's `iroh-relay` beside it — one install line, version-locked to the
computers that dial it, the relay svrngs runs for everyone the one anyone
runs for themselves — and n0's address lookup dropped beside its relays,
each computer's relay learned from its group's record. So our own relay
left §2 for the order of work: svrngs's at `relay.svrnme.sh`
the default (DigitalOcean, US West, the operator's inclination since
2026-10-05; deploying it the operator's), a group's own by `svrngs new
--relay <url>`, and n0 none of the path (§4; DESIGN §8, risk 5; `ralph/next/relay/`).
A computer met its groups there from relay-default (a) (`cli::reach`,
DESIGN §3, "Where a computer meets its groups") until relay-park; before it, at
n0's relays and lookup, `RelayConfig::default()`
(`crates/svrngs-node/src/guest.rs:62-74`).

Then by the operator, 2026-10-13, asked what DigitalOcean account
`scripts/deploy-relay.sh` needs: "Can we actually just park this here right
at the gate and keep using n0 until I'm actually ready to start paying /
flip the switch for production?" So svrngs's relay is built and parked at
the gate: n0's relays and address lookup are the command's default again,
chosen — never a fallback when svrngs's relay or a group's fails — until
the operator flips the switch. The switch is one constant naming a closed
set, n0's or svrngs's at `relay.svrnme.sh`; flipping it, then
`deploy-relay.sh`, then `deploy-page.sh` is the operator's, in that order
(§2; DESIGN §7). Nothing the relay queue built goes: `svrngs relay`, each
computer's relay in its group's record, `new --relay`, the deploy scripts
and their guard. A group that keeps its own relay is still met there, and
a link carries its issuer's relay in either posture, so a phone's page
dials only the relays its link and its group name and asks n0's lookup
for nothing, before the switch or after it (`ralph/next/relay/CHARTER.md`,
2026-10-13).

Then by the operator, 2026-10-15, of the grapevine demo app: "And then I
think that's our V1 cut line"; asked, "arch, groups, grapevine; V1 on n0"
and "arch → groups → grapevine". So V1 is the arch queue, the groups queue
and the grapevine demo app, in that order, meeting its groups at n0's
relays, and the switch to svrngs's relay comes after V1, not before it as
2026-10-12 asked; #134 and #135, whose moment is the switch, come with it
(§1, "V1"; `ralph/next/grapevine/CHARTER.md`; svrn-docs
`rings/current/GRAPEVINE_DEMO.md` §12.3).

By the operator, 2026-10-12, after Demo C's dry run (its finding 3): "let's
not make access limits arbitrary AND hard to update. If someone wants to
allow more concurrent users we shouldn't add dumb footgun configs that
people can't update easily through UIs". So a lend whose lender names no
limit lets in everyone in its groups at once, the 3 `share` once chose for
them gone; and its lender changes the limit on their home page, in place,
no command and no restart (DESIGN §3, "Too many at once"; bash-limit).

By the operator, 2026-10-12, "Yes that sounds good", on the seat's reading
of Ostrom: the hand-on is a collective-choice act, a removal operational
enforcement, and enforcement never unmakes an office already given — "the
office isn't the person". So a hand-on survives the removal of the one who
handed on (DESIGN §6, "Removal has one decider"); every removal and every ask to
remove is shown to everyone in the group (DESIGN §6); and a keeper the group
chooses, and a ladder of sanctions, are written to wait (§2).

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
  (§2 keeps it waiting, with its trigger).

By the operator, 2026-10-10, before the synced folder made production
(`ralph/next/lean/SURVEY.md`):
- "Queue them increase the budget, but let's also see if there are
  opportunities to economize": the lean rows cut what the product does not
  carry; main's cap stays 15,350, and the branch `folders`' may rise to
  16,500 at most while it merges main in (syp-3), coming back to main's
  with the last merge.
- the budget counts the product: `demo/runs/` apart, as `apps/` and tests
  are, a recorded run's driver kept as it ran (lean-1, DESIGN §5).
- a group's folder is synced onto its members' disks, not streamed: group
  folders' loopback endpoint and its one-way share are dropped on the branch
  `folders` (syp-1), and watching a member's films stays their lent
  Jellyfin's. A folder fetched on open, by verified range, waits for a member
  whose disk cannot hold the group's folder (§2).

By the operator, 2026-10-09, during scoped use made production:
- group folders' second spike runs now, in a worktree beside scoped use,
  not after it (§4, §2): a spike's code lives on its own branch, and how
  folders become production is decided from its verdict.
- a group's folder is more than its films in one Jellyfin: a folder synced
  both ways on its members' disks, written by any of them, is spiked next
  on the same branch, before group folders' production rows (§4, §2).

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
  14,750 to 15,200 for it (§4, "The budget's moves").

By the operator, 2026-10-08, after the middleware spike:
- the library carries the weight. The door takes the middleware's general
  pieces as hooks a lend declares over HTTP — a credential per person,
  that credential kept out of answers, a script first in the lend's pages
  — and an app's middleware keeps only its own nouns; before group folders
  (§4, "Then"; DESIGN §3, "The room", call 6). The repository's budget rose
  from 14,000 to 14,400 for it (§4, "The budget's moves").

By the operator, 2026-10-06:
- a phone opens each app at an address of its own under the page's host,
  and only so: the floor, every app under one address apart from the
  page's, is dropped (u6-00), since the catalogue run found it carried none
  of five self-hosted apps, each asking for its own root
  (`demo/runs/catalogue/2026-10-05/`), and the repository stood at 13,952
  of 14,000 lines with the room door to come. `svrngs new` refuses a page
  whose host does not serve the page at a name nobody made, saying what
  such a host must do (DESIGN §3, "Lent apps", call 1; DESIGN §7; §2 has the trigger);
- middleware is a spike after Demo B, and no U6 row builds it: a lent app
  that forwards to another, run only by its lender, with svrngs shipping no
  runtime or SDK for it (DESIGN §3, "The room", call 6, with its invariants;
  "or plugin format" dropped by the operator, 2026-10-08, above). Its
  spike, 2026-10-08, was kept by its bars read for their
  spirit, as the operator read them that day ("the bars are guidance to
  align us at the start, the spirit of them are what guides how we
  iterate"): Jellyfin's zero sign-in is a middleware in `apps/jellyfin/`
  (DESIGN §3, "Lent apps", call 4; `demo/runs/mw/2026-10-08/`).

By the operator, 2026-10-05, after five reads on a real TV
(`demo/runs/u6/2026-10-05-screen/reads.json`):
- the screen is the app's own client, and svrngs gives the devices in a
  room reach to a lend through a member's computer on that network ("we're
  adding value, but we're not taking media on tvs and end user devices
  into our scope"); nothing in svrngs knows AirPlay, Cast, DLNA, a TV or a
  player, and AirPlay and Cast from a page wait in §2 (DESIGN §3, "The room",
  calls 1 to 4). This replaces the screen handoff of 2026-10-03, below;
- old TV browsers are not supported: the LG's Chrome 87 could not open the
  page ("a losing cause");
- the room's address comes from a member's computer on the screen's
  network, not from a phone and not from a public address per lend (§2);
- Demo B: five friends at Dee's, Sam's library across town, the people in
  the room on the TV's WiFi; its gate unchanged (§4, "Demo B").

By the operator, 2026-10-05:
- north star 2 holds for the apps people already self-host: `svrngs share
  8096` lends Jellyfin as it comes, and someone who never used it opens it
  from the list, plays and seeks, on a computer or a phone. Films is dropped,
  and Demo A plays in stock Jellyfin; group folders are an experiment after
  U5, settled in the wild. The seven calls that build it are DESIGN §3's "Lent
  apps, as they come" (over "Demo A plays in Films", 2026-10-04; §4, "Demo A");
- the ceiling's fixes 1–3 — the digest, signatures checked once, the
  register, each kept by a tail fold — after Films (read here as after
  Demo B, §4's rule); dialling at once stays a §2 row;
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
  carrying only what the joiner cannot ask the issuer for, and no word DESIGN §1
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

## 4. What was built

The units, each upstream inventory, the demos and what each run measured, as
written when each landed. This is a record, not in force: where it and
DESIGN disagree, DESIGN holds. A reference into DESIGN here names that file
as it stood at `6c7b8296`, where every measurement still sat beside its rule.

**One experience, done delightfully: the group's films, in the Jellyfin a
member already runs, on any phone and the TV in the room.** Sam lends
Jellyfin as it comes; everyone in the group opens it from the list and
plays it here or on the TV; no port forwarding; the lender's password never
leaves their computer. Nothing else is built until Demo B passes, except the
two canaries and U5's catalogue run, which keep Jellyfin from bending the
infra into a media platform. (Until 2026-10-05 this was Films, an app of
our own with one shelf across every member's library; the operator dropped
it for Jellyfin as it comes, DESIGN §3 "Lent apps, as they come"; §3.)

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
  the lender made (DESIGN §3, call 4). Jellyfin kills
  running transcodes by device id and play session
  (`DynamicHlsController.cs:1504`, in a source read once and held in
  neither repository), so whether Bo seeking leaves Cy's play alone is
  u5-3's to measure, never assumed.
- *Too many at once* — how many a lend lets in at once is its lender's
  choice, and none chosen lets in everyone in its groups: no number stands
  in for one nobody chose (§3, 2026-10-12). Its lender sets, changes or
  clears it where they already are — on their home page, beside what their
  computer lends, or `svrngs share --limit` — and the door takes it at its
  next request, with no restart. Past it the door refuses with its reason
  in a person's words, the figures, and that the number is the lender's
  choice, theirs to change (`door.rs`, `busy`); nothing turns it into a
  choice of another lender.
- *The same room* — the TV plays through the app's own client, which
  reaches the lend through a member's computer in the room (DESIGN §3, "The
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
| `commonwealth-transport::iroh` and `mesh-reach::guest` | dial by key: a loopback bridge per peer for the dialer (`IrohTransport`, `HttpBridge`, `pump`); for the dialled, an accept loop that copies each stream into the node's loopback listener (`IrohAcceptor`); the relayed endpoint and its discovery setting | the dialling half forked: one ALPN, no IP transport, no bench relay pin; its bridge then cut (arch-peer), a request riding a bi-stream of the connection itself, as the server side's does, since the bridge's port carried any program on the computer to a member's door as this computer's key. The accept loop forked, and what it copies into replaced: hyper serves the node's router on each accepted stream, the key iroh verified reaching handlers as a typed request extension, so peer routes have no loopback port and no caller is served as local for lacking a header (`ring_routes.rs:65-72`). Upstream names this its upgrade path (`commonwealth/crates/commonwealth-transport/src/iroh.rs:23-24`) |
| `mesh-reach`'s port: `PeerTransport`, `PeerContact`, `PeerEndpoint`, `TrafficClass` | one seam over IP and iroh; a port per class of traffic | replaced: one transport, so a concrete type (RUST_ARCH: a trait only at a real boundary); a contact is a member's key; one ALPN constant, and an enum when a second protocol comes |
| `commonwealth-transport::iroh_identity_forward`, with `kernel-types::member`'s header scheme | stamps the dialer's verified key on every request head it parses and strips any the client typed | not forked: on the stream a handler reads the key from the connection, so no header can claim one, and only hyper parses a request. Upstream's rewrite passes the rest of a connection through untouched after a chunked body (`iroh_identity_forward.rs:419`, `iroh_routed_forward.rs:159`), so a member could name another member's key on the next request. U3's door sets the verified headers on the request it proxies, from the extension |
| `commonwealth-media::origins` (`OriginRegistry`, `admit_spliced_origin`) and `commonwealth-transport::iroh_routed_forward` | a table of many programs' origins by ALPN and path prefix | replaced: U1's node serves one router, its sync route, to members; U3's lent services are routes on it |
| `host-kit::shell::RouteBundle` | named route bundles a shell mounts | replaced by an `axum::Router` |
| `rail.rs`: `err` | the refusal body | forked with the route |
| `rail.rs`: `append`, `RailQuery`, `LiveBuffer`; the live route (`ring_routes.rs:163`) | the record API an app writes through; the nudge that wakes the round after a write; the live lane | U3, as the loopback record API: append, with the nudge, landed in u3-2 (`crates/svrngs-node/src/rail.rs`, `ring_sync::append`); the live lane in u3-3 (`LiveBuffer` and its doors in `rail.rs`, the live route in `ring_routes.rs`, the fan-out in `live.rs`) |
| `rail.rs`: the sync doors (`digest` to `compact`) and `attest` | the inference daemon reading this process's journals over loopback | left behind: one process per computer |
| `ring_routes.rs`: the checkpoint route (`:208`) | exports one ring as a document | waits (§2). It checks no ring's roster, so any member the acceptor admits can export any ring the node holds |
| `RailsDaemon` and `RailsNode` (`lib.rs:232-664`): `RingSyncHost`'s implementor (`ring_sync.rs:700-733`) and the wiring in `start` and `run` | the journals, endpoint, membership, nudge, loopback listener, acceptor and round, in one process | replaced by the node: a `svrngs-node` type holding the rail, endpoint, transport, loopback listener, acceptor and round. Gossip, join, the mesh store, the work plane, presence, the endpoint's self-heal and `rails.toml` stay behind |
| `commonwealth-transport::identity` | the node key on disk | U2, with the `svrngs` command, which owns a device's keys; the node takes its key as an argument |

Four facts follow. *One key signs and dials:* upstream signs a ring's acts
with the key its endpoint proves (`lib.rs:400-402`), so a member's actor is
its iroh id and the register names every node the round may dial; since
DESIGN §6's key per device per group, bound to the person (bash-devices), the
register carries each device's key with its person's first key beside it
(`Member::person_key`). *A bare key is dialled:* upstream dials the relay and
addresses the mesh gossiped, and refuses a bare key
(`commonwealth/crates/commonwealth-transport/src/iroh.rs:182-237`); with no
gossip, iroh's address lookup, upstream's default
(`mesh-reach/src/guest.rs:79-88`), resolves a member's key, so the fork accepts
a bare key when lookup is on, and tests seed addresses as upstream's do. *No
presence:* the round dials every standing member's computer, all at once
(asleep-1), and one that is offline costs its own dial's timeout each
round, the others' exchanges not waiting on it; a browser's key is never
dialled (u4-1, DESIGN §3 call 2). *The door asks the register too:*
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
| `mesh-join-vocab/src/deep_link.rs`: `build_https_guest_link` and `parse_https_guest_link` (`:244-300`), with `percent_encode` and `parse_query_params`; `build_join_link` (`:421-436`) | the-link's https form: every secret in the fragment, a parser that never reads left of the `#`, the dial as `iroh=`; and the client's form, `sovereign://join/<key>` | forked, then made short and opaque (u2-8; operator, 2026-10-04, §3): one base64url token carrying only what the joiner cannot ask the issuer for — a format byte (`Format`, a closed set), the issuer's key, which it dials, and 16 secret bytes the link's signing key is derived from (BLAKE3 `derive_key`, a named context), and the issuer's relay only when it runs without address lookup (DESIGN §11, risk 5). A client's link with no relay is 80 characters, where the seven-field https link was 412, with the issuer's network addresses in it. The issuer answers the group's seed and its `Invite`'s id to the link's key (DESIGN §6). Two wrappers carry the one token: the client's link, `svrngs://join/<token>`, upstream's `sovereign://` shape, which `new` and `link` always print; and `<origin>#<token>`, printed beside it only for a group that keeps a page origin, so nothing depends on a hosted page unless a group names one. `svrngs join` takes either, or the bare token. The origin is the group's setting on each computer, not a flag each call reads: `new --origin` keeps one, `join` keeps the one its https link was printed on and none from a client's link, and `link --origin` changes it. A computer never fetches the origin; it dials the issuer by key over iroh. A phone (U4) loads the page there, which dials over iroh in turn, so the origin is trusted with that page's code and never sees the fragment, which is why it can be anyone's. The https form with the key in its path (`:396-413`) stays behind: a host reads a path. A token whose random part spells a word DESIGN §1 keeps from people is drawn again |
| `commonwealth/crates/commonwealth-rails/src/found.rs:46-114`, `join.rs:116-229`, `internal.rs:222-295`; `commonwealth-discovery/src/membership.rs:12-75` | the mesh invite: a 48-bit key, reusable until a 24-hour lapse (`membership.rs:53`), only its hash kept, in `mesh.json` outside any record. The joiner dials the founder by key and posts `/internal/join` with a proof that signs neither the key nor the mesh (`commonwealth-transport/src/identity.rs:130-141`); the founder checks and persists before answering | the handshake's shape forked: the joiner dials the issuer by key and proves it holds both the link's key and its own, over the group and the request; the issuer persists before answering. What it writes is an `Admit`, and every use of a link is in the record: the `Invite`, its lapse, its closing. The lapse stays upstream's day. The mesh snapshot, `MemberRecord` and `node_id` stay behind. Landed: `crates/svrngs-node/src/redeem.rs` |
| the acceptor U1 built (`crates/svrngs-node/src/node.rs`, `admit_dialer`) | closes every dialer standing in no group the node holds | kept for members. Redemption is served on a second ALPN, the one a key in no register may speak, with one route; `alpn.rs` becomes the enum it promised (landed: `Alpn`, and `Admit::Serve` names the router each protocol reaches) |
| `sovereign-grants`' guest grant (`guest_grant.rs`, `guest_session.rs`) | one QR code serves a room; its state is a map in memory | not forked: its state is lost on a restart. Its shape, one link for a room, is what U2 keeps, in the record |
| `commonwealth-transport/src/identity.rs:26-126` | the node key: a 32-byte seed at `<data_dir>/node_key`, written through a temporary file and a rename, mode 0600; an I/O error falls back to a key nobody keeps | forked with edits as the command's device key. An unreadable key refuses: a computer never silently becomes someone new (ARCH 6). The mesh proof and dial-info signing (`:128-184`) stay behind |
| `commonwealth-rails/src/cli.rs` and `main.rs` | four verbs, parsed by hand, no framework; exit codes 0, 1 and 3; a tokio runtime built by hand | the shape of `crates/svrngs`: `new <name>`, `join <link>`, `link <group>`, and the bare command, which runs the node for every group the computer holds. No parser dependency |
| `commonwealth/crates/commonwealth-rail/examples/stage_member_act.rs:94-157` | an `Admit` signed by a member, timed strictly after everything held | its signing steps are the redemption's write; its timing rule moves into the door (DESIGN §6) |
| `sovereign/crates/sovereign-cli-mesh/src/ring_cmd/mod.rs:356-470`, `ring roster add` | joining a ring: two people copy a hex key by hand | replaced by the link; nothing forked |
| svrngs's own `valid_namespace` (`crates/svrngs-node/src/lib.rs:43-50`) | a group's id is a name the caller types, so two groups called House on one node collide | a group's id is the hash of its seed — the founder's key and the name they go by, the group's name and random bytes drawn at `new` — so a seed the issuer answers for a link proves the id. The name is what a person reads (landed: `crates/svrngs/src/group.rs`, in the node's `group.rs` since start-group; the founder's name added, since the register binds every seed key to a person) |

Four facts follow. *Only the issuer redeems:* a link works while its
issuer's computer is reachable; Sam's always-on computer is, in both demos,
and any member redeeming waits in §2. *A new link closes the old one:* an
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
lines, though DESIGN §5 sets tests apart; U2 moves them beside the files they test
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
| `sovereign/crates/sovereign-cli-mesh/src/publish_cmd.rs` (`svrn publish <name> <port>`) and `mesh_media/offer.rs` (`probe_origin`, Jellyfin's well-known ports) | writes a lend into `config.toml`; probes a port for a known service | the verb's shape and the probe, with edits: `svrngs share <port>` writes a lend declaration into the data dir, and works whether or not svrngs is already running (a running node reads the declarations; one per data dir, `crates/svrngs/src/cli.rs` `claim`). Landed (u3-5): `crates/svrngs/src/share.rs`, its probe, table and decision in the node since v1ui-lend (a) (`crates/svrngs-node/src/share.rs`); a port alone, on 127.0.0.1; the kind the given one, else the port's from a table kept as data (`well_known.txt`, 8096 — the door speaks plain HTTP, so upstream's TLS port 8920 is not shared by number — and since bash-person 4533 and 8083), else refused; the name the given one, else the kind; the limit the given one, else none, which lets in everyone in its groups at once, printed (bash-limit: the 3 that stood here was a number nobody chose, and a declaration naming none, once refused, now holds none); the secret read from a prompt, unechoed on a terminal, sent as `authorization`; `share` runs the node unless svrngs already runs, and `svrngs unshare <name>` withdraws a lend |
| `commonwealth/crates/commonwealth-media/src/declared.rs:51-77, 169-229` | the lender's secret: a header value in a 0600 file under a 0700 dir, named by its header, never logged, never in anything shared | forked with edits: the lend's secret stays in the data dir and never enters the record. A lend's public part (name, kind, limit) is answered by its lender's node, never stored: the list of what members lend is computed by asking each standing member, as upstream's offers were (`commonwealth-media/src/reach.rs:131-168`), so an offline computer's lends are absent rather than stale. Landed (u3-4): `crates/svrngs-node/src/declared.rs`, the store under `<data dir>/secrets/lends/<lend>/`, a failed hardening failing the write; the declaration `<data dir>/lends/<lend>.json` (`lend.rs`), read per request. Landed (u3-5): the declaration holds the port, the kind, the limit and the refused paths, written and withdrawn only by `lend::declare` and `lend::withdraw`, and its limit changed only by `lend::limit` (bash-limit, the home page's `/lent`); `GET /lends` on the member ALPN answers a dialer standing in a group held what this computer lends now and how many people use each (the door's count of the people with a request in flight, `door::Using`; since u3-6, the people its limit counts); `Node::lends` asks every standing key at once and answers each lend with whose it is and an address on this computer's loopback, the dialler's bridge then `/lend/<name>/` (since u5-2 its origin, `origin.rs`; since arch-peer a request on the stream, no bridge), naming a computer that did not answer |
| `commonwealth-media/src/identity.rs:118-165` (`admit_spliced_origin`) and `commonwealth-transport/src/iroh_identity_forward.rs:297-328` (`rewrite_head`), with its tests at `:620-702` | refuse a non-member and a member outside the allow list; strip every client identity header and every header about to be declared, compared case-, padding- and control-byte-blind; then set identity, then the declared secret | the rules forked into the door's handler, the code not (§4, U1's inventory): the person and key come from the `Dialer` extension and the register, and one prefix of svrngs's own is stripped whole. The tests come over as tests. Landed (u3-4): `crates/svrngs-node/src/door.rs`, `/lend/<lend>/<path>` on the member ALPN; `x-svrngs-person` and `x-svrngs-key` set, every client header under `x-svrngs-` stripped; a client's padded or control-byte header name never reaches the door, since hyper refuses the request (400) |
| `sovereign/crates/sovereign-daemon/src/serve_client.rs:616-655` (`forward_stream`), the strip lists of `sovereign-pods/src/worker_inference_proxy.rs`, and `commonwealth-transport/src/iroh_routed_forward.rs:318` (`route_by_prefix`, its traversal guard) | a GET proxy that streams a response and passes `Range`; hop-by-hop strips; refusing `..`, `%2e`, `%2f` | with edits: every method, a streamed request body, `If-Range`, and the lend's refused paths matched as `route_by_prefix` matches a prefix. Upstream streams no request body anywhere. Landed (u3-4): one hyper connection per request, both bodies streamed; a refused path compared percent-decoded, case-blind and with repeated slashes as one; a response header holding the secret dropped |
| `commonwealth-transport/examples/media_bridge_bench.rs` (its Range-serving origin) and `commonwealth-rails/tests/two_daemons.rs:208-346` (`picky_origin`, 401 then 200) | prove Range sha-exact through the splice; prove a declared credential arrives | forked as test fixtures. Landed (u3-4): `crates/svrngs-node/tests/door/origins.rs` |
| `commonwealth/crates/commonwealth-rails/src/rail.rs`: `append_act` (`:244`), `log_answer`, `LiveBuffer` (`:781`), `err`; `api.rs:167` (`check_loopback`) | the record over loopback: append, log, and a live buffer drained by polling; loopback is the auth | forked with edits as the loopback record API: append, log and live only; the attestation branch goes with attest; the roster becomes the register. An app's act is signed by its member's node, with `on_behalf_of` naming the person the door verified when the app serves someone else. Landed (u3-2): append, log and the people with their keys (`GET /v1/rail/people`, from the register; since v1ui-door (a) each key with what holds it, `device`: `computer` or `browser`), on 127.0.0.1, each caller refused unless on loopback (`crates/svrngs-node/src/api.rs`); live in u3-3: `POST /v1/rail/live` offers a payload of at most 4096 bytes to the group's other standing keys, `GET` drains this computer's buffer for one group, each payload with its sender's key and person key since onair-live (`rail::Live`) |
| `sovereign/crates/sovereign-daemon/src/routes_rail_live.rs:76-196` (`push_ephemeral`) and `ring_routes.rs:163` (the live route) | fans a live payload out to each online member | with edits: dialled over `IrohTransport` to the register's standing keys; `LiveEnvelope` (`crates/svrngs-node/src/peer_wire.rs:85`) gains its user (ralph/REVIEW_FINDINGS.md #11). Landed (u3-3): `crates/svrngs-node/src/live.rs`; the live route admits a dialer standing in the group its envelope names and refuses the rest, as the sync route does |
| `serving-policy-core/src/fair_sched.rs:163` (`fair_share_cap`) and `try_grant` | divide a budget evenly among the active; granted, queued or shed | the shape of the share and the limit, counted in people, not slots. Landed (u3-6): `crates/svrngs-node/src/door/limit.rs`, pure, the time an argument; a person counted while a request is in flight or within `WINDOW`, 30 s, of one; nothing queues: a newcomer past the limit is refused 503 with the reason and how many are using it; the share, the declared upload over the people counted, in `x-svrngs-declared-share-bps` on the request and its answer. The upload is the lend's `upload_bps`, from `svrngs share --upload <Mbit/s>`; a lend declaring none is told no share |
| `commonwealth-transport/src/iroh_path.rs` (`PeerPath`) and iroh's `Connection::paths()`, `Path::is_relay`, `PathStats` | whether a peer's path is direct or relayed; per-path bytes, RTT and congestion window | read for the trace a demo watches; never mistaken for capacity. Landed (u3-6): `iroh::Via`, the connection beside the `Dialer`, whose selected path's address, relay, RTT and bytes the door traces at info with each share |

Four facts follow. *The upload is declared:* the lender states it, measured by
hand as Demo A's run-of-show already does, and the share the door tells each
request says it is a declared figure (ARCH 6); measuring it waits in §2.
*Lends are asked, not stored.* *The local network is guessed:* iroh says
nothing about it, so a viewer whose path is a private, link-local or
unique-local address is taken to be in the lender's home and not counted,
and the trace says it guessed. *Localhost is the trust boundary:* any process
on the lender's computer can reach a lent port and type the identity header,
as it can reach any localhost service; upstream's per-process tie waits in
§2. Jellyfin is GPL-2.0-only, so svrngs talks to it at arm's length, over its
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
| `ring-runtime/src/lib.rs:1-203` (153) | one export, `dial_guest(dial, token, path)` (`:62-63`): parses `<hex>@<targets>` and refuses one with no http(s) relay URL (`:91-99`); binds an endpoint, `presets::Minimal`, a fresh `SecretKey::generate()` on every dial, `RelayMode::Default`, `GUEST_ALPN` (`:119-123`); waits for `online()` (`:139`), connects to the relay address (`:142-145`) and opens one bi-stream (`:159`); writes a hand-built `GET … Connection: close` (`:166-168`), reads `read_to_end(512 * 1024)` (`:178`) and finishes the send half only after it (`:185-187`); every layer pushed to `window.__trace` as it happens (`:27-35`) | forked into `web/` as svrngs-web (u4-3), upstream's file first with only `kernel-types` replaced by svrngs-core; then the person's key from the page's sealed store in place of `generate()` (call 1), the member ALPN, and a dial by key: the token's relay, else iroh's address lookup, which the N0 preset does in a browser over HTTPS (`PkarrResolver::n0_dns()`, `iroh-1.0.2/src/endpoint/presets.rs:122-128`), never refusing a bare key — until relay-default, since which the token's relay alone, a bare key refused as upstream refused one, n0 asked for nothing. The trace stays. The hand-built request and the buffered read go in u4-4: hyper's HTTP/1 client over each stream, any method, a streamed request body, the response body a `ReadableStream`, the send half open until the response ends, no cap. Landed (u4-4): `web/src/http.rs`; `connect(key, token)` answers the dialled computer, whose `fetch(method, path, headers, body)` answers a page's `Response`, its body handed over as each chunk arrives off the stream |
| `ring-runtime/Cargo.toml:1-34` (14) | a cdylib in its own `[workspace]` (`:32-34`), so the host workspace never builds it; "The LOCKED pin … measured WORKED at exactly these versions" (`:21-26`): iroh =1.0.2, wasm-bindgen =0.2.128, wasm-bindgen-futures =0.4.78, js-sys =0.3.105; `kernel-types` by path (`:30`) | forked with its pins and its own workspace (u4-3), svrngs-core in place of `kernel-types`. iroh 1.0.2 has no wasm feature: its build script turns the browser on by target (`iroh-1.0.2/build.rs:7`) |
| `ring-runtime/web/index.html:1-45` (27) | the static page: a CSP of `default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self' wss: https:; img-src 'none'; …; base-uri 'none'` (`:25`); `referrer` `no-referrer` (`:24`), so the fragment never leaves in a Referer; the script by absolute path, `/ring/app.js?v=__VERSION__` (`:43`) | forked (u4-3); its words become DESIGN §1's; `no-referrer` and the strict CSP kept; the shell's `/ring/` paths follow wherever a group's origin serves the page |
| `ring-runtime/web/app.js:1-68` (47) | reads the fragment's `token`, `iroh`, `path` and `s` (`:25-29`); `init()`, then races `dial_guest` against 120 s (`:41-45`); pulls status and body out of the trace text (`:49-53`); registers the service worker after load, best effort (`:64-68`) | forked (u4-3), then the joining page (u4-5): the fragment is svrngs's token, parsed by svrngs-core (call 5); it asks the person's name once, makes and seals the key (call 1), asks and redeems at the issuer naming itself a browser (call 2), keeps the key and the group's name, and lists what the group's people lend. Landed (u4-5): `web/app.js` and `web/store.js`, `web/src/join.rs`, and the node's `GET /people/lends` |
| `ring-runtime/web/sw.js:1-34` (25) | a cache-first shell cache named by content hash (`:5-11`); same-origin GETs only (`:28-34`): "The dial itself is not HTTP to this origin — it is iroh — so requests to it never reach here" (`:26-27`) | forked (u4-3), the shell cache kept; then the runtime (u4-6, call 3): it opens the endpoint from the sealed key when it starts and again after the browser stops it, and serves every request under its scope through u4-4's client to the lender's door, `/lend/<name>/<path>`. Landed (u4-6): `web/sw.js`, a module, with `endpoint(key, token)` and its `dial(lender)` in `web/src/lib.rs` — one endpoint per key, since a relay sets an older endpoint with the same key aside when another binds (`iroh-relay-1.0.2/src/server/clients.rs:80-92`) |
| `scripts/build-ring-runtime.sh:1-119` (75) | checks for the wasm target (`:31-35`) and a `wasm-bindgen` CLI of exactly 0.2.128 (`:36-49`); on macOS uses Homebrew LLVM's clang and llvm-ar, since Apple's clang has no wasm32 backend (`:52-76`); `RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --release --target wasm32-unknown-unknown` (`:81`), the cfg for `kernel-types`' getrandom 0.3 (`:77-80`); `wasm-bindgen --target web` (`:85-86`); `wasm-opt -Oz` when installed (`:91-97`); a 12-hex content hash stamped into the three shell files (`:103-107`) | forked as `scripts/build-web.sh` (u4-3). Whether the getrandom cfg stays once `kernel-types` is gone is a change of meaning, its own commit |
| `landing/scripts/build-ring-runtime.sh:1-24`, `landing/scripts/verify-ring-deployed.sh:1-92`, `landing/vercel.json:1-73`, `landing/package.json:13` | build into `landing/ring/` and deploy one origin: "The trailing slash is LOAD-BEARING" (`build-ring-runtime.sh:7`), after "a phone hung on the wall link" (`:11-12`); a redirect `/ring` → `/ring/` (`vercel.json:4-10`); `sw.js` uncached and the wasm immutable (`:64-71`); a deploy checked by fetching the page's script as a browser resolves it | not forked: which origin serves the page, and how, is a group's. Their facts go into u4-8's package for the operator: a page served at a path needs its trailing slash, `sw.js` uncached, the wasm immutable |
| `landing/ring/` (generated, committed) | the built site; `wasm/ring_runtime_bg.wasm` is 4,295,444 bytes (`git ls-tree -l`); its glue reaches the relay with `new WebSocket` | not forked. Upstream's one figure for a page's weight. Ours, measured (u4-8, `demo/runs/u4/2026-10-05/`): the page fetches 5,103,571 bytes, 1,209,407 brotli, its wasm 5,027,239 without `wasm-opt`; a cold join took 15.35 MB from an origin sending no cache headers, the wasm fetched by the page, the worker's install and the worker's start (`ralph/REVIEW_FINDINGS.md` #51); since u4-9 the wasm is fetched once, by one URL the page and the worker share, and a cold join took 5.34 MB (`demo/runs/u4/2026-10-05-reach/`) |
| `kernel-types/src/alpn.rs:26` | `GUEST_ALPN = b"cwth/guest/0"`, the one spelling both ends import | not forked: svrngs's protocols are its `Alpn` (`crates/svrngs-node/src/alpn.rs:29-30`), spelled in svrngs-core from u4-2, so the page and the node spell each name once |
| `mesh-join-vocab/src/deep_link.rs:228-300` | the https link's build and parse, never reading left of the `#` | forked already (U2): `crates/svrngs/src/link.rs`; u4-2 moved the token's parse, now `crates/svrngs-core/src/link.rs`, and the signing of the ask and the redemption into svrngs-core (call 5), leaving the command `issuer.rs`: the issuer as iroh dials it; a fresh token's secret, drawn there until start-home moved it beside the link's `Invite` into the node (`crates/svrngs-node/src/invite.rs`), where `svrngs link` and the home page both make a link |
| `sovereign/crates/sovereign-contracts/src/guest_pages/ring_shim.js:1-171` (105), with `shim.rs` and `guest_pages.rs` | `window.ring` over plain `fetch` to a LAN door: the bearer from `location.hash` (`:5`); a name claimed by `window.prompt` and `POST /v1/guest/session`, its handle kept in `localStorage` (`:30-61`) | not forked: a guest on a LAN door, which never dials iroh and for whom the host signs. svrngs's phone is a person with a key in the register. Both retire on svrngs's bar (`docs/RING_SPEC.md:26`) |
| `sovereign/crates/sovereign-daemon/src/guest_door.rs:617-729` (104) | the LAN door's proxy: the request body buffered to 16 MiB (`:619`, `:646`); headers forwarded but `host`, `connection`, `content-length` and `accept-encoding` (`:659-667`); the response buffered whole (`:689`) and only its status, `content-type` and body returned (`:681-696`) | not forked: it drops `Content-Range` and streams nothing. The other end of a phone's streams is U3's door (`crates/svrngs-node/src/door.rs`) |
| `sovereign-mobile/src-tauri/src/iroh_bridge.rs:14-17`; `scripts/ring-room-phone/phone.mjs` | a native Tauri app's iroh, its key "ephemeral (fresh key per app run)"; a Node stand-in for a phone browser, with a fake `location`, `localStorage` and `fetch` | not forked: a native phone app waits (§2). The browser tests drive real engines with Playwright, Chromium and WebKit with iPhone emulation (u4-7) |
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
group trusts its origin with that code (DESIGN §11, risk 5). This is read from
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
`docs/RING_APP_LIBRARY.md:315`). §3's browser door, a group opening its own,
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
DESIGN §3's list. Paths under `mesh_media/` are
`sovereign/crates/sovereign-cli-mesh/src/mesh_media/`.

| Upstream | What it does there | Becomes |
|---|---|---|
| `mesh_media/viewer.rs:304-463` (`provision`, 323 lines without its tests), with `:143-196` and `:216-296` | finds or makes one Jellyfin user, sets its password, writes its policy over the origin's own — `POST /Users/{id}/Policy` replaces rather than patches, and needs `AuthenticationProviderId` and `PasswordResetProviderId` (`:21-27`, `:143-175`) — reads the policy back (`:177-196`), signs in by `AuthenticateByName` (`:443`) and keeps `MediaBrowser Token="…"` (`:461`); a token already held is proved by one `GET /Users/Me` (`:216-296`). An `/Auth/Keys` key is admin-equivalent (`:6-12`); a non-admin sets no password (403, `:205-208`) | zero sign-in's shape (DESIGN §3, call 4): a Jellyfin user per person, made by the lender's computer with a key the lender gives once. Forked into `apps/jellyfin/mw/src/viewer.rs`, behaviour kept first, then a user per person in place of one viewer, as the middleware's first module (mw-1, `62560f0`, `717f433`); never into the infra (call 3) |
| `docs/RING_APP_LIBRARY.md:802-816` | three ways a lent service keeps its users, cheapest first: the app reads the member header; the lender's credential injected, one shared viewer; an account per member, made on `Admit` and disabled on `Remove`. Mapping the member header onto an app's trusted proxy header is "From memory — verify per app" (`:810-811`) | the first is DESIGN §1's "an app that takes the signed-in person from a request header", which u5-catalogue-dry's census checks per app against each app's own docs; the second is what U3's dry run broke on; the third is zero sign-in |
| tag `e0-bot`, `docs/DESIGN.md:580-587` and `:878` | the door masked the secret in response bodies because "Jellyfin writes its token into playback info and into the HLS playlists' subtitle, trickplay and variant URLs"; a lend refused `/Sessions/Logout`, "which would sign the whole group out"; `Remote-User` and per-person accounts waited for "a named service that needs them" | not forked. The body claim was never measured: u5-3 (c) asks whether any byte a person receives carries a secret the lend holds, the transcode included. Jellyfin is now the named service |
| `.canon/sources/notes/invariant/35104a7e.md:3-9`; `scripts/cw-media-demo.sh:31-40`, `:54-56` | Jellyfin 12.0.0 takes its credential only from `Authorization: MediaBrowser … Token="…"`: `X-Emby-Token`, `X-MediaBrowser-Token` and `?api_key=` each answer 401, and the inner space and quotes are load-bearing | a fact `apps/jellyfin/` and u5-3 build on: a lend's secret displaces the client's whole `authorization` (`crates/svrngs-node/src/door.rs:113-119`, `:386-412`), the client's device with it. Nothing forked |
| `mesh_media/offer.rs:94-97` (`WELL_KNOWN_ORIGINS`) | 8096, then 8920 | forked already (U3): `crates/svrngs-node/src/well_known.txt`, 8096 alone, so `svrngs share 8096` names Jellyfin |
| `commonwealth/crates/commonwealth-media/src/reach.rs:94-95` | refuses a member's own node: "its {} is already local" | left behind: u5-1 lists this computer's own lends, where `Node::lends` skips them (`crates/svrngs-node/src/node.rs:475-477`), and u5-2 serves them through this computer's door's rules |
| `quality/campaigns/ring-apps-shelf.md:187` | Jellyfin is GPL-2.0-only: "API interop at arm's length only" | held: svrngs talks to Jellyfin over HTTP and its API and copies none of its code |
| svrngs `demo/runs/a/dry-2026-10-04/` (`dry.sh`, `drive.mjs`, `README.md`) | Jellyfin 12.0.0 through the door on one computer, the macOS app's server with data dirs of its own: the wizard over the API (`dry.sh:60-76`) and the lend's secret minted (`:88-95`); opened with nothing typed, a sign-in (`README.md:35-37`); a member's sign-in logs the lend's token out for everyone, the door's header carrying the lend's one `DeviceId` (`:39-50`); with a placeholder sign-in written into the page's local storage (`drive.mjs:41-47`), it plays in 1,137 ms and each seek plays again within 123 ms (`README.md:52-61`); an API key ruled out (`:63-67`); `/socket` 404 through the door, 101 direct (`:69-73`); no secret in a received byte but the token Jellyfin handed the browser signing in (`:81-88`); three at once and a transcode not measured (`:90-93`) | u5-3's base: its scratch Jellyfin is `dry.sh`'s, and what the run found is what u5-3 measures and u5-4 solves (call 2). The placeholder sign-in stays a driver's trick: no person can write it |
| svrn-docs `rings/runs/strm-2026-10-05/` (its commit `457f120`) | the group-folder experiment's first run: Jellyfin 12.1.0 over two `.strm` stubs; every browser played the holder's MP4 from the stub's URL; WebKit, alone and as an iPhone, remuxed the MKV through Jellyfin's own HLS, reading the whole file unpaced; FAIL on its pre-registered byte-share bar (93.0% and 85.8%) | §2's group folders, after U5; no U5 row builds on it |

Four facts follow. *Lends shared the key's origin until u5-2:* on a phone
every lend was under the page's scope, where u4-6's test had a lent page's
script unseal the person's key (DESIGN §11, risk 5), and on a computer a lender's
lends shared the bridge to that lender, its loopback port beside the record
API's; since u5-2 each opens at an origin of its own (DESIGN §7). *Two nodes on one host dial on loopback only:* iroh moves a
connection dialled on loopback onto this computer's private address, which
the door guesses home and tells no share (`ralph/REVIEW_FINDINGS.md` #33),
so U5's two-node tests build every node with `RelayConfig::loopback_only`,
as the door's do (`crates/svrngs-node/tests/door/main.rs:56`). *A sign-in is
the app's:* the door sets the person on every request, and an app that
reads no such header asks its own sign-in; Jellyfin is one (U3's dry run),
and which of DESIGN §1's apps read one is the census's to find. *Jellyfin's own
cross-origin rules were never measured* in either repository; with each lend
at an origin of its own (call 1), stock jellyfin-web asks none.

**U6's inventory.** What upstream at `e1a43bcd2`, and svrngs at `d95039c`,
have for the room door, and what each becomes; written from the seat's
survey (`ralph/next/u6/UPSTREAM.md`), made for a handoff to a TV that the
operator has since taken out of scope (DESIGN §3, "The room"), and kept here only
where the room door uses it, each path:line read again at u6-0. Upstream
has a LAN door that lives only while it is wanted and an address picker,
and nothing measured on a TV; its door's proxy buffers, so U4's inventory
left it. svrngs serves a lend only on an iroh stream or on loopback.

| Upstream or svrngs | What it does there | Becomes |
|---|---|---|
| `sovereign/crates/sovereign-daemon/src/guest_door.rs:412-499` (`serve`, `wait_for`) | the LAN guest door's listener: `0.0.0.0:9744` when no bind is named (`:434-443`), bound only while a grant is live, the grant store polled (`:455-456`, `:491-499`), shut down gracefully at the last expiry (`:477-486`) — "safe by LIFETIME, not by address" (`:429-431`); `ConnectInfo` on its service (`:472-476`) | its shape, not its code: the room door's listeners live while the command runs, bound on the private address alone, so safe by both. Not forked: it waits on a grant store svrngs does not have (`:454`) and serves the daemon's router and auth (`:475`) |
| `sovereign/crates/sovereign-cli-mesh/src/mesh_guest_link.rs:37-55` (`advertised_base`), `:63-96` (`guest_bind_url`) | "the door names the PORT, this machine names the ADDRESS" (`:10-12`): a wildcard or loopback bind is never advertised (`:45-53`), no candidate is absence (`:34-36`), and the LAN before tailscale, measured: "the default picked 100.104.36.28 while the room's phones needed 192.168.1.7" (`:67-73`) | its rules, written smaller (the next two rows). A device on the room's WiFi cannot reach tailscale's range, so here "before" is "instead of": the private LAN address or absence, never tailscale's. Not forked: it reads a config's bind and `relay_candidates` |
| `commonwealth/crates/commonwealth-discovery/src/mesh_discovery.rs:56-77` (`classify_ip`), `:98-124` (`relay_candidates`), `:237-313` (`local_ip_candidates`) | every IPv4 address outside 100.64/10 is "lan", a public one included (`:58-66`); the interfaces from the `if-addrs` crate, which svrngs does not carry, minus loopback and link-local (`:259-283`); a container bridge listed before tailscale (`:21-32`); the outward route's address by a UDP connect that sends nothing, its fallback (`:290-298`) | not forked: its "lan" would pick a public address, which call 3 refuses, and iroh lists the interfaces already (next row). The classifier is std's `Ipv4Addr::is_private`, which `guess` uses already (`crates/svrngs-node/src/door/limit.rs:54-65`). The UDP connect is kept, as the tie-break: on this host it answers `192.168.1.4`, the address the TV reached in the reads |
| iroh 1.0.2 `Endpoint::addr()` (`src/endpoint.rs:1196`, its addresses gathered at `src/socket.rs:1814-1955`), netwatch 0.19.1 `LocalAddresses::new` (`src/ip.rs:34-93`) | with the wildcard bound, every interface that is up, paired with the bound port (`socket.rs:1926-1953`); loopback only when there is nothing else (`:1937-1940`), link-local IPv4 likewise (`ip.rs:63-66`, `:86-93`); the port-mapped and reflexive addresses its reports find (`socket.rs:1814-1875`); no interface's name. Measured: a node here logged `dial=Some("…@192.168.1.4:49713")` (`demo/runs/a/2026-10-05/data/alex-run.log:3`) | **the picker's source, written smaller** (u6-1's row leaves the choice here), with no new crate: a pure function over the addresses `ip_addrs()` lists and the outward route's — the private IPv4 address the route leaves from, when it is among them; else the one private IPv4 address listed, when there is one; else absence, said so. With no name kept, a bridge in a private range (a container's, `172.17.0.0/16`) is told from the LAN only by the route, which does not leave through it. A node built `loopback_only` lists loopback alone (`crates/svrngs-node/src/guest.rs:215-219`), so the picker's tests plant the lists |
| svrngs `crates/svrngs-node/src/origin.rs:59-107` (`lent`), `:119-158` (`carried`) | each lend's origin on loopback: the lend its `Host` names, this computer's own through the door's one body as `Who::Own` (`:68-79`), another's dialled as this computer's key, a request on the stream to its lender's door (`:90-106`; the bridge until arch-peer), a WebSocket spliced once both sides switch (`:145-153`) | **the room door's body**, one with it (call 3): the lend named by the listener's port in place of the `Host`, which on a room's device is the private address |
| svrngs `crates/svrngs-node/src/api.rs:64-85` (`router`), `:132-160` (`from_this_computer`), `:165` (`check_loopback`) | the loopback API's listener, where each lend's origin is served beside the record's routes behind one check that the caller is on this computer | not reused: the room door is a listener of its own that answers no route of the record's. Its caller check has `from_this_computer`'s shape and `guess`'s home as its predicate — private, link-local or unique-local — so "this network" has one decider |
| svrngs `crates/svrngs-node/src/door.rs:736-805` (`lend_for`), with `guess` | the lender's door: the person from the dialer's key and the register (`:736-765`); the path's address guessed home or away (`:805`) | unchanged (call 3): the lender's door sees the room door's computer as its person, counted once however many devices in the room ask, its path guessed as now — from across town, away and counted |
| svrngs `crates/svrngs-node/src/node.rs:300` (`Node::lends`), `:367-370` (`origin`) | `/v1/lends` answers where this computer's browser opens each lend | gains a served lend's room address (u6-1, DESIGN §7) |
| svrngs `apps/web-conformance`, `crates/svrngs-node/tests/conformance.rs` | the conformance canary (§4) | the room door's check: its routes at the private address, from a client not on loopback (u6-1) |

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
| U2 join by link | `svrngs new <name>`, `svrngs join <link>` and `svrngs link <group>` on a computer, and `svrngs` alone running the node; the link is one short token — the issuer's key and the link's secret, and the issuer's relay, with lookup or without it since relay-park (c) — and the issuer answers the group's seed and its `Invite`'s id to the link's key; redeeming at the issuer's node writes one `Admit`, co-signed by the new key; ids over everything signed, the previous act among it; the door signing after everything it holds (DESIGN §6) | three people open one link and each is admitted once; opening it again admits nobody twice; a lapsed or closed link admits no one; a dialer without the link's key is closed with no answer; nothing the command prints uses a word DESIGN §1 keeps from people |
| U3 lend a service | the member door: a route that proxies to a lent service, the verified person set from the connection; `svrngs share <port>` with its lend declaration; streaming and byte ranges through the door; each request's share of the lender's upload; the people-at-once limit; the loopback record API (the rail routes and the live lane, forked); the list of what members lend | the app sees the verified person; a header the client typed is stripped; the lender's secret appears in no byte a member receives; with three streams, each is told a third of the declared upload, and a fourth person is refused with a reason |

**Demo A — Sam's films, three homes at once** (after U5; operator,
2026-10-05, in Jellyfin as it comes, where on 2026-10-04 it was put in an
app of our own once U3's dry run found no way into Jellyfin's own player
that kept the bars — DESIGN §11, risk 7). Sam lends Jellyfin with one command and
sends a link. Alex, Bo and Cy join from three homes and each plays a
different film from Sam's library in Jellyfin's own player through the
lend, in their own browser, at the same time, seeking freely; none stalls
another, and the door tells each its share of Sam's upload, in Sam's log.
Sam's password never left Sam's computer; nobody made an account (DESIGN §3, "Lent
apps", call 4, on how that clause meets a lender's middleware).

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U4 the browser | a phone opens a group's https link, on the page origin the group named, and is in: its own key, a dial by key to the issuer to join and to each lender for what they lend (DESIGN §3, "Joining"), and the runtime that renders a lent app through it — fetch, POST, streaming, byte ranges; the page is static files any origin can serve | a stock phone browser plays a lent film and seeks |
| U5 any app on a port, Jellyfin as it comes | what an app cannot yet ask (DESIGN §3): the groups this computer holds and what their people lend, on the record API (u5-1); every lend at an origin of its own, a pass in place of the person's key, the web carried whole, its live socket included (u5-2); stock Jellyfin measured through a lend and the way past its sign-in decided (u5-3), then built (u5-4); DESIGN §1's list of apps measured on five of them (u5-catalogue-dry); Demo A's dry run (u5-9) — on the DESIGN §3 surface alone, and nothing that knows Jellyfin outside `apps/jellyfin/` | after `svrngs share 8096`, someone who never used Jellyfin opens it from the list, from a computer and from a phone, in Chromium and WebKit, plays, and seeks three times, each playing again within five seconds; two people at once, one seeking five times, never restart the other's play; no byte a person receives holds a secret the lend keeps; the conformance canary's routes all pass on a computer and with the wildcard, and on the floor (dropped since, u6-00) all but the two it cannot give, each named; DESIGN §1 names only apps the census supports |
| U6 reach for the room | the room door: a member's computer serves the lends its person names to its own network while the command runs, each at a port of its own on its private address, as that person (u6-1); Jellyfin's own client for a screen measured through it, and what its lender sets written in `apps/jellyfin/` (u6-2); Demo B's dry run (u6-9) — and no screen, player or media word in svrngs (DESIGN §3, "The room") | a client on the room's network, not on loopback, passes the conformance canary's routes through the room door; a lend not named is not served, nothing is served once the command stops, and a request from off the network is refused; the address picked is the private LAN's, or absent; Jellyfin's own client for a screen, stood in for by jellyfin-web, signs in, plays and seeks three times, every request through the room door |

**Demo B — movie night** (after U6; operator, 2026-10-05: the library is
across town, and the people in the room are on the TV's WiFi). Five friends
at Dee's, phones only; Sam's library is across town. Before the evening
Dee sets up two things, once: Dee's computer runs the room door for Sam's
library, and Dee's TV has Jellyfin's own app (DESIGN §3, "The room"). Sam's
group names a page origin, so its link opens in a phone's browser. Sam
texts the link; each opens it, opens Sam's library, and puts a film they
pick on the TV. Nobody installs or signs up. The run-of-show is
`demo/b.md`.
Gate: four of five newcomers get from the link to the film on the TV within
three minutes, unassisted.

**The canaries** run in `scripts/check.sh` from the moment their surface
exists, each a small app in `apps/` using only DESIGN §3:
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
  each app at an address of its own (DESIGN §7), in Chromium and in WebKit as an
  iPhone, by `crates/svrngs-node/tests/conformance.rs`; a step of its own in
  `scripts/check.sh`, printing each route's verdict per way. Every route
  passes on a computer and with the wildcard; the floor's mode went with
  the floor (u6-00). WebKit's seeks back to back are a rate,
  not a gate (operator, 2026-10-06): each run prints and keeps their verdict
  beside the same seeks straight at a canary over the same link
  (`web/tests/control.mjs`) and the host's load, and `scripts/check.sh`
  counts every run's misses on this host (§2 has the trigger).
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
  (DESIGN §3, "The room", call 6, bar 4).
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
connection, and an asleep member cost a ten-second timeout each round (two
seconds since events-asleep, `dial::DIAL_WITHIN`). Each
fix is the tail fold `Held` already keeps for writes
(`crates/svrngs-runtime/src/journal.rs:36-58`); prototyped behind switches,
they took that round from 73 s to 10 ms (svrn-docs
`rings/current/THE_CEILING.md`, its runs under `rings/runs/`). Dialling at once
stays §2's row. A commit signs no view (arch-view): each act carried its
writer's whole sync digest, read only by the signature over it, 39% of a line
at 2 members and 88% at 25, and a commit checked every signature held to make
it. Without it a line is about 540 bytes at any size, and at 25 members and
10,000 acts a commit took 0.064 s where it took 0.79 (one run each,
`demo/runs/view/2026-10-15/`). Acts already holding one are read and verify
as they did.

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
middleware spike (DESIGN §3, "The room", call 6; kept, 2026-10-08), then made
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
own, 39 svrngs's, request carried whole, discovery and opening not, DESIGN §3,
"Lent apps", call 6), then scoped use, an operation
its owner offers beside their data, to one app acting for one person
(`rings/current/ACL_AND_SCOPED_USE.md` §4, §9; run, 2026-10-09: every
bar passed, held by 46 of the service's 98 lines and two of the door's checks,
DESIGN §3, "Lent apps", call 6), each made production
from its data, as the middleware was — Movie Night by four rows
(`demo/runs/mn/2026-10-08/README.md` §9): a lent page told the lends it
reads, whose and where and who did not answer, on a phone and a
computer alike, with no pass in its hands; svrngs's refusals named as
its own; a newcomer let in by every lender at once, not at each one's
next round; a lend opened at a path from a page that reads it (landed,
mnp-1 and mnp-2: Movie Night's plumbing 39 lines to 12, `find` 28 to 1,
bar 2 passed for a phone that never opened that Jellyfin, bar 4 asleep
before the list and bar 5's letter passed, `demo/runs/mnp/2026-10-09/`;
DESIGN §3, "Lent apps", call 6) — and
scoped use by four (`demo/runs/su/2026-10-09/README.md` §10), three
of them next, the computer first (operator, 2026-10-09; svrn-docs
`rings/current/SCOPED_USE_MADE_PRODUCTION.md`): the app named on the
pass, and by a computer's origin, which keeps a page to its reads
(landed, sup-1; DESIGN §3, "Lent apps", call 6); a
person's yes and stop kept by the lend's door, said on a page no app
runs at — on a computer the home page, `svrngs.localhost` (landed,
sup-2; DESIGN §3, "What an app gets"), and on a phone the group's page (landed,
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
alone reaches on any device of theirs, waited on another device for the
same person, which bash-devices landed (DESIGN §6, "A key per device per group";
`demo/runs/devices/2026-10-12/`), and is not yet queued;
group folders (DESIGN §3, call 7 of "Lent apps"; §2), synced onto each member's
disk and not streamed (operator, 2026-10-10, after the mesh-Dropbox
spike's four bars on one host, `demo/runs/sy/2026-10-09/`; §3), made
production on the branch `folders` by its queue `syp`, main merged in as
the lean rows cut what the product does not carry — recorded runs'
drivers, tests in counted files, code nothing reaches, code written twice
(`ralph/next/lean/SURVEY.md`; lean-1, lean-2 and lean-3's first cut in at
syp-3): the rows the spike's README's §9 lists (landed, syp-4: DESIGN §3, call 7
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
merged into main by the operator (2026-10-11, `4475a89`; §3); then
Demo C's units, below (operator, 2026-10-10): removal (U7, the keeper of
DESIGN §6, with forks that void nothing; landed, u7-removal: Demo C's removal
check passed on one host, every lender refusing the removed person 0.048 s
after the keeper's command, `demo/runs/u7/2026-10-11/README.md`), before
any group of strangers;
onboarding — a computer's svrngs got from the group's link, the link handed
to it, the group's folder placed for the person, the link previewing as an
invitation in any chat (DESIGN §3, "A chat's door is the group's link") — with
the middleware's P1 follow-ups as its parts (operator, 2026-10-08;
`demo/runs/mw/2026-10-08/README.md` §7, items 5 and 7; its item 1,
`share`'s words, answered by the sign-in's declaration, mwp-1): a lender
keeps a library back from the people the middleware makes accounts for;
Demo A's steps with nothing typed, and its dry run (its other four, P2 and
below, wait in that §7) (landed, v1-onboarding: one line from the group's
page to a newcomer in the group with a member's photo on their disk in
6,489 ms, once on one host, DESIGN §7; `jellyfin-mw --keep-back`; Demo A's dry
run with nothing typed, nobody asked to sign in, three of Bo's ten
converted seeks missing 5 s on a host loaded past its cores,
`demo/runs/a/2026-10-11/`); then the library, the folder and a join measured
between computers in two homes, through a relay; then our own relay, n0
out of the path — its relays and its address lookup — built, and parked
at the gate until the operator's switch, which comes after V1 (operator,
2026-10-12, 2026-10-13 and 2026-10-15; the table below, DESIGN §11 risk 5); then
Demo C. The phone
replica spike waits in §2 (operator, 2026-10-10). After that the core grows
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
Its steps have run three times on one host, every person a program: the
dry run against the published page found eight things
(`demo/runs/c/dry-2026-10-12/`), the bash queue closed seven (the phone's
typed name is the operator's), the dry run again, on the page and tarball
built from its commit and served on its host, held each closing
(`demo/runs/c/dry-2026-10-12-rerun/`), and once more on one `svrngs relay`
on that host with n0 out of the path held them again, no address of n0's
opened, and found a relay for the group alone playing a phone nothing
(`demo/runs/c/no-n0-2026-10-13/`). What stands before Demo C is the two
homes, a phone's lend through a relay for its group alone, the phone's
typed name and publishing that build, the last two the operator's. Its
computers and phones meet their groups at n0's relays, the build's chosen
default, until the operator flips the switch to `relay.svrnme.sh`, when
they start paying for it, after V1 (§2; §3, 2026-10-13 and 2026-10-15).

| Unit | What lands | Bar, watched failing first |
|---|---|---|
| U7 removal | landed, u7-removal: one keeper, the group's maker at first, sequences removals, handed on only by its own act; anyone's `svrngs remove <name> --group <group>` is an ask until the keeper's is the cut, by hash; a removed key cannot remove, ask or hand on; forks void nothing; a removal or hand-on wakes the round (§3; DESIGN §6) | within 30 s of the keeper's removal every lender refuses the removed person in a person's words, and nothing new in a folder reaches them; their own `Remove` counts nowhere; what they held stays theirs. Watched failing: take 1, a lender not the keeper refused at 43.3 s, at its next round; passed at take 2, 0.048 s, on one host (`demo/runs/u7/2026-10-11/`) |
| onboarding | landed, v1-onboarding: on a computer, svrngs got from the group's link by one line, `install.sh` and four tarballs served beside the page, the link handed to it, the home page opened and the group's folder at `~/svrngs/<group>`; the link previewing as an invitation in any chat; the middleware's P1 follow-ups, above | a newcomer's computer from the link to the group's three things within five minutes, unassisted, one line pasted in a terminal and nothing else typed. Measured, not yet judged by people: once on one host, the paste to a member's photo on disk 6,489 ms, loopback, nobody reading; Linux could-not-judge (`demo/runs/onboarding/2026-10-11/`) |
| the dry runs | Demo C's steps on one host, every person a program. The first, against the published page, found a name merging a stranger into a person and Navidrome's admin given by it, Navidrome lent only by `--kind`, a limit of 3 nobody chose, logins as names, the phone's typed name, running computers said not to answer, no folder for the founder, and a newcomer's 51–56 s wait for the lends; bash-first-minute, bash-limit, bash-person and bash-devices closed all but the typed name | the machine's floor under each bar, never the bar: again on what landed, each newcomer from the link to film, album and photo in 12,736–13,762 ms, the lends listed 346–403 ms after the line, only the asleep computer said not to answer, each person their own Navidrome account by `x-svrngs-user`, a second `ana` refused at the line and told how to add a device; a limit's count could-not-judge on one host, Apart never-ran (`demo/runs/c/dry-2026-10-12-rerun/`) |
| two homes | the library, the folder and a join measured between computers on two networks, through a relay, each path as iroh selected it | measured, not judged: the numbers Demo C's bars are read against |
| our own relay | the relay queue (`ralph/next/relay/`), built and parked at the gate, switched on after V1 (§3, 2026-10-15): what breaks with n0 off, run on one host first (landed, relay-n0-off: with n0 off no computer dials another — svrngs's transport refuses a key with no relay, address or lookup before iroh is asked, `crates/svrngs-node/src/iroh.rs:229-234` — so only joining by link holds, and a phone joins and sees nothing lent, `demo/runs/n0-off/2026-10-12/`); each computer's relay kept in its group's record, folded in svrngs-core, and every member dialled by its key and the relay its group names — so no computer asks n0's address lookup, and none dials a key blind (landed, relay-in-record: with n0 off every step n0-off failed passes on one host — a photo on both other disks in 268 ms, every home page and the phone listing the film, the phone playing it, a photo reaching Alex in 580 ms with Sam stopped, `demo/runs/relay-in-record/2026-10-12/`); `svrngs relay`, the relay in the svrngs binary, which anyone runs: a name it is reached at, its certificate by Let's Encrypt, by default admitting only keys that stand in a group its computer holds, `--open` for one like svrngs's own, kept running as `install.sh` keeps svrngs (landed, relay-verb: a key in no group refused on 127.0.0.1, `--open` holding each client to 8,800,000 bytes a second from a film at Demo A's bitrate, `demo/runs/relay-rate/2026-10-12/`; its certificate the operator's run, DESIGN §3); svrngs's relay at `relay.svrnme.sh` the command's default with `n0_services` off, `svrngs new --relay <url>` for a group that runs its own, kept in its record so every member and phone reaches it; the page dialling the relays its link and its group name; `scripts/deploy-relay.sh` written for the operator, the relay answering before any build whose default it is is published (landed, relay-default: three computers made, joined and syncing through a group's own relay on 127.0.0.1, none with an address lookup nor tracing an n0 host, `crates/svrngs/tests/default_relay.rs`; the page in Chromium and WebKit with every address off this host refused it, a link naming no relay refused in a person's words; deploy-relay.sh written, never run, and deploy-page.sh refusing while the relay does not answer, DESIGN §7); Demo C's dry run again through `svrngs relay`'s own `relay::run` on 127.0.0.1, open and then admitting only the group (landed, relay-run: no process the run started opened an address of n0's or `relay.svrnme.sh`'s in 561 samples of `lsof -i` per process, each svrngs off the host only to the router's port mapper, no trace naming n0; the group's relay admitting its five standing computers and refusing Bo's, removed, and Fay's and Gus's, newcomers; the newcomers 5.2–6.8 s slower than the rerun's under a like load, the phone's film 3,200 ms after its play against 12,255 through n0's; and through the group's relay Cy's phone in and its film never playing, the page opening a lend from a fresh key holding a pass (`web/app.js:22-25`) that `relay::Admits` refuses, the join page shown in the film's place; Fay and Gus told the sender did not answer, not that the relay refused them, `demo/runs/c/no-n0-2026-10-13/`); then parked at the gate (relay-park; §3, 2026-10-13): n0's relays and lookup the command's default again, chosen, svrngs's relay one switch away, a closed set the build names in one constant; a group that keeps its own relay still met there, with n0's beside it; a link carrying its issuer's relay in either posture, so the page asks n0's lookup for nothing; `deploy-page.sh` publishing a build that meets its groups at n0's, and one at svrngs's only once its relay answers (landed, relay-park: `cli::MeetAt` and `cli::SHIPPED`, `crates/svrngs/tests/shipped.rs`; a computer holding only a group with its own relay homed there, while with n0's named beside it iroh chose n0's `usw1-1` over a plain-HTTP relay on 127.0.0.1, DESIGN §3; a joiner at n0's keeping none of n0's relays its link carries; the shipped command's link carrying `usw1-1` and the page joining through it in Chromium in 5,256 ms and WebKit in 5,608 ms, `demo/runs/park/2026-10-13/`; the guard shown each way, DESIGN §7) | no process of a group's, computer or phone's page, opens an address of n0's, sampled per process (held, relay-run); a key that stands in no group of a relay's computer refused there, traced (held, relay-run); a phone playing a lend through a relay for its group alone (failed, relay-run: open for a row); a newcomer refused at a group's relay told so; the command as shipped meeting a group that keeps no relay at n0's relays and lookup, and one that keeps its own there with n0's beside it, and a link from a computer with lookup carrying its relay (held, relay-park); a real phone on cellular through `relay.svrnme.sh`, the operator's run once they flip the switch (`demo/runs/c/no-n0-2026-10-13/README.md`, "The public half") |

**Start — a friend lends their model, and the group uses it** (an early
demo, for anyone getting started; operator, 2026-10-13, "an early demo
document for anyone getting started"; its run-of-show and the newcomer's
document both `demo/start.md`, its bars the seat's proposal and the
operator's to set). Sam, with no link, opens the group's page, pastes the
founder's line, makes a group on the home page and a link to invite Bo, and
lends llama-server, `svrngs share <its number> --kind llama-server`; Bo
opens the link, pastes its line, and chats with Sam's model from Bo's home
page, and a program on Bo's computer asks it at the address the page
copies. Proposed gate: Sam from the page to the model lent within five
minutes, two lines the only lines; Bo from the link to an answer within
three on a computer, one line pasted the only thing typed but the
question; the program answered at that address again after svrngs starts
again; no account, no key asked for but a program's own field. It runs on
what the start queue landed (`ralph/next/start/`): an address that stays
put, llama-server lent with nothing to look up, the home page for someone a
bit afraid, making a group there. Its story ran once on one host, every
person a program (`demo/runs/start/2026-10-13-story/`): the computer's half
whole, from no link to an answer; a phone stops at llama-server's chat page,
which answers a phone's page with nothing it can show until the phone's page
asks for gzip. Whether a reader given only the document reaches an answer is
start-doc's reader run, `demo/runs/start/2026-10-13-reader/`. What stands before its first run
with people is publishing that build (the operator's) and the phone's gzip.

**On air — three primitives an app gets, proved by an app** (after the
start queue; operator, 2026-10-12: "what's the minimal primitives we would
want to have in svrngs to allow someone else to realize a vision like
this? This is largely just a test of the infra"; 2026-10-13: "Add the on
air spike too"; its design svrn-docs `rings/current/ON_AIR.md`, its queue
`ralph/next/onair/`). A stranger's app, in `apps/on-air/`, airs an MP3
from one member's computer to the others in its group and to a page anyone
opens, and asks svrngs for three things it lacks: bytes by hash, for an
app, with a lifetime (landed, onair-blob); a signature for an app, checked
anywhere with the key alone (landed, onair-sign); the live lane naming who
spoke (landed, onair-live). svrngs runs no public door for it:
signed, hash-named bytes need no trusted carrier. Its harness runs first on
svrngs as it is, red, and that run is the reproduction
(`demo/runs/onair/`); each primitive then lands as a row of its own, proved
by that app's bars alone, and none is made general — for a second app or
another kind of bytes — before the app has proved it; then the three run
together on one host. What 10,000 listeners would need — a CDN, a second
machine, HLS and phones — is the operator's.

**Events — the primitive event and its reference app** (after onair;
operator, 2026-10-13: "I want the run after onair to be our event
primitives + reference app (we can layer on this taps concept later)";
svrn-docs `rings/current/EVENTS.md` §9). The primitive event is three acts
in a group's record and nothing new in svrngs: an event posted to each
group it draws from, each record holding only its own people's answers,
and guests as names a member brings. Its reference app lives in
`apps/events/`, in the shape of `apps/expenses/`, and runs on one host
with its predictions written first; whatever that run finds svrngs lacking
becomes a row of its own, said as svrngs's. A secret two people share, for
the question after the party, comes later, and so do taps, layered on the
event. Its demo, the launch story (operator, 2026-10-13: "so many stories
start with an event, who was invited and who actually came"; its
run-of-show `demo/events.md`, its bars the seat's proposal and the
operator's to set): Sam posts a dinner once to The House and dWeb; Bo,
Fay and Di answer from computers and Cy and Eve from phones, each seeing
only their own group's answers; Bo brings Maya by name and alone says she
came; the next morning Sam's page says, per group, who was invited (who
is in the group now, so Fay, who joined after the post) and who came,
with nothing shown as a refusal. Its story ran on one host, every person a
program (`demo/runs/events/2026-10-14/`): every bar held but one, a person
told the host's computer is asleep only after 30 s, iroh's dial giving up.
events-asleep measured a working dial, 3–347 ms from a computer and 66–103
ms from a phone through n0's relay, and capped the dial to a lend's
computer at 2 s by a rule committed before the data (`dial::DIAL_WITHIN`,
`demo/runs/asleep/2026-10-14/`), 4 s by the same rule once its capped take
saw a just-started computer's dial take 1,013 ms (events-wait): told then,
from a computer and from a phone. A phone already holding a dial to a
computer that sleeps without a goodbye still waited 30 s (that run's
`NEEDS_HUMAN.md`), until events-wait held that dial only while the
computer sends within the cap, a keep-alive each second: told in
4,010–4,271 ms where it was 34,042–34,068, the keep-alive costing the
phone's relay flow 217.5 bytes a second while the page holds the dial
(`demo/runs/wait/2026-10-14/`).
events-links gave the app the link that opens the event, which the host
copies per group and sends in that group's chat; the demo's link take
(`demo/runs/events/2026-10-14-links/`) opened it on a phone in 1,753 and
1,801 ms and on a computer by one paste on its home page, and told a
person the host's computer was asleep in 4,018–4,221 ms on the event's
address, stopped or frozen. What stands before its first run with people
is starting the app, which takes a terminal and a copy of svrngs; and a
phone tapping the link while the computer it joined through sleeps, told
only after 30 s, the page's dial to that computer left as iroh's (that
take's `NEEDS_HUMAN.md`).

**What an app could not yet ask, at U5**, each with the U5 row that adds it (the
seat's survey, `ralph/next/u5/UPSTREAM.md` §(c), every line read again at
u5-0):
- *The lender's own lend, opened* (u5-1 lists it; since u5-2 it opens at
  its origin on this computer through this computer's door's rules, its
  secret injected and its limit counted, one body with the door,
  `door::Who::Own`). Its secret lives only in the door's store, and nothing
  on loopback carries it to the lend.
- *A lend at an origin apart from the person's key* (u5-2, calls 1 and 6
  below). On a phone every lend was under the page's scope, the origin
  that holds the key (DESIGN §11, risk 5), and on a computer a lender's lends
  shared the loopback port of the bridge to that lender. Since u5-2 each
  lend opens at an origin of its own: on a computer
  `http://<label>.localhost:<port>/` (`crates/svrngs-node/src/origin.rs`),
  the home page's port, kept as it is (DESIGN §7), which `/v1/lends` answers as
  each lend's `origin`; on a phone as DESIGN §7 says,
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
for "you're next" (nothing queues, U3's inventory, above); a live payload's
sender, which a drain does not name (`rail.rs:592-605`; §2); and a phone's
own writes, which reach the record through an app naming the person in
`on_behalf_of` (U3's inventory, above).

**The sign-in made production** (DESIGN §3, "The room", call 6, at
`6c7b8296`; operator, 2026-10-08): the spike's middleware was
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
  already (U1's inventory, above), its rewrite carrying a person's headers
  where it carries the lend's one secret. `iroh_routed_forward.rs:247`
  (`mint_tie`) and `:260` (`tied_pubkey`): not forked; the answerer
  believes loopback as the middleware did, so a process on the lender's
  computer asking it gets a person's credential, and is the lender's
  own (§2 keeps the tie's trigger). Nothing upstream swaps a secret in
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

**The budget's moves** (DESIGN §5's "Lean, and kept lean" at `6c7b8296`). `svrngs-core` and the repository stay under the
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
(DESIGN §3, "The room", call 6): of its 1,050 lines of code, about 600 are any
app's, less the 330 the door and the command already have, about 270
new, and 400 with the door's traces and refusals (the seat's estimate);
and from 14,400, at 14,398, on 2026-10-08, the sign-in having taken 426
against that 400: for its decision in one place and one person's sign-in
never holding another's (`ralph/REVIEW_FINDINGS.md` #87, #88), and the
one header the scoped-use spike may add ("Then", above); and from 14,500,
at 14,466, on 2026-10-08, for Movie Night made production ("Then", above;
`demo/runs/mn/2026-10-08/README.md` §9): svrngs's refusals named as its
own (~30), a newcomer let in by every lender at once (~50), a lent page
told the lends it reads on a phone and a computer (~75), a lend opened
at a path (~25), and ~70 for the traces and refusals they carry, 250
by the seat's estimate; and from 14,750, at 14,644, on 2026-10-09, for
scoped use made production, the computer first ("Then", above; svrn-docs
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
merge (§3). And `svrngs-core`'s from 3,000, at 2,981, on 2026-10-12, for
one person, many devices (bash-devices; the operator, "yes to both";
svrn-docs `rings/current/ONE_PERSON_MANY_DEVICES.md` §6, "the core: about
+80"): a device link's person on an `Invite`, the fold's person by their
first key and `person_of`, removal and hand-on resolving a name to one
person — 3,051 with them, and no function of the core without a caller
outside the tests to make room from. And both on 2026-10-12, for svrngs's
own relay, n0 out of the path (above; §3, 2026-10-12; `ralph/next/relay/`):
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
