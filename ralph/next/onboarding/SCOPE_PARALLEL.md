# Demo deadline — parallel ownership and integration contract

Operator, 2026-11-04 local / 2026-11-05 UTC: use DeepInfra DeepSeek Flash,
parallelize independent work, reduce repeated check.sh runs, demo in a couple
of hours. Model `deepinfra/deepseek-ai/DeepSeek-V4.1-Flash` passed a live
OpenCode probe with variant `medium`. All queue roles use it.

Resource plan: the pool's two-Rust-lane floor is 16 GB, while current available
memory is 11 GB. Do not bypass that guard or stop another session's resident
model. Run this native/backend queue serially with small cargo concurrency;
run the separate `onboarding-ui` queue serially in
`/Users/alexsbryan/dev/svrngs-lanes/onboarding-scope-ui` on branch
`ralph/onboarding-scope-ui`, with **no cargo in that UI worker**. Both model
workers write concurrently; only the native side compiles/tests Rust.

The supervisor stopped the old serial worker and landed its released
grouped-home handback at `e659dd1f`. No personal/V1.6 data was changed. The
cap formatting at `dbfa37a8` changes no cap values and adds five counted
Python lines; account for that in the refreshed economy baseline.

## Order and ownership

- Grouped-home reconciliation closes the committed handback with affected
  fast/browser checks, not another full gate. It owns only its reconciliation
  evidence and genuine fixes to that presentation, not scope controls.
- Scope economy remeasures the landed baseline. Its survey and cancellation
  prototype already exist; do not recreate either. Finish current fast checks
  and the updated record, then release the backend work.
- **`onboarding-scoped-lending`: backend worker in main.** Own Rust production code,
  declaration/membership/migration/enforcement/OIDC/raw cancellation, CLI,
  Give Rust callers and their non-browser Rust tests. Own node protocol
  version and the wasm export described below. In `home.rs`, own API request/
  response handlers; leave the static served-module table to the UI lane.
  Update DESIGN's lending-policy paragraphs, not its UI composition section.
  Adjust only the non-JS cap as necessary under the existing preapproval.
- **`onboarding-scope-ui`: separate onboarding-ui worker/worktree.** Own home/page and shared UI JavaScript,
  the discovery-request header in `web/page/dial.js`, JSDoc/types, audience
  selector/browser drivers and browser harness registration. In `home.rs`,
  only add necessary served-module entries; do not edit the backend handlers.
  Reuse the current components and group data. Update DESIGN's corresponding
  home-page UI description, not its lending-policy paragraphs. Adjust only
  the JS cap under the existing preapproval.
- Keep the original total growth guardrails across both lanes, not per lane:
  +700 non-JS / +235 JS maximum review bounds, zero core growth. Report any
  small served-module non-JS addition so final integration accounts for it.
- Commit explicit paths. UI stays in its own worktree and does not merge/push
  main or run cargo. The final integration/audit worker may merge the completed
  UI branch after verifying its queue is done, its tree clean and its diff
  matches ownership; never merge into an actively edited UI worktree. This
  narrow integration authority is delegated by the supervising operator.
  Do not change the other worker's files to fake a finished interface.

## Shared API shape — implement this, do not negotiate divergent shapes

- Lender-local declaration: `groups: [stableGroupId, ...]`. Empty means paused;
  legacy unchosen has no group reach and needs selection. Carried app's
  existing `group` marker is unchanged.
- Owner `GET /lent`: preserve `{lent, signing}`. Each declared row includes
  `groups: string[] | null` (null = legacy/needs selection; [] = paused).
  The local admin response includes the full selected audience; peer discovery
  discloses only appropriate shared IDs and is independently authorized.
- Create `POST /running`: existing `{port,name,kind}` plus required `groups`.
  Validate explicit selections and lender standing; never default missing
  selection to all groups. Preserve the existing successful result shape.
- Audience edit `POST /lent`: `{lend, to: {groups: [ids]}}`; success is 204.
  It preserves every other configuration field and the OIDC credentials.
- Group names/options come from the existing HOME groups, not a new endpoint
  or guessed names. Use stable IDs, existing name/date disambiguation, visible
  explicit confirmation, and current refusal/retry UI machines.
- Backend bumps the existing node protocol VERSION to 5 and exports
  `current_version() -> u32` from `web/src/lib.rs`, returning that one constant.
  New `/lends` and `/people/lends` callers send `x-svrngs-version` with this
  version. Backend refuses unmarked/old discovery and unscoped old replies;
  the header is compatibility data, never permission. UI adds the header at
  the existing phone discovery edge using `core.current_version()`.
- A UI-only worktree can lack the backend's generated wasm export/types.
  Record that integration check as pending until the backend lands; never add
  a compatibility fallback that treats an unknown scope as everyone. Prove UI
  behavior against explicit scoped fixtures. Final integrated strict types,
  browser and real-node proofs must pass.

## Checks — one integrated full gate, focused checks per lane

The operator's deadline policy supersedes intermediate full-gate instructions
in older row text/director reports. It does not remove a final gate or allow
unproven security behavior.

- Both workers run lean/counts, formatting/lint/type checks applicable to their
  files and meaningful new red/green regression tests. Backend additionally
  checks the workspace/wasm and targeted HTTP/pass/OIDC/raw/migration/Give
  tests. UI runs its Node page laws/fixtures and prepares the authenticated
  browser driver/registration. Its Rust/browser compilation and backend-wasm
  type integration are explicitly deferred to main integration; no cargo in
  the UI worker, and no claim that those pending checks passed.
- Browser tests still take `/tmp/svrngs-gate.lock`; the lanes can write code
  and run independent non-browser checks concurrently. Do not run two full
  gates, or a full gate for every commit/continuation/row.
- Reconciliation uses existing recorded full-browser evidence plus fresh
  affected checks and the now-passing count. Scope economy uses its existing
  scratch proof plus the refreshed count/seam review. Neither launches
  check.sh again just to close an intermediate row.
- UI worker completion means its own focused checks/evidence and commits complete;
  pending other-lane integration is named honestly. Do not claim the whole
  product verified. Keep source changes red-first and trace refusals.
- Once backend is ready, the existing final audit checks the UI queue's DONE
  marker/log in that worktree, merges its completed branch, and owns reconciliation of the
  API/types/budget/docs and the real-node House/Trip denial/withdrawal scene,
  followed by **one locked full scripts/check.sh on the final integrated
  product**. A real failure or product change justifies a rerun; unchanged
  documentation alone does not. Keep all full-gate steps intact.
- Prepare local scene artifacts from the gated product; record source hashes.
  Do not trigger another full gate through the scene-preparation driver when
  the same product has already passed. Run build/checksum/HTTPS probes and
  retain the final gate's exact product provenance.
- Preserve the original audit's source review, ledger and clone/noun analysis;
  prepare the instrument once, not once per changing intermediate tree. If
  analysis or people evidence remains pending, report it separately rather
  than calling the project fully complete. No fabricated people run.

No public deployment, provider spend, push, changed trust floor or new product
features are authorized by this scheduling change. Reserve time for the final
integrated gate and the operator's Mac–Fedora demo.
