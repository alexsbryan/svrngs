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
The installer recovery gap at that baseline is covered by the arrival row:
`web/install.sh` now arms EXIT/signal recovery when it stops an existing
service, and disarms it only after startup succeeds. Red/green evidence is in
`demo/runs/onboarding/2026-11-04-arrival/README.md`.

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
- [x] onboarding-store-proof 67731737b — depends [onboarding-economize] — ONE ROW, a
  survey, a local proof and a live Free proof, no product code and no public
  deployment (the operator, 2026-11-03: "we can plan on a free tier upstash
  deploy (but again we don't deploy now)"; 2026-11-04: "I just want us to move
  forward with this"):
  - **(a) The free tier's boundary.** From Upstash's official pages and the
    operator-provided live database (`https://true-pheasant-225442.upstash.io`,
    key in Keychain service `svrngs-upstash`, account `alexsbryan`), record the
    Free plan's limits (commands/month, storage, bandwidth), how a database
    stays free, and what the account sees when a limit is reached. Cite each
    URL or live run. A page that cannot be read is could-not-judge, never
    assumed. The live key is never printed or committed.
  - **(b) Atomic operations, locally and live.** Verify create-if-absent,
    compare-and-close and a tombstone that survives expiry against a locally
    installed Redis and the live Free database. `proof.py`/`ATOMIC.md` hold the
    local red/green plants; `live.py`/`live.txt` hold the live smoke test
    (exactly one owner, exactly one close, a refused retry, a permanent
    reservation, and an error the client can map to unavailable). No public
    deployment.
  - **(c) Self-hosting.** Record the official Redis install path and what a
    self-hosted resolver would still need (TLS, auth, persistence, backup,
    an operator). This proves a path exists; it does not build one.
  - **(d) Phrase space and limits, together.** The operator approved the
    proposal on 2026-11-04 ("The rest is reasonable too"): EFF Short Wordlist
    1, five words (51.69 bits), 30/min and 300/day per IP, 20k/month global,
    100 active secrets, 100k reservations or 100 MB, 8 KiB bodies, one-day
    invitations. Record the values in `NEEDS_HUMAN.md`; no adapter before the
    reset guard below.
  - **(e) Graceful failure at every limit.** The operator's answer, 2026-11-04:
    "We should fail gracefully in the case we hit any of those limits." Define
    the resolver's policy for commands/day/month, bandwidth, storage, request
    size, rate limits, timeouts and provider 5xx/429: the host and attendee see
    "joining is temporarily unavailable" with retry guidance, never an ended
    invitation; existing members are unaffected; no partial publication or
    close; no paid upgrade or automatic retry storm; the host's pending receipt
    and the underlying invitation remain recoverable. The policy handles any
    non-success, so it does not depend on undocumented provider codes.
  - **(f) A reset guard.** Keep a permanent generation key in the live store.
    If the key is missing, or the store otherwise looks reset, refuse all
    short-address publication and lookup with "joining is temporarily
    unavailable" until the operator reconciles; never reuse the URL namespace.
    A test proves a missing generation refuses service rather than reopening a
    phrase.
  - **Check:** every claim cites a URL or a local/live run; the atomic tests'
    red and green pasted; the reset guard red on a missing generation; no
    product lines; `scripts/check.sh` unchanged and not required for this row.
  - **Unattended safety:** if the tree starts with another row's tracked
    changes, commit them as `WIP: <row>` first; never discard them. This row
    does not run the full gate.
  - **Operator decision, 2026-11-04:** the operator provided the live Free
    database and said "I just want us to move forward with this." The official
    written-answer gate is superseded by the live proof and the reset guard.
    The paired phrase/limits are approved. No public deployment or paid usage.
- [x] onboarding-native-publication b3f8c45ff — depends [onboarding-economize] — ONE
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
    - **A limit or provider error fails gracefully.** A resolver answer of
      429, 5xx, timeout, quota/bandwidth/storage exhaustion or an
      unreadable body leaves the invitation valid and the receipt pending,
      says "joining is temporarily unavailable" with retry guidance, and
      never looks like an ended invitation or a closed publication.
  - **(b) The change.** Extend `Host::link`/`MintInvite` and the home page's
    authenticated route; keep the receipt with `declared::write_secret`.
    Reuse `svrngs_node::http::client()`; do not add a second HTTPS client.
    The browser gets no cross-origin access to the resolver and no record
    API bearer token. The publication client treats any non-success as the
    graceful failure above; it never retries in a tight loop and never
    upgrades a plan.
  - **Check:** (a) red first on each named plant; the existing invitation,
      join and home-page tests unchanged by name; `scripts/check.sh` once.
- [x] onboarding-resolver e900f4eab — depends [onboarding-store-proof, onboarding-native-publication] — ONE ROW, red first, and `scripts/check.sh`
  once, at the close (ONBOARDING §3):
  - **(a) Tests: THE RESOLVER IS REPLAY-SAFE AND FAILS GRACEFULLY.** These
    fail at HEAD:
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
    - **Every provider limit fails gracefully.** Commands/day/month,
      bandwidth, storage, request size, rate limits, timeouts and 5xx/429
      return one person's "joining is temporarily unavailable" answer with
      retry guidance; no partial write, no reopened address, no lost
      reservation, no paid upgrade and no tight retry loop. A limit is
      never said as "ended" or "not found".
    - **A reset store is refused.** A missing or mismatched permanent
      generation key makes every short-address publication and lookup answer
      "joining is temporarily unavailable" until the operator reconciles;
      the resolver never reuses the URL namespace after a reset.
  - **(b) The change.** Build the resolver handler against the live Upstash
    Redis Free database (`https://true-pheasant-225442.upstash.io`; key from
    Keychain service `svrngs-upstash`, account `alexsbryan`, never printed or
    committed) and the atomic semantics proven by `onboarding-store-proof`.
    Keep it replaceable and hold no group record or shared files. No provider
    framework, no public deployment. The adapter uses the phrase space and
    limits the operator approved, the graceful-failure policy above and the
    generation/reset guard.
  - **Check:** (a) red first on each plant, including a retry that reopens a
      closed address, an old phrase reused for a later invitation, a
      simulated 429/quota/bandwidth answer, and a missing generation key;
      `scripts/check.sh` once; no public deployment. Run the gate under
      `lockf -k /tmp/svrngs-gate.lock`; rerun `leave_race` and
      `the_home_page_lends_what_runs_here` before calling either a failure.
- [x] onboarding-host-controls 23f258103 — depends [onboarding-native-publication] —
  ONE ROW, red first, and `scripts/check.sh` once, at the close
  (ONBOARDING §2):
  - **(a) Tests: THE HOST'S CONTROLS.** These fail at HEAD:
    - A group's home page offers **Invite people here**; its result shows a
      large, copyable short address, when it ends, and that this computer
      must remain reachable.
    - **Close joining** ends admission through that invitation; a refusal
      says why and offers retry.
    - The address and controls survive reloading and restarting.
    - A resolver limit or failure is shown as "joining is temporarily
      unavailable" with a retry, never as an ended invitation; the
      underlying invitation stays usable.
  - **(b) The change.** Extend the home page's typed machine and view. Use
    the existing `Copy` and group-link components; no new page and no
    cross-origin browser access to the resolver.
  - **Check:** (a) red first; Chromium browser tests; `scripts/check.sh`
      once.
- [x] onboarding-native-arrival c0b58cd14 — depends [onboarding-resolver, onboarding-host-controls] — ONE ROW, red first, and `scripts/check.sh` once,
  at the close (ONBOARDING §2, §3 "Extend the native arrival"):
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
    - A resolver limit or failure says "joining is temporarily unavailable"
      with retry guidance, and the attendee is never told the invitation
      ended.
  - **(b) The change.** Extend `web/page/name-form.js`, `join.js`,
    `computer.js`, `web/install.sh` and the existing folder opener. No
    second identity for the browser and no second folder API.
  - **Check:** (a) red first, including a literal shell name, a failed join
      followed by a restart, and a simulated resolver limit;
      `scripts/check.sh` once.
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
  - **Staging, 2026-11-04 (supervisor on the operator's behalf):** use the
    live Upstash Free database (`https://true-pheasant-225442.upstash.io`;
    key in Keychain service `svrngs-upstash`, account `alexsbryan`; never
    print or commit it). Run the resolver handler locally with a small
    adapter on this Mac; build and serve the canonical page and matching
    Mac/Linux downloads locally from the scene commit; use the LAN/tailnet
    address for the attendee laptops. No public deployment, publish or push.
    The operator supplies the host plus two attendee laptops, one fresh
    install, and is present for the run. If they are not present, prepare and
    verify the staging, record the people step as never-ran, and stop; never
    simulate the welcome.
  - **People bar (provisional, supervisor on the operator's behalf; the
    operator may revise):** each attendee joins from the typed address
    without verbal help; each reaches the correct folder and adds a text
    file; no second name prompt, duplicate membership or wrong folder; one
    interrupted join retains identity; closing refuses a new arrival through
    both the short address and a copied ordinary invitation while existing
    members still exchange files. Record assistance and time; one run is not
    a rate. The operator judges the human welcome from the record.
