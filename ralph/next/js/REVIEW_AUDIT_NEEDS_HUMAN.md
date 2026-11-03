# REVIEW-audit-js-auto-1 — required instrument absent

## Decision requested

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

The final scripts/check.sh has not run: this is a blocked partial audit,
not row closure. After restoring the instrument, finish the clone/noun
checks and architectural review, fix small behavior-preserving findings,
commit, then close via the locked gate required by the queue.
