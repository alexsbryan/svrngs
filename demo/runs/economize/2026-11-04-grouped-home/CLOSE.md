# Grouped home B — reconciliation close

B is implemented behind `/?ui=b`; A remains the default. The approved JS cap
is 8,188 (`BUDGET.md`, commit `11e36265`), with 8,187 actual counted lines.
The capability inventory and complete browser recheck passed as recorded in
`README.md` and `browser-recheck.log` (98 passed, 2 ignored).

This intermediate close runs the focused affected checks only, per the
operator's demo-deadline policy (`SCOPE_PARALLEL.md`). It does **not** run a
full `scripts/check.sh`; the one integrated locked full gate belongs to the
final audit after the backend/UI lanes merge. The prior `closing-check.log`
passed through the `pages` step (lean, security gates, lint/format/types,
library tests 588 s, canaries, browser 163 s) but is incomplete and has no
exit-0 verdict. Its remaining scenes/conformance/app-law steps passed in the
earlier `check.log` run. The Xenia A/B study remains unrun.

## Fresh focused checks, 2026-11-04 (reconcile)

`python3 scripts/lean_check.py` (exit 0):

```text
crates/svrngs-core     3486 / 3494 lines
.                     19795 / 19833 lines
js                     8187 / 8188 lines
swift                     0 / 1 lines
```

The non-JS count is 19,795 (was 19,790 at the handback) because `dbfa37a8`
added five behavior-preserving budget-layout lines to `scripts/lean_check.py`;
no cap value changed. JS is one line below its exclusive cap.

Static affected checks, all exit 0:

```text
cargo fmt --all --check                          exit 0
web/node_modules/.bin/oxlint                     exit 0
web/node_modules/.bin/oxfmt --check "**/*.{js,mjs,cjs}"  exit 0 (294 files)
node --test web/tests/grouped.test.mjs           exit 0 (tests 3; pass 3; fail 0)
node scripts/types.mjs                           exit 0 (130 files)
node scripts/typed-lint.mjs                      exit 0 (126 + 3 files)
```

Focused browser proof under `lockf -k /tmp/svrngs-gate.lock`, red first:

```text
$ GROUPED_HOME_PLANT=missing-wifi cargo test -p svrngs-node --test browser \
    grouped_home_keeps_capabilities_and_group_identity -- --nocapture
AssertionError [ERR_ASSERTION]: all original group controls retained
test result: FAILED. 0 passed; 1 failed; 99 filtered out

$ cargo test -p svrngs-node --test browser \
    grouped_home_keeps_capabilities_and_group_identity -- --nocapture
grouped home: {
  "groupControls": [23, 23, 23],
  "homeControls": {"acts": 6, "join": 2, "lent": 5, "make": 3, "open": 2, "running": 1},
  "navigation": true
}
test result: ok. 1 passed; 0 failed; 99 filtered out
```

The named plant removes only B's presentation module in the browser, omitting
WiFi/program-address controls; A's inventory is unchanged and the comparison
fails on the missing capability. Without the plant the same gate passes. No
product source was edited.

## Integration handoff

Pending other-lane integration, named honestly: the backend/UI API merge, the
real-node House/Trip denial and bounded-withdrawal scene, and the single locked
full `scripts/check.sh` on the merged product all belong to the final audit
(`REVIEW-audit-onboarding-auto-1`). This close claims no passing full product
gate. No deployment, publish or push.
