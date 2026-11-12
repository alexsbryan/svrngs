# Check-tier extraction

Contract/ledger written before extraction: [`docs/CHECKS.md`](../../../../docs/CHECKS.md).
This refactors check ownership and selection, not application authority behavior.
All work is uncommitted; the existing creator/HolyArrs/xenia changes are preserved.

## What moved, and what stayed real

- Four pure JavaScript/source contracts moved from the Rust browser target to
  `web/tests/question.test.mjs`. All 15 acts/decider outputs, four first-open
  examples, both pages' source-wiring assertions, alias distinctions, two opposite
  time zones and all nine lookup outcomes remain. UTC dates were independently
  read with `date -u -r` before extraction. Display dates now use a fixed input
  rather than a race with local midnight.
- Two exact writer/reader row expectations now have one golden home in that Node
  contract. The real native workflows still compare actual DOM to the tested
  shared formatter and retain their reach/signing/response/ownership assertions.
- Three original seek-judgment calibration cases moved to `seek_judgment`.
  They consume the same actual Rust instrument and explicit observations. Real
  media playback still uses that instrument in integration.
- Five resolver cases moved from Redis hooks to direct tests of the real handler:
  original phrase/signature vectors, provider failures, invalid ingress, signed
  BOM bytes, and retry guidance. Rejected ingress still asserts zero store calls;
  its test transport fails if touched. No store implementation is imitated.
- Redis/Lua state, all write-boundary faults, concurrency, expiry, counters and
  AOF restart remain `resolver.integration.test.mjs`. CSS content type and actual
  WebKit computed style remain `page-server.integration.test.mjs`.
- The existing isolated `resolver/adapter.test.mjs` is now in the quick catalog;
  the old web-only glob had not included it.

`web/tests/run.mjs` owns unit/integration/apps discovery. Unit and integration
are disjoint and exhaustive over platform Node tests; unknown and empty selections
fail explicitly. Full check.sh retains every tier and the apps' own tests.
`--fast` stops before actual Node integration. The touched planner queries the
same catalog and preserves conservative unknown/shared-instrument reach.

Published build/CSP, real ingress/signing, origin behavior, group replication,
long-lived-use closure, persistent resources and media stay integration checks.
Publication's `#[test]` restarts real processes; it was not moved merely because
its annotation looks synchronous.

## Observed timings

These are individual runs on this checkout, not performance distributions:

| Record | Result / elapsed |
|---|---|
| `baseline-node.json` | Combined Node tests passed, 16.736 s |
| `extracted-node-before-retirement.json` | Moved Node contracts passed before retiring old copies, 0.190 s |
| `extracted-seeks-before-retirement.json` | Original three Rust calibration cases passed at their new target, 1.613 s |
| `final-unit.json` | Complete isolated Node tier passed, 0.652 s |
| `isolated-no-services.json` | Unit tier passed with Redis hidden from PATH and an absent Playwright browser installation, 0.610 s |
| `seek-restored.json` | Warm isolated Rust calibration passed, 1.418 s |
| `node-integration-restored.json` | Actual Redis/WebKit tier passed, 15.822 s |
| `fast-gate.json` | Static checks plus isolated laws passed, 10.902 s |
| `phone-rendering.json` | Changed native phone rendering/write workflow passed, 48.093 s including compilation |
| `reader-rendering.json` | Changed native reader/Stop workflow passed, 89.244 s including compilation |

Native size budgets did not rise: core 3,506, non-JS 21,259 and JS 8,506.
The catalog replaces duplicate shell discovery; the planner's fixed-point closure
needs no separate grew flag. Those savings fund explicit integration reach.

## Controls observed failing

| Record | Actual fault and observation |
|---|---|
| `touched-red.json` | The old selector omitted renamed integration tests/helper/service sources. New plan assertions failed before selector changes. |
| `classifier-red.json` | Letting unit accept integration files made both synthetic and real catalog assertions fail; the unknown/empty positive control still passed. |
| `alias-zone-red.json` | Removing UTC from joined-day formatting made the opposite-zone alias contract fail. |
| `resolver-bom-red.json` | Stripping a signed BOM made the real handler call the forbidden store transport; the migrated zero-call assertion failed. |
| `seek-boundary-red.json` | Changing >5,000 to >=5,000 made the original inclusive-boundary calibration fail. |
| `resolver-reopen-red.json` | The retained Lua reopen plant failed authenticated lifecycle in actual Redis after the file/helper move. |
| `browser-dependency-control.json` | With browser executables absent, the real WebKit test failed explicitly. This is a dependency-absence control, not a behavioral refusal. |

Every source plant was reversed by its exact inverse edit. No checkout/restore
discarded other work. The first integration rerun (`node-integration-green.json`)
caught a missing ownerKey import in the extracted helper: that ReferenceError was
fixed and is not counted as a security/control witness. Restored unit, integration,
calibration and planner checks passed afterward. All 13 planner tests pass,
including the actual standalone calibration target and Node-only service reach.

## Complete gate

**Full gate passed, exit 0 in 1,794.463 s (29 min 54 s).** `full-gate.json`
records the command and input hashes. Its raw log stays at
`target/check-tiers/full-gate.log`. The gate explicitly passed isolated page laws
(1 s), page integration (16 s), all Rust checks/tests, the complete browser suite,
all 18 real scenes, conformance and the apps' tests. The four ancillary WebKit
seek pairs passed in this run too; they remain observations rather than a claimed
performance distribution. `gate.py` executes every step,
verifies the existing hooksPath without rewriting Git config, and uses the same
per-run SSH -4 adjustment to the real Podman VM that the earlier runs required.
No required test tier, refusal assertion or timing/resource limit is removed.

For the next edit, start with `node web/tests/run.mjs unit`, one direct Node law,
or a focused Rust target. Start the relevant actual integration only when its
boundary changes; run the complete gate after focused checks settle.
