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

## The day at a 15-minute dial, 2026-10-28 07:43 to 16:58 Pacific

The same tunnel, never restarted (`up_s` from 74,154 to 97,256), carried
through an ordinary day, dialed every 900 s. The record is in `day15/`.
The operator ended it at 17:05: "I think we've learned what we needed for
the day".

| What | Verdict | What it rests on |
|---|---|---|
| answered | **failed** by the letter | 37 of 38 answered. The one miss, at 15:14, is a 15-minute gap, and the bar allows none past 15 minutes. The answers either side took 11.4 s and 6.5 s. |
| bytes | **the prediction failed, the other way** | 247.0 MB in 9.3 hours, **26.6 MB an hour**, against 12.6 dialed every minute. It went up, not down. |
| battery | the same | Settings › Battery: Phone tunnel **30%** of the day's use by 17:03, flagged "Poor coverage during use"; the phone used 71% by then, against its 38% average. |

## Diagnosis: the cost is waking, not dialing

- **iOS put the provider to sleep and woke it 1,741 times in 9.3 hours,**
  against 571 on the first day and 112 overnight (`day15/trace.log`).
- **The rhythm is a loop.** After a wake the provider stays up a median
  2.9 s (10th percentile also 2.9 s: a fixed grace), sleeps, and is woken
  again a median 4.5 s later. The gap between wakes is 9.0 s at the
  median (5.4 to 23.7 s, 10th to 90th percentile).
- **The bytes sit at the wakes.** Of the day's 247 MB, 208.7 MB (84%)
  fell in 10-second sample windows that held a wake: 135.5 KB a window
  with one wake, 234.7 KB with two or more, against 30.4 KB with none
  (`day15/samples.jsonl`, `day15/trace.log`).
- **A dial each minute hid it.** The first day and the night, dialed
  every minute, had a third and a fifteenth as many wakes. Steady traffic
  kept iOS from putting the provider to sleep, so they paid only the
  awake rate, about 11 to 13 MB an hour: the same 30 KB per 10 s the
  wakeless windows show today.

**The likeliest mechanism, not yet confirmed.** iroh is written for a
process that never stops. Asleep, its timers stand still; woken, every
overdue one fires at once inside the 2.9 s grace: a relay ping (every
15 s), a path heartbeat (every 5 s), a net report with QUIC and HTTPS
probes to every relay (every 20 to 26 s), maybe a reconnect to the relay.
The provider is asleep again before the answers arrive, and the answers
wake it. Poor coverage stretches every exchange, and so the loop.

**Two costs, then, each with its own cure:**
1. **Awake at rest, about 11 to 13 MB an hour:** iroh's timers. Fine on a
   charger and WiFi, as the night showed.
2. **Each wake, about 100 to 200 KB, and a radio that never rests:** the
   loop. It is what made today 30% of the battery.

## What iroh does at rest, from its own trace, 2026-10-28 17:16 to 17:39

A fresh start of the tunnel with iroh's trace at debug (`74a3c65a`),
locked on cellular for 22 minutes and never dialed. The record is in
`iroh-trace/`. **The loop didn't reproduce:** iOS never slept the provider
(`slept` 0, a sample every 10 s), so this run shows the awake cost alone.
13.0 MB an hour, the first day's and the night's rate.

