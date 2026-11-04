# Does a locked iPhone answer a dial? Predictions, before the run

Phase 1 of svrn-docs `rings/current/PHONE_REPLICA.md` §5, cut to the one
question that can end the replica outright, risk 1 of its §7: **does the
phone's endpoint answer a dial by its key while the phone is locked?** Apple
DTS said a tunnel is not kept alive while locked ("100%, no",
[dts-756941]). The operator chose this path on 2026-10-27, after the door
spike (`../../phone-tunnel/2026-10-26/README.md`): "Ok let's go that path".

**What runs.**
- **On the phone:** the door spike's tunnel, built with a probe
  (`tunnel/src/lib.rs` there, `PROBE`, `answering`). Its endpoint, the
  tunnel's key `fe352c1f…`, accepts `svrngs/probe/0` from the dialer's key
  alone and answers one line of vitals: `up_s`, `footprint`, `available`,
  and `slept` and `woke`, the provider's sleeps and wakes as iOS calls
  them. Under the probe it opens no lend, so nothing of the door wakes it.
  The app turns on iOS's on-demand rule, so iOS restarts a tunnel it
  stops, and a restart shows as `up_s` starting again.
- **The dialer, `dialer/`,** on the Fedora box, always on. Every 60 s it
  dials the phone's key, waiting up to 30 s, and at the same moment makes a
  control dial through a relay to an endpoint of its own. Each slot gets
  one verdict in `dials.jsonl`: answered; failed (the control answered and
  the phone didn't); could-not-judge (neither answered); never-ran (a slot
  the schedule owed and missed).

**Plants, already watched failing at a desk on 2026-10-27,** with the same
core on the Mac:
- a key nobody holds: failed, every slot, the control answering;
- the probe naming another key: "closed unanswered" in the trace, and
  failed at the dialer.

On the phone, the run starts with the tunnel off for three minutes, so the
dialer's first slots are failed.

**Sessions.**
1. The tunnel off for 3 minutes.
2. On, unlocked, for 5 minutes.
3. Locked, on WiFi, for 60 minutes.
4. Locked, on cellular with WiFi off, for 60 minutes.

The phone is left untouched through 3 and 4. The operator writes down when
each one starts, and whether the phone is on power.

## The bar, PHONE_REPLICA §5's suggestion for its "answered" bar

For each locked hour: **at least 90% of judged dials answered within 30 s,
and no run of misses longer than 15 minutes.** Under PHONE_REPLICA's
suggested bars, the operator's to set. The operator chose this path, and a
change before the run lands here first.

## Predictions

| Session | Expected | Chance | Why |
|---|---|---|---|
| Off | every slot failed, the control answering | 95% | the plant |
| On, unlocked | all answered, under 2 s after the first | 90% | the desk run answered in 1 to 459 ms |
| Locked, WiFi | **fails the bar**: answered under 50%, with gaps past 15 min | 65% | DTS; a developer counted sleeps every 42 s on iOS 16 and 6 s on 17 [f-737280]. Against it: a dial is a packet arriving on the provider's own socket, which may wake it. |
| Locked, cellular | fails the bar, worse than WiFi | 70% | the same, and the carrier's NAT drops a mapping that isn't refreshed while the phone sleeps |
| `slept` and `woke` | both above 0 once locked | 80% | the provider's `sleep` and `wake` are iOS's to call |

**If it fails as predicted:** the replica stops here. DESIGN's always-on
computer stays the answer to risk 1, and the phone stays a door that dials
out. PHONE_REPLICA's phase 2 and its risks 4 to 6 aren't spent on. **If
it passes:** phase 2, the 24-hour pocket run, is worth its cost.

## The pocket day, committed 2026-10-27 10:35 (`4dcf165c`), before it ran

The operator, 2026-10-27, after the locked hour: "I can keep the tunnel on
while using the phone throughout the day with normal use patterns". The
dialer keeps running, from slot 137 until the operator stops it tonight.
This is PHONE_REPLICA §5's phase 2, at its suggested bars, with the
operator's ordinary day as the routine.

