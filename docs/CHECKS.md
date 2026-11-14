# Check the contract at its smallest real boundary

The default edit loop runs the actual decision/formatting code with explicit
inputs. Start a browser, native process, relay or service only when that mechanism
is part of the claim. A fixture supplies observations or inputs; it does not
reimplement authorization. Fast presentation tests are not signing/I/O evidence.

## Commands and ownership

| Check | Command | Required mechanism |
|---|---|---|
| One UI contract | `node --test web/tests/creator-consent.test.mjs` | Node, actual shared formatter |
| All isolated Node laws | `node web/tests/run.mjs unit` | Node; source files, crypto and bounded shell/Git checks; no browser, native build or service |
| Node resource/browser integration | `node web/tests/run.mjs integration` | Actual Redis/Lua persistence, expiry and restart; actual WebKit/CSS application |
| Seek judgment calibration | `cargo test -p svrngs-node --test seek_judgment` | Actual pure Rust instrument, explicit recorded measurements; no browser/host sampling |
| One real creator workflow | `cargo test -p svrngs --test scenes creator -- --test-threads=1` | Native publication/signing/version routes and computer/phone browsers |
| That the shipped binary reaches n0 | `cargo test -p svrngs --test n0` (check.sh `n0`) | n0's relays and lookup on the internet, the shipped `svrngs` as built; could-not-judge when n0 does not answer iroh alone |
| That a browser decodes a lent film | `cargo test --test browser playback:: && cargo test --test conformance seeks_back_to_back:: && cargo test -p svrngs --test scenes night_played::` (check.sh `playback`) | Actual Chromium and WebKit media decoders, and Jellyfin's player on the night scene's TV through a lend's worker; the one place a test waits on playback |
| Complete regression gate | `scripts/check.sh` | Every tier, before shipping |

`--fast` keeps its static checks and isolated Node laws. The complete gate also
runs Node integration and the apps' tests. `--touched --files <path>...` is a
conservative regression selector, not an excuse to erase another session's work.
Unknown reach still runs the complete set. Shared instrument changes are held to
all their consumers. The whole gate remains the gate.

## Structural classification

Node tests that require a browser/service use `*.integration.test.mjs`.
Other `*.test.mjs` files under `web/tests/` and `resolver/` are isolated laws. `web/tests/run.mjs`
owns discovery, classification and execution; the touched planner reads its
integration catalog rather than maintaining a second list. `apps/` tests remain
the apps' own tier. Unknown tier names and empty selections report never-ran and
fail. The union of unit and integration must equal the previous complete Node
test set, with no overlap or missing cases.
The already-isolated `resolver/adapter.test.mjs` is now included in the unit
catalog; the old page-laws glob had not covered it.

## Extraction ledger — implemented and verified

