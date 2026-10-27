# svrngs give — any port through the door, a caller check, and Give as its own app (operator, 2026-10-25)

svrn-docs `rings/current/GIVE.md` is this queue's exploration, and its §12 lists what is still open. This queue does three things:

- **Lends take any port.** North star 2 promises "if you can run it on localhost, you can share it", and today the door carries HTTP alone. Factorio is UDP and Minecraft Java is TCP.
- **The record API stops trusting loopback.** `api.rs:12` says "Loopback IS the auth", and on a Mac running podman it is false: a container reaches the Mac's `127.0.0.1` through gvproxy (measured 2026-10-25, GIVE.md §5). This fix is owed whether or not Give exists.
- **Give ships as its own package.** A group can keep an app in rootless podman, one keeper at a time. Give lives in `apps/give/`, on svrngs's public surface and nothing else.

There are seventeen rows:

1. the economizing pass, the estimate, and the ceiling, with the three questions the charter says are awaiting the operator;
2. the token for programs;
3. the session for browsers, with the container on a Mac refused;
4. the UDP door;
5. the TCP door;
6. Play, on each member's computer;
7. the raw-port scenes and the operator's game sessions;
8. the cut: a member no longer standing loses an open raw connection, closed by the lender;
9. Give's runtime and its two cards;
10. Give keeping an app, starting from the shelf;
11. moving it;
12. taking it over, and the stale copy;
13. Give's scenes and the people runs;
14. Give's page saying when it cannot read the record;
15. install;
16. Give's install line on the group's own page host, ours the default;
17. the docs and a release, prepared for the operator.

**Waits, each with its trigger:**

- **Automatic failover:** a house rule in canon, once a group asks.
- **WAL shipping:** a group loses data it cared about.
- **Members carrying image layers:** a takeover fails because the registry was down, counted.
- **Holders who opt in:** given state too large for every member to hold.
- **The real address passed to a game (PROXY protocol):** a group asks.
- **LAN discovery, consoles and Bedrock:** a group asks, counted.
- **A cap on Give's own lines:** Give lands.

- [x] give-economize 509696f99 — depends [] — ONE ROW, each cut its own commit with its behaviour kept, and `scripts/check.sh` once, at the close — the economizing pass before any raise (AGENTS.md "Done"):
  - **(a) A survey, nothing to product.** Using the instruments of the latest `demo/runs/economize/*/SURVEY.md`, FIND what the product carries that it does not need. Look first in what this queue touches: `crates/svrngs-node/src/api.rs`, `home.rs`, `origin.rs`, `door.rs`, `lend.rs`, `node.rs`, `room/`, `crates/svrngs/src/cli.rs` (share and unshare), and `web/install.sh`. Write it to `demo/runs/economize/<date>-give/SURVEY.md`, each candidate with its lines.
  - **(b) Cuts, behaviour kept.** CUT what the survey finds, each cut its own commit, with the caps lowered to where they land.
  - **(c) An estimate.** ESTIMATE each later svrngs row's lines in four ledgers (core, library, `apps/`, surface items) from the sites it touches, beside GIVE.md §9's sizings:

    | Row | GIVE.md §9 |
    |---|---|
    | give-token plus give-session | library +160 to +310 net, core 0 |
    | give-udp | library +160 to +290, core +13 to +25 for the `Alpn` variant and the raw `Source` |
    | give-tcp | library +115 to +195 |
    | give-play | library +215 to +350 |

    `docs/PLAN.md` §1 then names this queue, in the commit that says so. Then NEEDS_HUMAN.md, asking the operator for:
    1. one ceiling for give-token through give-play, core and repository;
    2. a yes or no on overturning WHAT_CAN_ACT_AS_YOU §4's "Accept it, and say so" for programs meant to be contained;
    3. the two trust floor lines the charter names, each word for word.

    Then stop.
  - **Check:** every cut's tests unchanged by name; `python3 scripts/lean_check.py` before and after.
  - **THE OPERATOR DECIDED, 2026-10-25,** on `demo/runs/economize/2026-10-25-give/NEEDS_HUMAN.md`:
    1. **The ceiling is the estimate's top:** +1,005 repository and +5 svrngs-core over the caps the pass left, so at most 19,233 and 3,244, for give-token through give-play. Each row raises only what it lands, to one past where it lands. A row past its range stops for the operator.
    2. **§4: yes.** The caller check overturns WHAT_CAN_ACT_AS_YOU §4's "Accept it, and say so" for programs meant to be contained, and keeps it for the person's own programs, which read the token.
    3. **The trust floor:** both lines are approved with their reasons word for word as NEEDS_HUMAN.md §3 gives them, Play's held on each of its two bind lines. Each lands in its row's commit with a `Trust floor:` line citing this decision.
