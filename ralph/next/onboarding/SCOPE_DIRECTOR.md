# Director decision: reconcile grouped-home before lending close

2026-11-04, `onboarding-scope-economize`, director attempt 1.

## Decision

Accept the worker's ownership boundary. Split the inherited-product closure
into `onboarding-grouped-home-reconcile`, immediately before and a dependency
of `onboarding-scope-economize`; correct the source order in PLAN §1 too.
Return that closure to the active grouped-home owner, rather than sending
another lending worker around the same unfinished checkpoint. This is queue
repair under CHARTER §Director, not approval of presentation growth.

The prerequisite must establish a landed passing product baseline, including
its full locked gate. It first seeks supported behavior-preserving savings.
No cap raise is authorized here. A remaining grouped-home budget shortfall
after its own economizing pass goes to the operator with options and costs,
as CHARTER §Director requires. Do not assume savings exist or decide to
discard the opt-in prototype for its owner.

Then the lending worker remeasures that baseline and current lending seams,
updates LENDING_SCOPE's headroom and finishes its existing close checks.
Its already-recorded scratch proof remains historical evidence; the
real-node withdrawal proof remains in implementation. All existing checks,
guardrails and the operator's lending-only approval stay required.

## Reproduced evidence

Inspected both requested session logs, including their closing commands:

- `ctl/log/sessions/onboarding-scope-economize-1.out:1600-1676` records the
  first concurrent-tree blocker and continuation.
- `ctl/log/sessions/onboarding-scope-economize-2.out:448-634` records the
  checkpoint failures and reconciliation escalation. Those failures describe
  that checkpoint, not all of today's working tree.

`git log --oneline -12` identifies HEAD `1403f7b6`, after `c3fa0ef7`
(`WIP: grouped-home`), the survey `0f2a2e02` and scratch proof `d5dcfb63`.
`git show --stat c3fa0ef7` confirms the 14-file grouped-home checkpoint.
`git diff --numstat 7ad807f1..HEAD -- crates scripts web resolver apps/give`
lists only grouped-home source/tests; there is no committed lending product
or cap change in that range. `git status --short` still shows active edits:

```text
 M crates/svrngs-node/src/home/page/grouped.js
 M web/tests/grouped.mjs
 M web/tests/grouped.test.mjs
?? demo/runs/v1.6/2026-11-03/publish.json
?? demo/runs/v1.6/2026-11-03/publish.log
```

`commons_work crates/svrngs-node/src/home/` reports active owner claim
`ring-32fb4a66f5b1a57a`, intent: opt-in B group overview/workspace/computer
navigation preserving A controls and capability behavior, with browser parity.
The queue correction is the handback; the director edits no product files.

Fresh foreground reproduction held `/tmp/svrngs-gate.lock` throughout all
six commands, using `lockf -k /tmp/svrngs-gate.lock python3 -c` with
`subprocess.run` for each command and its printed return code:

| Command | Actual result |
|---|---|
| `python3 scripts/lean_check.py` | exit 1: core 3486/3494; non-JS 19790/19833; JS 8183/8090; Swift 0/1; `BROKEN js has 8183 lines of code, budget 8090: raised only by the operator, with the reason in the commit` |
| `cargo fmt --all --check` | exit 1: `tests/browser/grouped.rs:10`, expands the one-line `drive(...).await` call |
| `web/node_modules/.bin/oxlint` | exit 0, no diagnostics |
| `web/node_modules/.bin/oxfmt --check "**/*.{js,mjs,cjs}"` | exit 1: `web/tests/grouped.mjs`, one file needs formatting |
| `node scripts/types.mjs` | exit 0: `types: 0 errors in 130 files` |
| `node scripts/typed-lint.mjs` | exit 0: jsconfig.json 126 files; jsconfig.workers.json 3 files |

At this observation JS needs 94 fewer counted lines to pass the exclusive
8090 cap (maximum 8089). This is a shortfall, not a claim that such cuts are
available. The owner has already corrected the checkpoint's complexity and
type errors. Browser/full gate are **never-ran by this director**; no passing
product verdict is inferred. The moving working tree must be measured again
when its owner lands. No gates or budgets were edited.

## Worker handoff

Run the new prerequisite first, coordinating with the active owner. Finish
its measured budget/gate reconciliation and full locked gate before closing
it. If a separate budget decision proves necessary, stop for the operator
there, with the completed economizing evidence. Once it lands, resume
`onboarding-scope-economize` as specified in STATE; do not reimplement the
survey or spend lending's incremental allowance on the prerequisite.
