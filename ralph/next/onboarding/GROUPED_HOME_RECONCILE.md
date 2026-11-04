# Grouped-home reconciliation — measured budget decision

2026-11-04, `onboarding-grouped-home-reconcile`, base `d206e79e`.

## Ownership and scope

The existing owner, claim `ring-32fb4a66f5b1a57a`, is still actively finishing
the prototype. During this worker's inspection, the owner added DESIGN/PLAN
changes, the economy README, and started the locked full gate. The worker
announced reconciliation in commons claims `ring-0e5b3b1dda186e36` and
`ring-23d132bfbeb02e30`. No handback has been observed. No product file was
edited, staged or checkpointed by this worker. The owner's existing changes
and the two V1.6 publication artifacts are preserved.

`STATE.md:360-379` requires owner coordination, supported savings, no cap raise,
and a committed product tree with a fresh locked full gate exiting 0.
This report is progress evidence, not closure of that row.

## Measured remainder and economy review

Fresh command and actual output (exit 1):

```text
lockf -k /tmp/svrngs-gate.lock python3 scripts/lean_check.py
crates/svrngs-core     3486 / 3494 lines
.                     19790 / 19833 lines
js                     8187 / 8090 lines
swift                     0 / 1 lines
BROKEN  js has 8187 lines of code, budget 8090: raised only by the operator, with the reason in the commit
```

The cap is exclusive: 8,089 is the largest passing JS count. The current tree
therefore needs **98 fewer counted JS lines**, or operator approval of a cap
of **8,188**, subject to a final recount when the owner lands. Other caps need
no increase. The owner's pre-edit count was 8,066: the current addition is
121 JS lines, consuming 23 available lines and leaving the 98-line shortfall.

Inspected current `home/page/grouped.js:11-122`, `home-page.js:88-149`,
`group-read.js:39-60`, and `home.js:33-46`, alongside the owner's inventory in
`demo/runs/economize/2026-11-04-grouped-home/README.md:15-43`:

- The group and home places reuse the original feature components and mutation
  machines. There is no duplicate control drive to delete.
- Hidden groups remain mounted to supply the existing all-group access reads.
  Removing those reads or selecting only one group changes behavior.
- Ordinary links already provide routing/history; there is no additional router,
  preference store or navigation listener to remove.
- B's remaining module owns its distinct navigation, context/refusal messages,
  version-review link, welcome continuation and disclosure composition. Removing
  them changes the requested comparison or recovery behavior. Packing template
  code to fewer lines is not a supported saving.
- The existing welcome/folder opener and changed-version data are reused. No
  duplicate folder API, invitation machine or consent policy was introduced.

**Cuts established by this worker: none.** No speculative saving is credited,
and no cap or gate has changed. This is a bounded review of the actual new
presentation and its integration, not a claim of zero duplication everywhere.

## Fresh affected checks

All commands below ran in the foreground under
`lockf -k /tmp/svrngs-gate.lock`. The six static commands ran sequentially
inside one locked Python `subprocess.run` driver, which printed each exit code.

```text
cargo fmt --all --check                                      EXIT 0
web/node_modules/.bin/oxlint                                 EXIT 0
web/node_modules/.bin/oxfmt --check "**/*.{js,mjs,cjs}"         EXIT 0
  All matched files use the correct format. (294 files)
node --test web/tests/grouped.test.mjs                       EXIT 0
  tests 3; pass 3; fail 0
node scripts/types.mjs                                      EXIT 0
  types: 0 errors in 130 files
node scripts/typed-lint.mjs                                 EXIT 0
  jsconfig.json: 126 files, exit 0
  jsconfig.workers.json: 3 files, exit 0
```

Browser green (exit 0):

```text
lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs-node --test browser grouped_home_keeps_capabilities_and_group_identity -- --nocapture
grouped home: {
  "groupControls": [23, 23, 23],
  "homeControls": {"acts": 6, "join": 2, "lent": 5, "make": 3, "open": 2, "running": 1},
  "navigation": true
}
test grouped::grouped_home_keeps_capabilities_and_group_identity::chromium ... ok
test result: ok. 1 passed; 0 failed; 99 filtered out
```

The existing new-test plant was also exercised without source edits (exit 101):

```text
lockf -k /tmp/svrngs-gate.lock env GROUPED_HOME_PLANT=missing-wifi cargo test -p svrngs-node --test browser grouped_home_keeps_capabilities_and_group_identity -- --nocapture
AssertionError [ERR_ASSERTION]: all original group controls retained
  [missing WiFi opens, program addresses and address-copy controls]
test result: FAILED. 0 passed; 1 failed; 99 filtered out
```

The subsequent unplanted command waited for the owner's full-gate lock and
was terminated by the tool at 600 seconds without producing output. It is
**never-ran**, not a browser failure. The earlier green remains the observed
unplanted result; a red-then-green sequence and the full closing gate are owed.

## Operator fork

1. **Recommended if retaining the requested B surface:** approve only the
   measured grouped-home JS remainder (currently +98 cap, 8,090 to 8,188),
   after owner handback and final recount. Then run the required full locked
   gate on the committed product and address any other failures. This costs
   98 lines beyond the existing cap; it spends none of lending's approval.
2. Keep the cap and ask the owner to establish at least 98 behavior-preserving
   JS savings. None is established here; the completion cost is unknown.
3. Defer or narrow B explicitly. This changes the requested comparison and is
   an operator product decision, not a behavior-preserving economy cut.

No option is applied by this report. The active owner's full-gate log is
`demo/runs/economize/2026-11-04-grouped-home/check.log`; at inspection its
lean step had reproduced the same shortfall and its later steps were running.