- [x] give-token 48c28c4d — depends [give-economize] — ONE ROW, red first, and `scripts/check.sh` once, at the close (GIVE.md §5):
  - **(a) Tests: A PROGRAM ASKS WITH THE TOKEN.** These fail at HEAD:
    - A request on loopback with no token is refused on every record route: `/v1/rail/{append,log,live,people}`, `/v1/groups`, `/v1/lends`, `/v1/folders`, `/v1/blob`, `/v1/removals`, `/v1/handovers`, `/v1/leaves` and `/v1/room`. The refusal names the token file.
    - The same request with the token is answered as today.
    - The token file is mode 0600, minted once, and kept across restarts beside `API_FILE`.
    - A plant that skips the check turns them red.
  - **(b) The change, the node's.** The token is minted and kept as `identity.rs:60-76` keeps a key. The check is one layer beside `from_this_computer` (`api.rs:103-107`). The callers send it:
    - `ask_running` (`cli.rs:915-930`);
    - `web/install.sh`'s curl (`:170-182`);
    - the test helpers through `tests/common/computers.rs`;
    - every `apps/` caller: events, pantry, doc, on-air, expenses' canary, and the movie-night, jellyfin and taste checks.

    `api.rs:12`'s "Loopback IS the auth" and the `NotLoopback` text are said anew. The trust floor gains the gate "a caller without the token is refused".
  - **Check:** (a) red first, green after; every `apps/` test and check green; the scenes green.
- [x] give-session 9a2e0b239 — depends [give-token] — ONE ROW, red first, and `scripts/check.sh` once, at the close (GIVE.md §5):
  - **(a) Tests: A BROWSER COMES THROUGH AN OPEN.** Browser tests in Chromium and WebKit, failing at HEAD:
    - The home page and a lend's origin, asked with no session, answer only "Open svrngs from your computer", naming the opener below. No terminal is needed (WHAT_CAN_ACT_AS_YOU §9: "nothing needs a terminal").
    - A new verb, `svrngs open`, opens the home page with a single-use, expiring code that becomes the session cookie. So does `install.sh`'s open.
    - **The opener.** `install.sh` puts a clickable opener in place that runs `svrngs open`: an app in `~/Applications` on macOS, and a `.desktop` entry on Linux. The "Open svrngs from your computer" page names it in the person's words ("Open svrngs from your Applications"). A run of `install.sh` checks that the opener opens the home page with a session. On a host where this cannot run, the check is never-ran with its reason.
    - A lend opened from the home page carries a code for its own origin, on `oidc/provider.rs:190-230`'s `Codes`. A replayed code is refused.
    - At a lend's origin, a request with no `Sec-Fetch-Site` is the lend's own only with the token (`origin.rs:85-88`).
    - **The kept paths:** the home page and the group page open; a lend opened from either; a phone's yes; a newcomer opens Money and Notes answering no question; a browser already opened once keeps working from a bookmark.
  - **(b) The change.** The session check sits beside `home.rs::only_itself` (`:172`) and in `origin.rs`. The exact-`Origin` guard stays as a second layer.
  - **(c) A recorded run on a Mac with podman.** A container's request to `host.containers.internal:<api port>` is refused, and the home page still opens, in `demo/runs/give/<date>-container/README.md`, with its commands. Without podman this is never-ran, with its reason.
  - **Check:** (a) red first, green after; the kept paths green throughout; the scenes green.
  - **THE DIRECTOR DECIDED, 2026-10-26,** on `demo/runs/give/2026-10-26-session/NEEDS_HUMAN.md`: the full decision is in `DECISION.md` there.
    - **A capability in the read address.** `/svrngs-reads` answers each read lend's `address` as `http://<read>.localhost:<port>/svrngs-as/<cap>/`. The cap is the token's HMAC over (page's lend, read lend), from `Sessions`, its input kept apart from a host cookie's.
    - **Where it is taken.** `origin.rs::lent` lets the prefix in when the cap is this computer's for (`app_of`'s app, this lend), and that app is another lend. The prefix is stripped before anything else; `app_of` still names the app.
    - **A navigation under the prefix** goes 303 to the home page's `/to?lend=&at=`. A cap never sets a lend's session.
    - **The three red browser tests.** Their drivers take the read's address from `/svrngs-reads`, every assertion kept. Add one assertion: the plain origin, fetched with no cap, is refused.
    - **The budget.** 18,446 at fbba26ca; the row's top is 18,495 (SURVEY.md:222). Past it after an economizing pass, stop for the operator.
