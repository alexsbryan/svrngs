# The OTel Demo benchmark: diagnosis through the projection, against raw telemetry — 2026-11-25

**The question** (`PROTOCOL.md`, committed before any run): with only the
site's projected view, no vendor contract, and the semantic-conventions
profile alone, can a fresh agent diagnose real faults in telemetry
nobody shaped for it? How much worse does it do than with raw telemetry?

**The app** is OpenTelemetry Demo 3.1.0. I didn't write it, its
instrumentation or its faults. It ran under `compose.site.yaml`, with
the site's collector pipelines ahead of the demo's own redaction.

**The arms** are two fresh agents per fault, the same prompt
(`record/PROMPT.md`), the same tool over a svrngs lend. The only
difference is what the view serves: **P** projected, **R** raw.

## Result

| Fault | P | R | What decided it |
|---|---|---|---|
| F1 payment fails ~50% | 3/4 | 3/4 | Both read the "gold" label the demo writes *on* failure as the cause |
| F2 one product fails | 4/4 | 4/4 | Equal hashes kept "one product" for P |
| F3 cache grows | *4/4* | *0/4* | **Confounded**: the window still held F2 (my protocol flaw) |
| F3b same, rerun clean | 4/4 | 4/4 | P read the cache's growth from a shape's *length* |
| F4 email memory leak | 2/4 | 3/4 | No memory metric in either arm; R committed to email, P declined |
| F5 non-US shipping 10 s | 3/4 | 3/4 | One international order in the window; neither saw "international" |
| **Unconfounded total** | **16/20** | **17/20** | |

**Leak check:** `emitRawPii` was on in every fault, so services put raw
emails, card numbers and CVVs on spans. P passed all 6 windows: none of
the 3–24 personal values harvested from R's same window appeared
anywhere in P's routes. The plant (the same search in R) failed all 6.
R's F4 report names customers' email addresses; no P report has any.

**One run per arm per fault, so these are not rates** (AGENTS.md 7). The
1-point gap is F4, where both arms had almost nothing to go on, and R's
agent committed where P's held back with the same durations in view.
That's within what agents vary by. The measured cost of the projection
on these five faults is small, and the leak protection held throughout.

### My predictions, checked

| Predicted | Happened |
|---|---|
| P 4/4 on F1 and F5 | 3/4 on each: scope missed, in both arms alike |
| P loses scope on F2 (the product id is shaped) | **Wrong**: equal hashes said "one product" |
| F3/F4 hinge on metrics | F3b: P needed none (a shape's length). F4: the service exports no memory metric at all |
| R 4/4 on all five | 17/20: misleading telemetry (F1) and a thin window (F5) cost R the same as P |

## What the run taught the projection (each a next change)

1. **A shape's length is a number's magnitude.** P diagnosed F3b from
   `demo.product.count` growing from 2 to 8 digits. That's the signal
   here, and leakage for an amount or a date of birth. Lengths need
   bucketing, or a contract that types numbers.
2. **Event names can be free text.** `Error: Product Catalog Fail Feature
   Flag Enabled` passed the name rule, every token being a word. Hold
   event names to a vendor's list, or shape them by default.
3. **Metric attribute values the profile doesn't know are shaped**
   (`type: user/system`). P saw series it couldn't tell apart. Add the
   conventions' state and type enums.
4. **Conventions written another way are shaped:** Rust's
   `feature_flag_key` versus `feature_flag.key`. Map the common
   spellings, or let the vendor's contract name its own keys.

## What went wrong in the run, and how each was caught

- **Setup** (all before the first fault, `4ec5e6a6`):
  - 3.1.0's `.env` runs `latest` images, so they're pinned.
  - The collector won't start without the socket it was denied.
  - Moving the proxy's port broke every caller's address.
  - The view dropped every metric. Each was seen in the telemetry or the
    logs, then fixed.
- **The leak checker's own false alarm.** It first matched
  `server.address`, which names services, so it now takes only keys
  about a person. That was before the first fault.
- **The window confound.** R diagnosed F2's leftovers in F3, because
  30-minute windows met faults 9 minutes apart. The fix, fresh views at
  each reset, landed as its own commit (`f27f1f29`) before F3b.
- **Logs dated null.** Shipping's Rust logs have no `timeUnixNano`, so
  `/logs` dropped them for both arms. P found it in F5. A log should take
  `observedTimeUnixNano`.

## Threats to validity

- **Every agent recognised the demo** and said so, as the prompt asked.
  The scores rest on cited telemetry, and both arms share the bias, but
  agents might know the demo's faults by name.
- **One run per cell, and a 30-minute window** with 3 minutes of clean
  traffic before each fault.
- **Faults on a synthetic load** of 5 users. F5's scope needed more
  international orders than the load made.
- **Scoring is mine,** against ground truth written before the runs.

## Run it

    podman-compose …   # via ./demo.sh up (vendor-cache/ holds the pinned 3.1.0 source)
    ./lab.sh up        # the site and vendor svrngs nodes, both lends, Play on
    ./fault.sh <flag> <variant>   # PROTOCOL.md's steps; then the two agents with record/PROMPT.md
    node leakcheck.mjs && node leakcheck.mjs --plant
    ./lab.sh down && ./demo.sh down
