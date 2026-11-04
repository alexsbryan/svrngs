# Grouped-home B — approved measured JS remainder

The operator approved the requested +98 JS cap on 2026-11-04 after the
grouped-home economy review and reconciliation report. This is separate from
selected-ring lending's minimum-growth preapproval. The grouped-home owner
released claim `ring-32fb4a66f5b1a57a`; its product and other evidence are
left to the reconciliation worker's scoped landing.

No core/non-JS cap changed. The JS cap is exclusive. Actual commands:

```text
$ python3 scripts/lean_check.py  # before the raise, current B input
crates/svrngs-core     3486 / 3494 lines
.                     19790 / 19833 lines
js                     8187 / 8090 lines
swift                     0 / 1 lines
BROKEN  js has 8187 lines of code, budget 8090
exit 1

$ python3 scripts/lean_check.py  # approved cap 8188
crates/svrngs-core     3486 / 3494 lines
.                     19790 / 19833 lines
js                     8187 / 8188 lines
swift                     0 / 1 lines
exit 0

$ python3 scripts/lean_check.py  # planted web/grouped-budget-plant.js
crates/svrngs-core     3486 / 3494 lines
.                     19790 / 19833 lines
js                     8188 / 8188 lines
swift                     0 / 1 lines
BROKEN  js has 8188 lines of code, budget 8188
exit 1
```

The plant was exactly one counted line:
`export const groupedHomeBudgetPlant = true;`. It is removed. The restored
`python3 scripts/lean_check.py` exited 0, reporting core 3,486/3,494,
non-JS 19,790/19,833, JS 8,187/8,188 and Swift 0/1. The original full BROKEN
diagnostics also include the standard operator-raise instruction.

The preserving pass established no deletion: B composes the existing
components/machines, folder opener, native navigation and version data;
removing the remainder drops requested presentation or capability. Its
source inventory and proof are in `README.md`. Budget proof does not replace
the fresh full gate owed on the reconciled product tree.