- [x] give-udp ce9c2523a — depends [give-session] — ONE ROW, red first, and `scripts/check.sh` once, at the close (GIVE.md §6):
  - **(a) Tests: A UDP PORT IS LENT.** Two nodes on loopback (`loopback_only`), failing at HEAD:
    - `svrngs share <port> --udp` lends a UDP port, and a member's datagrams reach it and come back.
    - A packet larger than `max_datagram_size` arrives whole.
    - A dropped fragment drops that packet alone.
    - A key not standing is refused.
    - The declaration's limit counts people across flows.
    - A flow idle past its expiry closes, traced.
  - **(b) The change.**
    - **Core:** a fifth `Alpn` beside `Member`, `Redeem`, `Lend` and `Blobs` (`crates/svrngs-core/src/alpn.rs:14-25`), dispatched where `Alpn::Lend` is (`node.rs:403`).
    - **`lend.rs`:** a raw `Source` beside `Port` and `Carried` (`:183-187`).
    - **The lender:** one socket per flow, on `127.0.0.1:0`, connected to the lent port, with the trust floor's line.
    - **The carrier:** iroh's QUIC datagrams (`endpoint/connection.rs:907`, `:969`, `:998`), in a small frame of flow, sequence, index and count, reassembled on the far side.
  - **Check:** (a) red first, green after; a plant that skips reassembly turns the large-packet test red.
- [x] give-tcp 6acead15c — depends [give-udp] — ONE ROW, red first, and `scripts/check.sh` once, at the close (GIVE.md §6):
  - **(a) Tests: A TCP PORT IS LENT.** These fail at HEAD:
    - `svrngs share <port> --tcp` lends a TCP port, and a member's bytes reach it and come back.
    - Two connections at once run on two streams, and one stalled stream holds back neither the other nor the record's sync.
    - A half-close is carried.
    - A key not standing is refused, and the limit is counted.
  - **(b) The change.** One iroh bi-stream per connection, on the one connection kept per computer.
  - **Check:** (a) red first, green after.
  - **THE OPERATOR DECIDED, 2026-10-26,** on `demo/runs/give/2026-10-26-tcp/NEEDS_HUMAN.md`: "Go for both", this row's raise and give-play's. Landed for this row by the operator's session: the cap 18,868 (c0d3b829, its plant named) and PLAN §1's ledger line (0c002e14). What remains is `scripts/check.sh` once, at the close.
- [x] give-play db4b68713 — depends [give-tcp] — ONE ROW, red first, and `scripts/check.sh` once, at the close (GIVE.md §3 scene 2, §6):
  - **(a) Tests: A MEMBER PRESSES PLAY.** Browser tests in Chromium and WebKit, failing at HEAD:
    - A raw lend's row on a member's home page shows Play.
    - Pressing it binds `127.0.0.1:<port_of(label)>` (`room.rs:63`), UDP or TCP as the lend is, and the row says "Connect to 127.0.0.1:<port>", with the address copied, in the lend's own name.
    - The port is the same after a restart.
    - Stop unbinds it, and nothing listens while it is off.
    - A member who stops standing loses the listener.
    - **It is listed under "What acts as you".** While Play is on, anything on this computer can reach that lend as this member, since a raw port carries no token. So every Play that is on is a line in "What acts as you", beside the WiFi lines (capability-what-acts), with its Stop: "Open on this computer: <the lend's name> (<lender>'s), while Play is on. Stop". That Stop is the same act as the row's own.
    - The room's tests are unchanged by name.
  - **(b) The change.** An owner task on the room's model (`room/owner.rs:67`, `:133`), its on-state kept as `room.json` keeps the room's, and the row in the cell beside the WiFi toggle (`home/home.js:158-163`). The trust floor's Play line comes with it.
  - **Check:** (a) red first, green after; the room's tests green throughout.
  - **THE OPERATOR DECIDED, 2026-10-26:** "Go for both": this row's raise is approved in advance, to one past where it lands, while it stays inside give-play's range and the ceiling (at most 19,233 and 3,244). A worker's edit of `BUDGET` is refused as a CI bypass (give-tcp-1.out), so commit the row's work, write the raise as a ready diff in `NEEDS_HUMAN.md`, and stop; the operator's session lands it. Past the range or the ceiling, the row stops for the operator as before.
