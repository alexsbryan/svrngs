# Onboarding's current-base economizing survey, 2026-11-03

`onboarding-economize` (`ralph/next/onboarding/STATE.md`), product base
`75f9fd3b`. The source survey at `dece85b1` predates the completed pages'
JavaScript queue, join-while-running and the updater. This survey completes
the inherited first measurement before choosing cuts. It introduces no
product code or new check. The full closing gate is recorded separately.

## Baseline

At `75f9fd3b`, `python3 scripts/lean_check.py` reported (exit 0,
`data/lean-before.txt`, rerun by `measure.py survey`):

| Rung | Count | Cap | Headroom |
|---|---:|---:|---:|
| `crates/svrngs-core` | 3,477 | 3,478 | 1 |
| `.` | 19,441 | 19,442 | 1 |
| `js` | 7,499 | 7,500 | 1 |
| `swift` | 0 | 1 | 1 |

The feature estimate cannot fit under these caps without cuts or a later
operator-approved raise. This measurement grants neither.

## Change since the original source survey

`git diff --stat dece85b1..HEAD -- crates web apps scripts` reported 161
changed paths, 11,905 insertions and 6,747 deletions. This raw diff includes
tests and the pages' readability refactor, so it is not a product-line
estimate. The exact current rung counts above are the budget baseline.

The JavaScript queue folded the repeated phone `trace`, `wasm`, `sending`,
`titled`, `hex` and PKCS8 construction, and the home page's HTTP helpers
(`../2026-11-03-js-read/SURVEY.md`, compared to current `web/page/dial.js`,
`web/page/page.js`, `home/question.js`, `web/authority.js`, and
`home/page/routes.js`, where `home/` is `crates/svrngs-node/src/home/`).
Those savings already belong to the current count; do not subtract them
again. The barrel `home/ui/kit.js` is gone; rebuilding it to compress imports
would reverse the pages' explicit owning-module imports (DESIGN §5).

## Instruments and their limits

Run from the root:

```sh
python3 -I demo/runs/economize/2026-11-03-onboarding/measure.py survey
python3 -I demo/runs/economize/2026-11-03-onboarding/measure.py public
python3 -I demo/runs/economize/2026-11-03-onboarding/measure.py compact
python3 -I demo/runs/economize/2026-11-03-onboarding/measure.py sites
```

`measure.py` is a recorded-run driver. It calls the earlier instruments
unchanged, recording their output and exit codes under `data/`:

- **Rust public/dead-code surface:** give-economize's `demote.py`,
  `repub.py` and `cargo_warnings.py`, over a `git archive 75f9fd3b` scratch
  in `target/onboarding-econ/scratch`. The prior pass's three manual
  visibility restorations are named in the driver. `cargo check --workspace
  --offline --message-format=json` passed at native round 24; the separate
  `web/` workspace passed for `wasm32-unknown-unknown` at round 5.
  `data/warnings-native.txt` and `warnings-wasm.txt` include
  `build-finished success: True`. Each restoration round's `repub` output
  and the final successful rounds' rendered diagnostics/verdicts are retained;
  intermediate full compiler logs are not committed. `compact` removes
  cargo artifact metadata from the retained final logs.
  This is an internal-use inventory, not permission to break a library API.
- **Counted test-cfg inventory:** give's `cfgtest.py` found 50 lines,
  `data/cfgtest.txt`. Read below; its count includes module braces.
- **Line-window inventory:** give's `windows.py 4`, over tracked, non-test
  `.rs`, `.js`, `.sh`, `.py` in crates, web and scripts, vendor apart
  (`data/windows-4.txt`). It finds exact normalized line windows, not
  semantic clones. Argument lists, imports and overlapping windows remain
  visible rather than being called deletions.
