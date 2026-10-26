# svrngs give — any port through the door, a caller check, and Give as its own app (operator, 2026-10-25)

svrn-docs `rings/current/GIVE.md` is this queue's exploration, and its §12 lists what is still open. This queue does three things:

- **Lends take any port.** North star 2 promises "if you can run it on localhost, you can share it", and today the door carries HTTP alone. Factorio is UDP and Minecraft Java is TCP.
- **The record API stops trusting loopback.** `api.rs:12` says "Loopback IS the auth", and on a Mac running podman it is false: a container reaches the Mac's `127.0.0.1` through gvproxy (measured 2026-10-25, GIVE.md §5). This fix is owed whether or not Give exists.
- **Give ships as its own package.** A group can keep an app in rootless podman, one keeper at a time. Give lives in `apps/give/`, on svrngs's public surface and nothing else.

There are fourteen rows:

1. the economizing pass, the estimate, and the ceiling, with the three questions the charter says are awaiting the operator;
2. the token for programs;
3. the session for browsers, with the container on a Mac refused;
4. the UDP door;
5. the TCP door;
6. Play, on each member's computer;
7. the raw-port scenes and the operator's game sessions;
8. Give's runtime and its two cards;
9. Give keeping an app, starting from the shelf;
10. moving it;
11. taking it over, and the stale copy;
12. Give's scenes and the people runs;
13. install;
14. the docs and a release, prepared for the operator.

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
- [ ] give-token — depends [give-economize] — ONE ROW, red first, and `scripts/check.sh` once, at the close (GIVE.md §5):
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
- [ ] give-session — depends [give-token] — ONE ROW, red first, and `scripts/check.sh` once, at the close (GIVE.md §5):
  - **(a) Tests: A BROWSER COMES THROUGH AN OPEN.** Browser tests in Chromium and WebKit, failing at HEAD:
    - The home page and a lend's origin, asked with no session, answer only "Open svrngs from your computer", with the command.
    - A new verb, `svrngs open`, opens the home page with a single-use, expiring code that becomes the session cookie. So does `install.sh`'s open.
    - A lend opened from the home page carries a code for its own origin, on `oidc/provider.rs:190-230`'s `Codes`. A replayed code is refused.
    - At a lend's origin, a request with no `Sec-Fetch-Site` is the lend's own only with the token (`origin.rs:85-88`).
    - **The kept paths:** the home page and the group page open; a lend opened from either; a phone's yes; a newcomer opens Money and Notes answering no question; a browser already opened once keeps working from a bookmark.
  - **(b) The change.** The session check sits beside `home.rs::only_itself` (`:172`) and in `origin.rs`. The exact-`Origin` guard stays as a second layer.
  - **(c) A recorded run on a Mac with podman.** A container's request to `host.containers.internal:<api port>` is refused, and the home page still opens, in `demo/runs/give/<date>-container/README.md`, with its commands. Without podman this is never-ran, with its reason.
  - **Check:** (a) red first, green after; the kept paths green throughout; the scenes green.
- [ ] give-udp — depends [give-session] — ONE ROW, red first, and `scripts/check.sh` once, at the close (GIVE.md §6):
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
- [ ] give-tcp — depends [give-udp] — ONE ROW, red first, and `scripts/check.sh` once, at the close (GIVE.md §6):
  - **(a) Tests: A TCP PORT IS LENT.** These fail at HEAD:
    - `svrngs share <port> --tcp` lends a TCP port, and a member's bytes reach it and come back.
    - Two connections at once run on two streams, and one stalled stream holds back neither the other nor the record's sync.
    - A half-close is carried.
    - A key not standing is refused, and the limit is counted.
  - **(b) The change.** One iroh bi-stream per connection, on the one connection kept per computer.
  - **Check:** (a) red first, green after.
