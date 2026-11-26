# The rerun on the fixed projection (b8d33b6c) — projected arm only, all five faults

First the two results that depended on what the fixes took away (F2,
F3b), then the other three, one agent each, with the same prompt and
protocol. The raw arm sees nothing different, so it wasn't rerun.

| Fault | Before the fixes | After | What the agent used in place of what was removed |
|---|---|---|---|
| F2 one product fails | 4/4 (read the free-text event name) | **4/4** | the flag's key and value, and the per-product split by hash. It listed the event name (now shaped) as unseen and didn't need it |
| F3b cache grows | 4/4 (read exact digit counts) | **4/4** | length bands: 2–3 → 4–7 → 8–15 digits, then a reset at each restart |
| F1 payment fails | 3/4 | **3/4** | the same miss as before: the loyalty label the demo writes on failure (h fb13c8bc04, 6/6) read as the cause |
| F4 email leak | 2/4 | ***0/4, confounded*** | see below |
| F5 non-US shipping | 3/4 | **4/4** | the fixes *added* what scored: shipping's logs now in /logs (the flag and delay logs on exactly the 2 slow orders of 26), and `feature_flag_key` = `intlShippingSlowdown` readable, from which it inferred international orders, which it flagged as unverified |

**After the fixes, projected arm: 15/16 on the four clean faults (F1, F2,
F3b, F5), against 14/16 before.** F4 is confounded. One run each: not a
rate.

**F4's rerun is confounded twice, so it says nothing about the
projection.**
1. *Cumulative counters carry history across the fresh-view reset.* The
   agent found 7 failed charges in checkout's cumulative
   `rpc.server.call.duration` counts. They were F1's, run 15 minutes
   earlier: fresh views drop old spans and logs, but not what a counter
   already counts. That's a second protocol flaw, of F3's kind. The
   remedy is to restart the services, not only the views, between faults.
2. *The fault hadn't shown.* Email sat at 90 of 105 MB with no restart
   (`F4-witness.txt`; the first run had one), so every order did get its
   confirmation. The agent's arithmetic was right for what was there.
Scored 0/4 against the ground truth, and recorded as inconclusive.

**The leak checker's second false alarm.** F4's first check failed: 1 of 8
(`F4-leakcheck.txt`). Caught with `--debug`, the value was `10.89.1.22`,
a service container's IP. It is a `client.address` in one span (shaped)
and a `server.address` in another, where it passes as a machine's
address. The checker now drops a harvested value the raw arm also shows
as a machine's own address. Three reruns passed; the plant on R failed.
No personal value reached P in any window.

Leak check: P passed in both windows (`F2-leakcheck.txt`,
`F3b-leakcheck.txt`), and the plant on R failed in both. One run each:
the fixes cost no diagnosis here, which isn't a rate.

Also fixed for the rerun, in lab.sh:
- the views now start `--fresh`, since read from the start they took
  longer than the share's wait over a 200 MB file;
- the lab waits for both views' ports before sharing.
