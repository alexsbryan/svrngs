# The phone's door — what the spike found

Run 2026-10-26, 23:08 to 23:38 Pacific, on an iPhone 14 (`iPhone14,7`,
iOS 26.6.2), against the lender PREDICTIONS.md names. Closed early by the
operator: "Can we just close this up? Do we have enough signal for a
scrappy startup level of execution?" S2 and S3 never ran.

**The answer: yes, it can be done.** The node's own endpoint and member's
half of a lend run in an iOS packet tunnel extension at about 6 MiB, under
a 50 MiB cap. They serve a group's lend on the phone's 127.0.0.1 to Safari
and Jellyfin's app while the tunnel's app is not in front, carry a 1080p
stream directly at 6 to 10 Mbit/s over the phone's cellular network, with
no relay, and refuse a key no group holds. What is
left is production's tuning, not the question.

## The bars

| Bar | Verdict | What it rests on |
|---|---|---|
| P0 reached | **passed** | Safari and Jellyfin's app at `127.0.0.1:8096` reached the extension's listener and the lend: 28 connections from the tunnel's key in `s0/lender.log`. |
| P1 plays, WiFi | **never-ran** | The film played only after the phone left WiFi (below). S0, on WiFi, carried Jellyfin's pages, not a film. |
| P1 plays, cellular | **could-not-judge** | The operator, after: "I ran this on cellular network". About 4 of 30 minutes, 23:28 to 23:32, with a scrub, and no screen recording, so no stall count. In those minutes: 6.4 to 10.3 Mbit/s held, 38.0 on the scrub, the lend open at every sample (`s1/samples.jsonl`). |
| P2 fits | **could-not-judge** | By its letter it waits on P5. Peak footprint 6.4 MiB across S0 and S1, against a bar of 40. |
| P3 the background | **never-ran** | S3 never ran. S1's film played with the tunnel's app out of the way, which is half of P3. Through a locked phone was not tried. |
| P4 refused | **passed** | Key `326c3679…` refused `NotAdmitted` on the phone (`s4/trace.log`), and `dialer not admitted` under `transport` on the lender (`s4/lender.log`). |
| P5 the instrument | **could-not-judge** | Two of its three clauses met, twice. The extension was killed after samples of 44.83 and 45.25 MiB, with no `stop:` traced (`s5/trace.log`). The operator started both runs, the second "to make sure I didn't miss something with the jetsam". No JetsamEvent report was written: the phone's newest is from 2026-10-20 (`xcrun devicectl device info files --domain-type systemCrashLogs`). |
| P6 the path | recorded | Direct in S0 and S1, on cellular from 06:13 UTC. Relay in S4 and S5, each under a minute from a fresh start. |

**The instrument holds by its spirit, not its letter.** P5 was there to
show that the sampler's `footprint` is the number iOS kills on. At every
sample from S0 to S5, `footprint + available` was 50.00 MiB, and both
kills came where `available` ran out. A JetsamEvent report was the third
witness, and iOS 26.6.2 wrote none for an extension killed at its limit.
Read so, P5 and with it P2 pass; by the letter, they stay as above, and
the reading is the operator's.

## Departures from the design

- **The film was synthetic:** 31 minutes of ffmpeg's `testsrc`, 1080p30
  H.264 with noise, averaging 6.4 Mbit/s, since the lender's library had
  only 1-minute films. It is at
  `~/Movies/house-mesh-demo/Phone Tunnel Test (2026)/` on the lender, not
  committed.
- **No screen recordings,** so no stall was counted by hand.
- **S0 used Safari as well as Jellyfin's app,** as P0's "another app".
- **S1 ran on cellular, not WiFi,** so it was S2's network for S1's
  length. The lender's own sync dials to the tunnel's key list the phone's
  addresses (`s1/lender.log`, "holding these addresses"): `192.168.1.14`,
  the lender's LAN, until 06:13 UTC; then `71.140.180.29:1309` (whois: AT&T,
  SBC Internet Services), active through the film. The lender's home
  address with the phone's port, `69.181.177.160:50019`, also stays
  listed as active, which these logs don't explain. They are the sync
  connection's, not the lend's, so they say which networks the phone was
  on, not which carried each byte.

## Found, for production

1. **The lend's connection dropped three times in its first five minutes**,
   the last as the phone moved from WiFi to cellular,
   `ConnectionLost(TimedOut)` at 06:09:52, 06:10:47 and 06:13:04 UTC
   (`s0/trace.log`), and then held for 19 minutes, 14 of them idle (`s1/`).
   The member's dial is held to a 4 s idle cap with a keep-alive each
   second (`crates/svrngs-node/src/dial.rs:57`, `held()`), set on a laptop
   ("Cellular, a second host and a release build were not measured"). Each
   drop's lend was opened again at the next sample, within 10 s. A drop
   mid-film is a stall, so this is the first thing production instruments,
   before choosing a cap for phones.
2. **libc at 0.2.189** for every iOS build until netdev's
   `in6_ifreq` use is fixed (DESIGN.md §1, §9).
3. **`SystemConfiguration` is linked** for netwatch's `SCDynamicStore`
   calls, which iOS's headers mark unavailable: an App Review question
   (DESIGN.md §9).
4. **App groups need the Xcode app once:** `xcodebuild
   -allowProvisioningUpdates` makes the team profiles but won't register a
   new app group (`ios/project.yml`).
5. **Not measured:** a stream through a relay, for a carrier where
   holepunching fails; 30 minutes on any network; and a locked phone. The
   design reads a relayed cellular failure as the relay's, not the
   tunnel's (DESIGN.md §7).
