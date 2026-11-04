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

At HEAD `75f9fd3b`, `python3 scripts/lean_check.py` reported core 3,477/3,478,
`.` 19,441/19,442, and `js` 7,499/7,500. The operator approved the upper
estimate on 2026-11-03, so the caps are now **3,494 core**, **19,833 `.`**
and **8,090 `js`** (`scripts/lean_check.py`); each landing still raises only
to where it lands. The operator chose Upstash Redis Free for the planned
resolver, with no paid overage and no deployment now. `onboarding-store-proof`
must verify that plan before any adapter.

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
`NEEDS_HUMAN.md` records the operator's budget/storage answers. The
full closing gate first failed (`4555a32e`, `data/check-first.txt`): Podman
was unavailable and Chromium's final lending redraw assertion failed.
Podman was started and the isolated browser retry passed without edits.
The unchanged full retry then passed in 1,141 seconds, exit 0, recorded in
`data/check.txt` by `measure.py close`. The pass is complete and the operator
has answered both questions in `NEEDS_HUMAN.md`; storage proof is next,
before an adapter. The approval-only closing recount on 2026-11-03,
`python3 scripts/lean_check.py` (exit 0), remains:

```text
crates/svrngs-core     3477 / 3494 lines
.                     19441 / 19833 lines
js                     7499 / 8090 lines
swift                     0 / 1 lines
```

## Rows

- [x] onboarding-economize fb0e3e8b2 — depends [] — ONE ROW, the pass before any
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
- [ ] onboarding-store-proof — depends [onboarding-economize] — ONE ROW, a
  survey and a local proof, no product code and no deployment (the operator,
  2026-11-03: "we can plan on a free tier upstash deploy (but again we don't
  deploy now)"):
  - **(a) The free tier's boundary.** From Upstash's official pages, record
    the current Free plan's limits (commands/month, storage, bandwidth),
    how a database stays free (no card, a spending cap, or pausing at the
    limit), and what the account sees when the limit is reached. Cite each
    URL and the date read. A page that cannot be read is could-not-judge,
    never assumed.
  - **(b) Atomic operations, locally.** Verify the exact primitives the
    resolver needs against a locally installed Redis (or an in-repo test
    double only where the row says why): create-if-absent, compare-and-close,
    and a tombstone that survives expiry. A test script under
    `demo/runs/onboarding/2026-11-04-store/` runs red on a non-atomic
    create/close and green on the chosen primitives. No Upstash database is
    provisioned.
  - **(c) Self-hosting.** Record the official Redis install path and what a
    self-hosted resolver would still need (TLS, auth, persistence, backup,
    an operator). This proves a path exists; it does not build one.
  - **(d) Phrase space and limits, together.** Propose the wordlist (source
    and licence), phrase length, entropy, reservation lifetime, request
    rate and abuse limits, with the arithmetic shown. Write the
    recommendation and exact choices in `NEEDS_HUMAN.md`, then stop for the
    operator's answer. No adapter or deployment before that answer.
  - **Check:** every claim cites a URL or a local run; the atomic test's red
    and green pasted; no product lines; `scripts/check.sh` unchanged and not
    required for this row.
  - **Director disposition (2026-11-04 UTC): held for the operator.** Local
    red/red/green proof reproduced; it does not certify hosted recovery.
    The paired phrase/limits answer is still required by CHARTER §3 and (d).
    Before this row can pass, resolve the selected Free endpoint's durable
    reservation/closure recovery and bandwidth-exhaustion behavior, as required
    by ONBOARDING §5. Options, costs and exact provider questions are in
    `NEEDS_HUMAN.md` under “Director decision”. No adapter until that proof
    and the operator's paired answer; the row and source order stay open.
- [ ] onboarding-native-publication — depends [onboarding-store-proof] — ONE
  ROW, red first, and `scripts/check.sh` once, at the close (ONBOARDING §3,
  "Short addresses over existing invitations"):
  - **(a) Tests: A HOST PUBLISHES AND CLOSES.** These fail at HEAD:
    - The host mints a short-address publication through the home page's
      existing authenticated boundary, and the publication receipt is kept
      owner-only on this computer.
    - The status survives reloading the page and restarting svrngs.
    - Close invalidates the underlying `Invite`, including a copied long
      link, before it reports admission closed; existing participants stay.
    - A stale close does not invalidate a newer invitation.
    - A lost response, failed publication, restart and failed withdrawal
      each recover; network requests stay outside the native invitation
      lock; a late response rechecks the current invitation before it is
      treated as current.
  - **(b) The change.** Extend `Host::link`/`MintInvite` and the home page's
    authenticated route; keep the receipt with `declared::write_secret`.
    Reuse `svrngs_node::http::client()`; do not add a second HTTPS client.
    The browser gets no cross-origin access to the resolver and no record
    API bearer token.
  - **Check:** (a) red first on each named plant; the existing invitation,
      join and home-page tests unchanged by name; `scripts/check.sh` once.
