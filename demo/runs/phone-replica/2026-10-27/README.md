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

## The pocket day, 10:35 to 20:58 Pacific

The operator carried the phone through an ordinary day with the tunnel on:
4 h 31 min with the screen on and 7 h 2 min off, by Settings › Battery.
The dialer ran from slot 137 to 760 and was stopped at 20:58. The phone's
own record is in `phone/`: `samples.jsonl`, a line every 10 s with no gap
over a minute, and `trace.log`.

| Bar | Suggested | Verdict | What it rests on |
|---|---|---|---|
| answered | at least 90% of judged dials within 30 s, and no run of misses past 15 min | **passed** | 617 of 624 answered (98.9%), none could-not-judge; the 7 misses were single dials, one at 15:07 and six between 18:15 and 18:35, so the longest gap was 2 minutes. Median 169 ms, 90th percentile 813 ms, slowest 19.6 s. 337 direct, 280 relay. |
| memory | peak at most half the 50 MiB cap, no termination | **passed** | Peak footprint 7.2 MiB, the least `available` 43.0 MiB. `up_s` never reset, so iOS never stopped the tunnel all day. |
| battery | at most 5 points over the day beyond a baseline day | **failed** | No baseline day was recorded, but none is needed to judge it: Settings › Battery charged Phone tunnel with 10% of the day's use, against Firefox's 12% and Qobuz's 11%, on a day that went from a full charge to 4%. That's about 9 to 10 points, roughly twice the bar. The screenshot isn't committed, since it names the operator's other apps. |
| data | at most 10 MB in 24 h | **failed** | The endpoint's own socket counters: 158 MB in 10.4 hours (104 MB up, 54 MB down), about 365 MB a day. It was steady at 11 to 15 MB an hour through the day, and 19 to 26 MB an hour after 18:00 on relay paths, so it's what the endpoint does at rest, not the operator's use. Settings › Cellular's figure wasn't recorded. |

**iOS slept the provider and dials woke it.** `slept` and `woke` stayed
at 0 until 16:42, then reached 571 each by evening (`phone/trace.log`),
and the dials kept being answered throughout. The prediction that a sleep
would make the phone unreachable was wrong both ways: it slept, and was
still reached.

**Where the bytes go isn't attributed.** The counters are every byte the
endpoint's sockets carried: the probe's dials, holepunching on each new
connection, relay pings, and iroh's net reports, every 20 to 26 s
(PHONE_REPLICA §4). An hour of iroh's per-kind metrics at a desk
attributes them.

## The night on a charger, 2026-10-27 21:09 to 2026-10-28 07:43 Pacific

The phone charged overnight on WiFi with the tunnel on. The record is in
`night/`: `dials.jsonl` (the dialer's), `samples.jsonl` and `trace.log`
(the phone's, cut to the night).

| Bar | Verdict | What it rests on |
|---|---|---|
| answered | **passed** | 634 of 635 answered (99.8%), the one miss at 21:59 (`connect: timed out`). Median 139 ms, 90th percentile 243 ms, slowest 1.5 s. 417 direct, 217 relay. |

`up_s` ran unbroken from 36,859 to 74,108 s: the same tunnel since the
first day's 09:45. iOS slept the provider 110 more times overnight (611
to 721), and every dial still woke it. Peak footprint 7.8 MiB.

**The bytes didn't care that the phone was idle:** 132.9 MB over 10.6
hours, 12.6 MB an hour, the same rate as the day of ordinary use. So the
cost is what the endpoint does at rest, dialed once a minute. Battery and
data aren't judged on a charger and WiFi, by the predictions.
