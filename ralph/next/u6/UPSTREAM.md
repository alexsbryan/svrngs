# U6 — what upstream and svrngs have for the screen handoff (the seat's inventory notes, 2026-10-05)

Read-only survey of commonwealth-ai at e1a43bcd2/HEAD and svrngs at e0-bot/HEAD. Lines labelled "general knowledge, unverified here" are not measurements. U6's inventory row writes DESIGN §8 from it. Input to the queue, not a record.

## U6 (the screen handoff): what exists upstream and in svrngs, and what is missing

**Short answer.** Nothing in commonwealth-ai at `e1a43bcd2`/HEAD, or in svrngs at `e0-bot`/HEAD, implements AirPlay, Cast, DLNA, Remote Playback, a TV URL, a stream token or a remote control. There is no TV/casting code at all.
- What exists is one research table row (the shelf), the detailed `e0-bot` design (prose, written for a native app), and some pieces U6 could fork:
  - upstream's LAN guest door, which binds `0.0.0.0:9744` only while a grant is live;
  - its "derive the LAN address" helper;
  - an in-memory TTL token store;
  - a Range test origin;
  - iroh's `Endpoint::addr()`, which already lists the node's LAN addresses.
- Measurements cover iroh splice throughput, Range through the splice, and relay rates. Nothing was measured on a TV or with Safari/Chrome casting.

**Repo state.** svrngs `main` moved during this session, from 632d5d3 to e32a4ac. Commit d3a39b7 added U4's inventory and the seat's five calls to DESIGN. svrngs DESIGN line numbers below are at HEAD e32a4ac. commonwealth-ai HEAD is b1d992295; where a file moved I give the new path (blob identity checked with `git rev-parse`).

Size command (the same method as svrngs `ralph/next/u4/UPSTREAM.md:16-22`):
`git -C <repo> show <rev>:<path> | sed -n 'A,Bp' | awk '{l=$0; sub(/^[ \t]+/,"",l); if(inb){if(l~/\*\//)inb=0; next} if(l~/^\/\*/){if(l!~/\*\//)inb=1; next} if(l==""||l~/^\/\//)next; c++} END{print c+0}'`

### (a) Table

**commonwealth-ai @ e1a43bcd2**

