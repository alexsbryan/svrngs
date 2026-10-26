# The phone's door — a spike, designed before anything is built

Written 2026-10-26, before any code. The operator: "let's design the spike
and spec out the cost", then "let's write the full design. I'll get the
developer account settled before we start", then "Can we do it even more
efficiently?". Its case is svrn-docs `rings/current/WITHOUT_BEING_WATCHED.md`
§8: a phone app's one job is to be a network endpoint, carrying the phone's
own apps to a group's lends, from anywhere, with the app closed. Code
references are svrngs at `9ab61bf5`.

The bars below are proposals. The operator sets them, and `PREDICTIONS.md`
is committed beside this file, with what is expected of each, before the
first session starts. `README.md` says what each met, after.

The first version of this design (`e2b1a158`) carried the phone's packets
through a userspace TCP stack, copied the member's half of a lend, and took
Android and a Mac harness along. This one cuts all four (§2). That version
stays the fallback if §5's first session fails.

## 1. The question

Can an iroh endpoint, inside an iOS packet tunnel extension and under its
memory limit, carry a phone app's TCP connections to a friend's lend, and
keep carrying them with the app closed and the phone locked?

No app found in svrn-docs `rings/current/PHONE_REPLICA.md` §7, nor in a
search on 2026-10-26, runs iroh in a Network Extension. Delta Chat ships
iroh in an App Store app, but in the app, not a tunnel. The packet tunnel's
limit is about 50 MiB, measured by developers since iOS 15, and Apple's
engineer says it may be lower on older devices (Apple Developer Forums,
thread 73148). The full svrngs node on the operator's Mac has a physical
footprint of 20 MB (`footprint` of the running node, 2026-10-26), with
every group, journal and door, and no video in flight.

**Already answered, on 2026-10-26: it builds.** The whole node checks for
iOS once libc is held at 0.2.189:

```
git archive HEAD | tar -x -C <scratch>; cd <scratch>
cargo update -p libc --precise 0.2.189
cargo check -p svrngs-node --target aarch64-apple-ios   # Finished
```

At the workspace's lock (libc 0.2.190) it fails: libc 0.2.190 made
`in6_ifreq` macOS-only, and netdev, which iroh pulls in through netwatch,
still uses it on every Apple target (`netdev-0.46.3/src/os/darwin/ipv6_addr_flags.rs:19`).
iroh 1.0.2 alone fails and passes the same way. The spike's crate keeps a
lock of its own, so the workspace's is untouched.

PLAN §2's row "a native phone app" holds a different spike: an endpoint in
a network extension that answers a computer's dial for 24 hours. This one
dials out. They share the first half, an endpoint alive in a tunnel under
its cap, so what this one measures informs that row.

## 2. What it tests, and what it leaves out

It tests:
- iroh in a packet tunnel at all;
- memory against the limit, while a film plays;
- throughput, and whether each path is direct or relayed;
- the background: the app closed, the phone locked;
- refusal of a key that stands in no group.

It leaves out the person's headers, names for lends, joining from the app,
the app's screens, App Review and battery. Battery is noted, not judged.
The lend is **raw TCP** (give-tcp; `svrngs share <port> --tcp`), not a
page, and Jellyfin signs the phone in with its own account, so nothing
here depends on who svrngs says the asker is.

What the first version carried, and this one doesn't:

- **No packets.** The extension listens on `127.0.0.1:8096`, as Play does
  on a computer (`crates/svrngs-node/src/play.rs:236`), and Jellyfin's app
  connects there. The tunnel routes nothing; it is up only so the system
  keeps the extension running. No `tun-rs`, no TCP stack, no address table.
  iOS proxy apps are known to serve a local proxy from their tunnel
  extension this way; that other apps reach it here is S0's first check,
  not assumed.
- **No copy.** The extension links `svrngs-node` and calls its own member's
  half, `door::tcp::carried` (`crates/svrngs-node/src/door/tcp.rs:76`),
  with an endpoint from `build_relayed_endpoint` (`crates/svrngs-node/src/iroh.rs:51`).
  What it measures is the node's code, not a copy of it, and §9 loses a
  question.
- **No Android.** `VpnService` has no cap like this one, and the question
  is the iPhone's. Android is production work after a pass.
- **No Mac harness.** On a Mac, Play is this exact path, and it is in use.

## 3. Setup

**The lender.** The operator's Mac, running svrngs at the run's commit, in
a group of the run's own: `svrngs share 8096 --tcp --kind jellyfin`, with
music in the same Jellyfin library for the locked phone.

**The tunnel's key.** On the Mac, `svrngs link <group> --device` makes a
link that joins as the Mac's person (DESIGN §3, "Joining"). A second
svrngs with a data directory of the run's own redeems it:
`SVRNGS_DIR=<run>/data/tunnel-key svrngs join <link> --no-run`. Its
`identity` file is a 32-byte ed25519 seed (`crates/svrngs/src/identity.rs:41`),
which the spike's build embeds, with the lender's key. It is a spike's
key: never shipped, and removed from the group after the run. The
lender's door admits it, since a raw lend is opened only by a key standing
in a group the lender holds (`door/udp.rs` `opening`, `door.rs:1225`).

**Devices.** An iPhone on iOS 16 or later, and an older iPhone if one is at
hand, since the limit may be lower there. Each one's model and OS version
go in the record.

## 4. What is built

Everything lives under this directory. A recorded run's driver is outside
the size budget (`scripts/lean_check.py:15`), and no check runs it.