- [ ] give-play — depends [give-tcp] — ONE ROW, red first, and `scripts/check.sh` once, at the close (GIVE.md §3 scene 2, §6):
  - **(a) Tests: A MEMBER PRESSES PLAY.** Browser tests in Chromium and WebKit, failing at HEAD:
    - A raw lend's row on a member's home page shows Play.
    - Pressing it binds `127.0.0.1:<port_of(label)>` (`room.rs:63`), UDP or TCP as the lend is, and the row says "Connect to 127.0.0.1:<port>", with the address copied, in the lend's own name.
    - The port is the same after a restart.
    - Stop unbinds it, and nothing listens while it is off.
    - A member who stops standing loses the listener.
    - The room's tests are unchanged by name.
  - **(b) The change.** An owner task on the room's model (`room/owner.rs:67`, `:133`), its on-state kept as `room.json` keeps the room's, and the row in the cell beside the WiFi toggle (`home/home.js:158-163`). The trust floor's Play line comes with it.
  - **Check:** (a) red first, green after; the room's tests green throughout.
- [ ] give-play-scene — depends [give-play] — ONE ROW, its plant red first (AGENTS.md 5):
  - **(a) The scene, played.** A raw-port scene in the gate's "scenes" suite: two people, two computers, stand-in UDP and TCP echo servers lent, each person pressing Play and their bytes crossing. It is red on a plant that unbinds the listener.
  - **(b) Run-of-show, before any run.** `demo/factorio.md` and `demo/minecraft.md`, written before the first run (`demo/README.md`).
  - **(c) The operator's sessions.** NEEDS_HUMAN.md asks the operator to set the bars, then to play:
    - Factorio between two homes, once on a direct path and once with the relay forced, counting desyncs and recording the ping Factorio reports on each path;
    - Minecraft Java the same way.

    Then stop.
  - **Check:** the scene red on its plant, then green, in the full gate.
- [ ] HUMAN-give-half — depends [give-play-scene] — the operator, 2026-10-25: "Let's do half the rows." The svrngs half runs (give-economize through give-play-scene), and the loop stops here, before Give's own rows. This row is checked [x] by the operator, or the operator's delegate on the operator's word, once the svrngs half has landed and the operator says Give's half goes.
- [ ] give-app-runtime — depends [HUMAN-give-half] — ONE ROW, in `apps/give/` (not counted, reported). The kill clause holds (CHARTER):
  - **(a) Tests: A CARD RUNS.** These run against a stand-in card (a small published image whose door is HTTP):
    - A card is the app's `compose.yaml` with an `x-svrngs` block: title, about, door (http, tcp or udp), state, and asks, each ask with its link and the environment its yes sets. Parsing refuses `build:`, `privileged`, custom networks, host networking and any bind mount from the host, each refusal naming its field.
    - `give check <card dir>` passes when the stand-in is pulled by digest, started rootless, healthy by its own `healthcheck`, snapshotted (pause, a reflink copy where the filesystem has one and a plain copy where it does not, unpause), stopped, restored, started again, and healthy again. A plant that breaks the restore fails it, naming the step.
    - On macOS the podman machine is created and started when absent, sized for what it keeps, with no terminal opened.
    - Without podman, every test is never-ran with its reason.
  - **(b) The runtime.** One module naming no svrngs type: `start`, `health`, `snapshot`, `restore` and `stop` over a card and a state directory, on podman's CLI. Containers are named `svrngs-test-*` in tests and removed by the test that made them.
  - **(c) The two cards.** `apps/give/cards/minecraft/compose.yaml` and `apps/give/cards/factorio/compose.yaml`, as GIVE.md §4 gives them, with their digests pinned. Minecraft's `VERSION` is pinned, and its EULA is an ask. Each card carries its `nouns.txt`. Each passes `give check` on this host, or is never-ran with its reason.
  - **Check:** `give check` on the stand-in red on its plant, then green; both cards checked.
