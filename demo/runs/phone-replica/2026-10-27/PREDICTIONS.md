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
