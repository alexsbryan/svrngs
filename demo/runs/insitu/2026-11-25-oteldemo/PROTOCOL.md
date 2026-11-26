# The OTel Demo benchmark: what each run is, written before the first

Pre-registered 2026-11-25, before any agent run. Changes to this file after
the first run are commits of their own, with the reason given.

**The question:** with only the site's projected view (no vendor contract,
the semantic-conventions profile alone), can a fresh agent diagnose real
faults in telemetry nobody shaped for it? And how much worse does it do
than with the raw telemetry?

**The app:** OpenTelemetry Demo 3.1.0 (`vendor-cache/src`, sha256
`8c461c53…4777`), its `compose.yaml` with `compose.site.yaml`, and the
load generator at its default 5 users. The site's collector pipelines
(`otelcol-site.yml`) take OTLP before the demo's own redaction.

**The arms**, each a fresh agent, the same prompt and the same tool
(`view.sh <route>`, GET only, over a svrngs lend), differing only in what
the view serves:
- **P (projected):** `siteview.mjs` in projected mode;
- **R (raw control):** `siteview.mjs --raw`, nothing hidden: what an
  unredacted SaaS receives.

**Each fault:** reset all flags and restart both views from the file's end, so the window holds only this fault (added after F3, whose window still held F2: see F3-scores.md); at least 3 minutes of clean traffic; set the
flag (and `emitRawPii` on, for the leak check); 4 minutes for it to show;
start P and R together, with the complaint. No hint beyond the complaint.

| # | Flag (variant) | The complaint the agents get | Ground truth (the demo's source) |
|---|---|---|---|
| F1 | `paymentFailure` 50% | "Some customers say checkout fails." | payment's charge throws "Payment request failed. Invalid token." for about half of charges (`payment/charge.js:43-48`); failing spans carry `demo.user_context.loyalty_level=gold` |
| F2 | `productCatalogFailure` on | "A customer says one product page shows an error." | product-catalog's GetProduct fails for product `OLJCESPC7Z` only (flag targeting); every other product is fine |
| F3 | `recommendationCacheFailure` on | "The site feels slower than last week." | recommendation's cache grows by about a quarter on each miss (`recommendation_server.py:86-87`): ListRecommendations gets slower and its memory grows |
| F4 | `emailMemoryLeak` 1000x | "Some customers never got their order confirmation." | email pads each confirmation 1000x and keeps it (`email_server.rb:71`); memory climbs to its 100 MB limit, and it is killed and restarted, failing sends meanwhile |
| F5 | `intlShippingSlowdown` 10sec | "Some customers say checkout takes forever." | shipping sleeps 10 s for addresses outside the US (`shipping_service.rs:65-92`); US orders are unaffected |

**Scoring**, by the operator against the ground truth, from each agent's
final report alone. One point each:
1. **Service:** names the service where the fault is.
2. **Mechanism:** names what goes wrong there (the failing operation and
   its kind: an error thrown, one item, a growing cache, memory to a limit,
   a fixed delay).
3. **Scope:** says who or what is affected (a share of charges, one
   product, recommendations, confirmation emails, non-US addresses).
4. **Evidence:** cites telemetry that really shows it, checked against the
   view.
Recorded too: wall time, tool calls, and whatever the agent says it
couldn't see. One run per arm per fault: 5 pairs, not a rate.

**The leak check:** with `emitRawPii` on, the services put raw
`user.email`, card numbers and CVVs on spans. Every route of the P view,
read after each fault, must hold none of them. The marker values are read
from the R view of the same window, so the check doesn't depend on
knowing them in advance. Plant: the same check against R must fail.

**Predictions** (mine, before any run):
- P scores 4/4 on F1 and F5, where error types, span names, status codes
  and durations say it all.
- P loses scope on F2, because the product id is a value the projection
  shapes, so "one product" shows only as one repeating shape.
- F3 and F4 hinge on metrics. P does as well as R if the runtimes export
  memory metrics under code names, and worse if not.
- R scores 4/4 on all five.

## The confidence round (added 2026-11-26, before its first run)

**The claim:** on the fixed projection (`b8d33b6c`), an agent with the
projected view diagnoses these five faults as well as one with raw
telemetry, and no personal value crosses the projection.

**The design:**
- Each fault gets 3 runs per arm. Runs already made on the fixed
  projection count; P has one each for F1, F2, F3b and F5, and R has one
  each for F1, F2, F3b, F4 and F5 (R's view never changed). R's F4 from
  the first round counts.
- F1, F2, F3b and F5 get 2 more per arm, and F4 gets 3 P and 2 R. All
  runs for one fault start together, in the same window.
- **Second protocol fix:** before each fault the shop's services restart,
  so no cumulative counter carries an earlier fault. The rerun of F4
  found F1's failed charges in checkout's counters. The collector,
  flagd and the stores keep running.
- F4 waits after its 4 minutes until email has restarted once, then up to
  4 minutes more, so the fault has shown.
- The scoring is PROTOCOL.md's rubric, by me, from each report alone.

**Decision rule:**
- **Confident** if, on every fault, P's mean is at least R's mean minus
  0.5, and the leak check passes in every window with its plant failing.
- **Not confident**, with the faults named, if either fails.
- With 3 runs per cell this shows a large gap, not a small one, and the
  report will say so.