| Previous location / claim | New owner | What must be preserved |
|---|---|---|
| `browser/acts.rs::what_acts_as_you_is_said_in_one_wording` | Direct question contract | All carrying/reading/Stop/selected-lend/decider outputs; independent expected dates |
| `browser/opens.rs::the_open_says_what_an_app_does_in_one_wording` | Direct question contract | Four exact examples; both pages call the shared formatter |
| `browser/main.rs::the_index_s_row_is_judged_once_for_both_pages` | Direct question contract | Full/on results; neither page owns competing full/sign-in rules; both call opens |
| `browser/main.rs::a_lends_alias_never_collides_within_a_group` | Direct question contract | Namesake/device distinctions, one carried copy, UTC+14/UTC-11 equality and all nine exact/ambiguous lookup results |
| `browser/main.rs::film_seeks_by_load` | `seek_judgment` Rust target | All three original calibration tests, including missing measurements, unknown load, 5,000 ms boundary and actual refusal/abstention distinction |
| Isolated phrase/vector, provider-failure and rejected-ingress cases from resolver tests | Direct resolver contract | Original assertions and actual signature bytes; rejected ingress makes zero store calls |
| Resolver lifecycle/consumption/limits/expiry/restart | Node integration | Actual Redis and original Lua, concurrent races, before/after write faults, AOF restart and timeout behavior |
| Page server stylesheet check | Node integration | Real HTTP content type and WebKit computed style; not merely a string or stub |
| `browser/main.rs` runtime test's play, three seeks and play again after the worker stops | `browser/playback.rs::a_lent_film_plays_and_seeks_under_its_worker`, both engines, check.sh `playback` | Each seek again within 5 s judged by `seeks_judged`, a miss past `load::BUSY` could-not-judge; Chromium's worker stopped and run again; the `<video>`'s ranges at most 2 MiB naming bo (#53) |
| Plays in the runtime, the no-store hand-over test and guests.rs (the runtime's own film test retired: its ranges, bounding and refusal marks are `lend-worker.test.mjs`'s laws, its bytes `door::a_range_is_served_206_and_416_through_the_door_sha_exact`'s) | `main.rs::film_answered`: the film's first MiB through the lend's worker, 206, `video/mp4`, its part named, sha-exact, before and after the worker stops | Every non-playback assertion of each test; the hand-over's trace read after the worker has answered |
| films scene's play, Demo A's seeks and the other's watch | person.mjs `streamed`: each person's own film's stream, 200/206 `video/*`, from their own computer's lend address; seeking's bar is `playback.rs`'s, Demo A's numbers its recorded instrument's (check.sh `pair`) | Signed in as themselves, devices apart, Sam's password in nothing received, the token plant |
| holyarrs H5's play, `seek(50)` and Jellyfin's now-playing | `streamed` through Bo's lend, by its worker on the phone, not Jellyfin's own address; `/Sessions` names Bo | H1-H4, H6-H8 |
| night scene's TV playing within three minutes, paused and sought from Ana's phone within 10 s | `night.rs::night_played`, check.sh `playback`; `night` in the scenes step asserts the TV received Ana's film from the room's address, 200/206 `video/*`, and asked nothing else off it but jellyfin-web's Chromecast sender script | Quick Connect sign-in as the room, nobody made an account, the bars' could-not-judge past `BUSY` (people.rs `late`) |
| `conformance.rs` canary's `seeks` route | `conformance.rs::seeks_back_to_back`, both ways and engines, check.sh `playback` | Chromium a gate with load could-not-judge, WebKit a rate kept in `target/seeks-webkit.log`; the canary keeps its other ten routes |

Keep these integration checks: actual HTML build/CSP, native ingress/signing,
per-person accepted code, real folder replication, withdrawals, resource limits,
long-lived connection closure, media playback and browser-origin behavior.
Media playback has one home, the `playback` step: decoding is what varies with
the host's load and its media stack, so every other test asserts the bytes a
film is answered (`film_answered`, or a scene's stream responses, person.mjs
`streamed`) and none waits on `currentTime` (demo/runs/flakes/2026-11-12).
`#[test]` alone does not imply purity: publication's test restarts real processes.
Exact writer/reader row text formerly repeated in two native workflows now has
independent golden assertions in the direct question contract. The native cases
still compare their actual DOM to the shared formatter, plus all their original
signing, reach, response and ownership observations. Copy edits have one golden
home without removing the real rendering check.

## Verification contract

Run the migrated contracts before retiring their original copies. Keep every
original assertion and all legitimate controls. Observe the moved contracts fail
on a relevant source fault, and the tier classifier fail on a misclassified
integration suite; compilation failure or zero tests is not that witness. Check
the touched planner's unit/integration/helper/service reach. Measure isolated
and service-backed runs separately. Complete the full gate once after those
focused checks pass; do not use it as the wording edit loop.

The first extraction is recorded in `demo/runs/check-tiers/2026-11-11/`:
isolated Node laws 0.652 s, real Node integration 15.822 s, warm Rust calibration
1.418 s, and the complete gate exit 0 in 1,794.463 s. The unit tier also passed
with Redis absent from PATH and an unavailable Playwright browser installation.
These are individual timing observations. The record names the compiled/source
faults that the migrated checks caught before the full gate passed.

## Seam migration ledger

The gate's tests should judge their claims on deterministic paths: no
browser, no socket, no relay, no wall-clock bar, no outside daemon, no
decoder. Each fragile mechanism, a seam, is judged in one contained place,
whose test claims only that the seam works, with the four verdicts
(passed, failed, could-not-judge, never-ran), as the `playback` step does.
The logic around a seam is claimed elsewhere, from explicit inputs.

The seams, each named by its letter in `docs/seams.tsv`:

| Seam | What varies | Seen fail as |
|---|---|---|
| B browser | Playwright's Chromium or WebKit, its scheduling and its layout | most of the browser step's 114 recorded failures |
| W leaving a page | WebKit reports the loads a navigation cancels as page errors | holyarrs H8's growing list of messages |
| M decode | the host's media stack and load | §"Media playback has one home" |
| Q QUIC | iroh/noq endpoints dialling on loopback | noq endpoint.rs:648 aborts the binary (C3) |
| R relay or n0 | a relay process, or n0's relays and lookup on the internet | relay retake races; n0 outages (the events scene at 119dd235, Sam's and Bo's home relay at n0 lost mid-scene) |
| P socket | a TCP/UDP listener or connection on the host | a prober's connection counted by a stub (C2) |
| C clock | a duration, rate or "within N s" judged on a loaded host | carried's 15 s bar, acts.rs's rate (C4) |
| D daemon | podman, Jellyfin, Redis | podman's socket refusing; Redis absent |

A child process alone (X in the map) is not a seam: a spawned `svrngs` that
opens no socket is as repeatable as a function. A node started and never
dialled still binds a UDP endpoint and its API port, so it counts as Q and P.

### Where it stands

`docs/seams.tsv` has a row for every test outside the unit tests, and for
each unit test that crosses a seam: its seams, whether it is `fragile`,
`deterministic` or `ignored` now, or `moved` (retired, its claim held
where `moves to` names), its claim with the line that asserts it,
where that claim moves, and the harness missing for the move. A unit that
moves a test changes its row in the same commit.

On 2026-11-12, at 15607df4, the gate's Rust tests (unit tests and every
target check.sh runs, ignored ones apart): 1,019, of which 471 (46.2%) cross
no seam and 548 cross at least one: P 538, Q 516, R 241, B 193, C 97, D 22,
M 7. The target is 95%: at most about 51 fragile tests, each the contained
test of its seam. The Node tiers apart: every isolated law is
deterministic; `page integration` is 13 tests on Redis and WebKit, kept as
they are (§"Keep these integration checks"); the apps' tier has 161 calls,
45 of them crossing a seam.

Of the 548, by target: browser 113, door 60, pages 36, the node's own unit
tests 36, folders 32, carried 24, home 20, scenes 20, install 16, share 14,
record_api 13, join_by_link 12, and 152 in 34 smaller targets.

