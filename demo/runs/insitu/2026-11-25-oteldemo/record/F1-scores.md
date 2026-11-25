# F1 — paymentFailure 50% — "Some customers say checkout fails."

Agents started 2026-11-25T22:44:5xZ, together. P: 7 tool calls, 59 s.
R: 8 tool calls, 65 s. Both recognised the shop as the OTel Astronomy Shop
and said so, as the prompt asked.

| Point | P (projected) | R (raw) |
|---|---|---|
| Service: payment | 1 | 1 |
| Mechanism: charge throws (charge.js:48), surfacing as checkout PlaceOrder INTERNAL → frontend 422 | 1 (exception type, code location and chain; the message only as a shape) | 1 (also read the message: "Payment request failed. Invalid token…") |
| Scope: about half of charges, at random | **0**: "every charge of one loyalty level (h c0ec56c85f) fails" | **0**: "gold-tier customers; 8/8 gold failed, 0 non-gold" |
| Evidence that really shows it | 1 (/services 20 calls / 8 errors; /errors; trace 7fc7a4b3) | 1 (same, plus counts by loyalty level before and after onset) |
| **Total** | **3/4** | **3/4** |

**The shared miss.** The demo writes `demo.user_context.loyalty_level=gold`
onto a charge span *because* it is about to fail (`payment/charge.js:46`),
after the level was set at random. So every failed span reads gold, and
successes keep their own levels. Both arms took the label for the cause.
R had the text "…loyalty_level=gold" in the message as well, which did not
help it. Neither arm could have told the two apart without the code. That
is telemetry that misleads, not a cost of the projection.

**What P said it couldn't see:** the error message and stack trace text
(shapes only), the loyalty level's value (a 4-letter lowercase shape), and
which flag payment evaluated.
**What R said it couldn't see:** which flag was evaluated and its value,
and the code at charge.js:48.

**Leak check** (`F1-leakcheck.txt`): P passed (none of 24 personal values,
6 of them emails). The plant failed (R: 25 of 25).
