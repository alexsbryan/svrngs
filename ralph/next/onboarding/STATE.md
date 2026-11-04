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
- [x] onboarding-welcome 4764c59e9 — depends [onboarding-native-arrival] — ONE ROW, the operator's real Fedora feedback (2026-11-04; `demo/runs/onboarding/2026-11-04-scene/FEDORA.md`), red first:
  - **The observed miss.** Installation appeared to work, but arrival at the
    home page with three groups overwhelmed the operator and gave no sense
    of welcome or invitation to contribute. `home-page.js` scrolls to an
    anchor while still rendering every group and management place.
  - **The correction.** Give the native-install arrival an explicit, focused
    welcome to the joined group, identified by its stable id, not its name.
    Lead with "Welcome to <group>", the actual shared folder, what sharing
    there means, and one obvious **Open this folder** action. Keep unrelated
    groups and management controls behind an explicit way back to all groups.
    Ordinary home-page use still offers all groups. Reuse the existing typed
    page machine, session/open destination and folder opener; do not add an
    onboarding identity, folder API, or general tour.
  - **The host's welcome.** The optional haiku invitation stays in ordinary
    group-owned `WELCOME.txt`, as ONBOARDING §2 says; never hard-code a haiku
    requirement or group content into the platform. Make the first step
    toward the shared folder and its welcome note clear. Do not claim a
    welcome note exists if it has not been read or established.
  - **Proof.** Reproduce the current three-group arrival in a browser test
    before the change. Afterward it leads with only the intended welcome,
    exposes the correct path/opener, and has a working way back to all groups.
    Cover same-name groups, loading/unread/missing target, explicit navigation
    away and refresh, and the ordinary non-arrival home page. No polling read
    may throw someone back into the welcome after they chose all groups.
    Keep identity, retry and existing-install behavior. Trace decisions and
    refusals. Tests must fail on the old all-groups arrival, not merely check
    for the new heading. Update DESIGN/ONBOARDING with the implementation.
  - **Close.** Stay within the approved budget; run the affected page/installer
    checks and locked full gate. Prepare matching local scene artifacts and
    give the supervisor the minimum Fedora retry instructions. No public
    deployment, push, or unsolicited change to Fedora's personal data. The
    scene remains open for the operator's actual observations.
- [x] onboarding-portal-reminder e720c815b — depends [onboarding-welcome] — ONE ROW, the operator's request (2026-11-04): bare `svrngs` while already running should remind the person where to open their local UI rather than tell them to stop it.
  - **Behavior.** If the bare run command finds this data directory locked by
    an existing svrngs, print a helpful success message with the actual home
    address, `http://svrngs.localhost:<kept port>/`, and "Run `svrngs open`
    to open it in your browser." Keep this a reminder, not an unsolicited
    browser launch. A browser origin not yet enrolled still uses `svrngs open`
    to establish its authenticated session; printing the address grants no
    access. Do not tell someone to stop a healthy node just to find its page.
  - **Reuse.** `cli.rs` parses no args as `Verb::Run`, currently falls through
    to `claim(&root)?`, and returns `Refusal::Busy`. Reuse
    `svrngs_node::api::kept_address` as the authoritative address reader, the
    existing home hostname and `open()` path. No port guessing, second
    address file, new service or credential in output/debug logs. If the
    address is missing or unreadable, say so with the existing typed errors
    rather than invent a URL or falsely report a reachable portal.
  - **Proof.** First reproduce with a real running node and a second bare
    invocation: the old stop-and-run-again refusal must fail the new check.
    Afterward assert the reminder's URL matches that node's actual port,
    the first process remains running, and no second node/browser starts.
    Keep the no-running-node path (bare command starts svrngs), `svrngs open`
    and other verbs' lock behavior. Cover missing/malformed saved addresses.
    Trace the branch at debug; update command help/design documentation with
    the changed behavior and run affected CLI checks plus the locked full
    gate. Stay within the approved budget and preserve the active welcome
    worker's work. No deployment or push.
- [x] onboarding-oidc-setup-help 7a621499f — depends [onboarding-portal-reminder] — ONE ROW, the operator's Jellyfin setup feedback (2026-11-04): the page prints issuer/client credentials but gives no actionable way to configure Jellyfin.
  - **Observed gap.** "Signing people in with svrngs. In the app's own
    settings, choose OpenID Connect" implies a completed setup and a stock
    settings form. Turning the switch on only creates the provider/client;
    Jellyfin needs its SSO-Auth plugin and app-side configuration. Never
    include the operator's pasted client secret in evidence or a commit.
  - **Reuse and evidence.** Read `apps/jellyfin/README.md` and the successful
    `demo/runs/oidc/2026-10-21-jellyfin-2/` trial before changing guidance.
    It used Jellyfin 10.11.11 / SSO-Auth 4.0.0.4, DisableHttps,
    DoNotValidateEndpoints (issuer validation kept), preferred_username,
    a Branding login button, and KnownProxies=127.0.0.1. The issuer is reached
    by the app server on the lender, not on the attendee's localhost; a
    container's loopback is different. Verify current upstream instructions
    and compatibility; upstream currently marks the plugin archived.
  - **Correction.** Make the generic UI/CLI wording distinguish credentials
    created from app configured, and explain the next step clearly. Put
    Jellyfin-specific prerequisite, install/configuration navigation,
    field mapping, login-button setup and an end-to-end verification path
    in its app-owned guide. Reuse existing guidance/data surfaces; keep app
    integration code out of the core/node and avoid a new setup framework.
    Do not silently enable all libraries or grant administrator permissions.
  - **Proof/close.** Pin the truthful setup state and actionable next-step
    wording in the existing page/CLI checks, watching the new check fail
    first. Do not claim a user is signed in because a toggle generated
    credentials. Update affected docs, run affected checks and the locked
    full gate if product behavior changes. No unsolicited installation or
    changes to the operator's Jellyfin server; the real configuration walk
    is brokered separately with the operator. Stay within approved budgets.