Counted in `iroh-trace/trace.log` over the 22 minutes:
- **58 net reports**, one every 20 to 26 s ("scheduling periodic_stun to
  run in 23s"). The first probed all four of n0's relays; after that,
  three probes a report, QUIC address discovery (QAD) over v4 and v6 to
  the nearest two relays (118 to usw1, 61 to use1), plus 12 HTTPS probes.
  Each QAD probe is a fresh QUIC handshake with a relay.
- **90 relay pings and pongs,** one every 15 s: tiny.
- **12 pkarr publishes:** tiny.

So the net reports carry nearly all of it: about 82 KB a report, 158 an
hour. Their interval is a constant, "Pick a random duration between 20
and 26 seconds (just under 30s, a common UDP NAT timeout on Linux, etc)"
(`iroh-1.0.2/src/socket.rs:2004-2008`, `new_re_stun_timer`): the report
doubles as the keep-alive of the UDP socket's NAT mapping, so direct paths
can be punched. `NetReportConfig` can turn off the HTTPS probes and the
captive-portal check, not the interval.

**When the loop happens is still open.** Across the runs, wakes come
only off the charger and only after hours: none on the first day until
16:42, then 32 to 179 an hour as the day wore on; overnight, none from
22:36 to 07:41 on the charger; on 2026-10-28, from 07:44, minutes after
the phone came off the charger at 07:11, rising from 50 to 320 an hour
by 15:00. A fresh start locked on cellular for 22 minutes had none.

## The net report at 5 minutes, and the loop seen from inside, 2026-10-28 17:54 to 18:22

The tunnel built with `iroh-restun-5min.patch`, a fresh start locked on
cellular for 28 minutes, dialed every 300 s. The record is in `restun/`.

**The patch held, and didn't help.** The periodic report was scheduled at
300 to 306 s, and ran twice (`starting direct addr update (Periodic)`).
But the bytes rose to **21.6 MB an hour** (9.98 MB in 28 minutes), since
this time **the loop happened**: 64 sleeps and 64 wakes. The predictions
for bytes and reports failed. Of the dials, 6 of 7 were answered, in 0.5 to
3.6 s; the first, `failed`, came before the tunnel was restarted.

**What each cycle does** (`restun/trace.log`, around wake 20 at 01:09:25):
1. iOS wakes the provider, and the cellular interfaces it lists have
   changed: iOS keeps eleven, `pdp_ip0` to `pdp_ip10`, and brings data
   contexts up and down as the modem sleeps.
2. iroh's netwatch reads that as **`link change detected is_major=true`**
   and runs a full net report (`do_full=true`): QUIC probes to the relays
   over v4 and v6, HTTPS probes, a captive-portal check. Then, often, a
   second full report straight after.
3. On a link change iroh pings the relay to check the connection, and
   waits half a second (`Sending ping` 31.137, `Ping timeout` 31.639). iOS
   has already put the provider back to sleep by then, so the pong can't
   arrive: **`Lost connection to relay server: Ping timeout`**, and a new
   relay connection, TCP, TLS and a websocket, on the next wake.
4. Going to sleep changes the interfaces again: another major link change.

Counted over the 28 minutes: 129 major link changes and 129 full reports
(`starting direct addr update (LinkChangeMajor)`), 64 relay reconnects
after a ping timeout, against 2 periodic reports. About 150 KB a cycle.
The phone's public address answered by every probe stayed the same
(`71.140.182.158:5823`): the network didn't change, only the interface
list did.

**So the cost has two causes, and the loop's is not the report's
interval:**
1. **Awake at rest, about 13 MB an hour:** the net report every 20 to 26
   s (`What iroh does at rest`). The 5-minute patch removes it; no wake
   then pays it.
2. **Each sleep and wake, about 150 KB:** iOS's cellular interfaces flap
   with the modem, and iroh treats each flap as a major network change:
   a full report each time, and a relay reconnect when its half-second
   check straddles a sleep.

Neither is a setting in iroh 1.0.2. Both are fixable there: a report
interval a caller can set; and on iOS, a link change that leaves the
default route and the public address as they were treated as minor, with
the relay's check not failed by a sleep.

## netwatch's trim fixed, 2026-10-28 19:11 to 19:39

Both patches, `iroh-restun-5min.patch` and `netwatch-trim-digits.patch`,
a fresh start locked on cellular for 28 minutes, dialed every 300 s. The
record is in `fixed/`.

**The fix did what it was for, and the bytes didn't move.** Major link
changes fell from 129 to 2, as predicted. The loop still ran: 53 sleeps
and wakes. The bytes were **20.5 MB an hour**, against 21.6 before the fix
and a prediction of under 2. All 7 dials were answered, in 0.47 to 2.2 s.

**Why** (`fixed/trace.log`):
- **A minor link change still runs a report.** iroh's
  `handle_network_change` calls `re_stun(LinkChangeMinor)` for a minor
  change too (`iroh-1.0.2/src/socket.rs:1661-1675`), and netwatch still
  signals a change whenever any interface's flags move, AWDL's included,
  since its watched state compares every interface. So 109 minor changes
  ran 109 reports (`starting direct addr update (LinkChangeMinor)`), some
  full, against 4 periodic ones.
- **A report that straddles a sleep misreads the relays.** Latencies of
  1.1 to 1.2 s were measured across a sleep, against 40 to 90 ms awake, and
  the preferred relay swapped each time: `home is now relay` 63 times,
  between usw1 and use1. Each swap republished to pkarr (`Publish endpoint
  info to pkarr (info changed)`, 63 times) and moved the relay connection
  (14 lost, 25 redialed).

So the classification bug was real but not the whole cost. Each wake still
pays a report, and a report taken across a sleep moves the home relay.