- [x] give-play-scene 32953fc8d — depends [give-play] — ONE ROW, its plant red first (AGENTS.md 5):
  - **(a) The scene, played.** A raw-port scene in the gate's "scenes" suite: two people, two computers, stand-in UDP and TCP echo servers lent, each person pressing Play and their bytes crossing. It is red on a plant that unbinds the listener.
  - **(b) Run-of-show, before any run.** `demo/factorio.md` and `demo/minecraft.md`, written before the first run (`demo/README.md`).
  - **(c) The operator's sessions.** NEEDS_HUMAN.md asks the operator to set the bars, then to play:
    - Factorio between two homes, once on a direct path and once with the relay forced, counting desyncs and recording the ping Factorio reports on each path;
    - Minecraft Java the same way.

    Then stop.
  - **Check:** the scene red on its plant, then green, in the full gate.
  - **THE OPERATOR DECIDED, 2026-10-26,** on `demo/runs/give/2026-10-26-play-scene/NEEDS_HUMAN.md`: "I think this is good enough for the game server for now -- we can do the two-home session another time." No bars are set now. The two-home sessions of `demo/factorio.md` and `demo/minecraft.md` wait for the operator, and are never-ran until they are played. Beside them stands a rehearsal on one host with Factorio 2.1.23, a UDP port lent and Play, the operator at the client: 9 minutes, no desync, no drop, its path on this host and so neither of the two the bars would name (`demo/runs/factorio/2026-10-26-rehearsal.json`, d5f65cdb). (a) and (b) landed and the full gate passed (give-play-scene-1.await.log); close the row with PLAN §1 saying the sessions wait, then stop at HUMAN-give-half.