| path:lines | what it does | size | dependencies | constraints |
|---|---|---|---|---|
| `quality/campaigns/ring-apps-shelf.md:181` (same at HEAD, identical) | the gateway row: "one LAN HTTP endpoint `hash + Range → 206`, never 200" | prose | none | a research verdict (the passes of 2026-09-17, `:6-8`), not a measurement; no code ever followed it |
| `ring-apps-shelf.md:183-196`, `:257`, `:271-272` | the play section; casting row at `:191`; library row at `:257` ("LAN gateway + `<video>` on faststart MP4/H.264 … Cast DMR + HDMI browser floor"); gaps at `:271-272` | prose | — | lists as unverified "Safari MKV; whether Cast accepts plain `http://` LAN URLs" (`:191`) |
| `sovereign/crates/sovereign-daemon/src/guest_door.rs:412-499` (HEAD `svrn/crates/sovereign-daemon/src/guest_door.rs`, identical) | `serve` and `wait_for`. Binds `0.0.0.0:DEFAULT_GUEST_PORT` (9744) TCP when no bind is configured (`:434-443`), only while a live grant exists, and shuts down gracefully at the last expiry (`:455-487`). "safe by LIFETIME, not by address" (`:429-431`) | 67 (whole file 534) | axum 0.8, tokio, `sovereign_grants::GuestGrantStore`, daemon `AppState`, `sovereign_contracts::guest_pages::DEFAULT_GUEST_PORT` (`guest_pages.rs:56`) | plain HTTP; polls (`POLL`); tied to the daemon's AppState and auth layer. The closest fork candidate for a LAN listener that opens only while a handoff is live |
| `guest_door.rs:617-729` | the LAN door's proxy (`proxy_request`, `proxy_get`) | 104 | reqwest | buffers the body to 16 MiB (`:619`, `:646`), buffers the whole response (`:689`), returns only status, content-type and body (`:681-696`). So it drops `Content-Range` and cannot serve a TV. U4's inventory already says "not forked" |
| `sovereign/crates/sovereign-cli-mesh/src/mesh_guest_link.rs:16-96` (HEAD `cmnwlth/crates/sovereign-cli-mesh/src/mesh_guest_link.rs`, identical) | `port_of_bind`, `advertised_base`, `guest_bind_url`: "the door names the PORT, this machine names the ADDRESS" (`:10-12`). Never advertises `0.0.0.0`/loopback (`:45-54`); prefers LAN over tailscale (`:67-80`) | 45 | `commonwealth_discovery::mesh_discovery::relay_candidates`, `SetupConfig` | composes `http://{ip}:{port}` only; with no candidate the answer is absence (`:34-36`) |
| `commonwealth/crates/commonwealth-discovery/src/mesh_discovery.rs:39-125` and `:237-313` (HEAD `cmnwlth/crates/commonwealth-discovery/src/mesh_discovery.rs`, identical) | `classify_ip`, `relay_candidates` (tiers tailscale > lan > ipv6), and `local_ip_candidates`: `if_addrs::get_if_addrs()` minus loopback and link-local, with a UDP-connect fallback | 60 + 48 | `if-addrs` crate (svrngs does not carry it), serde | classifies every non-CGNAT IPv4 as "lan", public addresses included (`:56-66`). The docker-bridge-first caveat is at `:21-32` |
| `commonwealth/crates/commonwealth-discovery/src/mdns.rs` | `mdns-sd` 0.12, advertises `_commonwealth._tcp.local` between nodes | 350 | mdns-sd | finds nodes, not TVs. A browser cannot do mDNS, and svrngs DESIGN:657 keeps local discovery waiting |
| `sovereign/crates/sovereign-grants/src/guest_grant.rs:282-400` | `GuestGrantStore`: tokens in memory with a TTL, `issue`/`live`/`revoke`/`drain_dead`/`spawn_reaper`; token and clock injected (`:44-50`); deliberately not persisted (`:32-38`) | 69 (whole 326) | tokio, serde | svrngs DESIGN:467: "not forked: its state is lost on a restart" (said of U2's links). For a short-lived TV token, in-memory may be acceptable, but that is a new decision |
| `commonwealth/crates/commonwealth-transport/examples/media_bridge_bench.rs` (HEAD `cmnwlth/…`, identical) | a Range origin plus a rate and stall harness | 696 | tokio, iroh | already forked as svrngs test fixtures (next table) |
| `sovereign/deploy/mesh/WORK_PLANE.md:363-372, 416-500, 538-585` (HEAD `cmnwlth/deploy/mesh/WORK_PLANE.md`, path rewrites only) | the measurements in (b) | prose | — | — |
| `docs/THE_LINK.md:61-63, 217-222`; `docs/RING_APP_LIBRARY.md:308-323, 565-571, 618-631` | the LAN door is plain HTTP for guests; the https origin exists for a secure context | prose | — | see (d) |
| `quality/campaigns/ring-room.toml:69-81` | upstream's only "TV": "the machine wired to the projector" plays through the loopback tunnel; status "open" | prose | — | no casting involved |
| `commonwealth/crates/commonwealth-media/src/reach.rs:1-10` | "the loopback URL a player is pointed at" | — | — | upstream media reach is loopback only |
| `sovereign-mobile/` | a Tauri chat client | — | — | no TV or casting code (grep for video, media, cast, airplay, tv) |

**svrngs @ e0-bot** (design only; a grep of the whole `e0-bot` tree outside `docs/DESIGN.md` for airplay, cast, dlna, upnp, mdns, bonjour, unguessable finds nothing)

| path:lines | what it does | size | constraints |
|---|---|---|---|
| `docs/DESIGN.md:541-576` | "A TV in the room": the full TV path | 36 lines of prose | assumes a native app: AVKit route picker, the Cast SDK, a media background mode (`:757-758`, `:556-557`) |
| `:528-529` | the "serve to a screen" route | — | accepted only from the always-on node owner's own devices |
| `:92` | `group.play(name, paths, {on: here \| screen})` | — | — |
| `:249` | screen protocols as an enum: `AirPlay`, `Cast`, `Dlna` | — | — |
| `:943-954` | bars: play-on-a-tv, finishes-a-film | — | — |
| `:1009-1010`, `:1020-1027` | risks 12 and 16 | — | — |

**svrngs @ HEAD (what the node serves today)**

| path:lines | what it does | size | dependencies | constraints |
|---|---|---|---|---|
| `crates/svrngs-node/src/door.rs:1-709` | the member door. `/lend/<lend>/<path>` reaches `127.0.0.1:<port>` (`:642`); both bodies stream; Range, If-Range and Content-Range pass (`:25-26`); limit and share | 539 | axum, hyper, http-body-util | serves only on an iroh stream: refuses without a `Dialer` (`:453-458`); person comes from the dialer's key (`:473`); drops `upgrade` as hop-by-hop (`:120-129`) |
| `crates/svrngs-node/src/door/limit.rs:49-63` | `guess`: a private, link-local or ULA address means Home, uncounted and told no share | 94 non-test | — | a relayed or unseen path counts as Away |
| `crates/svrngs-node/src/lend.rs` | the lend declaration: port on 127.0.0.1, kind, limit, refused paths, upload | 315 | serde | — |
| `crates/svrngs-node/src/api.rs:92-102` | `check_loopback`: the record API refuses any non-loopback bind | 103 | axum | a TV listener cannot reuse this bind |
| `crates/svrngs-node/src/guest.rs:155-209`, `:300-509` | `build_relayed_endpoint` binds every interface unless `loopback_only` (`:57-61`, `:202-207`); `HttpBridge` binds only 127.0.0.1 (`:361-376`) | 140 (bridge) | iroh =1.0.2, tokio | — |
| `crates/svrngs-node/src/node.rs:60-69`, `:234-268` | `Node::lends`: each lend's address is the loopback bridge plus `/lend/<name>/`; `Node::endpoint()` is at `:221` | 334 | — | — |
| `crates/svrngs-node/src/iroh.rs:67-80`, `:362-385` | `format_dial_string(&endpoint.addr())` lists relays and then `ip_addrs()`; `Via::observe` reports the selected path's address and whether it is relayed | 633 | iroh | — |
| `crates/svrngs-node/tests/door/origins.rs:87-200` | `range_origin`, forked from `media_bridge_bench.rs`: answers 206, 416, `Content-Range`, `video/mp4` | 288 | tokio, sha2 | test fixture; a ready origin for U6 tests |
| `~/.cargo/registry/src/*/iroh-1.0.2/src/endpoint.rs:1184-1198`, `:1269-1284`, `:1286-1300`, `:1437-1447` | `addr()` = relay plus the IP addresses bound on interfaces (native); in wasm, "there will never be any direct addresses" (`:1298`); `bound_sockets()` returns wildcard sockets | — | — | the meeting point can read its own LAN IPs from `endpoint().addr().ip_addrs()` with no new crate; it must filter them (tailscale, docker, public) |

### (b) Measured facts, verbatim

**The shelf row, and the rows around it.** These are recorded research findings, not measurements made in the repo.
- `ring-apps-shelf.md:181`: "| gateway | one LAN HTTP endpoint `hash + Range → 206`, never 200 | no sink speaks iroh; MapLibre Native OOMs on a 200 to a range request |"
- `:174`: "seekable counter mode is MANDATORY — any chained mode silently kills range requests"
- `:179`: "one verified random range request per probe (~17 KiB)"
- `:183`: "### play — open. Shelf: remux not transcode; hls.js byterange; Jellyfin is closed to us."
- `:187`: "Jellyfin | **GPL-2.0-ONLY** … API interop at arm's length only."
- `:190`: "`EXT-X-BYTERANGE`: a computed `.m3u8` points every "segment" at a byte range inside the ONE existing file"
- `:191`: "| casting | Cast Default Media Receiver (no registration); DLNA via `rupnp` 3.0.0 (MIT/Apache); AirPlay free in Safari via `x-webkit-airplay` | | adopt | floor is a browser on an HDMI stick. Unverified: Safari MKV; whether Cast accepts plain `http://` LAN URLs |"
- `:193-194`: "MKV is the enemy, not the transport: no Chrome `<video>` support, not on Chromecast's list (Firefox 145 added it). HEVC in Chrome is GPU-only."
- `:271-272`: "Cast receiver on plain-http LAN URLs" is listed among the verification gaps.

**Choosing a LAN address.**
- `mesh_guest_link.rs:69-71`: "a GUEST LINK is opened by a phone on the venue's WiFi, which cannot reach 100.64/10. Measured: the default picked 100.104.36.28 while the room's phones needed 192.168.1.7."
- `guest_pages.rs:50-52`: port 9744 was chosen after "measured 2026-09-22 — `ring host` would have collided with its own rail".

**iroh splice measurements (WORK_PLANE.md).** All on one machine; the cross-network bar was "could-not-judge" (`:565-566`).
- Range survives the splice: "`206 Partial Content` with a correct `Content-Range: bytes a-b/2147483648`, `200` for the full GET, and `416` … all crossed unaltered" (`:451-454`).
- Viewers do not share a connection: "N viewers are N QUIC connections" (`:422`); "Four concurrent 100 MiB viewers each held 3.7-4.8 Gbit/s" (`:431-432`).
- Seeks are expensive: "Every viewer pays a full QUIC handshake, and so does every SEEK" (`:434`); "p50 189 ms / p90 1327 ms / 2 dial timeouts with a fresh connection per gossip round, against 38.8 ms / 391 ms / 0 when the connection is reused" (`:438-439`). Therefore "the shim's HTTP client must pool connections and its origin must speak keep-alive" (`:441-442`).
- Throughput: through `HttpBridge`, minimum 4583 and median 5099 Mbit/s (`:477`); "the splice costs about 5.6x" (`:480`).
- Soak: 25 Mbit/s for 600 s, three runs, max gap 87-89 ms, zero gaps over 500 ms (`:490-494`).
- Relay: "7.8 Mbit/s is an UPPER bound on the relay rate for that run" (`:553-555`); "`--relay-only` is a request, not a guarantee" (`:561-563`).

**Relayed readings (TWO_MACHINE_RUNBOOK.md, HEAD `bench/lanes/media_bridge/`).**
- `:148-150`: "The first honest relayed reading was taken and it is **8.7 Mbit/s against a 25 Mbit/s bar** … `path=relayed direct=[]`"
- `:214`: "RuggedFox's uplink to a neutral endpoint is 21.1 / 25.7 Mbit/s"
- `:232-235`: cloud VM on a mixed path: "one stream 87.0 Mbit/s over 25 s and 55.2 over 623 s with 3 reconnects and four >1 s gaps".

**svrngs, finding #33** (`ralph/REVIEW_FINDINGS.md:875-881`). This confirms iroh lists and uses the LAN address:
- "The door logged "door: guessed in this computer's home" 39 times, each `addr=Some(192.168.1.4)`. Both nodes bind iroh's wildcard, and iroh moves the connection dialled on loopback onto this computer's private address"
- the loopback knob's test "sees `[0.0.0.0:…, [::]:…]`".

**svrngs, Demo A dry run** (one computer, headless Chrome, `demo/runs/a/dry-2026-10-04/README.md:26`).
- `:55-58`: "play starts in 1,137 ms, and seeks to 60, 20 and 75 s each play again in 65, 123 and 7 ms … (`stream.mp4?Static=true`, five Range requests answered 206), 14.9 MB received".
- `:69-73`: "Jellyfin's live socket does not cross the door … the door drops `Upgrade` as hop-by-hop … Jellyfin's remote control and live session updates do [need it]."
- `data/none.requests.tsv:67,74,106`: Jellyfin's web client loads `chromecastPlayer-plugin…chunk.js` and Google's `cast_sender.js` (https at `:74`, http at `:106`). This is a HAR fact about Jellyfin's own player, which no unit builds into (DESIGN:733-734).

**Browser dial.** `ralph/DECISIONS.md:9948-9960` (commonwealth-ai): headless Chrome for Testing 147 on Linux, relay-only, `HTTP/1.1 200 OK`. No phone has ever dialled.

**Not found.** I searched both repos at both revisions for airplay, chromecast, `x-webkit-airplay`, `WebKitPlaybackTarget`, `webkitShowPlaybackTargetPicker`, `remote.prompt`, `RemotePlayback`, dlna, upnp, ssdp, AVTransport, Apple TV, TV, mixed content, and "service worker" next to video. There is no measurement of:
- LAN serving to a TV;
- Range requests from TVs;
- Apple TV or Chromecast behaviour;
- AirPlay or Cast from a `<video>` whose source is a service worker, a blob, or MSE;
- mixed content.

### (c) Gaps: what U6 needs that nothing provides

1. **No LAN listener.**
   - Every svrngs bind is loopback (`api.rs:92-102`, `guest.rs:361-376`), and the door serves only iroh streams (`door.rs:453-458`).
   - The fork candidate is the lifetime-bound bind at `guest_door.rs:434-487`, without its proxy.
2. **No unguessable or one-time stream URL, and no "serves only the device it was handed to" binding.**
   - The nearest shape is `GuestGrantStore` (bearer plus TTL, in memory).
   - `e0-bot`'s rule is prose only: by receiver address for Cast/DLNA, and for AirPlay "the first device other than the phone to fetch it" (`e0-bot DESIGN.md:560-561`).
   - General knowledge, unverified here: neither the Cast Web Sender nor Safari's AirPlay tells a page the receiver's IP, so the first-fetcher rule would cover both. Whether Safari fetches the URL on the phone before handing off is unknown.
3. **No route for a phone to tell a node "serve this to a TV".** `e0-bot`'s route (`:528-529`) admits only the owner's own devices, which excludes Demo B's friends at Sam's (DESIGN:613-616).
4. **Proxying another's lend is forbidden today.**
   - Call 4 says "No node proxies another's lend" (DESIGN:132-135), yet Demo B's shelf spans "three members' libraries" (DESIGN:615). So the meeting point serving a lend that lives on Dee's computer contradicts call 4.
   - Through the door, the TV stream would carry the meeting point's key and person (`door.rs:473-476`, `:585-603`), not the phone person's.
   - When the meeting point sits in the lender's home, `guess` marks it Home, so it is uncounted and told no share (`limit.rs:49-63`).
5. **A node cannot reach its own lend through the door.** The door refuses anything off an iroh stream (`door.rs:453-458`). I found no self-dial path in `iroh.rs` or `node.rs`.
6. **Nothing tells a relay-only page which computer shares its WiFi.**
   - A browser has no direct addresses (`iroh endpoint.rs:1298`), so a node sees the phone only as relayed (`iroh.rs:362-385`).
   - `e0-bot` solved this with a native direct dial (`:552-555`).
   - Local discovery waits in §9 (DESIGN:657).
7. **No LAN address picker in svrngs.** `endpoint().addr().ip_addrs()` holds every interface address. Upstream's `advertised_base` and `relay_candidates` show the needed filtering (LAN before tailscale, never a wildcard), but they need `if-addrs`.
8. **Mixed content is unmeasured.**
   - The page is https: it needs a secure context (`RING_APP_LIBRARY.md:565-569`), and U4's service worker holds the endpoint (DESIGN:127-131). The TV URL would be `http://<LAN>`.
   - General knowledge, unverified here: Chrome auto-upgrades mixed audio/video to https and fails when that is not served. WebKit's handling of an http `<video src>` on an https page varies by version.
   - Also general knowledge, unverified: a Cast sender only passes a URL string, which the receiver fetches itself; whether the Default Media Receiver plays plain-http LAN media is unverified (shelf:191; `e0-bot` `:1026-1027`).
9. **The TV must be handed the node's URL, not the page's.** A `<video>` served by a service worker uses a page-origin URL. General knowledge, unverified here: a TV fetching that URL reaches the origin server, not the phone's worker, and AirPlay/Remote Playback cannot hand off blob or MSE sources.
10. **No remote-control channel, and none for "however many phones hold the remote"** (DESIGN:405).
    - The live lane skips browser keys (call 2, DESIGN:122-126) and carries at most 4096 bytes (DESIGN:505). WebSockets do not cross the door (dry run `:69-73`).
    - General knowledge, unverified: AirPlay is controlled only from the sending page's `<video>`. Cast control goes through the Web Sender's RemotePlayerController, and other senders can join only if they share the origin.
11. **Phone lock behaviour is unmeasured.** `e0-bot`'s lock claims assumed a native app with a background mode (`:555-559`). General knowledge, unverified: once the TV fetches from the node, playback no longer depends on the phone.
12. **Format and CORS.**
    - `e0-bot` wanted HLS for AirPlay/Cast and CORS for the Cast receiver (`:547`, `:567-569`); the door adds no CORS headers.
    - Chromecast does not list MKV (shelf:193-194).
    - Infra code may not use the words transcode, subtitle, film or jellyfin (`apps/films/nouns.txt`; DESIGN:215-218).
13. **"Cast from Chrome" probably means Android or desktop Chrome.** General knowledge, unverified: Chrome on iOS is WebKit and has no Cast Web Sender.
14. **No Range-on-TV data, and macOS Local Network permission on the meeting point is unmeasured** (`e0-bot` risk 12, `:1009-1010`).
15. **DLNA has no code** (shelf names `rupnp` 3.0.0, `:191`; it waits in §9, DESIGN:645).

### (d) Recorded decisions that bind U6

**svrngs DESIGN @ HEAD**
- `:97-98`: "the screen handoff: a node on a TV's network serves one stream at an unguessable local address the TV fetches, while a phone is the remote." `:100`: "An app that needs anything else is a decision, not a convenience."
- `:366-370`: "an always-on one is the group's meeting point and the origin a TV fetches from, its port mapped by iroh"
- `:405`: "*The same room* — one stream to the TV, however many phones hold the remote."
- `:611`: the U6 row, as quoted in the brief. `:613-618`: Demo B, with "browses one shelf across three members' libraries".
- `:645`: "| DLNA, and TVs that take no push | people whose TV takes neither AirPlay nor Cast ask, counted |". `:657`: "local discovery on one WiFi | Apple's multicast entitlement is granted".
- `:697-698` (operator, 2026-10-03): "the screen handoff is infra; phones first, through the browser".
- `:122-135` (seat, 2026-10-05), calls 2, 3 and 4: a browser's key is never dialled and the live fan-out skips it; the service worker holds the endpoint; "No node proxies another's lend."
- `:33-37`: the words rule (never key, node or relay in anything a person reads). `:215-218`: the nouns gate.
- `:222-229`: the repository ceiling is 14,000 lines, raised "for U4's page and its wasm and U6's handoff, about 2,100 lines between them by the seat's estimate."
- `:388-389`: "Sam's own home, on the local network, is never counted against it." `:513-516`: "The local network is guessed".
- `:707-709`: "Members lend their own servers to their own group; svrngs hosts nothing, and its relays carry only ciphertext."
- `:733-734`: "no unit builds a way into Jellyfin's own player."
- `:519-520`: Jellyfin only "at arm's length, over its API".

**Why the earlier TV path was cut**
- DESIGN:52-55: "We designed an app SDK, a wasm sandbox, signed manifests, TV casting and group compute in detail before one group had used anything, and the first change of direction made most of it moot."
- DESIGN:6-10: "the TV path … is this file at tag `e0-bot` … Build from it only what a §8 unit names."
- Commit d239848 parked it in §9 as "a TV in the room (AirPlay, Cast, DLNA from the member door) | Films passes its bars" (`d239848:docs/DESIGN.md:234`).
- Commit 8e4a984 (operator, 2026-10-03) brought it back as U6 ("The screen handoff is infra"), with only DLNA and TVs that take no push left waiting.

**What `e0-bot` decided** (`docs/DESIGN.md`)
- `:543-550`: every protocol "ends with the TV fetching an HTTP URL itself. That URL is the lingua franca, and the platform makes exactly one … at a path no one can guess, over plain http on its real address (never loopback, never through a resource loader), with CORS for Cast's receiver, until playback stops; behind it is the member door's tunnel to the lender. The phone is always the sender — the AirPlay route, the Cast session, the DLNA control point — and the remote."
- `:560-562`: the address-binding rule quoted in gap 2, and "A route change mints a new URL."
- `:1026-1027`: "Whether third-party AirPlay 2 TVs fetch the URL themselves, and whether Cast plays plain-http HLS, are unmeasured."
- `:1098`: "the TV path converged".
- Its LAN guest door was cut (`:871`, `:1088`), and its avoid list included "browser keys" (`:860`), which the current design reverses.

**commonwealth-ai**
- `docs/THE_LINK.md:85-90` and `:241-242` refuse a proxy that terminates plaintext.
- `THE_LINK.md:217-222`: "The LAN door is plain HTTP by design … Secure-context needs … belong to the https gateway tier, never to the LAN door".
- `RING_APP_LIBRARY.md:618-619`: "The LAN door stays as the no-internet path: plain HTTP, guests only, the host signs."
- `RING_APP_LIBRARY.md:623-624`: "a phone off the WiFi cannot reach a LAN bind at all".
