# Audit preparation: local embedding service unavailable

## Decision requested

Provide a reachable existing local embedding endpoint/model, or authorize
starting a specified local embedding service; also resolve the failed
TypeScript export and missing app-workspace coverage using the existing
instrument. Keep all acceptance checks. The developer binary is available
and works; the dispatcher path is no longer the blocker.

The director limits preparation to existing local services and requires a
missing endpoint/exporter to be reported as a concrete preparation failure
(`ralph/next/onboarding/REVIEW_AUDIT_NEEDS_HUMAN.md:74-82`). This request
does not seek an FTS substitute, synthetic embeddings or an audit waiver.

## Provenance and commands

Source HEAD: `029d4b847f277df31b40a2add97fd471e3fc4b92`. No product
changes since the prior ledger endpoint `b463b29b`: the actual diff is
`preparation.txt:15-21`. The ledger already in
`ralph/REVIEW_FINDINGS.md:13136-13171` therefore retains source net +1006
and tests net +1968 through this HEAD.

The executable SHA256 identities and upstream checkout HEAD are recorded
in `preparation.txt:5-13`. The upstream HEAD is provenance of the checked
source, not a claim that the binary was built at that commit. The developer
binary hash matches the director's witness.

All commands run from `/Users/alexsbryan/dev/svrngs` with:

```text
SVRNMESH_DATA_DIR=/var/folders/qf/4ntssn_10d598hw_5rjhx3740000gp/T/opencode/onboarding-audit-029d4b84
```

Writers use that root's `indexes` subdirectory, and readers explicitly
select `--corpus-id svrngs`. Local refresh derives `svrngs` from the
repository basename (`code_refresh.rs:171-182`), so its destination agrees
with the chunk writer and report readers. This is a dedicated new root,
not the user's existing `~/.svrnmesh` data. The graph stays local; no graph
database or corpus is committed. `prepare.py` captures commands, combined
stdout/stderr (terminal CR/trailing-space normalization only), exit status
and read-only SQLite coverage. It was run in
the foreground, exit 0; that exit means capture completed, not audit passed.

### Embedding index failed (exit 1)

Actual command, output and backtrace: `preparation.txt:23-71`:

```text
$ /Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli-dev code index /Users/alexsbryan/dev/svrngs --corpus-id svrngs --data-dir /var/folders/qf/4ntssn_10d598hw_5rjhx3740000gp/T/opencode/onboarding-audit-029d4b84/indexes --full
corpus-mcp: endpoint candidate ollama http://localhost:11434/v1 — unavailable: GET http://localhost:11434/v1/models: error sending request for url (http://localhost:11434/v1/models)
corpus-mcp: endpoint candidate llama-server http://localhost:8080/v1 — unavailable: GET http://localhost:8080/v1/models: error sending request for url (http://localhost:8080/v1/models)
corpus-mcp: endpoint candidate oicp daemon http://localhost:9741 — unavailable: GET http://localhost:9741/v1/models: error sending request for url (http://localhost:9741/v1/models)
Error: no embedder; pass --fts-only to build a keyword-only index
exit 1
```

This proves those discovery candidates unavailable during this run, not
that no other local service could exist. No configured model was changed,
service started, model downloaded, remote/paid endpoint used, or lexical
index substituted.

### Local SCIP export partially succeeded; refresh failed (exit 1)

Actual command and complete output: `preparation.txt:73-133`:

```text
$ /Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli-dev refresh --local --data-dir /var/folders/qf/4ntssn_10d598hw_5rjhx3740000gp/T/opencode/onboarding-audit-029d4b84/indexes
    ✓ rust: 28791 symbols, 165110 references
    ⚠ typescript: skipped (export failed)
    ✓ python: 29476 symbols, 136971 references
    ✓ 58267 symbols (+1)
    ✓ 302081 edges (+18)
    ✓ Done in 80 seconds
  Rebuilding LanceDB corpus index: no existing index ... — first build
  ✗ LanceDB rebuild failed: .../svrn-ingest index failed (its reason is above)
exit 1
```

An initial uncaptured refresh had already created the isolated graph;
captured repeats preserved it and added the capture driver's Python
symbols. The final capture follows a display-whitespace normalization fix
to the driver after the staged diff check flagged terminal progress spaces.
Every refresh failed at embedding discovery. The log above
contains the full reason, not just this abbreviated excerpt.