Recounted at c87af561 from the binaries (`cargo test --workspace --no-run`
and each `test = false` target's, every executable's `--list` less its
`--ignored`, the ceiling canary kept; each name keyed as this map keys it,
a unit test with no row taken as crossing no seam, as this map's rule
says, and no integration test left without a row): 954, of which 515
(54.0%) cross no seam and 439 at least one: P 431, Q 419, R 205, B 158,
C 79, D 10, M 7. By target: browser 92, folders 32, the node's unit tests
26, carried 24, pages 22, home 20, scenes 20, install 16, share 14,
record_api 13, join_by_link 12, and 148 in 34 smaller targets. R counts a
relay on 127.0.0.1 too: n0 is out of every CLI test (harness 8), not R.
The isolated Node laws, 234, all deterministic.

Recounted at aa90326d the same way, the `n0` target added
(demo/runs/flakes/2026-11-13): 956, of which 516 (54.0%) cross no seam
and 440 at least one: P 432, Q 420, R 206, B 158, C 79, D 10, M 7; the
two new, n0's contained test and a deterministic one of ad25eab2's. The
scenes still cross R, a relay on 127.0.0.1, so the share does not move;
no gate test but `n0`'s depends on n0. The isolated Node laws, 234.

Recounted at 5cfa2609 the same way (demo/runs/flakes/2026-11-13,
`recount-5cfa2609.out`), after harness 4: 956, of which 528 (55.2%) cross
no seam and 428 at least one: P 424, Q 408, R 206, B 158, C 79, D 10,
M 7. The 12: ring_sync's eight exchange tests, rail's three and
record_api's loopback refusal. What harness 4 frees beside these needs a
node with no endpoint (harness 3): blob's and the node's dialling unit
tests, and unit 6's whole nodes.

Recounted at 1c33d4ff, after harness 3 (`recount-1c33d4ff.out`): 957,
of which 602 (62.9%) cross no seam and 355 at least one: P 341, Q 320,
R 206, B 158, C 79, D 10, M 7. Units 4 and 6 moved 74 more onto the
wire. Left beyond the wire's reach: the CLI's and the scenes' (a raw
dial, a real listener, a relay), the browser and pages steps (B),
kept.rs's two that kill a member process.

After harness 3, kept.rs's restart moved to the wire once a node's
folders and store end with it (node.rs `Stop`): by the count
at 1c33d4ff, one test fewer crossing Q and P, 603 of 957 (63.0%)
crossing no seam, P 340, Q 319. Counted from the map, not from the
binaries (no test was added or removed in the gate's set).

Recounted at 630967b0 (`recount-630967b0.out`), after seam B's pass:
899, of which 617 (68.6%) cross no seam and 282 at least one: P 267,
Q 234, R 140, B 90, C 75, D 10, M 7. The browser step 92 to 43 fragile,
the pages step 22 to 3, home 20 to 13 (on the wire; what is left keeps
P: a lent service, a key's try, the port probe, the resolver's stub);
their claims to Node laws over the twin, the composed phone page and
cut-from-text edges, and to wire tests in tests/carried.rs and
surface.rs. The isolated Node laws, 309, all deterministic: with them,
926 of 1,208 (76.7%).

Recounted at 18cd2afb (`recount-18cd2afb.out`), after unit 7's pass:
900, of which 651 (72.3%) cross no seam and 249 at least one: P 237,
Q 209, R 133, B 90, C 71, D 10, M 7. join_by_link 12 to 2, share 14 to
11, new 7 to 2, meet 5 to 3, leave_page 6 to 4 (to `tests/verbs` on the
wire, a join's redeem by `Net::ask_once`); install 16 to 8 (install.sh
over a stubbed `curl` and a stand-in svrngs); record_api, removal and
the node's settled rounds held to a paused clock. Left in the CLI: each
verb that asks the svrngs running here (`ask_running`, a socket), a
page's host, a relay, a real svrngs installed.

The Node side, from 2026-11-13: the resolver's Redis suite keeps eleven,
each its seam's claim, since the Lua that Redis runs decides
(resolver/store.mjs `SCRIPT`) and a JS store that answered would be a
second decider; its whole-response bound moved to the unit tier on
node:test's mocked `setTimeout` (`resolver-contracts.test.mjs`). The apps'
servers' handlers are functions of their config, asked in the test's
process with svrngs's API a function: events' partial groups and
grapevine's asker and `heldBy`, no listener and no spawned server. The
pantry's, the vision bridge's and the whisper wrapper's too, every hop a
handler on `apps/pantry/in-process.mjs`, a socket's stand-in; P's own
claim for the pantry, its address across a restart, is
`apps/pantry/address.test.mjs`. The doc's four run its server's doc and
sync.js's pages in one process on node:test's mocked clock, the lid a
stopped server; P's own, the upgrade and ws.mjs's frames, is
`apps/doc/socket.test.mjs`. Events' page
says who sees whose answers on its handler (`apps/events/answers.test.mjs`);
its browser test keeps what only a browser does, a form pressed and the
Copy button's clipboard. Grapevine's §10 scene reads its page's
markup on its handler, a loop a turn of each person's pass
(`apps/grapevine/scene.test.mjs`); its browser test keeps its script's
send, reload and refusal. House rules' page runs its own index.js in
Node on a document of plain objects (`apps/canon/house-page.test.mjs`);
its browser test keeps its policy, module and one press per engine.
Movie Night's startup is B's own: a module held in flight.

### The contained test of each seam

| Seam | Contained place | What moves out of everything else |
|---|---|---|
| B | the browser step, one test per engine and page: the home page (Chromium, `the_home_page_says_what_it_is_for`), the group's page and joining by link (WebKit, `join_page`), a carried app's signer frame and cookie in both | a page's words and decisions to Node laws over the composed page (`web/tests/pages/enter.test.mjs` composes the group's page, `web/tests/pages/twin.mjs` runs the home page's); node answers to handlers |
| W | none needed: a scene says whose pages each step leaves, and an error from one is not judged, whatever its words (`web/tests/leaving.mjs`, its law `leaving.test.mjs`) | holyarrs H8's list of messages, gone |
| M | the `playback` step, as it is | nothing left |
| Q | `door::a_range_is_served_206_and_416_through_the_door_sha_exact` (bytes cross the door), `accept::a_handler_reads_the_dialers_verified_key_and_no_header_names_another`, `folders::a_change_by_any_member_reaches_another_s_directory` | header rules, refusals and decisions to door.rs's pure functions and to `door::router` in-process; syncing claims to an in-memory transport |
| R | `default_relay::a_group_s_own_relay_meets_every_member_and_n0_never`, `relay_fallback::a_fresh_contribution_syncs_after_the_public_reach_is_cut`; and n0 in `n0::the_shipped_svrngs_is_met_at_n0_by_a_second_computer` alone, check.sh's `n0` step: the shipped binary's Sam met at a home relay of n0's, Bo joining by the link alone, could-not-judge when n0 does not answer iroh before any svrngs runs, or again after one failed | n0 out of every other test (the CLI's and the scenes': harness 8); tier choices to a pure step (tiers.rs) |
| P | `play::stop_closes_it_and_nothing_listens_while_it_is_off`, and the home page's port probe run alone | ports to injected probes and in-memory upstreams |
| C | the ceiling canary; `lend_origin::a_lend_s_computer_asleep_is_said_away_within_the_cap` | every other bar to an ordering claim over a fake clock or `now` passed in |
| D | the films scene (Jellyfin), give (podman), `page integration` (Redis) | holyarrs and night's other claims to handlers and fixtures; give's keeper to fakes of Podman |

### Harnesses missing, smallest production seam first

1. A `Dialer` a test can build, so `door::router`, `holders_router` and the
   members' routes are asked in-process: in-crate tests already can
   (iroh/tests.rs `dialer`); `tests/` cannot (iroh.rs `Dialer` is private
   on purpose). Frees most of door, oidc and ring_routes' refusals.
2. The door's upstream behind the socket it dials (door.rs `connected`): an
   in-memory stream for a test, so a lend's stub answers with no port.
   Landed: `Net::service`, a connection to a lend's service at a port,
   TCP on loopback in `IrohTransport` and one end of a pipe on a `Wire`
   (`Wire::listen`; on_wire.rs `serve` answers it with a stub); the HTTP
   door and the TCP door both dial through it. Paid for by `connected`'s
   two warn arms made one, so the count did not move. lend_origin's 7,
   session's 4, carried's 5 (its lent services) and the door's 4 (the
   forged identity, the refusals, the lists, the authorize path; origins.rs
   `answer_on`) on the wire; kept there, the WebSocket through two doors (Q) and the
   dial's cap on a slow and an asleep path (C's contained test).
