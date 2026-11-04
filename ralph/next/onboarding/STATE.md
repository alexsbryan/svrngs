# svrngs onboarding — welcoming a room through self-service short invitations

The feature contract is `docs/ONBOARDING.md`. The operator chose this work
ahead of the phone-write sync row on 2026-11-03, while deferring publication
of recent multi-ring join-flow fixes (`docs/PLAN.md` §1). The sync row remains
after onboarding.

## The demo

The host creates and closes a short address from the group page. At least two
other laptops, one freshly installed, join under their chosen names, find the
correct shared folder without verbal directions, add a text file, and see
each other's files. Exercise an interrupted join and closing admission too.
The haiku is optional welcome text. The full run is in
`docs/ONBOARDING.md` §6; its human fluency bar is not judged by simulation.

## Current baseline

At HEAD `75f9fd3b`, `python3 scripts/lean_check.py` reports core 3,477/3,478,
`.` 19,441/19,442, and `js` 7,499/7,500. There is one line of headroom on
each rung. The estimate and current-base survey are not budget approval.

The shared HTTPS client extraction in the original estimate already landed
at `467d5d16`; reuse `svrngs_node::http::client()` and do not count it again.
The installer recovery gap described in `docs/ONBOARDING.md` §3 remains:
`web/install.sh` stops the running node before joining, and a failed join
exits before the later `keep` call restarts it.

The current-base survey is committed in `194aa994`, at
`demo/runs/economize/2026-11-03-onboarding/SURVEY.md`. It selected no
product cut; the clone instrument is could-not-judge, not a zero-clone
finding. The revised estimate in `docs/ONBOARDING.md` §4 is +230–391
non-JS and +345–590 JS, with possible +0–16 core included in non-JS.
`NEEDS_HUMAN.md` prepares the operator's budget/storage questions. The
full closing gate and its committed exit-0 evidence are still owed before
the pass is complete; its driver is `measure.py close` in that run directory.

## Rows

- [ ] onboarding-economize — depends [] — ONE ROW, the pass before any
  feature code or budget ask (AGENTS.md "Done"):
  - **(a) Current-base survey, nothing to product.** Start from the count at
    `75f9fd3b`, compare the onboarding source survey at `dece85b1` with the
    current tree, and examine the specific reused paths in
    `docs/ONBOARDING.md` §4. Use the latest economize surveys' applicable
    instruments. Record each candidate with its site and lines in
    `demo/runs/economize/2026-11-03-onboarding/SURVEY.md`. Record an
    unavailable instrument as could-not-judge; do not substitute a manual
    scan as a clone report.
  - **(b) Cuts, behavior preserved.** Land only candidates the survey
    supports. Each cut gets its own commit and unchanged existing tests; if
    no cut is supported, say so. Lower caps to where cuts land, never raise
    them.
  - **(c) Remeasure and estimate.** Recount core, `.`, and `js`; update each
    remaining row in `docs/ONBOARDING.md` §4 from current sites. Stop in
    `NEEDS_HUMAN.md` for the operator's budget ceiling and resolver
    storage/provider choice. No feature implementation, cap edit, adapter,
    deployment or hosting purchase before those decisions.
  - **Check:** survey committed before a cut; `python3 scripts/lean_check.py`
    before and after each cut; existing tests unchanged by name; full
    `scripts/check.sh` once at the close of the pass.
