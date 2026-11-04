# Closing reconciliation: grouped-home checkpoint blocks current gates

## Decision requested

Return the unfinished grouped-home prototype to its owner/director for gate
and budget reconciliation, then resume `onboarding-scope-economize` against
that landed baseline. This is not a request to approve scoped-lending's
already-authorized minimum growth. Do not expand that authorization to the
separate presentation prototype.

The owner remained active: after the recorded checks, `git status --short`
again showed edits to `home/page/grouped.js`, `web/tests/grouped.mjs` and
`web/tests/grouped.test.mjs` (the browser driver grew substantially). Those
post-checkpoint edits are preserved for that owner, not staged into this
unit. Counts and failures below describe the recorded checkpoint run,
not the continuously changing working tree.

The user required inherited tracked work to be committed first. Inspected
all inherited diffs and four new source/test files and checkpointed them in
`c3fa0ef7` (`WIP: grouped-home`), preserving their contents. The active commons
claim `ring-32fb4a66f5b1a57a` identifies a separate owner of this work. Neither
V1.6 publish artifact was modified or staged.

## Fresh evidence

`reconcile-fast.txt` contains complete command/output/exit records, run in
the foreground under `lockf -k /tmp/svrngs-gate.lock`. No checks or caps changed.

```text
$ python3 scripts/lean_check.py
crates/svrngs-core     3486 / 3494 lines
.                     19790 / 19833 lines
js                     8174 / 8090 lines
swift                     0 / 1 lines
BROKEN  js has 8174 lines of code, budget 8090: raised only by the operator, with the reason in the commit
exit 1
```

Compared with the survey's `7ad807f1` base: 0 core, +1 non-JS, +108 JS,
entirely inherited grouped-home product growth. This unit still has zero
product growth, cuts and cap changes. Remaining passing non-JS headroom is
42 lines. JS exceeds the maximum passing count by 85 lines; it has no
headroom. Give was not edited; its recorded source count remains 2,669.
The lending estimate remains +400–700 non-JS / +135–235 JS, zero core,
Give separately +10–25. Once the unrelated baseline passes, use its actual
headroom and adjust only necessary lending caps to measured count + 1,
with the required named red/green plants.

Other current results:

- Passed: principle pins, trust floor, advisories (warnings retained),
  vocabulary, all three grouped page-law tests, workspace cargo check.
- Failed: rustfmt at `tests/browser/grouped.rs:10`; formatting in
  `home/page/grouped.js`, `web/tests/grouped.mjs`, `grouped.test.mjs`.
- Failed: complexity 11 over 10 at `home/page/grouped.js:56`.
- Failed: TS18048, possibly undefined `selected`, at `grouped.js:51`;
  typed lint also rejects the nullable number conditional at line 49.
- Never-ran in this reconciliation: browser and full `scripts/check.sh`.
  The known failed constituent checks already prevent a passing full gate.
  Prior passing product evidence is preserved, not attributed to this new base.

## What remains

The assigned survey/prototype work is complete, with the four disabled
transport guards red before green within the 1,250 ms scratch ceiling
(`PROTOTYPE.md`). There were no supported economizing cuts. No new product
policy or gate was introduced in this session.

After the grouped-home owner resolves its failures and budget, remeasure,
update the baseline/headroom in `docs/LENDING_SCOPE.md`, and run current fast
checks and the required locked full gate against the final product tree.
Then close this row. The real-node two-second withdrawal scene belongs to
the subsequent implementation row, not this scratch proof.