**`tunnel/`, in Rust.** A static library in a crate of its own, outside
the workspace, as `web/` is, depending on `svrngs-node` by path, with its
own lock holding libc at 0.2.189. It holds:
1. the endpoint, from the embedded seed, with n0's relays and lookup;
2. a `TcpListener` on `127.0.0.1:8096`, handed to `door::tcp::carried`
   with the lender's key and the lend's name;
3. the sampler: every 10 seconds, one JSON line to a file in the app
   group's container, `{t, footprint, available, up, down, open, path, plant}`.
   `footprint` is `phys_footprint` from `task_info(TASK_VM_INFO)`, the
   number the system's memory killer counts; `available` is
   `os_proc_available_memory()`; `path` is `direct`, `relay` or `none`;
4. every branch traced at debug under target `tunnel`, beside the node's
   own `door` traces;
5. two plants, chosen when the tunnel starts, not by a build: `stranger`
   dials with a fresh key that stands in no group; `leak` allocates and
   touches 1 MiB a second until the system kills the extension;
6. a C ABI: `tunnel_start(plant, log_dir)` and `tunnel_stop()`.

**iOS.** An Xcode project with two targets. The app: one switch, a picker
for the plant (none, stranger, leak), passed in the tunnel's
`providerConfiguration`, and a button that shares the sampler's file out.
The packet tunnel extension: an address and no routes, then
`tunnel_start`. The Rust is built for `aarch64-apple-ios`, installed here,
into an XCFramework by a script in this directory. One build is signed
once, with the operator's paid developer account, on their own device; a
free personal team cannot use Network Extension at all (PHONE_REPLICA,
source f-746373).

About 250 lines in all: some 100 of Rust, 120 of Swift, 30 of script.

## 5. The sessions, in order

Each one starts with the sampler running and the phone's screen recording
on, and ends by saving the sampler's file, the recording, and the lender's
debug log for target `door`.

| # | Session | Bars |
|---|---|---|
| S0 | **Preflight, and the gate.** The tunnel starts, the sampler writes, and Jellyfin's app, at `http://127.0.0.1:8096`, lists the library. If it can't reach the listener, the spike stops here and §7 says what follows. | P0 |
| S1 | **A film on WiFi.** A 1080p film in Jellyfin's app, 30 minutes, the phone on the lender's WiFi. | P1, P2, P6 |
| S2 | **A film on cellular.** The same, WiFi off. | P1, P2, P6 |
| S3 | **Music, locked.** Finamp (open source, a Jellyfin client) at the same address, the app closed and the phone locked, 20 minutes. | P2, P3 |
| S4 | **The stranger.** The plant `stranger`; Jellyfin's app tries to connect. | P4 |
| S5 | **The leak.** The plant `leak`, last, since it kills the extension. The JetsamEvent report is taken from Settings, Privacy & Security, Analytics Data. | P5 |

A stall is counted from the screen recording, by hand, each with its time:
a stretch longer than 10 seconds where the picture or sound stops.

## 6. The bars, proposed

Each is one of four verdicts: passed, failed, could-not-judge, never-ran.

- **P0 — reached.** Another app's connection to `127.0.0.1:8096` reaches
  the extension's listener, seen in the `tunnel` trace and in the lender's
  `door` trace.
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

## 7. What each outcome decides

- **All pass:** production is designed next. The door's page path with the
  person's headers, joining in the app, the app's three screens (On, your
  groups, What acts as you), Android's `VpnService`, and the organization
  Apple's guideline 5.4 asks for.
- **P0 fails:** the packet path of the first version (`e2b1a158`, its
  §4.1) replaces the listener: `tun-rs`, a userspace TCP stack and the
  address table, about two rows more. S1 to S5 then run as written.
- **P2 fails:** before it counts as failed, the loop tries a current-thread
  runtime and smaller receive windows and buffers, and the sessions run
  once more. If it still fails, Play on an iPhone runs only while the app
  is open, and the page in the browser stays the door.
- **P1 fails only on cellular, relayed:** that is the relay's throughput,
  not the tunnel. It goes to GROUP_RELAYS as a finding, and this path
  stands.
- **iroh does not start in the extension:** the spike stops there, with
  what failed and why, and PLAN §2's replica row learns it too.

## 8. Cost

**Money: $99.** The Apple Developer Program, a year. The organization
guideline 5.4 asks for is for the App Store, not this spike. Devices are
the operator's. Cellular data: 1 to 2 GB for S2, since 30 minutes at
8 Mbit/s is about 1.8 GB.

**The work: one row,** `tunnel/`, the Xcode project and its build script,
with `PREDICTIONS.md` and the session sheet once the operator sets the
bars. One more is likely for the extension itself, whose logs come only
through Console.app and which is awkward to attach a debugger to. Two
more if P0 fails.

**The operator's hands,** which no worker can stand in for: enrolling,
about half an hour, then hours to two days for Apple's approval; signing
the app and its extension in Xcode and the first install, one to two
hours, once; the sessions, about two hours. Three and a half to four and
a half hours.

**Time:** two or three days from the account being settled.

## 9. Not decided here

- What a phone app stands as in a group. The tunnel's key stands as a
  computer of its person, because that is what the door admits to a raw
  lend today; a production app may be a computer in svrngs's terms, since
  it would redeem a link as a computer does (DESIGN §3, "Joining"), or a
  device kind of its own.
- How the workspace gets past libc 0.2.190 on iOS: held back, or netdev
  fixed upstream. A report to netdev is outward-facing, so it is the
  operator's to send.
- Names for lends on the phone, in place of 127.0.0.1 and a port, and what
  happens when two lends want one port.
- Whether the bars above are the operator's.
