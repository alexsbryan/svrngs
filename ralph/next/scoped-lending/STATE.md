# scoped lending — finish the onboarding queue's backend, merge its UI, one audit

Moved out of ralph/next/onboarding (operator 2026-11-05: "Finish scoped-lending
first", before main is pushed; the onboarding queue closed on its scene).
Contract, preapprovals and budget: ralph/next/onboarding/CHARTER.md and the
rows below, unchanged. The backend WIP is on main at 2547d91c with four
uncommitted test edits in ~/dev/svrngs (lend declarations naming `groups`).
The UI lane is done on branch ralph/onboarding-scope-ui (dd0ee048), unmerged.
Run by a strong-model worker, reviewed by the seat; no push until the audit's
full gate exits 0.

## Rows

- [ ] onboarding-scoped-lending — depends [onboarding-scope-economize] — BACKEND LANE, implement selected-ring lender enforcement from `docs/LENDING_SCOPE.md` and `SCOPE_PARALLEL.md`; minimal necessary actual growth preapproved, zero additional core product lines:
  - **Parallel ownership.** Own Rust policy/storage/CLI/HTTP/OIDC/raw code,
    wasm `current_version()` and Give's known-group Rust callers. UI is being
    built independently in `onboarding-scope-ui`; use the frozen JSON/version
    contract in `SCOPE_PARALLEL.md`, not a different shape. Do not edit UI JS,
    its type declarations, browser registration or the home served-module
    table. Those belong to the UI lane. Adjust only the non-JS budget rung;
    the total original growth guardrails apply across both lanes.
  - **Policy and migration.** Persist explicit stable group IDs on the
    existing lend declaration; no selection means no group access. New
    joins never widen an existing audience. Pause/stamp legacy declarations
    until explicit selection, fail closed on migration/read errors and keep
    old binaries from silently ignoring scoped grants. Preserve carried
    apps' intrinsic group ownership and the existing `Offered.group` marker.
    Audience-only edits preserve secrets, OIDC client and choices; serialize
    the entire read/modify/write across audience, limit and sign-in writers.
  - **One authorization.** Resolve the verified device/pass through an
    allowed shared-group membership witness and carry that identity through
    the decision. Handle overlapping groups and conflicting person keys
    explicitly. Enforce before HTTP forwarding/OIDC authorize, raw admission,
    choices and secret injection. Scope OIDC code provenance/redemption,
    discovery (including relayed lists), app reads, append context and page
    headers. Filter disclosure to appropriate shared groups. Add the small
    mixed-version discovery fence; a response-version bump alone is not
    enough because old clients accept newer messages. The fence is not an
    authorization credential. Keep existing reach-pass signatures.
  - **Withdrawal and interfaces.** Implement the fresh pass's measured
    bounded cut of affected HTTP bodies/WebSockets and raw connections;
    never close the whole shared transport or unrelated traffic. Keep owner
    administration and deliberate WiFi delegation explicit. Implement the
    existing CLI and home endpoints for explicit selections/audience-only
    edits, with current name/ID disambiguation, for the parallel UI lane.
    Update Give's known-group share call sites and current scripts. Do not
    silently reset credentials on a scope edit.
  - **Red-first proof.** House-only member succeeds; Trip-only member can
    neither discover nor contact the upstream lend. Try direct/copied URLs,
    stale passes, forged group headers, OIDC, TCP and UDP; prove absence of
    upstream contact. Cover overlap/order/identity, duplicate group names,
    joining a third group, current membership, concurrent narrowing versus
    other edits, restart, legacy/malformed scope and both old/new protocol
    directions. Prove active-stream withdrawal within the measured bound
    while another lend and group sync stay connected. Recheck carried apps,
    guests, app reads/writes, owner access, WiFi/Play and Give. Plant each
    guard's old broad behavior before the passing run; use existing harnesses.
  - **Landing.** Behavior-preserving moves and each behavior change are
    separate commits within this row. Update DESIGN in place with the
    implemented policy, migration and exact withdrawal semantics. Recount
    after each part. Only raise a cap when required, one past the actual
    count, under CHARTER's preapproval and with the fresh pass/plant cited;
    no up-front upper-bound raise and no core growth. Close with meaningful
    focused Rust/CLI/OIDC/raw/migration/Give checks and real-node House/Trip
    evidence under `SCOPE_PARALLEL.md`; no intermediate full check.sh. The
    final audit integrates UI and owns that one full gate. Report pending
    cross-lane integration honestly. No live personal-data migration, public
    deployment, service spend or push as a side effect of this row.
- [ ] merge-scope-ui — depends [onboarding-scoped-lending] — merge ralph/onboarding-scope-ui into main against the frozen JSON/version contract in SCOPE_PARALLEL.md; resolve conflicts behaviour-preserving; focused UI/page-law/types checks.
- [ ] REVIEW-audit-onboarding-auto-1 — depends [merge-scope-ui] — final integrated audit of this queue's implementation range, 194aa994 through the merged backend/UI HEAD, against `docs/principles/ARCH_PRINCIPLES.md`; REUSE AND SIZE, WITH DATA — read: `docs/principles/ARCH_PRINCIPLES.md` — check: LINT
  - **Demo-deadline integration owner.** Read `SCOPE_PARALLEL.md`. Integrate
    Check the separate onboarding-ui worktree's queue/DONE marker and clean
    committed branch as specified there, then merge that finished branch.
    Use the runner's `status --workdir <ui-worktree> --queue onboarding-ui`
    and `git -C <ui-worktree>`/the shared object store for that inspection;
    do not trigger an external-directory permission failure with a broad
    filesystem scan. The supervisor has prepared a loopback embedding server
    at `http://127.0.0.1:8080/v1`, model `qwen-embedding-0.6b`, with a live
    numeric 1024-dimensional embedding probe. Reuse it if healthy; the
    supervisor owns its process (initial PID 83108). It needs no paid
    service or model download. Verify current readiness rather than assuming
    its historical PID is still alive, and do not stop an unrelated process.
    Do not merge an actively edited UI branch. Integrate
    both workstreams' exact API shape, wasm types/version header, UI served modules,
    Give callers, docs and combined counts. Apply only necessary remaining
    preapproved cap increments on the merged tree, with its evidence. Verify
    real-node House/Trip denial and bounded withdrawal, preserve unrelated
    traffic, then own the one final locked full gate. Intermediate rows use
    focused checks; do not re-run full gates solely because a row/commit ends.
    Prepare matching local demo artifacts from the gated product, recording
    hashes and reusing that gate for unchanged product bytes.
  - **Completion sequencing, 2026-11-04.** This existing row is the one
    remaining final audit, after implementation and before the people scene.
    Keep every original ledger, instrument/control, clone/noun, source-review,
    lint and locked-gate requirement. Cover the full implementation range,
    including work previously reported by completed auto-2; do not limit
    coverage to the latest audit's hash. Auto-2 remains historical evidence,
    not an excuse to skip auto-1's outstanding checks. No new cadence audit
    rows are needed. Finish the current product work first so the audit
    instruments are prepared once for the final reviewed tree.
  - **Supervisor resume context, 2026-11-04.** The last director ended on a
    rejected home-wide model search, not a completed analysis. The operator
    confirms main models live under `commonwealth-ai/sovereign/models`.
    The existing embedding model is
    `/Users/alexsbryan/dev/commonwealth-ai/sovereign/models/qwen-embedding-0.6b.gguf`
    (verified present, 1,197,629,632 bytes). `llama-server` is installed at
    `/opt/homebrew/bin/llama-server`. Use that existing model to start an
    owned loopback-only embedding process (for example `--embeddings
    --pooling last --host 127.0.0.1 --port 8080`); verify real embeddings and
    compatibility with the existing ingestor before the required controls.
    Stop only the process you started. No download, paid service or broad
    home-directory Glob is needed. Keep preparation in the isolated corpus
    already described below and read existing exporter controls for coverage.
    Reconciled grouped-home and scoped lending are prerequisites so the
    corpus/gate measure their committed product, rather than a changing tree.
    Preserve all audit requirements; report results through `ralph-result`
    rather than ending on a rejected tool call.
  - **(0) Prepare the existing instrument.** Director decision, 2026-11-04:
    use `/Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli-dev`
    (below, `AUDIT`), the existing owner of both audit verbs, instead of the
    absent dispatcher `target/debug/sovereign-cli`. Local preparation of a
    fresh code corpus and SCIP graph is part of this row, not an operator
    decision. Use the existing `code index` and local `refresh` surfaces;
    keep audit data in a new dedicated local directory, leaving other corpora
    intact. Record commands, executable identity, indexed source commit and
    coverage of the current tree (including new declarations and app sites).
    Pass the corpus id explicitly and use the same data root for writers and
    readers. Validate exact/near clone and repeated-noun detection on planted
    inputs in a separate scratch corpus before interpreting clean reports.
    Keyword-only indexing cannot satisfy the near-clone check. No external
    source edits, substitute analyzer, paid service or deployment is authorized.
    Any further preparation failure is reported with its actual command and
    output; it cannot become a pass or defer either analysis. Verified entry
    points and remaining preparation are in `REVIEW_AUDIT_NEEDS_HUMAN.md`.
  - **(1) Ledger.** Paste a per-unit net-line ledger for product code over
    that range — `git log --numstat --format=%s <hash>..HEAD`, summed by the
    subject's unit id, src apart from tests.
  - **(2) Clones.** Run `AUDIT code dry-report --corpus-id <id> --scope <dir>`
    for each crate dir the range touched and paste every exact or near clone
    with a side that is a symbol ADDED in the range.
  - **(3) Nouns.** For every new `struct`/`enum`/`trait` in the range, run
    `AUDIT code converge noun <Name> --corpus-id <id>`, and paste any noun
    with more than one definition.
  - **Review/close.** Fix here what is behaviour-preserving and small; record
    the rest in `ralph/REVIEW_FINDINGS.md` with both file:line sites. Complete
    the source/principle review, LINT and the locked full `scripts/check.sh`
    gate. All checks remain required; no focused-review exception applies.
