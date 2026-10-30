# HUMAN-ios-observe: predictions, before the operator's checks

`ralph/next/ios/STATE.md`, row HUMAN-ios-observe. The operator, 2026-10-30:
"Ok I'm ready, let's go".

**The build on the phone:** the spike's tunnel at the ladder's framework
(netwatch's trim fixed, iroh's reports by step; built 2026-10-28 19:52),
installed again with `key.sh`'s probe unset. So the tunnel opens its lend:
Jellyfin at `127.0.0.1:8096`, lent from the scratch lender on this Mac
(`b8109133…`). The step is "periodic", the closest to what would ship.

| Check | Expected | Chance | Why |
|---|---|---|---|
| 1. The cold start: from switching the tunnel on to Jellyfin's first frame | 3 to 6 s, so over Tom's 3 s | 65% | The endpoint binds and reaches a relay, the door dials the lender within 4 s (`dial.rs`), then Jellyfin's app signs in and buffers. None of it was timed in the spike. |
| 2. The handoff: music through the door while walking off WiFi | drops at least once | 60% | The door's dial gives up after 4 s of silence (`dial.rs` `held()`), and one of the spike's three early drops came as the phone left WiFi (phone-tunnel README). |
| 3. Safari locked: music from the group's page, the phone locked | stops within a minute of its buffer running out | 70% | iOS suspends a page's service worker in the background, and the lend's bytes come through it (`web/lend.js`). |
| 4a. The upload scan | accepted, with no refusal naming SystemConfiguration | 75% | The 26 functions link against the iOS SDK's own `SystemConfiguration.tbd`, so they're symbols iOS ships, not private ones; `API_UNAVAILABLE(ios)` is a header mark. |
| 4b. TestFlight internal: installed on the operator's phone from an individual's team | installs and the tunnel runs | 80% | Internal testing has no review, and the packet tunnel entitlement is granted to this team already. |
| 5. The house's chat | not predicted | — | People's answers, counted as they come. |

**What each decides** (STATE.md):
1. Over 3 s, the endpoint is kept warm, not started on use.
2. A drop gives a phone its own silence limit.
3. If music keeps playing, the pocket is no reason for the app.
4. A refusal adds `ios-sc`, and an install shows the demo needs no App
   Review.
5. If several people run a VPN, on-demand by domain comes into the queue.
