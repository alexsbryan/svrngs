# The confidence round — 2026-11-26

Pre-registered in PROTOCOL.md ("The confidence round", `1296348e`) before
its first run.

**The claim:** on the fixed projection (`b8d33b6c`), an agent with the
projected view diagnoses these five faults as well as one with raw
telemetry, and no personal value crosses the projection.

**The rule:** confident if, on every fault, P's mean is at least R's mean
minus 0.5, and every window's leak check passes with its plant failing.
There are 3 runs per arm per fault; `scores.tsv` has every one with its
source, made either this round or on the fixed projection before it.

## Diagnosis: met on every fault

| Fault | P (projected) | P mean | R (raw) | R mean | P ≥ R − 0.5 |
|---|---|---|---|---|---|
| F1 payment fails ~50% | 3, 4, 4 | 3.67 | 3, 3, 3 | 3.00 | yes |
| F2 one product fails | 4, 4, 4 | 4.00 | 4, 4, 4 | 4.00 | yes |
| F3b recommendation cache grows | 4, 4, 4 | 4.00 | 4, 4, 4 | 4.00 | yes |
| F4 email memory leak | 1, 3, 3 | 2.33 | 1, 1, 3 | 1.67 | yes |
| F5 non-US shipping 10 s | 4, 4, 4 | 4.00 | 3, 4, 4 | 3.67 | yes |
| **All** | **54/60** | **3.60** | **49/60** | **3.27** | |

**Where they differ, the projection helped:**
- **F1.** R read the payment error's text, "Invalid token.
  demo.user_context.loyalty_level=gold", and all three R runs blamed gold
  customers. P couldn't read that text. In two of its three runs it saw
  the loyalty hash on successful charges too, and said correctly that the
  failures are random, about 10%.
- **F4** is the hard fault for both arms: email exports no memory metric,
  and in this window no confirmation was visibly lost. Both arms found
  the restarts from email's counter resetting. Whether a run named email
  as the fault was a matter of the agent's nerve, not of what it could
  see.

## Leaks: none, after three fixes to the checker

In every window the projected arm held none of the personal values
harvested from the raw arm, and the plant (the same search over the raw
arm) failed every time.

The record keeps one failing file, `F3b-leakcheck.txt` (1 of 5).
`--debug` caught the value: `10.89.1.6`, a container's IP, matched as a
substring of `10.89.1.62`, a different server's address that passes.
That was the checker's bug, not a leak. Markers now match whole values.
Three checks of the same window then passed and the plant failed, but
only on the console: the window had moved on before they were saved.

The checker raised three false alarms across the benchmark, and each was
traced to a value before it was fixed:
1. `server.address`, a key the checker took as personal (before round 1);
2. a container IP seen as `client.address` in one span and `server.address`
   in another (the rerun's F4);
3. this substring match.

None was a value about a person.

## What "confident" covers here, and what it doesn't

- **3 runs per cell finds a large gap, not a small one.** It shows the
  projection costs no more than half a point per fault on these five,
  and that's all.
- **One application:** the OTel Demo 3.1.0, under synthetic load.
- **I did the scoring,** against ground truth written before the runs,
  from each report alone. Every agent recognised the demo, in both arms.
- **The view's `/metrics` lists 100 series by default,** and
  `process.memory.usage` falls past them in both arms. Two P agents said
  there was no memory metric. It's a usability fix (list names first),
  and it doesn't separate the arms.
- **Protocol fixes made along the way,** each before the runs they
  affect:
  - fresh views per fault (`f27f1f29`);
  - services restarted per fault (`1296348e`);
  - F4 waits for email's restart.
