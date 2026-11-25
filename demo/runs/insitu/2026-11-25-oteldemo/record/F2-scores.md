# F2 — productCatalogFailure on (product OLJCESPC7Z) — "A customer says one product page shows an error."

P: 10 tool calls. R: 4 tool calls, 37 s. Both recognised the shop.

| Point | P (projected) | R (raw) |
|---|---|---|
| Service: product-catalog | 1 | 1 |
| Mechanism: GetProduct fails INTERNAL for one product, by a flag | 1 (the flag key `productCatalogFailure` and its event passed; the value `true`) | 1 (also variant `on`, reason `targeting_match`) |
| Scope: one product only | 1: "exactly one product, h=b80dfa3cf4; 52/52 error, 9 others all ok", plus recommendations that include it | 1: "OLJCESPC7Z", the same split, plus recommendations |
| Evidence | 1 (tallies by product hash; traces 9a8fff6f, 1eee26e1) | 1 (tallies by product id; trace 9a8fff6f) |
| **Total** | **4/4** | **4/4** |

**Prediction wrong:** I predicted P would lose scope because the product id
is shaped. It didn't: equal values hash alike, so "one product" is one
repeating hash. P can't *name* the product, which a fix needs; diagnosis
didn't.

**Residual seen:** a span event named `Error: Product Catalog Fail Feature
Flag Enabled` passed the name rule (every token a word). It carried no
personal data here, but an event name can be free text. Event names
should be held to a vendor's list, as span names can be, or shaped by
default.

**Leak check** (`F2-leakcheck.txt`): P passed (none of 9). Plant: R failed
(8 of 9; the ninth left R's sampled window between harvest and search).
