# Does a locked iPhone answer a dial? What the run found

Run 2026-10-27 on the operator's iPhone 14 (`iPhone14,7`, iOS 26.6.2), with
the dialer on the Fedora box, as PREDICTIONS.md describes. `dials.jsonl` is
the dialer's record, one line a slot.

**Locked on WiFi, not charging, from about 09:53 to 10:34 Pacific: 42 of 42
dials answered** (slots 95 to 136), in 17 to 928 ms, by direct and relay
paths. `up_s` rose from 456 to 2,916 without a reset, so iOS never stopped
the tunnel, and `slept` and `woke` stayed at 0: iOS never called the
provider's `sleep`. The footprint held at 4.6 to 4.7 MiB. The operator
ended it at 41 of 60 minutes: "I'm gonna need my phone -- I think 30+
minutes are still a good read".

| Session | Slots | Verdict | What it rests on |
|---|---|---|---|
| Off | 0 to 86 | the plant, as expected | 87 slots failed, every control answered. Much longer than the planned 3 minutes, since the tunnel was switched on late. |
| On, unlocked | 87 to 94 | as expected | All answered. The first took 28.5 s with `up_s` 4, a dial waiting for the newly started tunnel; then 60 to 498 ms. |
| Locked, WiFi | 95 to 136 | **passed**, over 41 of the bar's 60 minutes | 42 of 42 answered within 30 s, and no miss at all. |
| Locked, cellular | — | never-ran | The operator needed the phone. |

**The prediction was wrong.** It gave the locked hour a 65% chance of
failing, on Apple DTS's "100%, no" [dts-756941], and expected sleeps once
locked. For a tunnel whose endpoint is dialed once a minute, iOS 26.6.2
kept it awake and answering. A dial each minute may itself be what keeps
the phone awake; it is the rate a group's computers would dial a replica
(`crates/svrngs-node/src/ring_sync.rs:105`), so it is the realistic case.

Not settled: cellular locked; a day in a pocket; battery and data; the
record on the phone (PHONE_REPLICA §5, phases 2 and 3).
