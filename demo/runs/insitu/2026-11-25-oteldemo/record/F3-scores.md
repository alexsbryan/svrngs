# F3 — recommendationCacheFailure on — "The site feels slower than last week." — CONFOUNDED

**Protocol flaw found here.** The view keeps 30 minutes; faults were about
9 minutes apart. F3's window still held F2's product-catalog errors
(t 1795646985187–1795647300856), so both arms faced two faults, one of
them over. PROTOCOL.md now restarts both views at each reset (its own
commit), and F3 is rerun as F3b. These scores stand as recorded, with
the confound.

| Point | P (projected) | R (raw) |
|---|---|---|
| Service: recommendation | 1 | **0**: diagnosed F2's product-catalog failure; "no latency changed" |
| Mechanism: a growing cache, slower calls, memory | 1: the flag flips on (`demo.feature_flag.recommendation_cache` False→True by minute), p95 3–4 → 84–216 ms, two restarts read from SDK counters resetting, "restarts point that way" (memory), not shown | 0 |
| Scope: recommendations on every page | 1 | 0 |
| Evidence | 1: per-minute percentiles, counter resets, traces c1ef9e8b and fa1e1c07 | 0 |
| **Total** | **4/4** | **0/4** |

P: 12 tool calls. R: 6 tool calls, 61 s. Both recognised the shop.

**Not a projection effect.** R had strictly more information than P. The
gap is one agent anchoring on the leftover errors, inside a window that
should not have held them: variance in agents, not the cost of the
projection. It counts as one pair, flagged confounded.

**Leak check** (`F3-leakcheck.txt`): P passed (none of 10). Plant: R failed
(4 of 4).