- [ ] onboarding-resolver — depends [onboarding-native-publication] — ONE ROW,
  red first, and `scripts/check.sh` once, at the close (ONBOARDING §3):
  - **(a) Tests: THE RESOLVER IS REPLAY-SAFE.** These fail at HEAD:
    - An unlisted random phrase maps to an ordinary invitation and its
      expiry; no public directory is offered.
    - Publication and closure are authenticated by the host's existing key
      and domain-separated app signature.
    - Creation and closure are atomic; retrying a publication does not
      reopen a closed address; an old phrase never leads to a later
      invitation.
    - Expired invitation secrets are discarded while the small reservations
      that keep those properties remain.
    - Expired and closed addresses say the invitation ended and how an
      existing member reopens svrngs.
  - **(b) The change.** Build the resolver handler against Upstash Redis
    Free's REST semantics proven by `onboarding-store-proof`; keep it
    replaceable and hold no group record or shared files. No provider
    framework, no deployment. The adapter uses the phrase space and limits
    the operator approved.
  - **Check:** (a) red first on each plant, including a retry that reopens a
      closed address and an old phrase reused for a later invitation;
      `scripts/check.sh` once; no network deployment.
- [ ] onboarding-host-controls — depends [onboarding-resolver] — ONE ROW,
  red first, and `scripts/check.sh` once, at the close (ONBOARDING §2):
  - **(a) Tests: THE HOST'S CONTROLS.** These fail at HEAD:
    - A group's home page offers **Invite people here**; its result shows a
      large, copyable short address, when it ends, and that this computer
      must remain reachable.
    - **Close joining** ends admission through that invitation; a refusal
      says why and offers retry.
    - The address and controls survive reloading and restarting.
  - **(b) The change.** Extend the home page's typed machine and view. Use
    the existing `Copy` and group-link components; no new page and no
    cross-origin browser access to the resolver.
  - **Check:** (a) red first; Chromium browser tests; `scripts/check.sh`
      once.
- [ ] onboarding-native-arrival — depends [onboarding-host-controls] — ONE
  ROW, red first, and `scripts/check.sh` once, at the close (ONBOARDING §2,
  §3 "Extend the native arrival"):
  - **(a) Tests: A PERSON ARRIVES ONCE.** These fail at HEAD:
    - Typing the short address shows who invited them and which group it
      is; the name is asked once and quoted literally in the installation
      command.
    - An existing installation is reused; retrying preserves identity and
      does not duplicate membership.
    - The installer restarts an existing svrngs after a failed join, and a
      refusal exits without leaving it stopped.
    - The opened local group page shows the actual folder path and **Open
      this folder**, taken from the node's answer, not a guessed
      `~/svrngs/<name>`.
    - The short address is never the installer's download source or the
      group's saved page origin; `/v1/open {at}` focuses the group.
  - **(b) The change.** Extend `web/page/name-form.js`, `join.js`,
    `computer.js`, `web/install.sh` and the existing folder opener. No
    second identity for the browser and no second folder API.
  - **Check:** (a) red first, including a literal shell name and a failed
      join followed by a restart; `scripts/check.sh` once.
- [ ] onboarding-scene — depends [onboarding-native-arrival] — ONE ROW
  (ONBOARDING §6):
  - **(a) Predictions before data.** Write
    `demo/runs/onboarding/2026-11-04-scene/PREDICTIONS.md` before any run.
  - **(b) The scene.** Host plus at least two other laptops, one freshly
    installed: each types the address, joins under their chosen name, finds
    the correct folder without verbal directions, adds a text file, and sees
    the others' files. Exercise an interrupted join and closing admission.
  - **(c) The record.** Every number in the README comes from the run's
    data; the human welcome is recorded, not judged by simulation. Stop for
    the operator's people bar.
  - **Check:** predictions committed before data; `scripts/check.sh` exit 0
      before the scene; no deployment or publish.
