# REVIEW-audit-onboarding-auto-1 — instrument preparation

## Supervisor resume context, 2026-11-04

The supervisor has now started the cached model locally: endpoint
`http://127.0.0.1:8080/v1`, model alias `qwen-embedding-0.6b`, initial PID
83108. A real `/v1/embeddings` probe returned a numeric vector of dimension
1024. Runtime log/PID files are in `target/onboarding-audit/`; these are not
committed audit results. Reuse the healthy supervisor-owned service for the
existing ingestor; still validate the actual clone/noun instruments and
their controls. Do not claim those analyses passed from this endpoint probe.

The last director stopped after a rejected home-wide search for an embedding
model. The operator confirms `commonwealth-ai/sovereign/models` is the main
model directory; a bounded check verified
`/Users/alexsbryan/dev/commonwealth-ai/sovereign/models/qwen-embedding-0.6b.gguf`
exists at 1,197,629,632 bytes. `/opt/homebrew/bin/llama-server --version`
reports 10180 (11b068d06). No model download or upstream source change is
needed. Start an owned loopback-only embedding process from this cached
model, verify the existing ingestor's discovery/model configuration and real
embedding output, then continue the required scratch controls and reports.
No home-wide Glob or broad permission change is needed. A loopback probe at
9741 currently refuses connection; that daemon is not a prerequisite when
the existing local embedding server can be started directly.

The audit now follows grouped-home reconciliation and scoped-lending
implementation so it measures the final committed product, not a changing
WIP baseline. It is the one remaining explicit final audit and covers
194aa994 through that final implementation HEAD, including earlier work;
completed auto-2 does not discharge the outstanding original analyses.
No clone/noun, source-review, lint or full-gate criterion is waived. The
earlier evidence below remains historical.

## Director decision, 2026-11-04

**Continue with local instrument preparation as part of the audit.**
CHARTER.md:107-115 permits re-scoping a row that cannot execute while
keeping every check. STATE.md now names the existing developer executable
and includes preparation of fresh local code/SCIP data. The audit stays
unchecked. Clone and noun analyses, their positive controls, the architecture
review, LINT and the locked full closing gate remain required. The prior
JavaScript exception is not extended. No acceptance criterion is waived.

The false premise was that absence of the dispatcher meant absence of its
audit implementation. The developer binary already runs both verbs; a new
analyzer or upstream rebuild is unnecessary to reach them. Source ownership
is visible in commonwealth-ai's
`code/crates/sovereign-cli-dev/src/code_cmd.rs:48-63` (dry-report route),
`src/lib.rs:168-195` (converge route), and `src/main.rs:10-12` (binary entry).
Paths in this paragraph are relative to that crate after the first one.

### Reproduced evidence

Starting HEAD `616c00ea`. `git status --short` showed only the two untracked
V1.6 publish artifacts. `git log --oneline -10` placed the worker's evidence
commit at HEAD. Read the assigned STATE.md row and the worker session
log `ctl/log/sessions/REVIEW-audit-onboarding-auto-1-1.out`; its final lines
record the commit and needs-human result. The original missing paths and
PATH lookup were reproduced; the dispatcher remains absent.

Actual successful commands (exit 0):

```text
$ shasum -a 256 /Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli-dev
8827d9afbd5ac993d9fd7462a6aea8897d4f76a0c157b73a51482ac8200df669  /Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli-dev
```

Using that absolute executable, `code dry-report --help`, `code index
--help` and `refresh --help` each ran, exit 0. Index help offers full
embedding or `--fts-only`; the latter is explicitly not a near-clone
measurement. The sibling `target/debug/svrn-ingest --help` also ran, exit 0.
No index build was run in this director session.

Both `code dry-report --scope crates/svrngs-node/` and `code converge noun
Receipt` ran and refused automatic corpus selection (exit 1). Each printed
the following first line, followed by the 47-corpus inventory:

```text
error: 47 indexed code corpora, none of them was built from this repo (/Users/alexsbryan/dev/svrngs) — pass --corpus-id:
```

This is not proof that every existing corpus is unusable; it is a refusal
to choose one as current. Explicit selection of a new audit id reproduced
the two distinct prerequisites, with actual output:

```text
$ /Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli-dev code dry-report --corpus-id onboarding-audit-616c00ea --scope crates/svrngs-node/
error: no chunk index at /Users/alexsbryan/.svrnmesh/indexes/onboarding-audit-616c00ea/chunks.lance — run `svrn project init` first
exit 1
$ /Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli-dev code converge noun Receipt --corpus-id onboarding-audit-616c00ea
error: no SCIP graph at /Users/alexsbryan/.svrnmesh/indexes/onboarding-audit-616c00ea/scip_graph.db — run `svrn project init` first
exit 1
```

### Worker execution scope

1. Prepare a dedicated local data root and current code corpus with the
   existing developer CLI and ingestor. Read their implementation/help before
   choosing flags. `code_index.rs:26-34,63-225` owns the chunk-index path;
   `code_refresh.rs:69-195` owns local SCIP refresh. Those paths are under
   commonwealth-ai's `code/crates/sovereign-cli-dev/src/`. In particular,
   local refresh derives the corpus name from project config or repository
   basename, not its `--name` argument (`code_refresh.rs:161-182`). Keep
   writer/reader paths aligned rather than assuming that flag sets the name.
2. Use the same isolated `SVRNMESH_DATA_DIR` for reports and preparation;
   report readers resolve `<root>/indexes/<id>` (`dry_report_cmd.rs:86-107`,
   `converge_cmd.rs:241-252`). Do not overwrite an existing user's corpus or
   edit upstream source/configuration. Use only existing local services; a
   missing embedding endpoint/exporter is a concrete preparation failure,
   not permission to use synthetic embeddings or lexical results.
3. Record source provenance and coverage, then validate the instruments on
   separate planted inputs. A help response and the errors above are entry
   point witnesses, not successful audit results or positive controls.
4. Complete all original reports and review through the eventual reviewed
   HEAD. Reconcile any source changes against the committed ledger. Land
   each small preserving fix separately; finish LINT and
   `lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh`, commit evidence,
   then report done only when every requirement is met. Use foreground runs
   or `ralph-result await` for preparation that exceeds the session budget.

Director closing checks: `git diff --check`, `python3 scripts/lean_check.py`,
`web/node_modules/.bin/oxlint` and `web/node_modules/.bin/oxfmt --check
'**/*.{js,mjs,cjs}'` all exited 0. Lean reported core 3486/3494,
`.` 19789/19833, JS 8066/8090 and Swift 0/1. Only queue instructions and
this decision record changed; the full audit gate is still owed by the worker.

## Original worker request and evidence (historical)

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
