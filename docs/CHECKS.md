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
| Plays in the runtime (now `a_lent_film_comes_under_the_worker_which_reaches_each_lender_itself`), both hand-over tests and guests.rs | `main.rs::film_answered`: the film's first MiB through the lend's worker, 206, `video/mp4`, its part named, sha-exact, before and after the worker stops | Every non-playback assertion of each test; the hand-over's trace read after the worker has answered |
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
3. A `Door` without a `Host`: `Folders::start` needs a bound endpoint
   (folder.rs `start`), so even an in-process door binds QUIC today.
4. An in-memory transport at `IrohTransport::fetch`/`send` and
   `ring_sync::To`, answered by the peer's `ring_routes::router`; and a holder
   at `Folders::fetch` for blobs. Frees folders, record_api's push and drain,
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
same reason as harness 4. Cost: the trait carries about ten methods
across 30 call sites, more than what is left under the cap after
harness 4 (see "Units, in order", unit 4, and the stop it names).

### Units, in order

Largest share and flake history first. Each lands per the verification
contract above: the deterministic test first, watched failing on a plant
named in its commit, every original assertion kept, the old copy retired
or shrunk to its seam claim in a commit of its own, this ledger and the
map changed with it.

| # | Unit | Tests | Moves to | Needs |
|---|---|---|---|---|
| 1 | browser step: claims the map finds covered by laws and handlers. 19 of 54 retired: an alias in two zones, a fragment's open and its words, a lend's script at a phone's page, a lost post (6 rows); and 15 on the home page (unit 5). Kept on the home page, unit 5's; 21 whose node answers are claimed only under `crates/*/tests` (door, carried, surface, lend_origin, guests, relay_fallback, a removed dialer's admission, the issuer's `/ask`: harnesses 1-4); 8 on a browser mechanism (a real alias landing, the wasm's streaming, a signer frame's cookie, a worker's update, a relay move, Cache across a restart, Web Locks across tabs, clipboard and request log) | 54 | `pages/enter.test.mjs`, `proxied.test.mjs`, `pages/home.test.mjs` | none |
| 2 | door, oidc, session, accept: pure rows and in-process routers. Landed for the door: `tests/door`'s 60 tests are 7 contained ones (bytes both ways, what the service and a member receive, each refusal family, live answers cut, what is lent, the authorize path, a newcomer through the ring), the rest `moved` in the map, their rules pure rows over explicit inputs and a rail on a tempdir (`door/tests.rs`, `oidc/tests.rs` with the token route by oneshot, `node/tests.rs`, `lend/tests.rs`) | 68 | door.rs pure functions, `door::router` | harness 1, then 2, 3; the live cuts and the limit's window, 7 |
| 3 | n0 out of the gate. Landed: the CLI's 25 and the scenes' 20 (harness 8); n0's one contained test is `tests/n0.rs`, check.sh's `n0` step, the shipped binary met at n0 by a second computer; `tests/shipped.rs` still pins `reach(SHIPPED)` with no socket | 45, 0 left | a relay on 127.0.0.1; n0 in its own step | harness 8 (landed) |
| 4 | the node's unit tests that dial (ring_routes, ring_sync, rail, iroh, blob); ring_routes' 7 landed, asked in-process | 36, 29 left | routers by oneshot with a `Dialer`; transport | harness 1, 4 |
| 5 | pages and the browser step's home-page tests. 29 retired to `pages/home.test.mjs`: 15 of the browser step's 23 (its words, the ask, the door, removals, refusals, the folder, the terminal line, links, the address and its Copy, a pasted link, two Trips, an unreadable room and register, the slowest computer as an order) and 14 of the pages step's 36 (what was done said once; five answers said once a press; a lapsed session at Leave and at an ask; WiFi opened in one foot; a folder opened twice; a group made, a group joined, each said read until it is, and Play in two lists, its page-home-rooms module gone). `the_home_page_says_what_it_is_for` shrunk to B's contained test: loaded under its policy with no page error, June's first look, look.css. Left, 7 browser tests, each asserting a node act only `crates/*/tests` or a socket also hold: a lend's declaration gone (limit) or holding its secret (sign-in), a removed computer told, a link's issuer, the room served on this WiFi, a planted opener and a tab by keyboard and mouse, ports probed. Left, 22 pages tests: the phone's page in WebKit (9), a node's write read back on `visible` or `again` (choices, lends, a bring, a join over QUIC), a visit or the screen on the real clock | 59, 30 left, one of them B's contained test | Node laws over the composed home page | harness 5 (landed); harness 7 for the clock, 1-4 for node acts |
| 6 | folders, carried, record_api, removal, surface, kept, canaries, relay_* | ~110 | in-memory transport and holder | harness 3, 4 |
| 7 | CLI: share, new, install, join_by_link, tcp, udp, meet, relay, remove, play, address. Landed: address's second bare command, its svrngs running here stood in for by the data dir's lock and its api and token files, so no node runs (X alone). Left: each other row's pure part sits inside a function that does I/O (`ask_running` cli.rs:980, the Share arm's prompts, `full_name`, `restart`'s service query, `relay::spawn`'s admission, update's fetch), so its split is a product change, waiting for the economizing pass | ~95, 1 done | `src/*_tests.rs` over the parts that print and decide | harness 8 (landed), small splits per row |
| 8 | scenes and holyarrs; H8's list of WebKit's messages replaced by whose page each step leaves (`web/tests/leaving.mjs`). Not shrunk yet: a scene is one test whatever it asserts, so a shrink moves no count; where a scene's claim has a deterministic home it is the apps' tier already (pantry, doc, events, canon, give's fold), and the rest wait on a harness the map names (a removal's stream cut on an in-process node, a WebSocket through the door, an in-process two-node record, a Jellyfin answers fixture). Their svrngs meets at relays on 127.0.0.1, n0 nowhere (unit 3) | 20 | handlers and fixtures; one scene per daemon stays | none for W; the rest as the map says |
| 9 | give's keeper, check and machine; landed: tests/fake answers as podman, svrngs's command and the record API do, `Keeper` is driven a round at a time at a set `now` (keeper.rs), `check` runs in process; keep.rs retired | 20, 1 left: check.rs's podman test, the contained one | fakes of Podman and the clock | none |

### Owed

- Nothing: the citation of `tests/door/main.rs:56` owed here is gone from
  `docs/DESIGN.md` since d779b190, and `docs/PLAN.md` cites :59.