- [x] HUMAN-give-half — depends [give-play-scene] — the operator, 2026-10-25: "Let's do half the rows." The svrngs half runs (give-economize through give-play-scene), and the loop stops here, before Give's own rows. This row is checked [x] by the operator, or the operator's delegate on the operator's word, once the svrngs half has landed and the operator says Give's half goes. **THE OPERATOR, 2026-10-26: "Let's restart the loop and slot in the fix somewhere where it makes sense"**: Give's half goes, after give-cut.
- [x] give-cut 3254ffaa8 — depends [HUMAN-give-half] — ONE ROW, red first, and `scripts/check.sh` once, at the close (the operator, 2026-10-26: "slot in the fix somewhere where it makes sense"; give-play-1.out's lesson, "Removing a member doesn't close their open port"):
  - **The gap.** The lender admits a raw use once, when a UDP flow or a TCP connection opens (`door/udp.rs` `admitted`, `PersonKey::resolved`), and never again. An open TCP connection runs until it closes, and a UDP flow until it is idle past `IDLE`, 60 s, so a game that keeps sending runs on after its member is removed. A lent page's door asks on every request (`door.rs:1099-1105`). Play closes its own listener when its member stops standing (play.rs), but that is the member's own computer, and a modified one would not. `svrngs remove` says "nothing new reaches them".
  - **(a) Tests: A MEMBER NO LONGER STANDING IS CUT.** These fail at HEAD, in `crates/svrngs/tests/udp.rs` and `tcp.rs`:
    - A member with a TCP connection open on a lend is removed, and within a few seconds the lender closes it: the member's program reads its end.
    - A member with a UDP flow that keeps sending is removed, and within a few seconds none of its packets reaches the lent port and nothing comes back.
    - The member's side is driven by `door::udp::carried` and `door::tcp::carried` directly, not by Play, so the member's own check is not what cuts them.
    - A member still standing keeps their connection through the same window.
    - A plant that skips the lender's check turns them red.
  - **(b) The change, the lender's.** While a raw lend's connection is open, the lender asks the dialer's standing again on the sweep it already keeps (`SWEEP`, 1 s), as `admitted` asks it, and closes the member's connection to that lend when it no longer stands, with a debug trace saying why. One check for both doors, in `serve`, which holds the connection.
  - **Check:** (a) red first, green after; give-udp's and give-tcp's tests unchanged by name; `python3 scripts/lean_check.py` before and after. Its range is +10 to +40 library and 0 core, inside what the ceiling for give-token through give-play left (at most 19,233 and 3,244). A worker's edit of `BUDGET` is refused as a CI bypass (give-tcp-1.out), so commit the row's work, write the raise as a ready diff in `NEEDS_HUMAN.md`, and stop; the operator's session lands it. Past the range, the row stops for the operator.
  - **THE OPERATOR DECIDED, 2026-10-26:** "Yes" to the raise to 19,111, landed by the operator's session (6019879b) at +16 and 0. The gate give-cut-1 started ran before it, so its size step fails; what remains is PLAN §1's ledger line and `scripts/check.sh` once more, at the close.
- [x] give-app-runtime 87db81a01 — depends [give-cut] — ONE ROW, in `apps/give/` (not counted, reported). The kill clause holds (CHARTER):
  - **(a) Tests: A CARD RUNS.** These run against a stand-in card (a small published image whose door is HTTP):
    - A card is the app's `compose.yaml` with an `x-svrngs` block: title, about, door (http, tcp or udp), state, and asks, each ask with its link and the environment its yes sets. Parsing refuses `build:`, `privileged`, custom networks, host networking and any bind mount from the host, each refusal naming its field.
    - `give check <card dir>` passes when the stand-in is pulled by digest, started rootless, healthy by its own `healthcheck`, snapshotted (pause, a reflink copy where the filesystem has one and a plain copy where it does not, unpause), stopped, restored, started again, and healthy again. A plant that breaks the restore fails it, naming the step.
    - On macOS the podman machine is created and started when absent, sized for what it keeps, with no terminal opened.
    - Without podman, every test is never-ran with its reason.
  - **(b) The runtime.** One module naming no svrngs type: `start`, `health`, `snapshot`, `restore` and `stop` over a card and a state directory, on podman's CLI. Containers are named `svrngs-test-*` in tests and removed by the test that made them.
  - **(c) The two cards.** `apps/give/cards/minecraft/compose.yaml` and `apps/give/cards/factorio/compose.yaml`, as GIVE.md §4 gives them, with their digests pinned. Minecraft's `VERSION` is pinned, and its EULA is an ask. Each card carries its `nouns.txt`. Each passes `give check` on this host, or is never-ran with its reason.
  - **Check:** `give check` on the stand-in red on its plant, then green; both cards checked.
- [x] give-app-keep 33bf0eadb — depends [give-app-runtime] — ONE ROW, red first (GIVE.md §3 scenes 1 and 2, §7, §7a):
  - **(a) Tests: THE GROUP STARTS AN APP.**
    - **The fold.** The keeper fact folds over Give's own acts: started, moved and taken over, each naming the snapshot and the card version it starts from. Two takeovers at once stand by admission order.
    - **The same fold, twice.** The fold is one module, built to JavaScript for the page as `apps/canon/build.sh` builds canon-core. Its tests run against both the page's build and the keeper program's.
    - **The keeper program.** It lends the app (`svrngs share`, with the card's door) only while the fold names this computer. It writes a snapshot into `~/svrngs/<group>/give/<app>/` at each interval and at a clean stop, each snapshot naming its card version. It unlends and stops when the fold names another computer.
    - **The carried page.** It shows the shelf of cards. "Start it for the group" asks the card's asks. A computer without Podman or the keeper program is asked once to install them. Each app shows "kept by <computer>".
    - **Scene 1** is played with the stand-in: a member starts the app, and another member reaches it.
  - **(b) The open question.** Who may keep is asked here (CHARTER), with any member as the default until the operator answers.
  - **THE OPERATOR DECIDED, 2026-10-26,** on `demo/runs/give/2026-10-26-keep/QUESTION.md`: "The first is fine": any member whose computer has Podman and Give's program may keep, as landed. A successor the house names waits for a group that asks.
  - **Check:** red first, green after; the fold's tests identical across both builds; svrngs's scenes green.
- [x] give-app-move 6a2893117 — depends [give-app-keep] — ONE ROW, red first (GIVE.md §3 scene 3):
  - **(a) Tests: MOVE IT TO ANOTHER COMPUTER.**
    - "Move it to…" asks the receiving computer's person, with the card's asks.
    - On their yes, the keeper takes a last snapshot, writes the move and stops, and the receiver starts from exactly that snapshot. The stand-in's state after the move equals its state before it.
    - A no leaves everything as it was.
  - **Check:** red first, green after.
- [x] give-app-takeover d069d0286 — depends [give-app-move] — ONE ROW, red first (GIVE.md §3 scenes 4 and 5):
  - **(a) Tests: THE KEEPER IS GONE, THEN BACK.**
    - A member whose computer has a copy is offered "Keep it here?" with the copy's time and the card's asks, once the keeper has not answered for the interval the row decides.
    - A copy that fails health, or exits within a minute, is offered the copy before it.
    - Two takeovers at once stand by admission order, and the later one becomes a copy.
    - The returning keeper stops, and its later state is kept as "<app> as <computer> had it at <time>", which anyone can Start instead.
    - Nothing is merged, and nothing is deleted.
  - **Check:** red first, green after.
- [x] give-app-scenes 7dd887994 — depends [give-app-takeover] — ONE ROW, its plants red first:
  - **(a) The scenes, played.** Give's scenes 1, 3, 4 and 5 join the gate's "scenes" suite with the stand-in. They are never-ran without podman, with their reason.
  - **(b) Run-of-show.** `demo/give.md`, written before any run.
  - **(c) The people runs.** NEEDS_HUMAN.md asks the operator to set the bars, then to run:
    - scenes 3 to 5 with a real Factorio world;
    - the fluency run, judged by a stranger: someone who says "I'm not good with computers" starts Minecraft for a group from the shelf and a friend joins, once with Podman installed and once without, counting minutes, decisions and questions.

    Then stop.
  - **Check:** each scene red on its plant, then green.
  - **THE OPERATOR DECIDED, 2026-10-26,** on `demo/runs/give/2026-10-26-scenes/NEEDS_HUMAN.md`: "Go with those":
    1. **Run 1, the kept world:** the candidate bars. Both markers are there after the move and after the takeover, no more than five minutes of play is lost, the later world is listed when Bo's computer comes back, and no terminal is opened past "Before the run".
    2. **Run 2, fluency:** the minutes, decisions and questions are counted and written down, with no pass mark.
    3. **Who:** the operator names them when the runs are played.
    4. **The order:** both runs wait for another time, as the game sessions do, and are never-ran until played. A one-host rehearsal of run 1 is possible here: Factorio 2.1.23 is installed through Steam (`~/Library/Application Support/Steam/steamapps/common/Factorio/factorio.app`, `demo/runs/factorio/2026-10-26-rehearsal.json`), which `ls -d /Applications/Factorio.app` does not find.
    5. **Finding 2** is a row of its own, give-page-down, before give-install.

    (a) and (b) landed. Close the row once its gate passes, with PLAN §1 saying the people runs wait.
- [x] give-page-down de180c825 — depends [give-app-scenes] — ONE ROW, red first, and `scripts/check.sh` once, at the close (the operator, 2026-10-26, on finding 2 of `demo/runs/give/2026-10-26-scenes/NEEDS_HUMAN.md`):
  - **The gap.** Give's page, left open while its computer's svrngs is down, says nothing. Each round's read throws "Failed to fetch", and the page goes on showing what it last read (`apps/give/page/index.js` `show`). A person whose svrngs restarts, during an update for example, reads a page that has silently gone stale. AGENTS.md 6: an absence is reported, never defaulted.
  - **(a) Test: THE PAGE SAYS IT CANNOT READ.** Beside Give's page test (`apps/give/page.test.mjs`, in Chromium), failing at HEAD:
    - With Give's page open and its computer's svrngs stopped, within one round the page says, in a person's words, that it can't read the group just now and will try again. Nothing it shows from before is offered as current, and no press acts on it.
    - With svrngs started again, the page reads as before, with no reload.
    - A plant that swallows the failed read turns it red.
  - **(b) The change.** In `show`, a failed read is said the way an answer that is not JSON already is.
  - **(c) A press while svrngs is down, said too.** Read at bc188e6a: `write()` (`apps/give/page/index.js:69`) lets its fetch throw with nothing to catch it, and the round (`:219`) skips `show()` while an ask, a move or a takeover is open, so a press such as "Agree and start" fails silently and its ask stays on screen as if it were waiting (give-page-down-1's lesson, seq 89). The same finding, so the same row: `write()` catches a failed fetch as `show()` now does and says, in a person's words, that it couldn't reach the group and nothing was done; the ask stays open to press again. A test in `page.test.mjs`, with `server.down()` while an ask is open, red at bc188e6a, and its plant (the catch removed) red again.
  - **Check:** (a) and (c) red first, green after; Give's scenes green; the four ledgers, `apps/` reported.
- [x] give-install 5c6e3b0d8 — depends [give-page-down] — ONE ROW:
  - **(a) Getting the keeper program.** `web/install.sh` offers Give's keeper program as an option, or the row gives its reason not to. The keeper program comes from the page host beside svrngs, with its checksum in `SHA256SUMS`. If `scripts/build-release.sh` must build `apps/give` for that, the row says what this couples, in distribution and not in code.
  - **(b) The ask in the page.** The carried page's install ask points at that one line.
  - **(c) Rosetta on a Mac with Apple silicon.** Factorio's card runs its amd64 build under Rosetta in Podman's machine, since the arm64 build segfaults making the map (`apps/give/cards/factorio/README.md:15-16`), and Podman 5.6 and later turn Rosetta off by default. The machine Give makes when there is none has Rosetta on. On a machine Give did not make that has it off, a card that needs it is refused before it starts, and Give's page says what to turn on, in a person's words, rather than letting the app crash. Never-ran where the host cannot show it, with its reason (give-app-runtime's "a Mac with Rosetta off", `demo/runs/give/2026-10-26-cards/`).
  - **Check:** `install.sh` run twice on a clean data dir, with and without the option; the checksum refused on a plant; (c) red first where this host can show it, or never-ran with its reason.
  - **THE DIRECTOR, 2026-10-26:** (a), (b) and (c) are committed (2d68d696, 2e6ca3b6, 8402864a), and `.` is at 19,120 of 19,109, reproduced. The raise to 19,121 is past the ceiling's rows (give-token through give-play), and the row names no range, so it is the operator's (CHARTER; AGENTS.md "Done"). The package is `demo/runs/give/2026-10-26-install/NEEDS_HUMAN.md`, and it recommends the raise land. Once it lands, the row's one `scripts/check.sh` closes it.
  - **THE OPERATOR DECIDED, 2026-10-26,** on `demo/runs/give/2026-10-26-install/NEEDS_HUMAN.md`: "Raise the cap". The cap is 19,121, landed by the operator's session (e5e113e7). What remains is `scripts/check.sh` once, at the close. Their answer on the install line's host is give-page-host, next.
- [x] give-page-host c5e32cd2d — depends [give-install] — ONE ROW, red first, and `scripts/check.sh` once, at the close (the operator, 2026-10-26: "always make the group.svrnme.sh configurable -- that's just our batteries included option"):
  - **The gap.** Give's install line names `https://group.svrnme.sh/` alone (`apps/give/page/index.js:23`, 2e6ca3b6), since Give's page cannot see where this computer's svrngs came from. A group on a page host of its own (README, "Run it with nobody in the middle"; `svrngs new --origin`, `svrngs link --origin`, install.sh's `<data dir>/origin`) would get svrngs and Give from ours. `git grep` over the product finds no other place where ours is the only choice: `cli.rs`'s mentions are help text, `DEFAULT_RELAY` gives way to a group's `--relay`, and the deploy scripts are the operator's, with their own flags.
  - **(a) Tests: GIVE'S LINE NAMES THIS COMPUTER'S PAGE.** These fail at HEAD:
    - A computer whose group's links carry another page (a stand-in https origin) is shown that page's install line on Give's page, with `--give`.
    - A computer that keeps no page is shown group.svrnme.sh's, the default.
    - A plant that fixes the line again turns them red.
  - **(b) The change.** The page address reaches Give on svrngs's public surface (fence 4), as one surface item: the record API says, beside each group it lists, the page that group's https links are printed on (the one `svrngs link` and the home page's `/link` print), and nothing where there is none. Give's line uses it, else ours.
  - **Check:** (a) red first, green after; the four ledgers. Its range is +5 to +25 library and 0 core. A raise is the operator's: commit the row's work, write the raise as a ready diff in `NEEDS_HUMAN.md`, and stop; the operator's session lands it on their yes.
  - **CORRECTED, 2026-10-26, by the row's worker: (b)'s route.** The list of groups the record API gives, `GET /v1/groups` (`crates/svrngs-node/src/surface.rs:76`), is served behind `not_from_a_page` (`api.rs:201-216`, layered at `:172`), which refuses 403 any request carrying `Origin` or `Sec-Fetch-Site`. Give's install line is shown by its page in a browser, before Give's program is on the computer, so a page beside `/v1/groups` never reaches it. The read Give's page does make, `GET svrngs/acts` at its own origin, already names its one group (`carried.rs:337`, `"group"`), so the page goes beside it there: `page`, the page `link` prints that group's https links on, `null` where none is kept, `{unread}` where it cannot be read. Still one surface item, on the carried route fence 4 names ("its own carried page"), and still one decider: `Host::printed_page` (`invite.rs`) is what `link_held` prints with. No check is weakened.
- [x] give-docs 5b46435e4 — depends [give-page-host] — ONE ROW:
- [x] REVIEW-audit-give-auto-1 46ed16234 — depends [give-docs] — audit the commits since the previous `REVIEW-audit-` row's hash (since this queue started if there is none) against `docs/principles/ARCH_PRINCIPLES.md`; REUSE AND SIZE, WITH DATA: (1) paste a per-unit net-line ledger for product code over that range — `git log --numstat --format=%s <hash>..HEAD`, summed by the subject's unit id, src apart from tests; (2) run `~/dev/commonwealth-ai/target/debug/sovereign-cli code dry-report --scope <dir>` for each crate dir the range touched and paste every exact or near clone with a side that is a symbol ADDED in the range; (3) for every new `struct`/`enum`/`trait` in the range, `~/dev/commonwealth-ai/target/debug/sovereign-cli code converge noun <Name>`, and paste any noun with more than one definition; fix here what is behaviour-preserving and small, record the rest in `ralph/REVIEW_FINDINGS.md` with both file:line sites — read: `docs/principles/ARCH_PRINCIPLES.md` — check: LINT
  - **(a) svrngs's documents.**
    - DESIGN §3 says the door carries any port and that the record API asks who is calling. §7 says how a computer gets Give.
    - `ARCHITECTURE.md`, `README.md` ("Lend what you run" names UDP and TCP), and PLAN §1 and §2 are updated.
    - svrn-docs `GIVE.md` gets a closing line pointing at what landed.
  - **(b) A release.** A release is prepared for the operator, and the row stops.
  - **Check:** `scripts/check.sh` once; docs-gate clean.
  - **CORRECTED, 2026-10-26, by the row's worker: docs-gate.** svrngs has none: `git grep -n docs-gate` finds only this row; it is commonwealth-ai's `cargo xtask docs-gate` (`scripts/ralph-check.sh:76` there). Read by its spirit, what a docs gate guards: every repository path the row's changed documents cite in backticks exists at the commit (a one-off script, `target/give-docs/cites.py`, red on a planted missing path, then green), and every code name they cite is found by `git grep`. No check is weakened, since none existed.
  - **NOTE, 2026-10-26, by the row's worker: the gate, twice, its failures each in code the row leaves alone.** Run 1 (at 361d12c0) failed one step: `apps/give/tests/keep.rs` `the_keeper_gone_then_back`, a count raced by the 4 s copy timer, fixed in ac76f67a. Run 2 (at e3348a0b; `git diff --stat 361d12c0 e3348a0b -- crates web scripts apps` names only that test) passed it, and failed two steps that run 1 passed on the same code. Both are follow-ups, not this row's:
    - `crates/svrngs-node/tests/removal.rs:614`, `leave_race::a_leave_takes_nothing_more_of_the_group`: Ana's copy held `["kim-last", "bo-unsent"]`, not `["kim-last"]`. Bo's round can reach Ana between the cut landing and the read, so the assertion on Ana's copy races a woken round. Kim's own copy, the one the test is about, held `["kim-last"]`.
    - The grapevine scene's step 12 (`crates/svrngs/tests/scenes/grapevine.rs:264-273`): "Sam's ask is not in Jo's copy 18s after Jo restarted", at a 1-minute load of 6.96 (15-minute 12.11), right after another tree's gate. Sam's log shows restarted Jo dialing Sam at 06:42:32.48. Sam's woken round then failed its exchange with Jo at 06:42:40.48 ("not answered within 4s", `iroh.rs:321`; `round_ms=7905`), and nothing tried again before the 18 s ran out. `RESTARTED_WITHIN`'s reasoning (`grapevine.rs:278-282`) assumes a second dial that is answered.
- [ ] give-redial — depends [REVIEW-audit-give-auto-1] — ONE ROW, red first, and `scripts/check.sh` once, at the close (the operator, 2026-10-27: "Let's fix the grapevine bug then release"):
  - **The bug, seen twice.** After a member's computer restarts, a computer that kept a connection to its old self cannot reach it for 18 s or more, so what it writes waits. The grapevine scene's step 12 fails on it (`crates/svrngs/tests/scenes/grapevine.rs:264-273`): "Sam's ask is not in Jo's copy 18s after Jo restarted".
    - On the Fedora box, in the gate's image on podman's own network (pasta), 5 runs of 5 (d5f65cdb's body): Sam's request on the connection kept to Jo's old self timed out after 4 s, and the fresh dial `iroh.rs` `fetch` sends it on again had no answer in 4 s, Jo's log showing no incoming connection. The host's network passed 3 of 3, so the gate's command there gives `--network=host`.
    - On this Mac, once (give-docs-2.await.log; the give-docs row's NOTE above): restarted Jo dialled Sam at 06:42:32.48, Sam's woken round failed its exchange with Jo at 06:42:40.48, "not answered within 4s" (`Asleep`, `iroh.rs:321`), and nothing tried again before the 18 s ran out.
    - Both times Jo's own dial reached Sam, so a connection to Jo's new self existed while Sam's requests went to the old one.
  - **(a) Instrument and reproduce first, nothing to product** (AGENTS.md 2). A test that restarts one member's computer while another keeps a connection to it, and measures how long until the other's next write reaches it, with the transport's traces at debug. Its rate, with predictions written first, in `demo/runs/redial/<date>/`: enough runs to read a spread here and on the Fedora box under podman's own network (`scripts/gate/README.md`'s command without `--network=host`). From the traces, name the cause: which connection the request rides, why the fresh dial is not answered, and what decides when the next try comes.
  - **(b) The smallest change the traces name**, red first: (a)'s test and the scene's step 12 fail at HEAD where they reproduce and pass after, and a plant that restores the cause turns them red. No wait is lengthened (`RESTARTED_WITHIN` stays, `grapevine.rs:283`), and no retry is added that the cause does not call for.
  - **Check:** (a)'s rate before and after, on both hosts; the grapevine scene green in ten runs in a row here; the Fedora box's image green on podman's own network if (a) shows the same cause there, else said with what it shows; `python3 scripts/lean_check.py`. The cap has one line of room (19,430 of 19,431). A raise is the operator's: commit the row's work, write the raise as a ready diff in `NEEDS_HUMAN.md`, and stop; the operator's session lands it on their yes.
  - **THE OPERATOR DECIDED, 2026-10-27:** "30 lines approved": a raise of up to +30 library and 0 core for this row, to one past where it lands. A worker's edit of `BUDGET` may be refused as a CI bypass (give-tcp-1.out); if it is, write the ready diff in `NEEDS_HUMAN.md` and stop, and the operator's session lands it. Past +30, the row stops for the operator.
