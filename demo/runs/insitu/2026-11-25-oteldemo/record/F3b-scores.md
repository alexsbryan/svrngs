# F3b — recommendationCacheFailure on, rerun with fresh views — "The site feels slower than last week."

The window held no earlier fault (product-catalog: 0 errors). P: 6 tool
calls, 59 s. R: 7 tool calls, 59 s. Both recognised the shop.

| Point | P (projected) | R (raw) |
|---|---|---|
| Service: recommendation | 1 | 1 |
| Mechanism: the cache grows on each miss; calls slow; memory; restarts | 1: "cache growing without limit", read from the **shape length** of `demo.product.count` (2 digits → 8 digits), with latency and two grow-crash-restart cycles | 1: the count itself (10 → 16,181,115), latency, `process.memory.usage` 66.9 → 88.9 MB, and restarts |
| Scope: recommendations | 1 | 1 |
| Evidence | 1 | 1 |
| **Total** | **4/4** | **4/4** |

**Two findings about the projection:**
1. **A shape's length is a number's magnitude.** P diagnosed the leak
   from `demo.product.count` growing 2 → 8 digits, with the value shaped.
   That's useful here, and leakage for a sensitive number (an amount, a
   date of birth). The shape channel, observed in use.
2. **Metric attribute values are shaped where the profile doesn't know
   the key** (`type: user/system/involuntary` on `process.cpu.time`), so P
   sees series it can't tell apart. P also said "no memory metric", though
   `process.memory.usage` was in its view. That was the agent missing it,
   made easier by shaped series labels. The profile needs the semantic
   conventions' metric state and type enums.

**Leak check** (`F3b-leakcheck.txt`): P passed (none of 6). Plant: R failed
(4 of 4).
