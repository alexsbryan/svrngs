# The phone's door — a spike, designed before anything is built

Written 2026-10-26, before any code. The operator: "let's design the spike
and spec out the cost", then "let's write the full design. I'll get the
developer account settled before we start". Its case is svrn-docs
`rings/current/WITHOUT_BEING_WATCHED.md` §8: a phone app's one job is to be
a network endpoint, carrying the phone's own apps to a group's lends, from
anywhere, with the app closed. Code references are svrngs at `652c3d6a`.

The bars below are proposals. The operator sets them before the first
session, and `PREDICTIONS.md` is committed beside this file, with what is
expected of each, before the first session starts. `README.md` says what
each met, after.

## 1. The question

Can an iroh endpoint, inside an iOS packet tunnel and under its memory
limit, carry a phone app's TCP connections to a friend's lend, and keep
carrying them with the app closed and the phone locked? The same question
goes to Android's `VpnService`, which is looser.

It is a question no one seems to have answered in public. No app found in
svrn-docs `rings/current/PHONE_REPLICA.md` §7, nor in a search on
2026-10-26, runs iroh in a Network Extension. Delta Chat ships iroh in an
App Store app, but in the app, not a tunnel. The packet tunnel's limit is
about 50 MiB, measured by developers since iOS 15, and Apple's engineer
says it may be lower on older devices (Apple Developer Forums, thread
73148). The full svrngs node on the operator's Mac has a physical footprint
of 20 MB (`footprint` of the running node, 2026-10-26), with every group,
journal and door, but no TCP/IP stack and no video in flight.

PLAN §2's row "a native phone app" holds a different spike: an endpoint in
a network extension that answers a computer's dial for 24 hours. This one
dials out. They share the first half, an iroh endpoint alive in a tunnel
under its cap, so what this one measures informs that row. Neither replaces
the other.

## 2. What it tests, and what it leaves out

It tests:
- iroh in a packet tunnel at all;
- memory against the limit, while a film plays;
- throughput, and whether each path is direct or relayed;
- the background: the app closed, the phone locked;
- refusal of a key that stands in no group.

It leaves out the person's headers, names for lends, joining from the app,
the app's screens, App Review and battery. Battery is noted, not judged.

To leave them out, the lends are **raw TCP** (give-tcp; `svrngs share
<port> --tcp`), not pages. Jellyfin and Navidrome sign the phone in with
their own accounts, so nothing here depends on who svrngs says the asker
is. The door's page path, which gives each request the person's headers,
is production work after a pass.

## 3. Setup

**The lender.** The operator's Mac, running svrngs at the run's commit, in
a group of the run's own:
- Jellyfin: `svrngs share 8096 --tcp --kind jellyfin`
- Navidrome, for the locked phone: `svrngs share 4533 --tcp --kind navidrome`

**The tunnel's key.** On the Mac, `svrngs link <group> --device` makes a
link that joins as the Mac's person (DESIGN §3, "Joining"). A second
svrngs with a data directory of the run's own redeems it:
`SVRNGS_DIR=<run>/data/tunnel-key svrngs join <link> --no-run`. Its
`identity` file is a 32-byte ed25519 seed (`crates/svrngs/src/identity.rs:41`),
which the spike's build embeds. It is a spike's key: never shipped, and
removed from the group after the run. The lender's door admits it, since a
raw lend is opened only by a key standing in a group the lender holds
(`door/udp.rs:313` `opening`, `door.rs:1225`). It stands as a computer of
the operator's person, which is what the door asks today; what a phone app
stands as is a production question (§9).

**The tunnel's network.** Address 10.77.0.2/24, MTU 1280, and only
10.77.0.0/24 routed into it, so nothing else on the phone goes through.
A table built in: 10.77.0.1:8096 is the Jellyfin lend, 10.77.0.1:4533 the
Navidrome lend. In Jellyfin's app the server is `http://10.77.0.1:8096`.

**Devices.** An iPhone on iOS 16 or later, and an older iPhone if one is at
hand, since the limit may be lower there. Any Android phone on Android 8
or later. Each one's model and OS version go in the record.

## 4. What is built

Everything lives under this directory. A recorded run's driver is outside
the size budget (`scripts/lean_check.py:15`), and no check runs it.

### 4.1 `tunnel/`, the core, in Rust