- [ ] REVIEW-audit-onboarding-auto-1 — depends [onboarding-oidc-setup-help] — audit the commits since the previous `REVIEW-audit-` row's hash (since this queue started if there is none) against `docs/principles/ARCH_PRINCIPLES.md`; REUSE AND SIZE, WITH DATA — read: `docs/principles/ARCH_PRINCIPLES.md` — check: LINT
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
- [x] REVIEW-audit-onboarding-auto-2 da503bace — depends [onboarding-oidc-setup-help] — audit the commits since the previous `REVIEW-audit-` row's hash (since this queue started if there is none) against `docs/principles/ARCH_PRINCIPLES.md`; REUSE AND SIZE, WITH DATA: (1) paste a per-unit net-line ledger for product code over that range — `git log --numstat --format=%s <hash>..HEAD`, summed by the subject's unit id, src apart from tests; (2) run `/Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli-dev code dry-report --scope <dir>` for each crate dir the range touched and paste every exact or near clone with a side that is a symbol ADDED in the range; (3) for every new `struct`/`enum`/`trait` in the range, `/Users/alexsbryan/dev/commonwealth-ai/target/debug/sovereign-cli-dev code converge noun <Name>`, and paste any noun with more than one definition; fix here what is behaviour-preserving and small, record the rest in `ralph/REVIEW_FINDINGS.md` with both file:line sites — read: `docs/principles/ARCH_PRINCIPLES.md` — check: LINT
- [ ] onboarding-scope-economize — depends [onboarding-oidc-setup-help] — ONE ROW, current-base reuse/economizing pass for selected-ring lending; the operator preapproved only minimum necessary growth (2026-11-04; CHARTER "Selected-ring lending"):
  - **Survey first.** Read `docs/LENDING_SCOPE.md`, its cited local/upstream
    surfaces and the current tree after the preceding fixes. Record product
    counts by core/non-JS/JS and Give, candidates with exact sites, and the
    smallest implementation in `demo/runs/economize/<date>-lend-scope/`.
    Recheck the membership witness, wire-version fence, full declaration
    mutation serialization, legacy migration and all current share callers.
    Existing stack/primitives outrank inventing a new store or policy engine.
    An unavailable instrument is could-not-judge, not zero duplicates.
  - **Cut only supported waste.** Commit the survey before any cut; each
    behavior-preserving cut is separate from policy changes. Lower caps to
    where cuts land, with a named red/green budget plant. Never meet a budget
    by packed code, renamed identifiers or relocating tests out of the count.
  - **Resolve the minimum design.** Prototype bounded body/WebSocket/raw
    withdrawal using current ownership/cancellation primitives, in scratch,
    before its production implementation. State its tested bound and update
    the source-grounded estimate/baseline. Need no extra core product lines,
    new dependency, service, ACL framework or reach-pass signature format.
  - **Close.** Existing tests unchanged for cuts, lean before/after, and the
    locked full gate if product code changed; document-only survey uses the
    recorded product evidence plus current fast checks. Report actual cuts,
    minimal remaining additions and the proposed incremental cap treatment.
    The budget is already conditionally preapproved: complete this row and
    continue to implementation without a redundant operator budget stop.
    Exceeding the survey's +700 non-JS / +235 JS guardrails, core growth or
    a wider design remains a new operator decision.

  Pass evidence: `demo/runs/economize/2026-11-04-lend-scope/` (`SURVEY.md`,
  `PROTOTYPE.md`, `CLOSE.md`). Survey committed before the prototype; no
  supported cuts or product/cap growth. Baseline remains core 3,486,
  non-JS 19,789, JS 8,066; Give Rust src separately 2,669. Scratch guard
  plants fail for body/upgrade-copy/raw TCP/UDP and green meets 1,250 ms;
  real-node two-second withdrawal remains implementation's proof. Current
  fast checks pass; closing inventory detected unrelated concurrent browser
  edits, retained for their owner before final done bookkeeping.
- [ ] onboarding-scoped-lending — depends [onboarding-scope-economize] — ONE ROW, implement selected-ring lending end to end from `docs/LENDING_SCOPE.md`; minimal necessary actual growth preapproved, zero additional core product lines:
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
    administration and deliberate WiFi delegation explicit. Add group
    selections and audience-only edits to existing typed UI/CLI, with current
    name/ID disambiguation. Update Give's known-group share call sites and
    the current scripts. Do not silently reset credentials on a scope edit.
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
    no up-front upper-bound raise and no core growth. Close with the locked
    full gate and a recorded real-node House/Trip scene; do not claim the
    people welcome from automated evidence. No live personal-data migration,
    public deployment, service spend or push as a side effect of this row.
- [ ] onboarding-scene — depends [onboarding-scoped-lending] — ONE ROW
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
