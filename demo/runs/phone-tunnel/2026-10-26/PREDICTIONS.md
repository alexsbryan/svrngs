# Predictions, before the first session

Committed before S0, so a result can't bend what was expected of it. The
bars are DESIGN.md §6's, P0 to P6, as proposed: the operator said, on
2026-10-26, "Ok you should be ready to run now", after being told the
proposed bars would stand unless they changed them. Each prediction gives a
chance and the reason for it, and the chances are guesses, not rates.

**The run.** Lender: svrngs at `24505c9e`, on the operator's MacBook Pro,
its own `HOME` and `SVRNGS_DIR`, in the group `phone-tunnel-run`, lending
Jellyfin 12.0.0 (Jellyfin.app) as `svrngs share 8096 --tcp --kind
jellyfin`, with `RUST_LOG=door=debug,play=debug,transport=debug`. Phone:
an iPhone 14 (`iPhone14,7`), iOS 26.6.2 (23G90). The tunnel is built from
this directory at the commit that adds this file.

| Bar | Expected | Chance | Why |
|---|---|---|---|
| P0 reached | passes | 70% | Proxy apps serve other apps from their tunnel extension on loopback; nothing seen here yet says an extension's loopback is apart from an app's. |
| P1 plays, WiFi | passes, path direct | 80% | Phone and lender on one network; on the Mac the same core went direct at once (PREFLIGHT.md). |
| P1 plays, cellular | passes | 50% | Holepunching from a carrier's NAT often fails, so it may be relayed, and how much a 1080p stream n0's public relays carry is unmeasured. |
| P2 fits | passes, peak 15 to 30 MiB | 65% | 9 MiB on the Mac with no stream; a film adds iroh's receive windows and copy buffers, and iOS counts memory as a Mac does. |
| P3 the background | passes | 85% | A connected tunnel's extension runs whatever the app in front is doing; music keeps the player app alive. |
| P4 refused | passes | 95% | Seen on the Mac; the lender's `transport` trace is on this time. |
| P5 the instrument | passes, killed near 50 MiB | 75% | A sample every 10 s at 1 MiB/s puts the last within 10 MiB by construction; the risk is a JetsamEvent report that doesn't name the extension, or the extension stopped for another reason first. |
| P6 the path | direct on WiFi, relay on cellular | — | Recorded, not judged. |

**Where it would be wrong.** If P0 fails, everything after it waits on
the packet path (DESIGN.md §7). If the footprint grows with each minute
of film rather than holding level, P2 fails late, in S1's last minutes,
not at once; the samples will show which.