- **Pages' structural/name lists:** the latest js-read `survey.py` and
  `survey.py more`, `data/js-survey.txt` and `js-more.txt`. Its sole four-line
  duplicate is PLAY/ROOM. Repeated top-level names are read in context,
  not assumed to be repeated implementations. The heuristic default/comment
  findings are readability prompts, not counted savings.
- **Clone report: could-not-judge.** `data/clone-availability.txt` records
  `command -v sovereign-cli`, three absent developer binaries, and both
  installed distribution binaries' `code dry-report --help`: "not available
  in this build", exit 2. Neither the line windows nor the source reading
  substitutes for a clone report. No claim of zero semantic clones follows.
- **Untouched product compile:** `cargo check --workspace --offline`, exit
  0 (`data/check-workspace.txt`). Per-file counts from lean_check itself
  are `data/sites.txt`, not raw `wc -l` counts.

## Reused paths, checked at the current base

The original estimate included 5–15 non-JavaScript lines to extract the
command's TLS client. That extraction is now shared in
`crates/svrngs-node/src/http.rs`, used by `crates/svrngs/src/cli.rs:1096`
and `crates/svrngs-node/src/update.rs:86` (`467d5d16`). Count zero new lines
for this reuse; the feature must not make a second client.

| Existing piece | Current site | Consequence for onboarding |
|---|---|---|
| Ordinary invitation and sequencer | `crates/svrngs-node/src/invite.rs:101-159,190-247`; `crates/svrngs-core/src/register.rs:116-137` | Reuse `Host::link` and `MintInvite`; HTTPS publication must run outside `Host::minting`. Nothing here stores a publication receipt or closes just the receipt's invitation. |
| Invitation read side | `crates/svrngs-runtime/src/lib.rs:227-247`; `crates/svrngs-core/src/project.rs:32-58` | `invite_for` finds the newest invite **naming that key**, not necessarily the issuer's currently open invite. A guarded close needs an authoritative current-issuer query; do not use that lookup alone to invalidate a later link. Keep a small optional core allowance. |
| Publication signatures | `crates/svrngs-core/src/app.rs:30-53` | Reuse `App::message/sign/check`; no new signing domain implementation. Receipt bytes, operation and replay identity still need one canonical definition. |
| Owner-only receipt writer | `crates/svrngs-node/src/declared.rs:119-161` | Reuse `write_secret`; no second atomic-file writer. The receipt and status/recovery state are new. |
| Existing host links and Copy | `home/page/group-link.js:58-94`, `group-links.js:23-70`, `home/ui/copy.js:47-72,104-114`; `home.rs:597-628` | Extend the link flow, keep device/guest paths; its current `shown` lives only in component state and is not restart recovery. Reuse Copy. |
| Name and native command | `web/page/name-form.js:30-48`, `web/page/join.js:96-142,245-304`, `web/page/computer.js:39-72` | Reuse NameForm and computer-line; route native arrival without browser redemption, carry `--as` quoted literally, and keep distribution origin separate from resolver origin. |
| Installer | `web/install.sh:67-94,113-158,160-229` | Keep platform selection, checking, placement, login startup and opener. The stopped service is not restarted if join's `no` exits at :157 before `keep` at :197. Recovery is behavior work, not a saving. The updater added a release-hash write; preserve it. |
| Real local folder and focused open | `crates/svrngs-node/src/surface.rs:171-186`, `home.rs:740-755`, `session.rs:415-419,650-665`, `home/page/folder.js:52-77`, `group-folder.js:15-27` | Existing answers supply exact folders; `/folder` checks the listed path, `/v1/open {at}` supports a local destination. Arrival needs focus/presentation, not a guessed path or another folder API. |

The original upstream resolver search remains evidence at `e1a43bcd2`
(`docs/ONBOARDING.md` §4). No new resolver/type is being implemented in this
pass. The existing OIDC code store (`crates/svrngs-node/src/oidc/provider.rs:191-240`)
is process-local/single-use, not durable replay-safe reservation storage;
do not use it as the short-address store.