Coverage (`coverage.txt`, raw zero-based graph coordinates):

- All seven new Rust declarations appear in the graph: Call at
  `crates/svrngs-node/tests/home/publication.rs:8`, Resolver at :13,
  Phase/Receipt/Request/Fault at `publication.rs:22,32,44,52`, and Welcome
  at `redeem.rs:160` (`coverage.txt:14-25`). This is graph presence, not
  validated clone/noun detection.
- The app graph contains Give's Rust workspace, but **no**
  `apps/canon/build/src/lib.rs` and hence no Request at its line 84
  (`coverage.txt:27-68`). That source definition was verified by reading
  `apps/canon/build/src/lib.rs:79-84`.
- A separate read-only query, exit 0, returned no rows:
  `SELECT file_path,count(*) FROM symbols WHERE file_path LIKE 'web/%' OR
  file_path LIKE 'resolver/%' OR file_path LIKE 'apps/canon/%' GROUP BY file_path;`
  The graph does not cover the JavaScript welcome/resolver/app sites. No
  clean result over those sites may be inferred from it.
- Python export also includes `target/onboarding-econ/scratch/...`
  (`coverage.txt:70-85`). This is not a tracked-source-only inventory.
  Production query scope excludes `target`, as printed in every noun run.

### Four crate reports failed; seven noun commands ran provisionally

Every command and its output is pasted in `preparation.txt:135-283`.
All four `code dry-report --corpus-id svrngs --scope crates/<crate>/`
commands (`svrngs-core`, `svrngs-node`, `svrngs-runtime`, `svrngs`)
exit 1 with `no chunk index .../indexes/svrngs/chunks.lance`.
Clone analysis is **could-not-judge**, not zero clones.

All seven `code converge noun <Name> --corpus-id svrngs` invocations
exit 0. Their provisional first-party counts are Call 0, Fault 1, Phase 1,
Receipt 1, Request 2, Resolver 0, Welcome 1. Call and Resolver are tests;
the printed production scope excludes them. The repeated noun output is:

```text
noun: Request
definitions (2), first-party production:
  svrngs-node                    crates/svrngs-node/src/iroh.rs:318
  svrngs-node                    crates/svrngs-node/src/publication.rs:43
```

These are raw tool coordinates (one before the actual declaration).
Source inspection shows an axum request alias at `iroh.rs:319` versus the
publication JSON-body struct at `publication.rs:44`: distinct concepts,
no preserving convergence fix established. The tool misses the separately
built House rules Request enum at `apps/canon/build/src/lib.rs:84`.
These are inventory observations only. **No positive controls were run**;
none of these counts is an accepted noun-analysis result. Exact/near-clone
and repeated-noun scratch controls remain never-ran, owed in full.

## Checks and remaining work

Current fast checks, all exit 0:

```text
$ python3 scripts/lean_check.py
crates/svrngs-core     3486 / 3494 lines
.                     19789 / 19833 lines
js                     8066 / 8090 lines
swift                     0 / 1 lines
$ web/node_modules/.bin/oxlint
[no output]
$ web/node_modules/.bin/oxfmt --check '**/*.{js,mjs,cjs}'
Checking formatting...
All matched files use the correct format.
Finished in 39ms on 291 files using 12 threads.
$ git diff --check
[no output]
$ command -v LINT ralph-result
/Users/alexsbryan/dev/commonwealth-ai/scripts/ralph-bin/ralph-result
```

No LINT executable/alias was found by that shell; the concrete lint/format
commands are named rather than counted as an invocation of LINT. Full
source/principle review, instrument controls, accepted clone/noun analyses
and `lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh` remain owed.
The full gate was not started in this blocked preparation checkpoint.
No source fix or new gate is claimed. No checks remain running. The two
pre-existing V1.6 publication artifacts remain untracked and untouched.

Resume with a specified reachable local embedder, repair coverage through
existing refresh/exporter surfaces, validate controls in a separate scratch
corpus, then complete reports/review and the locked full gate through the
eventual reviewed HEAD. Do not mark this row complete from these logs.
