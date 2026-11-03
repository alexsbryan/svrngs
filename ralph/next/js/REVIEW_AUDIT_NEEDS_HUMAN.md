# REVIEW-audit-js-auto-1 — required instrument absent

## Operator decision, 2026-11-03

Resolved: the operator approved a focused review of the actual JavaScript
changes against the agreed charter, the existing gate evidence, the focused
doc-scene rerun and the tested budget cap, then closing the loop on that
basis. The reply was "Yes that sounds good to me". STATE.md's audit row
now holds that acceptance criterion.

The original semantic-clone and cross-repository noun-convergence analyses
are explicitly deferred, not passed. No further external toolchain repair
or code-index build is part of this queue's closure. The partial ledger
and missing-instrument results below remain historical evidence.

The watching session subsequently restored the developer CLI and ingestor
through commonwealth's existing program-lift run-only checks, without
editing its source. A fresh source-code index was started and interrupted;
it is not an audit result. The proposed uncommitted snapshot helper was
removed when the work was brought back to the operator-approved scope.

## Director disposition, 2026-11-03

Continue under the operator-approved row committed in `54693711`, not
the superseded instrument-restoration request from `e4069050`. No replacement
instrument is selected and neither deferred analysis receives a passing verdict.
The focused review and every acceptance condition in STATE.md remain owed.

Verified in the main tree: `git status --short` was empty;
`git log --oneline -10` placed `54693711` after `e4069050`;
`git show 54693711 -- ralph/next/js/STATE.md` showed the explicit operator
decision and revised acceptance criteria. The worker's session log
`ctl/log/sessions/REVIEW-audit-js-auto-1-1.out:1339-1356` records the partial
commit and original escalation. `git diff --exit-code 54693711 --
ralph/next/js/STATE.md ralph/next/js/REVIEW_AUDIT_NEEDS_HUMAN.md` and
`git diff --check` both exited 0 before this disposition was added.

The next worker reads STATE.md's focused row, reviews the queue's source and
test changes, verifies the prior gate's coverage of the finished tree, runs
all named current fast checks, and updates `ralph/REVIEW_FINDINGS.md` with
the focused conclusions and explicit deferrals. Follow the row's requirement
for new validation if changes or failures invalidate the prior evidence;
commit the completed review before `ralph-result done`.

## Original decision request, resolved above

Restore/provide the sovereign-cli audit executable and a suitable current
corpus, or explicitly select a replacement instrument. The row requires
clone and noun reports; this worker cannot mark those checks passed from
a lexical scan. Changing that requirement would weaken the row's checks.

## Evidence

At HEAD 07f4b7a5, the requirement is ralph/next/js/STATE.md:112.
AGENTS.md:9 names the canonical principles pin at
docs/principles/ARCH_PRINCIPLES.md. The row's docs/ARCH_PRINCIPLES.md does
not exist. This path correction does not change the required checks.

Actual shell runs:

```text
$ target/debug/sovereign-cli code dry-report --scope crates/svrngs-node
zsh:1: no such file or directory: target/debug/sovereign-cli
exit 127

$ /Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli code dry-report --help
zsh:1: no such file or directory: /Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli
exit 127

$ command -v svrn sovereign-cli ralph-result LINT
/Users/alexsbryan/dev/commonwealth-ai/scripts/ralph-bin/ralph-result
exit 1
```

Glob `target/**/sovereign-cli` under
/Users/alexsbryan/dev/commonwealth-ai returned no files. No binary was
built: that would require choosing and preparing the external instrument
and its corpus, beyond a local path correction.

All four touched crates, apps/give/ and web/ were individually attempted
through Python subprocess calls to the specified dry-report executable.
All 17 noun candidates were individually attempted through converge noun.
Every launch returned the actual error:

```text
[Errno 2] No such file or directory: 'target/debug/sovereign-cli'; never-ran
```

## Committed progress and verification

ralph/REVIEW_FINDINGS.md's REVIEW-audit-js-auto-1 section contains the
per-unit ledger over 50dd06f1..07f4b7a5 (193 non-merge commits), its
independent endpoint reconciliation, the noun inventory, duplicate-name
sites and the missing-instrument verdicts. No production code changed.

```text
$ python3 scripts/lean_check.py
crates/svrngs-core     3477 / 3478 lines
.                     18861 / 18862 lines
js                     7338 lines, no cap until the js queue's jubilee
swift                     0 / 1 lines
exit 0

$ npm ci --prefix web
added 11 packages, and audited 12 packages in 1s
found 0 vulnerabilities
exit 0

$ web/node_modules/.bin/oxlint
[no output]
exit 0

$ web/node_modules/.bin/oxfmt --check "**/*.{js,mjs,cjs}"
Checking formatting...
All matched files use the correct format.
Finished in 47ms on 281 files using 12 threads.
exit 0
```

At e4069050 this was a blocked partial audit, not row closure. Its original
tool-specific completion requirement is superseded by the operator's
decision above. The focused closing review remains to be completed and
reported through the loop under STATE.md's revised acceptance criterion.
