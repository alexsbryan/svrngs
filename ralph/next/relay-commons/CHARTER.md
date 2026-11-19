# Our own relay — for connecting, not for carrying

## Direction and authorization

The operator, 2026-11-18: "the time has come for us to finally host our own
relays", with due diligence on scale and cost "ready for a viral moment".
The design was decided in chat that day and written in svrn-docs
`rings/current/RELAY_COMMONS.md` (at 9a37fa3). Its decisions are in PLAN §3,
2026-11-18. Then: "Let's start a branch off main and ralph this Plan up".
That is this queue's launch directive for the design, its six units of work
and this charter's envelope. Starting the loop is the operator's.

The design, in short; RELAY_COMMONS is normative:
1. **One relay of ours, n0's public relays behind it**, used only while ours
   does not answer. No n0 Pro. One name, `DEFAULT_RELAY`.
2. **The relay is for connecting.** Each computer gets a token bucket at the
   relay, ~1 Mbit/s steady and an 8 MB burst (iroh-relay `ClientRateLimit`).
   The numbers come from measurement: an AI answer at 0.057 Mbit/s, and
   Jellyfin's first open at 2.11 MB brotli, 5.66 MB decoded.
3. **When the share is spent, a code and words, never a list of apps.**
   - The signal: iroh 1.4's `Status::RateLimited`, counted in
     `relay_conns_ratelimited`.
   - The door answers a large request over our relay with 429 and a new
     `LendRefusal::Share`, and the page says why.
   - svrngs reconnects to clear it, after a minute with nothing large refused.
4. **At home a phone goes direct**: WebRTC as an iroh custom-transport path,
   the relay kept as backup.
5. **A group's own relay** comes first, and need not be left open.

The demo this moves: a phone opens a group's link through our relay, sees
the lends and chats at once, and plays a film at full speed on the lender's
Wi-Fi. Away from it, the phone is told plainly why the film waits. A
computer meets its groups at our relay, at n0's while ours is down, and at
ours again after.

## Budget

Headroom at `d59c8246` (`scripts/lean_check.py`):
- core: 3,499 of 3,539;
- all code but JavaScript: 21,474 of 23,101;
- JavaScript: 8,544 of 8,651.

Each row's allowance is its own ceiling, not a target. The queue's
estimate is about +560 Rust, of which +30 is core, and +100 JS. That fits
everything but JavaScript, which is tight.

- **The launch approves the work envelope, not a cap raise.** Economize
  first. Preserving cuts get their own commits and ratchet the caps down.
- **If measured necessary growth does not fit,** stop with the post-pass
  counts and the smallest ready cap diff (`ralph-result needs-human`). Do
  not edit BUDGET or the trust floor.
- **Tests:** at most 900 added code lines across the queue, on the existing
  harnesses.
- **Product docs:** DESIGN is rewritten in place in each behavior commit, at
  most 60 added lines in all.

## Reuse and feasibility

- **The spikes are the inventory.** svrn-docs `rings/current/webrtc-spike/`
  and `webrtc-transport-spike/` hold ours, Apache/MIT, and are ported, not
  rewritten:
  - `carrier/src/lib.rs`, the iroh custom transport;
  - the str0m loop, `native/src/main.rs`, its first tick given at once;
  - `page/src/lib.rs`, the two endpoints and a free `data_push`;
  - `web/sw.js` with `web/worker.html`, the worker relayed through the
    window.
- **What the spikes learned, which binds:**
  - iroh opens a custom address only before a peer has a selected path.
    So the lend's fresh endpoint dials WebRTC alone, then again with the
    relay. No iroh patch: n0's AI policy and #4602.
  - Paths are given up after 3 s with a 1 s keep-alive, on both ends.
    `network_change()` does not help.
  - wasm-bindgen refuses calls on an object while its async method holds it.
  - Host ICE candidates only, no STUN or TURN. A phone reaches its lender on
    the same Wi-Fi, and nothing past it: the trust floor's "this WiFi".
  - The lend shim captures every WebSocket a lent page opens. iroh lives in
    svrngs's own page and worker, never in a lent page.
- **New dependencies, authorized here and nothing else:**
  - iroh, iroh-relay and iroh-base 1.4.x, with the iroh-blobs that matches;
  - str0m 0.25 (rust-crypto). It resolves with iroh 1.4.1:
    `curve25519-dalek` 5.0.0.
  - Any further dependency is a stop.
- **Read before building:**
  - core `relay::Meeting`;
  - node `tiers.rs`, `door.rs` (`LendRefusal`) and `invite.rs`;
  - `crates/svrngs/src/relay.rs` and `cli.rs` (`SHIPPED`, `DEFAULT_RELAY`);
  - `web/src/lib.rs`, `web/lend.js` (refusals, `retry-after`) and
    `web/shim.js`;
  - the relay_tiers, join_page and browser harnesses.
- **One decider each:**
  - The relay's numbers live in its configuration alone.
  - The door's "large" line is the door's own.
  - n0's relays are named once, as the build names them.

## Boundaries and verification

- **The worker's scope is this worktree and synthetic identities.**
  - Not in scope: the operator's running svrngs and its data, other
    worktrees, `cw-jellyfin`, and the live llama-server.
  - Never: deployment, DNS, a purchase, push, main merge or release.
  - `deploy-relay.sh` is written and dry-run only. The box and its run are
    `HUMAN-` rows.
- **Production CSP and the trust floor stay unchanged.** WebRTC needs
  neither changed; a row that finds otherwise stops.
- **Every new check is planted and watched failing first** (AGENTS.md 5).
- **Focused checks close intermediate units, and no `check.sh` runs before
  the very end**, neither full nor `--touched` (operator, 2026-11-18: "I don't
  want us to run interminable check.sh runs -- let's just save that for the
  very end"). The commit hook's `--fast` is the only exception. The final
  audit runs the unchanged full `scripts/check.sh` in the `svrngs-gate`
  container (`scripts/gate/README.md`).
- **Real-phone and real-box results stay never-ran** until the operator runs
  them. The loop alone marks STATE rows.