3. A `Door` without a `Host`: `Folders::start` needs a bound endpoint
   (folder.rs `start`), so even an in-process door binds QUIC today.
   Landed as the Host's network half ("Harnesses 3 and 4: the
   design"): iroh.rs `Net`, its product impl `IrohTransport`;
   `Node::start_on`; the test impl node/tests_support.rs `OnWire` on a
   `Wire`, `Node::on_wire`, and common/on_wire.rs, which asks a node's
   API of its routes as a program on its computer would; `Wire::ask_as`
   asks a computer as a key on none (a phone's holder), as a test's
   splice dialled as one; `Wire::part` leaves one computer no path to
   another, as an address never seeded did; `Wire::asleep` makes one
   answer no dial, each ask waiting the dial's cap, as an address nothing
   answers at did; `Node::on_wire_listening` serves a node's API
   on a loopback listener and writes its `api` file, for a program outside
   the process.
4. An in-memory transport at `IrohTransport::fetch`/`send` and
   `ring_sync::To`, answered by the peer's `ring_routes::router`; and a holder
   at `Folders::fetch` for blobs. Landed at `fetch`: `iroh::Fetch`, its test
   impl `iroh::tests::InProcess` (bde74682, 34cad974); `send` and the
   holder go with harness 3, which a Host's transport needs. Frees folders, record_api's push and drain,
   removal's races, canaries' convergence.
5. Landed: `web/tests/pages/twin.mjs` runs the home page's own home.js in
   Node, every module loaded anew for each page (`pages/layout.mjs`),
   Preact drawing into a document of plain objects, each route answered
   from a recording in the shape its writer gives it; a law reads what was
   drawn, asked and done outside the page as data
   (`web/tests/pages/home.test.mjs`, twenty laws). No production seam.
6. A probe passed in at home.rs's `share::listening`, so no test scans ports.
7. `now` passed in where a bar is a timing: door.rs `unix_now`, carried's
   `spent`, the visit tests' end set from the real clock.
8. Landed: the CLI's spawned `svrngs` is `crates/svrngs/tests/common/as_main.rs`,
   the test binary run again as `main` but meeting at one relay on 127.0.0.1
   per binary, n0 nowhere, as home_group.rs `dee_s_computer` was alone:
   `Program` (share, join_by_link), address.rs, name.rs and install.rs's
   tarball, a script that is it. No production seam. The 25 tests that
   reached n0's relays and lookup reach 127.0.0.1 alone (a probe with
   `lsof` saw the shipped binary dial eight addresses on 443 and 80 in
   8 s, as_main's none); R stays in their rows, a relay process still.
   The scenes' too: each person's svrngs is scenes/people.rs `program`,
   the scenes' binary as `main` through as_main.rs `installed`'s script
   (a script, since a scene clears the environment, and give and
   HolyArrs's driver run the same one off the `PATH` or by its path); it
   meets at the scene's relay and the binary's, both on 127.0.0.1, the
   binary's the browser harness's relay behind its TLS front (scenes/main.rs
   `svrngs`), since a link carries its issuer's home relay first and a
   phone's page over TLS reaches a relay over TLS alone. No
   scene needed what only the real binary does (install, release paths,
   launchd), so none stays on it; `SCENES_BINARY` still pins the product
   for a rehearsal, and the xenia worlds run the product as built.
   Sampled with `lsof` over the scenes step's whole process tree
   (demo/runs/flakes/2026-11-13): every scene's svrngs opened 421 sockets
   to loopback and 13 to this host's LAN address (night's room), none
   outside, and no person's log names an n0 host, where every one did
   with the shipped binary.

### Harnesses 3 and 4: the design

One rule for both: a trait only where the network meets the node
(RUST_ARCH 7, a network client), owned beside the transport, with one
impl in the product and its test impl in a test file; no `cfg(test)`
fork or test-only branch in product code. A test impl decides nothing:
what a member answers is the answer of the router the node serves, so
every refusal a test watches is the production route's, and nothing
second stands beside a decider.

**Harness 4, a member asked in this process.** The boundary is the one
the round, a removal's tell and the live lane already share: a request
to the computer a `Contact` names, its answer read whole
(`IrohTransport::fetch`). It is `iroh::Fetch`, one method, in iroh.rs.
`ring_sync::To` is `(&dyn Fetch, &Contact)` and `RailApi::transport` an
`Arc<dyn Fetch>`; the node hands its one `IrohTransport` to both.
- Production impl: `IrohTransport`, its `fetch` unchanged (the
  connection kept, one retry on a fresh dial, `ANSWERED_WITHIN`).
- Test impl: `iroh::tests::InProcess`, each key's router asked by tower
  `oneshot` with the asker's `Dialer` inserted as `IrohAcceptor`
  inserts it; a key it serves nothing for is `Unsent::NoPath`.
- What it does not claim, and where that stays: the acceptor's admit
  decision and the verified key (`accept`'s tests), a request riding a
  kept QUIC connection (iroh/tests.rs
  `a_request_rides_a_stream_of_the_one_connection_kept_to_the_member`),
  a dial's cap (`a_computer_that_answers_no_dial_is_asleep_within_the_cap`),
  folders syncing (`folders::a_change_by_any_member_reaches_another_s_directory`).
- `IrohTransport::send`, a streamed answer (the door's and origin's),
  stays concrete: hyper's `Incoming` has no in-memory constructor, and
  none of the units below needs it.

**Harness 3, a Door without a Host.** The door's one tie to a socket is
`Door::folders`, whose `Arc<Host>` holds a bound `Endpoint` beside the
transport; an `iroh::Endpoint` exists only bound. So the seam is the
Host's network half, not the door: `Host::endpoint` and
`Host::transport` become one owned value behind a trait at the same
boundary, which `tiers.rs` (relays added and removed, the home relay and
address watched), invite.rs and join.rs (the key, a link's dial),
play.rs, a folder's Blobs dial (`Folders::fetch`, a QUIC `Connection`
handed to iroh-blobs) and `Fetch` all read. Its production impl is the
endpoint and transport as they are; its test impl answers members by
`InProcess` and a Blobs get from the holder's own store, so a door,
a folder and the record's routes run with no endpoint. Owner: node.rs,
where `Node::start_every` builds the Host. Not a second decider for the
same reason as harness 4. Landed: the trait has fourteen methods; the operator raised the cap +90 for it after the economizing pass (2026-11-13), and it took 85. A Blobs get on the wire is served by the holder's own provider and events over an in-memory pipe, so the folder's per-get decision runs as served.

**A link's issuer on the wire.** A join redeems on a connection dialled
for it alone on `Alpn::Redeem` (redeem.rs `call`), which took
`Net::dial`'s QUIC `Connection`, so no join ran on a `Wire`. The seam is
one more `Net` method, `ask_once`: a request asked of a key on an ALPN on
a fresh connection, the answer's head and body. Production impl:
`IrohTransport`, the dial and `on_stream` as `call` had them, uncapped;
test impl: `OnWire`, the issuer's acceptor decision and router for the
asker's key, as `send` is answered. Paid for by one `add_known_peer`
body where two stood (iroh.rs), so the count does not move. What it does
not claim: a fresh connection's dial (`Net::dial`, kept for the UDP
door's datagrams) and the acceptor's close (`accept`'s tests).

### Units, in order

Largest share and flake history first. Each lands per the verification
contract above: the deterministic test first, watched failing on a plant
named in its commit, every original assertion kept, the old copy retired
or shrunk to its seam claim in a commit of its own, this ledger and the
map changed with it.

| # | Unit | Tests | Moves to | Needs |
|---|---|---|---|---|
| 1 | browser step: claims the map finds covered by laws and handlers. 19 of 54 retired: an alias in two zones, a fragment's open and its words, a lend's script at a phone's page, a lost post (6 rows); and 15 on the home page (unit 5). Kept on the home page, unit 5's; 21 whose node answers are claimed only under `crates/*/tests` (door, carried, surface, lend_origin, guests, relay_fallback, a removed dialer's admission, the issuer's `/ask`: harnesses 1-4); 8 on a browser mechanism (a real alias landing, the wasm's streaming, a signer frame's cookie, a worker's update, a relay move, Cache across a restart, Web Locks across tabs, clipboard and request log). Then 4 more, the page halves to `pages/opens.test.mjs` (Play's line, copy and Stop; the apps' account; a reader's key; a native arrival's flow and welcome), the node halves already tests/home.rs's, play.rs's and install.rs's; kept, each for a node act no test outside the browser holds yet (a stop at `/reaching` refused `unasked`, a stopped visit dropped from `visiting`, surface.rs `to`) or the browser's own (a cold bookmark's redirects, a frame's or form's headers, a saved page across a restart, Web Locks, layout). Then carried's 4 (5 rows): a line past the budget rewound, the next computer after a stop, a reader's writes refused, Money and Events at their origin, to `carried.test.mjs` (lend.js and sign.js cut from text) and `pages/carried.test.mjs` (each app's index.js over its index.html), their node halves tests/carried.rs's; kept, the policy, a font, the signer frame and its cookie, a phone dialling on over a relay, and five whose node half no test outside the browser holds (`check` in `/v1/lends`, what a yes serves, a phone's `/shelf`, `/reaching` with `sends`, `/svrngs-reads` at a carried label); those five's node halves since on the wire in tests/carried.rs, and sends' 2 (4 rows) retired, the home page's send tick to `pages/sends.test.mjs`, and the policy-blanked card, its words drawn there too. Then a new version's yes on both pages and a phone's bring (3 rows), to `pages/runs.test.mjs` (the home page's POST /runs; the phone's `yes`, `through`, `running` and `carry`). Then 4 more on the wire and in laws: a Stop at `/reaching` holding from the app's next request, a visit listed on every computer and ended by its host's Stop (both pages), a carried app's one open (tests/surface.rs, tests/carried.rs, `pages/opens.test.mjs`). Then the lend worker's 4: a worker whose keys are lost, a film through it, a computer that did not answer as a dark window, a socket open again, to `lend-worker.test.mjs` (lend.js, open.js, store.js and hand.js cut from text); kept, a worker stopped by a real restart with a CryptoKey read from IndexedDB, the wasm's streaming, a real navigation and Back, frame-ancestors, DecompressionStream, and a lent link's `x-svrngs-label` and `x-svrngs-pages: none`, which door/tests.rs `a_service_is_told_the_page_of_each_group_its_asker_stands_in` and door/main.rs's forged-identity test now assert, so that test retired too, its computer's words `pages/links.test.mjs`'s. Then the home page's 10 (11 rows): a limit, signing in, a removal told, two of one name, whose computer did not answer by key, WiFi, the door's cells as an order, the screen, an older computer on both pages, an invitation's link, to `pages/home-acts.test.mjs`, removal.rs reading the ask by first key and `removed` in `/v1/groups`, synced_tests.rs the changer by first key; kept, the folder opened in a real tab (OPENERS not injectable) and lending what runs here (the real port probe, harness 6). Then the group's page's 9 and one engine of a tenth (12 rows): a removal, every removal and ask, a refusal apart, a computer named by its person, the door and index, a founder's line, the link's computer first, a link with no relay, two groups, the computer's line in WebKit, to `pages/group-page.test.mjs`, PAGE composed and its read run through `pages/phone-core.mjs`, a fake runtime in the wasm's place; kept, the dial of a standing key over a real relay, a lend's cold address and its alias landing, and the 3.3 s bar over a relay. Then join_page's device link (1 row) to verbs/join.rs and a law in `pages/enter.test.mjs`; its two route races (2 rows) claim the page's race alone, web/src/lib.rs `reach` dialling every route at once, with no pure piece to split (the race is iroh's dials), two tests since a link carries two relays at most | 54 Then, under the operator's rule of 2026-11-13 (a test asserting a seam asserts only it), acts' 2 (2 rows) retired: the group's page's "What acts as you" to `pages/phone-acts.test.mjs`, its pass hand.test.mjs's; a phone's page past the rate to door/tests.rs (`Using`'s TooFast, `read_too_fast` taking the lender). Then keep_going's 5, each shrunk to its seam, a browser's endpoint over a relay it learned or a saved page's Cache across a restart: the meets kept to enter.test.mjs, the readiness words to `unready.test.mjs` (store.js `unready` cut from text), the house's computers to relay_fallback.rs and folders.rs; a service worker's update and a saved page's join at home are seams whole. Then publication's restart test and its child retired: the controls to `pages/publication.test.mjs`, the restart and resolver calls home/publication.rs's. Then 9 rows found their seam whole, each claim a browser's mechanism with its decisions in laws already, so said as seams: gzip's DecompressionStream, the home page's policy and layout, two tabs' Web Locks, the signer's frame and cookie in both, and the navigations' real headers and redirects in both (a cold bookmark's refusal words to session_tests.rs). Then opens' 4 retired: a carried app's reads at its origin to tests/home.rs `opens` (its service in-process, harness 2), the rest to surface.rs, carried.rs, door's, session_tests.rs and the laws the map names; left, the ignored read of a real Immich. Then 10 more as seams: a cold address and an alias in both, their words to `pages/among.test.mjs` (the alias's How this reaches you out of it), the lend worker's restart, a lent page's navigation and Back, the clipboard, and the three over a real relay with a bar (whoever's awake, through another's computer, dialling on). | `pages/enter.test.mjs`, `proxied.test.mjs`, `pages/home.test.mjs` | none |
| 2 | door, oidc, session, accept: pure rows and in-process routers. Landed for the door: `tests/door`'s 60 tests are 7 contained ones (bytes both ways, what the service and a member receive, each refusal family, live answers cut, what is lent, the authorize path, a newcomer through the ring), the rest `moved` in the map, their rules pure rows over explicit inputs and a rail on a tempdir (`door/tests.rs`, `oidc/tests.rs` with the token route by oneshot, `node/tests.rs`, `lend/tests.rs`) | 68 | door.rs pure functions, `door::router` | harness 1, then 2, 3; the live cuts and the limit's window, 7 |
| 3 | n0 out of the gate. Landed: the CLI's 25 and the scenes' 20 (harness 8); n0's one contained test is `tests/n0.rs`, check.sh's `n0` step, the shipped binary met at n0 by a second computer; `tests/shipped.rs` still pins `reach(SHIPPED)` with no socket | 45, 0 left | a relay on 127.0.0.1; n0 in its own step | harness 8 (landed) |
| 4 | the node's unit tests that dial (ring_routes, ring_sync, rail, iroh, blob); ring_routes' 7 landed, asked in-process; ring_sync's 8 landed, the exchange over harness 4 (`ring_sync/tests/in_process.rs`); rail's 3, `RailApi` over it; the node's 6 that start a node and blob's 2, each node on a `Wire` (harness 3, `Node::on_wire`), a visit asked of the node's own acceptor by `Wire::ask_as`, the write's two rounds still settled by a 200 ms quiet window (C); blob's third, a fetch riding the kept connection, Q's own and kept over QUIC. Then iroh's ten-second bound, `whole` on tokio's paused clock (iroh/tests.rs `an_answer_whose_body_never_ends_is_slow_at_ten_seconds`). Left: iroh's 6 and guest's 2, Q's own seam, a dial's cap and the not-admitted close among them | 36, 9 left | routers by oneshot with a `Dialer`; transport | harness 1, 4 (landed: `iroh::Fetch`, `iroh::tests::InProcess`) |
| 5 | pages and the browser step's home-page tests. 29 retired to `pages/home.test.mjs`: 15 of the browser step's 23 (its words, the ask, the door, removals, refusals, the folder, the terminal line, links, the address and its Copy, a pasted link, two Trips, an unreadable room and register, the slowest computer as an order) and 14 of the pages step's 36 (what was done said once; five answers said once a press; a lapsed session at Leave and at an ask; WiFi opened in one foot; a folder opened twice; a group made, a group joined, each said read until it is, and Play in two lists, its page-home-rooms module gone). `the_home_page_says_what_it_is_for` shrunk to B's contained test: loaded under its policy with no page error, June's first look, look.css. Left, 7 browser tests, each asserting a node act only `crates/*/tests` or a socket also hold: a lend's declaration gone (limit) or holding its secret (sign-in), a removed computer told, a link's issuer, the room served on this WiFi, a planted opener and a tab by keyboard and mouse, ports probed. Then 19 of the pages step's 22 to `pages/again.test.mjs`: 12 on the home page over the twin (a Stop and a refused Stop, a yes from another tab, a lend from what runs here, three changes, the secret kept, a bring in the index and beside its button, a group joined, a guest's Stop, the door judged again, ten screen ticks on a held clock), 7 on the phone's page composed as web/app.js composes it (a guest at their link, a 5 s visit and an ended one on a held clock, a bring in the index and beside its button, full now on coming back, a refused yes); their node halves each named in the map. Left, 3 pages tests, each the browser's: a line kept in IndexedDB across a real reopen, the computer's line under its policy with Chromium's clipboard, a read again through the wasm endpoint the page bound; the first and the last of these claim their seam alone (seams.tsv `seam`), their words and decisions the laws' | 59, 30 left, one of them B's contained test | Node laws over the composed home page | harness 5 (landed); harness 7 for the clock, 1-4 for node acts |
| 6 | folders, carried, record_api, removal, surface, group_not_mesh, kept, canaries, relay_*. Landed: record_api's loopback refusal, its `RailApi` over a `Fetch` that reaches nobody (harness 4); folders' 26 on one `Wire` (harness 3), Q's contained test and the five whose holder is scripted over QUIC kept there; carried's 27, a key's raw HTTP at a computer asked by `Wire::ask_as` (on_wire.rs `bridge_as`), four still P by their lent service's loopback port (harness 2) and Taste by hand on a real llama-server; record_api's 10 more, then its last 2 with no socket, the listener's guard over literal addresses (the trust floor's gate, its name kept) and the token beside the api file (the port bound to surface's api-file test); removal's 9, a computer one cannot reach said by `Wire::part`; surface's 7, a computer that answers no dial said by `Wire::asleep` (its 3 s bar since on tokio's paused clock), the `api` file and the port it names kept over QUIC, that test shrunk to them, `/v1/groups`' exact row on the wire; group_not_mesh's 4, a stranger's and another group's member's dial by `bridge_as`, an address never told by `Wire::part`; canaries' 2, their computers on the wire, each record API on a loopback listener its `api` file names (`Node::on_wire_listening`), since the canary is a Node process (P and C kept). home's 20, the home page asked of its routes as a browser asks it (on_wire.rs `Browser`): seven now deterministic; twelve keep P, by a lent service its door dials (harness 2), a key's or account's try at its service, the port probe and scan (harness 6), or the join resolver's stub (`publication.rs` `send` takes no send, a product change); the link before a home stays on QUIC and two relays, its claim the wait for iroh's home report. kept's restart, Ana's node dropped and started again over her data dir, once node.rs `Stop` ended a node's folders and shut their store with it. Left: kept's 2 others, a member process stopped by SIGTERM and `kill -9` (the store's durability across a real kill) and a path iroh drops after a restart on a new port (Q's own) | ~134, 106 done Then home's 3 whose P was a lent service, its stub answered in this process (`head_on`, harness 2): a limit, svrngs's own answers to a page, the home page answering only itself; deterministic. | in-memory transport and holder | harness 3, 4 |
| 7 | CLI: share, new, install, join_by_link, tcp, udp, meet, relay, remove, play, address. Landed: address's second bare command, its svrngs running here stood in for by the data dir's lock and its api and token files, so no node runs (X alone). Then join_by_link's 10 to `tests/verbs/join.rs`, `cli::join_group` and `cli::link_to` called on one `Wire` and the issuer asked by `Net::ask_once` ("A link's issuer on the wire"); kept, `join` asking the svrngs running here (a real process over a relay on 127.0.0.1) and an older build's 404 at a socket, each `ask_running`'s. Then new's 5 and share's 3 to `tests/verbs/new.rs` and `tests/verbs/share.rs` (groups made and linked, the bare command's address, a lend's audience), a port's probe handed to `share::lend_of`; kept, a page's host asked whether it serves every name (`checked_page`, a listener), and share's lends reached through a door, its runs and its prompts (a process, a socket). Then meet's 2 and leave_page's 2 to `tests/verbs/meet.rs` and `tests/verbs/leave.rs` (a keeper's `link --relay` and a member's fold; a leave and a handed-on keeper's cut reaching a computer that was off, its place on the wire closed); kept, a join with every relay down, a public relay lost, `link --relay` asked of a running svrngs, and the home pages in a browser. Then install's 8 to install.rs's `line`, install.sh over the page's files by a `curl` stub on PATH, svrngs a stand-in that starts no node (X alone): both sum refusals, the platform's, a relay's plist and unit, the profile's PATH line, Give's agent and unit, and `restart`, this build's svrngs asked it alone; kept, 8 whose claim is the real svrngs running: a founder's line and the line run twice (it joins, keeps, opens and redeems a session), a refused join's restart (svrngs's refusal and identity), a hand-started svrngs's lock, a native arrival's one Admit, another browser's refusal page and redeem, and `new` and `join` placing the folder (no install.sh). Then four bars of "within N s, rounds 60 s apart" on the wire held to tokio's paused clock (`start_paused`), which moves only when every task waits, so each is an order before any timed round: verbs/join.rs's newcomer, record_api's woken round, removal's three tellings, node/tests.rs's settled rounds. Then home_group's two home-page tests (4 rows) and the driver they ran Dee's svrngs by: the node's half to `tests/verbs/home.rs` (home.rs `found`, `link`, `asked` and `join` asked as the page asks them on one `Wire`; `svrngs new` as `serve` runs it), the page's to two laws in `pages/home.test.mjs`; home_group.rs keeps its one deterministic test. Then the command's asks of svrngs running here through `cli::Here` (cli.rs; the product's is the data dir's `PathBuf`, the HTTP ask; tests/verbs/main.rs `Running` answers by the node's routes in this process), `cli::answer` being `run` less its print: remove's 2 to `tests/verbs/remove.rs`, `NotRunning` by the product's `Here` over a data dir that wrote no API; join_by_link's 2 (a join while svrngs runs, an older build's 404 by a router with no route), meet's `link --relay` and share's `--folder` to `tests/verbs`, the lock held as svrngs running holds it; install's `open --print` to `tests/verbs/open.rs`, the refusal page and the open's redeem asked of the node's routes. Then `Here` probes a lend's port and asks its lender at the terminal (`listens`, `prompt`, the product's `share::listens` and stdin as they were): share's sign-in, own-accounts and 4533 to `tests/verbs/share.rs`, svrngs running, nothing bound or typed. Then leave_page's last two (4 rows), its home pages in a browser: the node's half to `tests/verbs/leave.rs` (a leave asked as the page asks it, held as a leave on Sam's computer, at an old lend address and at a new link; a leave reaching the leaver's other computer off, on and off), the pages' to two laws in `pages/home.test.mjs`; leave_page.rs is gone. Then update's 5, the page's files read by a `Get` (update.rs `check_by`, `apply_at`; `fetch` the product's), no socket, its lines paid in update.rs itself. Left: each other row's pure part sits inside a function that does I/O (the Share arm's prompts, `full_name`, `restart`'s service query, `relay::spawn`'s admission), so its split is a product change, waiting for the economizing pass | ~95, 31 done | `src/*_tests.rs` over the parts that print and decide | harness 8 (landed), small splits per row |
| 8 | scenes and holyarrs; H8's list of WebKit's messages replaced by whose page each step leaves (`web/tests/leaving.mjs`). Not shrunk yet: a scene is one test whatever it asserts, so a shrink moves no count; where a scene's claim has a deterministic home it is the apps' tier already (pantry, doc, events, canon, give's fold), and the rest wait on a harness the map names (a removal's stream cut on an in-process node, a WebSocket through the door, an in-process two-node record, a Jellyfin answers fixture). Their svrngs meets at relays on 127.0.0.1, n0 nowhere (unit 3) | 20 | handlers and fixtures; one scene per daemon stays | none for W; the rest as the map says |
| 9 | give's keeper, check and machine; landed: tests/fake answers as podman, svrngs's command and the record API do, `Keeper` is driven a round at a time at a set `now` (keeper.rs), `check` runs in process; keep.rs retired | 20, 1 left: check.rs's podman test, the contained one | fakes of Podman and the clock | none |

### Owed

- Nothing: the citation of `tests/door/main.rs:56` owed here is gone from
  `docs/DESIGN.md` since d779b190, and `docs/PLAN.md` cites :59.
