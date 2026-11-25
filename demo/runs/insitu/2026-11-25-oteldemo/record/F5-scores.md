# F5 — intlShippingSlowdown 10sec — "Some customers say checkout takes forever."

In the window, 1 of 26 orders was international and slow: shipping
`/ship-order` max 10,003 ms, p50 0. P: 9 tool calls. R: 5 tool calls,
53 s. Both recognised the shop.

| Point | P (projected) | R (raw) |
|---|---|---|
| Service: shipping | 1 | 1 |
| Mechanism: a fixed 10 s wait in ship-order | 1: "checked a feature flag, then deliberately waited ~10 s". A flag log and a `delay_secs` log (2 digits, shaped) appear only on the slow order | 1: all 10 s inside shipping's handler, "almost exactly 10 s … an injected delay or timeout" |
| Scope: orders to addresses outside the US | **0**: 1 of 26 orders, intermittent | **0**: 1 of 26; noted that user's currency was CAD, and took it no further |
| Evidence | 1 | 1 |
| **Total** | **3/4** | **3/4** |

**Found by P, my bug, both arms:** shipping's (Rust) log records carry no
`timeUnixNano`, so siteview dated them `null` and `/logs` dropped them. P
reached them through `/trace`. A log should take `observedTimeUnixNano`
when it has no time.
**Projection cost seen:** Rust's tracing writes `feature_flag_key`, not
the convention's `feature_flag.key`, so the flag's name was shaped.

**Leak check** (`F5-leakcheck.txt`): P passed (none of 5). Plant: R failed
(5 of 5).