| Bar | Suggested | Expected | Chance | Why |
|---|---|---|---|---|
| answered | at least 90% of judged dials within 30 s, and no run of misses past 15 min | passes | 55% | The locked hour answered every dial. A day adds cellular, moving between networks, and iOS stopping the tunnel under memory pressure from other apps; the on-demand rule restarts it, but each restart costs a dial or two, and `up_s` shows it. |
| memory | peak at most half the 50 MiB cap, no termination | passes | 90% | 4.6 to 6.4 MiB in every run so far. |
| battery | at most 5 points over the day beyond a baseline day | could-not-judge | 80% | No baseline day was recorded. Settings › Battery's per-app share is recorded instead. |
| data | at most 10 MB in 24 h | fails | 55% | 1,440 dials a day, each a QUIC handshake of a few KB each way, plus iroh's relay pings every 15 s and net reports every 20 to 26 s (PHONE_REPLICA §4). |

**Tonight, the operator records:** Settings › Battery's share for Phone
tunnel over the last 24 hours, Settings › Cellular's figure for Phone
tunnel, the tunnel's samples and trace, and any reboot or time the tunnel
was switched off.

## Two more runs, committed 2026-10-27 21:09 (`5223095e`), before either ran

The operator asked for the simpler plan ("Can you do it more simply, more
resourcefully?") and chose it: "Yes, let's do that". Both runs reuse the
build already on the phone and the same dialer. Each writes its own
`dials.jsonl` on the Fedora box.

**The night: is a replica affordable on a charger?** The phone charges on
WiFi overnight with the tunnel on, and the dialer calls every 60 s, from
tonight until the operator turns the tunnel off in the morning. The night
is when laptops sleep, so it is the hours risk 1 is about. Battery and
data aren't judged, since a phone on a charger and WiFi doesn't pay them.

| Bar | Expected | Chance | Why |
|---|---|---|---|
| answered: at least 90% of judged dials within 30 s, and no run of misses past 15 min | passes | 80% | The locked hour and the day both passed; a still phone on power and WiFi is the easiest case. The risk is iOS's overnight maintenance, a software update or a restart. |

**The day: are the dials the cost, or iroh's timers?** The day after, an
ordinary day as on 2026-10-27, but the dialer calls every 900 s instead
of every 60 s. Judged against the first day's 10% of the battery and
about 365 MB a day (README.md, "The pocket day").

| What | Expected | Chance | Why |
|---|---|---|---|
| answered | passes, with each miss a 15-minute gap | 60% | One miss now fails the 15-minute rule. |
| bytes a day, by the endpoint's counters | falls by less than half, to over 180 MB | 60% | iroh's own timers run without any dial: a relay ping every 15 s, net reports every 20 to 26 s with HTTPS probes, and a path heartbeat every 5 s (iroh-1.0.2 `relay/actor.rs:71`, `socket.rs:109`). |
| battery share | falls by less than half, to over 5% | 60% | A relay ping every 15 s keeps the radio from resting, dials or not. |

**What each decides.** If the bytes and battery fall by more than half,
the dials were the cost, and the fix is ours: dial phones less often. If
not, iroh's timers are the cost: the replica is a charger-only feature
until n0 offers a low-power setting, which we ask for with these numbers.

## The net report at 5 minutes, committed 2026-10-28 17:50 (`4fcfea3f`), before it ran

The operator: "Build it". The tunnel built against iroh 1.0.2 with one
constant changed, the net report's interval from 20 to 26 s to 300 to 306 s
(`iroh-restun-5min.patch`, built by `IROH_PATCH=<patched copy>
../../phone-tunnel/2026-10-26/build-xcframework.sh`), iroh's trace still
on. A fresh start, locked on cellular for 20 minutes, dialed every 300 s
by the same dialer.

| What | Expected | Chance | Why |
|---|---|---|---|
| bytes an hour, awake | under 3 MB, against 13.0 | 75% | the reports were about 82 KB each, 158 an hour; at 5 minutes, 12 an hour, with relay pings and pkarr left |
| net reports in the trace | 4 to 6 in 20 minutes, against 58 in 22 | 90% | the patch, plus one at start and one on a link change |
| answered, of the 4 dials | all 4 | 70% | the relay keeps the phone reachable whatever the UDP mapping does; a direct path may be lost between reports, so answers may be relayed |

**What each decides.** If the bytes fall as expected and the dials are
answered, the report's interval is the phone's lever: we ask n0 to make it
configurable, with these numbers. If the bytes don't fall, the cost is
elsewhere, and the trace says where.

## netwatch's trim fixed, with the 5-minute report, committed 2026-10-28 19:07 (`cc48ee75`), before it ran

The operator: "second patch". netwatch 0.19.1's `is_interesting_interface`
means to ignore Apple's AWDL, low-latency WLAN and IPsec interfaces, but
`name.trim_end_matches("0123456789")` trims that whole string, never
trailing digits, so `awdl0` counts (`rustc` on the four names, as written
and fixed; the same line is in 0.19.3 and 0.20.0). In `restun/`, awdl0's
flags alternate between 34915 and 34850, up and running toggled, exactly
129 times, against 129 major link changes. `netwatch-trim-digits.patch`
trims the digits instead. The build carries it and `iroh-restun-5min.patch`
(`IROH_PATCH=… NETWATCH_PATCH=… build-xcframework.sh`). The same routine:
a fresh start, locked on cellular for 20 to 30 minutes, dialed every 300 s.

| What | Expected | Chance | Why |
|---|---|---|---|
| major link changes | 0 to 2, against 129 | 80% | awdl0's flapping no longer counts; a real change, say a cell handover, still would |
| bytes an hour | under 2 MB, against 21.6 with the loop and 13.0 awake | 70% | no per-wake report or relay reconnect, and a report every 5 minutes |
| sleeps and wakes | some or none; either way cheap | — | whether iOS sleeps the provider isn't ours; what each wake costs is |
| answered, of the dials | all after the tunnel starts | 75% | the relay connection is no longer torn down at each wake |

## The ladder, committed 2026-10-28 19:53 (`d556847b`), before it ran

The operator asked for "a comprehensive measurement" in place of one more
patch ("Rather than whack a mole …"), then "Do it". One build carries
`netwatch-trim-digits.patch` and `iroh-reports-ladder.patch`: a fresh copy
of iroh 1.0.2 whose net reports run as `SVRNGS_IROH_REPORTS` says, set by
the tunnel from the step the app's picker names. Stock iroh ignores it.
Checked on the Mac first: in 70 s, the relay step ran 1 report and the
stock step 3.

Each step is a fresh start of the tunnel, locked on cellular for 20
minutes, dialed every 300 s. The samples carry the step, so the record
splits itself.

| Step | What runs |
|---|---|
| bare | the tunnel up, no endpoint at all |
| relay | an endpoint and its relay connection; one report at start, none after, none on a link change |
| periodic | the relay step, plus a report every 300 to 306 s |
| stock | iroh's own reports, every 20 to 26 s and on every link change; the trim fix stays in |

| What | Expected | Chance | Why |
|---|---|---|---|
| wakes an hour, bare | under 60 | 60% | iOS's own wakes of an idle phone, with nothing of ours to answer |
| **wakes an hour, relay, against stock** | **under a third of stock's** | 60% | the traces put an iroh report on each side of most sleeps (`restun/`, `fixed/`): if its replies are what wake the phone, removing the reports breaks the loop. If not, the relay's pings, every 15 s from the phone and 16 to 20 s from the server, may be enough to keep waking it. |
| bytes an hour, relay | under 1 MB | 70% | relay pings and pongs are tens of bytes |
| bytes an hour, periodic | under 2 MB | 65% | 12 reports an hour at about 82 KB each, plus the relay step |
| bytes an hour, stock | 13 to 21 MB | 75% | the earlier runs |
| answered | none on bare; all on the other steps after their first minute | 75% | bare has no endpoint; the others keep their relay connection |

**What it decides.** If relay's wakes and bytes are both small, an
all-day replica is viable on iroh once reports stop reacting to link
changes and the interval can be set, and n0 gets that request with this
table. If relay's wakes stay high, the floor is the relay keep-alive, and
the request is a keep-alive a phone can afford, or a replica only on the
charger.
