# REVIEW-audit-onboarding-auto-1 — required instrument absent

## Decision requested

Provide/restore the required sovereign-cli executable with a current usable
code corpus, or explicitly revise this row's clone/noun acceptance criteria.
The worker cannot mark those analyses passed from lexical inventory. The
earlier JavaScript queue's focused-review exception was specific to that
row (`ralph/next/js/REVIEW_AUDIT_NEEDS_HUMAN.md:3-14`); this row still
requires the instrument. No acceptance criterion was weakened here.

## Evidence and attempts

Starting HEAD: b463b29b. Assigned requirement:
`ralph/next/onboarding/STATE.md:328`. The principle path named there did
not exist; `AGENTS.md:9-12` names `docs/principles/ARCH_PRINCIPLES.md`.
Both principle pins were read and only that path was corrected in the row.

Actual shell output:

```text
$ ls /Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli target/debug/sovereign-cli
ls: /Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli: No such file or directory
ls: target/debug/sovereign-cli: No such file or directory
exit 1

$ command -v sovereign-cli
[no output; exit 1]
$ command -v svrn
[no output; exit 1]
$ command -v LINT ralph-result
/Users/alexsbryan/dev/commonwealth-ai/scripts/ralph-bin/ralph-result
exit 0
```

All four touched crates and all seven new Rust declaration candidates were
individually attempted via Python subprocess. Each launch returned:

```text
[Errno 2] No such file or directory: 'target/debug/sovereign-cli' (never-ran)
```

Every command and result is pasted in `ralph/REVIEW_FINDINGS.md` under
REVIEW-audit-onboarding-auto-1. No clone result or noun convergence result
exists. No external source, toolchain or index was modified.

## Committed progress

The range is 194aa994..b463b29b, queue creation through the assigned row.
The per-unit ledger and independent endpoint check agree: +1006 raw source
lines net, +1968 test lines net. The source/test classification and explicit
subject-to-unit mapping are recorded with the ledger. A lexical inventory
finds Request at publication.rs:44 and apps/canon/build/src/lib.rs:84;
it is not a convergence verdict. No product fix is claimed.

Current lean, oxlint, oxfmt and git diff checks pass; exact outputs are in
the findings. No new gate was added. Full architecture review and
`lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh` remain owed, along
with the instrument checks and any fixes. No check is left running.

The two pre-existing V1.6 publish artifacts were inspected but left
untracked and untouched, as instructed. publish.json records a running
publication from source 75f9fd3b in /private/tmp/svrngs-release-v1.6; its
log contains build output. This worker did not resume that publication.

## Resume

1. Restore/provide the executable and corpus, or obtain an explicit row
   revision. A path correction alone does not remove instrument checks.
2. Run the four dry-reports and seven noun queries named in the findings;
   validate the instruments on plants before interpreting clean output.
3. Complete the source/principle review, record both sites for findings,
   and commit each small behavior-preserving fix separately.
4. Run the required lint and locked full gate, commit the evidence, then
   report done only once all acceptance criteria are met.