A static library in a crate of its own, outside the workspace, as `web/`
is. It depends on:
- `iroh = "=1.0.2"`, the lock's pin (`crates/svrngs-node/Cargo.toml:25`);
- `svrngs-core` by path, for `Alpn::Port` (`b"svrngs/port/0"`,
  `crates/svrngs-core/src/alpn.rs:48`). The core is pure and already builds
  for wasm;
- tokio, with a current-thread runtime first, since that holds less;
- `tun-rs`, for the tunnel's file descriptor, as its iOS guide describes;
- a userspace TCP stack: `ipstack`, else `smoltcp`, chosen by memory on the
  Mac harness (§4.4).

It holds:
1. **The endpoint.** Built from the embedded seed, with n0's relays and
   address lookup, as `guest.rs` builds one from `presets::N0`
   (`crates/svrngs-node/src/guest.rs:43-45`). The lender is reached by its
   key, from n0's lookup.
2. **The member's half of a TCP lend, copied, not linked.** From
   `door/udp.rs:313` (`opening`) and `door/tcp.rs:105` (`carry`): dial with
   `Alpn::Port`, write `tcp <lend>\n`, read `ok`; then each connection a
   bi-stream of its own, opened with a newline, answered `ok` or the
   lender's words, and copied both ways until both sides close. It is
   copied because `svrngs-node` pulls in `svrngs-runtime` and `notify`,
   which need not build for iOS, and making them would turn the spike into
   a port of the node. A production app links the node's half, or the
   half moves to a crate both use; that is §9's question, not this one's.
3. **The TCP stack's loop.** Each connection to 10.77.0.1:<port> is
   accepted, matched to its lend by the table, and handed to (2).
4. **The sampler.** Every 10 seconds, one JSON line to a file in the app
   group's container:
   `{t, footprint, available, up, down, open, path, plant}`. `footprint`
   is `phys_footprint` from `task_info(TASK_VM_INFO)`, the number the
   system's memory killer counts; `available` is
   `os_proc_available_memory()`; `path` is `direct`, `relay` or `none`, as
   iroh reports the lend's connection.
5. **Every branch traced at debug,** under target `tunnel`: a connection
   accepted, a lend opened or refused, a path changed, a stream closed.
6. **Two plants, by a build setting `PLANT`:**
   - `stranger`: dial with a fresh key that stands in no group;
   - `leak`: allocate and touch 1 MiB a second until the system kills the
     extension.
7. **Two entry points over a C ABI:** `tunnel_start(fd, seed, log_dir)`
   and `tunnel_stop()`.

### 4.2 iOS

An Xcode project with two targets:
- **The app:** one switch. It saves, starts and stops a
  `NETunnelProviderManager`, and shares the sampler's file out.
- **The packet tunnel extension:** it applies §3's settings, finds the
  utun descriptor as `tun-rs`'s iOS guide does (the older lookup can return
  nil on iOS 16 and later), and calls `tunnel_start`.

The Rust is built for `aarch64-apple-ios`, already installed here, into an
XCFramework by a script in this directory. Signing uses the operator's paid
developer account, on their own device. A free personal team cannot use
Network Extension at all (PHONE_REPLICA, source f-746373).

### 4.3 Android

A `VpnService`, whose `establish()` returns the descriptor directly, and an
activity with one switch, calling the same core over JNI, built with
`cargo-ndk`. Installed by hand; no store.

### 4.4 The Mac harness

The same core on a utun on the Mac, with `curl` and a browser at
10.77.0.1. Creating a utun needs root once, which the operator gives;
otherwise the core's tests drive the TCP stack from an in-memory pair of
packet queues. The loop develops and chooses the TCP stack here, by
`footprint` while a film plays, before any phone.

## 5. The sessions, in order

Each one starts with the sampler running and the phone's screen
recording on, and ends by saving the sampler's file, the recording, and
the lender's debug log for targets `door` and `play`.

