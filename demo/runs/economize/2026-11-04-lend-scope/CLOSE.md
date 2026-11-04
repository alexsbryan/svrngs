# Economizing pass: closing evidence

Source survey committed first as `0f2a2e02`. No supported cut followed it;
the prototype and evidence are recorded-run drivers, excluded by the
existing lean rules, not moved product implementation or tests. No production
policy, test, dependency or cap is changed by this unit.

## Row check

```text
$ python3 demo/runs/economize/2026-11-04-lend-scope/measure.py survey
core 3486 / 3494; non-JS 19789 / 19833; JS 8066 / 8090; Swift 0 / 1
lean exit 0; exact-window inventory exit 0
Give Rust src, separate same-rule measurement: 2669

$ python3 demo/runs/economize/2026-11-04-lend-scope/measure.py prototype
body guard disabled: exit 101 (1250 ms closure assertion)
splice guard disabled: exit 101 (1250 ms EOF assertion)
raw TCP guard disabled: exit 101 (1250 ms child EOF assertion)
UDP guard disabled: exit 101 (1250 ms child-drop assertion)
all guards enabled: exit 0
```

Full pasted observations and run commands: `PROTOTYPE.md` and
`prototype-{body,splice,raw,udp,none}.txt`. The source is the exact
`prototype.rs` executed in scratch. Tested local ceiling 1,250 ms, worst
green 982.261 ms; real-node two-second scope withdrawal remains the next
row's proof, including real WebSocket upgrade and iroh isolation.

```text
$ lockf -k /tmp/svrngs-gate.lock python3 demo/runs/economize/2026-11-04-lend-scope/measure.py fast
lean-after: exit 0; 3486 / 3494, 19789 / 19833, 8066 / 8090, 0 / 1
principles: exit 0
trust: exit 0; 10 policies, 11 listeners, 9 gates
advisories: exit 0, both workspaces (warnings retained in advisories.txt)
vocab: exit 0; 89 app nouns, 0 in infra code
rustfmt: exit 0
lint: exit 0
format: exit 0; 291 files
types: exit 0; 0 errors in 129 files
typed lint: exit 0; 125 files plus 3 worker files
page laws: exit 0; 138 tests, 138 passed, 0 failed/skipped
workspace: exit 0; cargo check --workspace --offline
unchanged-product (whole subjects including tests): exit 1
```

Each named `.txt` contains the actual command, complete output and exit;
trailing whitespace from terminal output is stripped by the recorder.
This summary does **not** label the entire driver invocation a pass. Its
last comparison detected a concurrent, unrelated browser-test edit:
`crates/svrngs-node/tests/browser/main.rs` added `mod grouped;`. New
`crates/svrngs-node/tests/browser/grouped.rs` and `web/tests/grouped.mjs`
also appeared during the session; these were absent in the starting tree.
They are not claimed, changed, staged or tested as this unit's work.

The separate `measure.py close` run records:

```text
$ git diff --exit-code 7ad807f1 -- crates web resolver scripts apps ':!crates/**/tests/**' ':!web/tests/**'
(no output)
exit 0
```

That is evidence of unchanged **tracked production**, not an assertion of
an unchanged test tree or a relaxation of a production gate. It is retained
alongside the failing whole-subject comparison, not in its place.

The assigned row explicitly permits recorded product evidence plus current
fast checks for a document-only survey. Its unchanged product's last full
gate was `lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh`, exit 0 in
1,269 s, recorded by `da503bac` at product base `e625f27b`; the full pasted
verdicts are retained in `prior-full-gate.txt` (`git show da503bac --
ralph/REVIEW_FINDINGS.md`). This session did not rerun that full gate or a
browser run. A new browser-test tree needs its own validation by its owner.

## Actual economy and handoff

- Cuts **0**, core growth **0**, non-JS growth **0**, JS growth **0**,
  Give growth **0**, caps changed **0**. Counts before/after identical.
- Necessary remaining implementation: **+400–700 non-JS / +135–235 JS**,
  Give separately **+10–25**, core **0**. `SURVEY.md` gives slices/sites,
  full mutation serialization, migration, caller/version fence and witness.
- Use the 43 non-JS / 23 JS lines available first. Each actual required
  increment raises only to `actual count + 1`, with its own named failing
  budget plant and passing recount. No budget plant is claimed here because
  no budget check or cap changes here. No redundant operator approval stop.
- Documentation-only pass proof is complete. Final `done` bookkeeping must
  wait until the concurrent owner lands/checkpoints its tracked browser edits;
  do not stage or discard them to clear this row. The pre-existing V1.6
  publish artifacts are still untouched and remain pre-session untracked.

## Post-commit concurrent-tree checkpoint

After `d5dcfb63`, the final foreground recount found the other work had
expanded from browser tests into production. Actual command/output:

```text
$ git status --short
 M crates/svrngs-node/src/home.rs
 M crates/svrngs-node/src/home/home.js
 M crates/svrngs-node/src/home/look.css
 M crates/svrngs-node/src/home/page/acting.js
 M crates/svrngs-node/src/home/page/group-read.js
 M crates/svrngs-node/src/home/page/group.js
 M crates/svrngs-node/src/home/page/home-page.js
 M crates/svrngs-node/src/home/page/join.js
 M crates/svrngs-node/src/home/page/make.js
 M crates/svrngs-node/tests/browser/main.rs
?? crates/svrngs-node/src/home/page/grouped.js
?? crates/svrngs-node/tests/browser/grouped.rs
?? demo/runs/v1.6/2026-11-03/publish.json
?? demo/runs/v1.6/2026-11-03/publish.log
?? web/tests/grouped.mjs
$ git diff --check
(no output; exit 0)
$ python3 scripts/lean_check.py
crates/svrngs-core     3486 / 3494 lines
.                     19790 / 19833 lines
js                     8156 / 8090 lines
swift                     0 / 1 lines
BROKEN  js has 8156 lines of code, budget 8090: raised only by the operator, with the reason in the commit
exit 1
```

The earlier passing measurements remain evidence for `7ad807f1` plus this
unit's documentation, not for this newer dirty product tree. This unit owns
none of the home/browser changes above. Continue after their owner commits
or checkpoints them: remeasure the new base, reconcile the estimate's
baseline and cap headroom, and run the required checks before reporting done.
Do not treat this unrelated +90 JS growth as selected-lending preapproval,
and do not change a cap to make someone else's unfinished work pass.

## Resumed checkpoint reconciliation

Inherited grouped-home files were inspected and checkpointed in `c3fa0ef7`
as required by the next session's instructions. The new locked fast-check
transcript is `reconcile-fast.txt`; `RECONCILE.md` records its failures,
fresh counts/headroom and the owner/director decision needed to resume.
The earlier passing fast checks do not describe this newer product baseline.
