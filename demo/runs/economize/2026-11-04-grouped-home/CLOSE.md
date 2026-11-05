# Grouped home B — verification summary

B is implemented behind `/?ui=b`; A remains the default. The approved JS cap
is 8,188 (`BUDGET.md`, commit `11e36265`), with 8,187 actual counted lines.
The capability inventory and complete browser recheck passed as recorded in
`README.md` and `browser-recheck.log` (98 passed, 2 ignored).

Closing command:

```sh
lockf -k /tmp/svrngs-gate.lock scripts/check.sh
```

`closing-check.log` records passes through the `pages` step, including lean,
security gates, lint/format/types, all library tests, canaries and the browser
suite. Library tests took 588 seconds; browser took 163 seconds.

This run is incomplete and has no exit-0 verdict. Its remaining
scenes/conformance/app-law steps are not claimed passed by this run;
those passed in the earlier `check.log` run. The Xenia A/B study remains unrun.