- [ ] give-app-keep — depends [give-app-runtime] — ONE ROW, red first (GIVE.md §3 scenes 1 and 2, §7, §7a):
  - **(a) Tests: THE GROUP STARTS AN APP.**
    - **The fold.** The keeper fact folds over Give's own acts: started, moved and taken over, each naming the snapshot and the card version it starts from. Two takeovers at once stand by admission order.
    - **The same fold, twice.** The fold is one module, built to JavaScript for the page as `apps/canon/build.sh` builds canon-core. Its tests run against both the page's build and the keeper program's.
    - **The keeper program.** It lends the app (`svrngs share`, with the card's door) only while the fold names this computer. It writes a snapshot into `~/svrngs/<group>/give/<app>/` at each interval and at a clean stop, each snapshot naming its card version. It unlends and stops when the fold names another computer.
    - **The carried page.** It shows the shelf of cards. "Start it for the group" asks the card's asks. A computer without Podman or the keeper program is asked once to install them. Each app shows "kept by <computer>".
    - **Scene 1** is played with the stand-in: a member starts the app, and another member reaches it.
  - **(b) The open question.** Who may keep is asked here (CHARTER), with any member as the default until the operator answers.
  - **Check:** red first, green after; the fold's tests identical across both builds; svrngs's scenes green.
- [ ] give-app-move — depends [give-app-keep] — ONE ROW, red first (GIVE.md §3 scene 3):
  - **(a) Tests: MOVE IT TO ANOTHER COMPUTER.**
    - "Move it to…" asks the receiving computer's person, with the card's asks.
    - On their yes, the keeper takes a last snapshot, writes the move and stops, and the receiver starts from exactly that snapshot. The stand-in's state after the move equals its state before it.
    - A no leaves everything as it was.
  - **Check:** red first, green after.
- [ ] give-app-takeover — depends [give-app-move] — ONE ROW, red first (GIVE.md §3 scenes 4 and 5):
  - **(a) Tests: THE KEEPER IS GONE, THEN BACK.**
    - A member whose computer has a copy is offered "Keep it here?" with the copy's time and the card's asks, once the keeper has not answered for the interval the row decides.
    - A copy that fails health, or exits within a minute, is offered the copy before it.
    - Two takeovers at once stand by admission order, and the later one becomes a copy.
    - The returning keeper stops, and its later state is kept as "<app> as <computer> had it at <time>", which anyone can Start instead.
    - Nothing is merged, and nothing is deleted.
  - **Check:** red first, green after.
- [ ] give-app-scenes — depends [give-app-takeover] — ONE ROW, its plants red first:
  - **(a) The scenes, played.** Give's scenes 1, 3, 4 and 5 join the gate's "scenes" suite with the stand-in. They are never-ran without podman, with their reason.
  - **(b) Run-of-show.** `demo/give.md`, written before any run.
  - **(c) The people runs.** NEEDS_HUMAN.md asks the operator to set the bars, then to run:
    - scenes 3 to 5 with a real Factorio world;
    - the fluency run, judged by a stranger: someone who says "I'm not good with computers" starts Minecraft for a group from the shelf and a friend joins, once with Podman installed and once without, counting minutes, decisions and questions.

    Then stop.
  - **Check:** each scene red on its plant, then green.
- [ ] give-install — depends [give-app-scenes] — ONE ROW:
  - **(a) Getting the keeper program.** `web/install.sh` offers Give's keeper program as an option, or the row gives its reason not to. The keeper program comes from the page host beside svrngs, with its checksum in `SHA256SUMS`. If `scripts/build-release.sh` must build `apps/give` for that, the row says what this couples, in distribution and not in code.
  - **(b) The ask in the page.** The carried page's install ask points at that one line.
  - **Check:** `install.sh` run twice on a clean data dir, with and without the option; the checksum refused on a plant.
- [ ] give-docs — depends [give-install] — ONE ROW:
  - **(a) svrngs's documents.**
    - DESIGN §3 says the door carries any port and that the record API asks who is calling. §7 says how a computer gets Give.
    - `ARCHITECTURE.md`, `README.md` ("Lend what you run" names UDP and TCP), and PLAN §1 and §2 are updated.
    - svrn-docs `GIVE.md` gets a closing line pointing at what landed.
  - **(b) A release.** A release is prepared for the operator, and the row stops.
  - **Check:** `scripts/check.sh` once; docs-gate clean.
