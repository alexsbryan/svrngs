# The port map: spike file → svrngs owner

relay-survey (c), 2026-11-18. The two spikes are ours (svrn-docs
`rings/current/webrtc-spike/` and `webrtc-transport-spike/`, Apache/MIT);
the queue ports them, it does not rewrite them (CHARTER, "Reuse"). This is
the mapping each porting row starts from, read at the spikes' READMEs and
sources on 2026-11-18. The findings that bind are CHARTER's, "What the
spikes learned"; each port's commit cites the spike file it came from.

| Spike file | What it is | svrngs owner (STATE row) |
|---|---|---|
| `webrtc-transport-spike/carrier/src/lib.rs` (136 lines) | the iroh custom transport: `Carrier`, `Bound`, `Sender`, `addr_of`, `TRANSPORT_ID`, `Send`; datagrams to and from a peer key, and nothing of iroh or WebRTC in it | `webrtc-node` and `webrtc-page`: one source both halves compile — the node's half as a module of `svrngs-node`, the page's half through `web/` as `web/src/lib.rs` already compiles `crates/svrngs-node/src/relay.rs` by `#[path]`. The node adds it with `Endpoint::builder(..).add_custom_transport` (spike `native/src/main.rs:73`) |
| `webrtc-transport-spike/native/src/main.rs` (340) | the computer's WebRTC half: `answer` (offer → answer), the str0m `Rtc` loop `run`, `say_paths`, `lan_address` | `webrtc-node`: svrngs-node's new WebRTC module. `answer` becomes the offer/answer over the new protocol on the group's existing dial; `run` becomes the node's task, host candidates only |
| `webrtc-transport-spike/page/src/lib.rs` (442) | the page's half: `signal_start`/`Signal` (the signalling endpoint), `data_start`/`Data` and the free `data_push`, `quick_paths` (3 s idle, 1 s keep-alive), `ask` (the kind-tagged stream ask) | `webrtc-page`: `web/src/lib.rs` (the group page's endpoint and the data endpoint's dials) and the carrier. `quick_paths` is the path config the `webrtc-node` row brings — 3 s paths, 1 s keep-alive — which `dial::held`'s connection idle (4 s, 1 s keep-alive, `crates/svrngs-node/src/dial.rs:59`) is not |
| `webrtc-transport-spike/web/sw.js` (48) | the data endpoint in a module service worker; datagrams to and from the window over a `MessageChannel`, transferred | `webrtc-page`: `web/lend.js` (the lend worker's endpoint) |
| `webrtc-transport-spike/web/worker.html` (86) | the window holds the `RTCPeerConnection` and the unordered, unreliable channel, and passes datagrams between it and the worker | `webrtc-page`: the group page's window half (the page that holds the dial, `web/page/`), and the lend window beside the worker |
| `webrtc-transport-spike/web/index.html` (102) | the spike's driver page | the harness: `crates/svrngs-node/tests/browser/` (its launcher `web/tests/browsers.mjs`), where the queue's Chromium and WebKit checks run |
| `webrtc-spike/server/src/main.rs` (385) | spike 1's str0m loop and film server, before iroh carried it: `answer`, `run`, `public_address`, `alter_fingerprint` (the A1 plant), `candidate_kinds` | `webrtc-node` (the same str0m loop as `native/src/main.rs`; the plants name the checks) |
| `webrtc-spike/page/index.html` (139) | spike 1's page: `RTCPeerConnection`, a channel, 2 MiB parts of a film | `webrtc-page`: the channel and the part fetching; svrngs's channel is unordered and unreliable (`worker.html`'s `{ordered: false, maxRetransmits: 0}`), not spike 1's ordered one |
| `webrtc-spike/page/sw.js` (24) | spike 1's worker answering each `/film/<a>-<b>` fetch from the window over a `MessageChannel` | `webrtc-page`: `web/lend.js` |
| `webrtc-transport-spike/patches/iroh-custom-open.patch` (20) | the iroh change that opens a learned custom address on live connections | **not ported**: CHARTER, "Do not send iroh's patch upstream", and the two dials are enough (spike README, "The iroh change") |
| `webrtc-transport-spike/run.mjs`, `webrtc-spike/run.mjs` | the Playwright drivers, plants included | `crates/svrngs-node/tests/browser/` and `web/tests/page.mjs` (the queue's browser checks) |
| `webrtc-transport-spike/build-page.sh`, `build-page-patched.sh`, `page-patched/` | the wasm builds, patched and stock | `scripts/build-web.sh` (the one build; no patched build lands) |

What each port keeps from the spikes' findings (CHARTER, "What the spikes
learned"): a fresh data endpoint dials WebRTC alone, then again with the
relay; paths are given up after 3 s with a 1 s keep-alive; host ICE
candidates only; datagrams go in through a free function, never a
wasm-bindgen method (`page/src/lib.rs:210-235`); iroh never lives in a
lent page (the shim takes over `window.WebSocket`, spike README, "The
iPhone run").
