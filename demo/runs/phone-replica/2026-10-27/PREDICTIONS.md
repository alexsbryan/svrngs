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

## The pocket day, added at 10:40 before it runs

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

## Two more runs, added 2026-10-27 at 21:10 before either starts

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