## Candidates and disposition before cuts

Line numbers below are product `75f9fd3b`, not the demoted scratch (which
splits grouped re-exports and shifts line numbers).

| Candidate | Site and amount | Disposition |
|---|---|---|
| Rust items uncalled by native product | `Pass::approve`, `crates/svrngs-core/src/pass.rs:283-295`; `Node::start/start_every`, `crates/svrngs-node/src/node.rs:267-275,306-315` | Keep: existing tests use them (`tests_grant.rs:305`, node `tests/carried.rs:1024`, `tests/scale.rs:591`, and CLI `tests/join_by_link.rs:82`). Removing them would change public interfaces and require editing callers. `Pass::approving` is called by the wasm build. |
| Unconstructed Read capability | `crates/svrngs-core/src/grant.rs:26`, one variant | Keep: the previous give survey §1 records the unwired read-policy design; `carried.rs:322-329` still filters by prefix. Removing or wiring it is a design/behavior decision, not this pass. |
| Native-only false positives | `dial.rs:46` `LINK_WITHIN`, `version.rs:41` `links`, `iroh.rs:48,66` exports; core re-exports in `warnings-native.txt` | Keep: web compiles separate uses, tests use exported transport helpers, and demotion generates private-interface/re-export warnings. No unreachable implementation deletion established. |
| Inline tests inside counted files | `http.rs:25-31`, 5 lines; `update.rs:345-391`, 42 lines; node's test support, 3 lines | Relocating the first two would lower the counter by 45 (47 minus two module declarations), but remove no implementation or test. No product saving banked from moving test text between paths. The node's three lines wire test support. |
| Link and locked-link argument lists | `invite.rs:151-159,192-198`, repeated four-line windows | Keep: one method acquires the lock and calls the held implementation. The sequencer fold was already cut by give-economize; no repeated drive here. |
| Browser nonce and redirect stores | `session.rs:118-156,181-247` (`Onwards` was the security pass's next candidate, lean_check comment :381) | Keep: nonces are a per-origin spent set; onwards is a proof-keyed, method/path-bound one-use redirect with chain count and independent capacity. Sharing the storage would need policy parameters and preserve no measured small cut. |
| PLAY/ROOM repeated read steps and owner turns | `home/page/play.js:111-153`, `wifi.js:123-167`; `play.rs:120-131,173-190`, `room.rs:163-176`, `room/owner.rs:71-89` | Keep in this pass: PLAY copies a local port and accumulates refusals; ROOM tracks closed lends and different answers. Existing audit finding #224 (`ralph/REVIEW_FINDINGS.md:12832`) already leaves the typed owner-turn consolidation beyond a small cut. Not counted as solved. |
| Other windows/names | `data/windows-4.txt`, `js-more.txt` | Struct fields/builders, HTTP extractor signatures, distinct response enums and flow contract names (`states`, `takes`, `io`, `view`). The known `decode` pair still differs in Unicode semantics (`web/open.js:62`, `web/sign.js:35`); folding it would change behavior. |
| Stale build instructions | `scripts/build-web.sh:149-159`, approximately 8 removable instructions | Keep separate: removing the obsolete `svrn mesh grant` instructions changes terminal output, so it cannot be banked as behavior-preserving. |

**Selected product cuts: none.** This survey supports no deletion or fold
that reduces implementation while keeping the public surface, output and
timing unchanged. It does not turn unavailable semantic analysis into a
proof that no larger refactor exists. Existing tests are unchanged; no new
gate was added and no plant is owed for a new check. No caps change.

## Closing the pass

Remeasure and the revised per-row estimate belong in `docs/ONBOARDING.md`
§4, with the operator's questions in `ralph/next/onboarding/NEEDS_HUMAN.md`.
The full unchanged `scripts/check.sh` must run once at the close. Until
its exit-0 log is committed, this survey is complete but the unit is not.