| # | Session | Bars |
|---|---|---|
| S0 | **Preflight.** The lender's lends answer from the Mac (`curl` through the harness). The phone's model, OS and battery are noted. The tunnel starts and the sampler writes. | none |
| S1 | **A film on WiFi.** A 1080p film in Jellyfin's app at `http://10.77.0.1:8096`, 30 minutes, the phone on the lender's WiFi. | P1, P2, P6 |
| S2 | **A film on cellular.** The same, WiFi off. | P1, P2, P6 |
| S3 | **Music, locked.** Amperfy (open source) against `http://10.77.0.1:4533`, the app closed and the phone locked, 20 minutes. | P2, P3 |
| S4 | **The stranger.** A build with `PLANT=stranger`; Jellyfin's app tries to connect. | P4 |
| S5 | **The leak.** A build with `PLANT=leak`, last, since it kills the extension. The JetsamEvent report is taken from Settings, Privacy & Security, Analytics Data. | P5 |
| A1–A4 | S1 to S4 on Android. | A1–A4 |

A stall is counted from the screen recording, by hand, each with its time:
a stretch longer than 10 seconds where the picture or sound stops.

## 6. The bars, proposed

Each is one of four verdicts: passed, failed, could-not-judge, never-ran.

- **P1 — plays.** The film plays to its 30th minute with no error and no
  stall over 10 seconds, on WiFi (S1) and on cellular (S2), each judged
  apart. The bitrate is recorded.
- **P2 — fits.** The sampler's largest `footprint` across S1 to S3 is under
  40 MiB, and no JetsamEvent names the extension. 40 rather than 50,
  because older devices may allow less. Could-not-judge unless P5 passed.
- **P3 — the background.** The music plays through all 20 minutes of S3
  with the phone locked.
- **P4 — refused.** The stranger's opening is refused as not admitted
  (`door/udp.rs`, `closed_unadmitted`), traced on the lender's side and the
  tunnel's.
- **P5 — the instrument.** The leak's extension is killed; its last sample
  is within 10 MiB of where the system stopped it; a JetsamEvent report
  names it. This is the plant that shows P2's instrument sees a kill.
- **P6 — the path.** Not judged: direct or relayed, per session, since it
  explains P1.
- **A1 to A4.** P1 to P4 on Android. Android's limits are looser, so P5
  does not apply, and A2 is footprint recorded, not judged.

## 7. What each outcome decides

- **All pass:** production is designed next. The door's page path with the
  person's headers, joining in the app, the app's three screens (On, your
  groups, What acts as you), and the organization Apple's guideline 5.4
  asks for.
- **P2 fails:** before it counts as failed, the loop tries the current-thread
  runtime, smaller receive windows and buffers, and the other TCP stack,
  and the sessions run once more. If it still fails, Play on an iPhone runs
  only while the app is open, and the page in the browser stays the door.
- **P1 fails only on cellular, relayed:** that is the relay's throughput,
  not the tunnel. It goes to GROUP_RELAYS as a finding, and this path
  stands.
- **iroh does not start in the tunnel at all:** the spike stops there, with
  what failed and why, and PLAN §2's replica row learns it too.

## 8. Cost

**Money: $99.** The Apple Developer Program, a year. A paid individual
membership is expected to run a Network Extension on the member's own
device; that is checked at enrollment, not here. The organization guideline
5.4 asks for is for the App Store, not this spike. Android costs nothing,
since it is installed by hand. Devices are the operator's. Cellular data:
1 to 2 GB for S2, since 30 minutes at 8 Mbit/s is about 1.8 GB.

**The loop's work, guessed.** Three or four rows:
1. `tunnel/` and the Mac harness, with the TCP stack chosen;
2. the iOS project and its build script;
3. the Android project;
4. `PREDICTIONS.md` and the session sheet, once the operator sets the bars.

About 600 to 900 lines, all outside the budget. One to three more rows are
likely for the extension itself, whose logs come only through Console.app
and which is awkward to attach a debugger to. This is where the estimate
is most likely to run over.

**The operator's hands,** which no worker can stand in for: enrolling,
about half an hour, then hours to two days for Apple's approval; signing
and provisioning the app and its extension in Xcode and the first install,
one to two hours; the sessions, about three hours with Android. Five to
seven hours over two or three days.

**Time:** about a week from the account being settled. The Android half can
start before it.

## 9. Not decided here

- What a phone app stands as in a group. The tunnel's key stands as a
  computer of its person, because that is what the door admits to a raw
  lend today; a production app may be a computer in svrngs's terms, since
  it would redeem a link as a computer does (DESIGN §3, "Joining"), or a
  device kind of its own.
- Where the member's half of a lend lives, so that the node and a phone
  app share one copy (AGENTS.md 8, one decider).
- Names for lends on the phone, in place of 10.77.0.1 and a port.
- Whether the bars above are the operator's.
