# Lending to selected groups — due diligence

Status: implemented by `onboarding-scoped-lending` and `merge-scope-ui`
(through 0fc5b189); `docs/DESIGN.md` describes the behaviour as built, and
this file stays the due-diligence record it was written as. On 2026-11-04
the operator approved the
minimum necessary line growth in advance and asked for as few queue rows as
necessary. `ralph/next/onboarding/STATE.md` holds the economizing pass and
implementation; its charter holds the incremental budget authorization.
There is no advance raise to the estimate's upper caps and no core growth
authorized for this work. Surveyed 2026-11-04,
starting at `cb171e7d65cd73b5683eb4136505a313b00592da`; the portal worker
was active in `cli.rs` and DESIGN. References below name the surveyed
symbols; implementation must recheck them after the current queue lands.
When surveyed, `DESIGN.md` described the then-implemented all-groups
behavior.

## Recommendation

Keep the audience on the lender's existing declaration, as an explicit set
of stable group IDs. Enforce it at the lender on every new use. Joining a
new group never edits or expands that set. Use current membership to admit
the people of selected groups; do not freeze a list of their device keys.

This needs no hosted service, database, new dependency, journal act, group
vote, new iroh endpoint, or new cryptographic grant. It does need coordinated
changes to the declarations, door, discovery and interfaces. Hiding a row in
one group is not authorization.

## What exists, and what can serve

| Existing surface | Evidence | Use |
|---|---|---|
| Lender-local JSON with strict parsing and atomic replacement | `lend.rs`: `LendFile`, `read`, `write`, `limit`; `declared.rs`: `write_secret`, `write_whole` | One policy home; preserve the client, secret and choices on an audience-only edit. |
| Stable group identity and membership | `group.rs`: `Seed::id`, `namesakes`; runtime `register.rs`: `standing`, `standing_anywhere` | Store IDs, show names with existing disambiguation; resolve membership within selected groups. |
| Group-limited carried apps | `lend.rs`: `Source::Carried`, `Offered::listed`; `carried.rs`: `NOT_ITS_GROUP` | Existing proof that group-limited listing/serving fits the stack. Keep carried apps intrinsically group-owned. |
| Common HTTP entry point | `door.rs`: `lend_for`, `holder_of`, `Who`, `Asker` | Apply audience before forwarding, OIDC authorization or issuing identity headers. |
| Raw TCP/UDP common admission and revocation | `door/udp.rs`: `serve`, `admitted`, `unstood`, one-second `SWEEP`; `door/tcp.rs`: `connection` | Reuse the existing machinery, reading the current declaration rather than a captured old clone. |
| Typed create/edit UI | `home.rs`: `ToLend`, `LendChange`, `lend_one`, `change`; `home/page/lend-found.js`, `lend-change.js`, `lent-row.js` | Add group selection to current flows; no second lending dashboard or form engine. |
| One protocol-version decider | `version.rs`: `VERSION`, `older`; `node.rs`: `ask_lends`, `answered` | Keep one version home; add a caller-side scope capability fence, not field-by-field guessing. |
| Standard-library file locking | `cli.rs`: `claim`, using `File::try_lock` | Reuse this primitive for serialized declaration mutations; no new lock crate or shared mutable state. |

Node paths in this document are under `crates/svrngs-node/src/`; runtime
paths are under `crates/svrngs-runtime/src/`.

### Upstream inventory

Checked commonwealth-ai at `d8b61abe55d29f573d71ca88fa0b698e2484608c`:

- `cmnwlth/crates/commonwealth-media/src/identity.rs:41-65,118-147` has a
  common origin-admission function over a verified mesh member and a local
  allow list. This is the right ownership pattern, but the list names mesh
  members, not groups, and an empty list means everyone.
- `cmnwlth/crates/commonwealth-rails/src/config.rs:79-95` keeps that media
  origin and member allow list in local configuration.
- `cmnwlth/crates/sovereign-cli-mesh/src/mesh_media/offer.rs:12-28` removes
  the allow list when re-offering without names, widening back to everyone.
  That default is specifically wrong for this change.
- `cmnwlth/crates/commonwealth-media/src/declared.rs` supplies the secret
  storage slice already forked here, not a group-audience implementation.

No directly reusable selected-group lender policy was found in those
surfaces. Forking the mesh admission code would import the wrong identities
and defaults. Extend the existing svrngs group/listing/door code instead.
This is a scoped source inventory, not a semantic-clone result.

## Policy and representation

For a declared HTTP/TCP/UDP lend, persist a `groups` array in its current
JSON file. It contains canonical IDs of groups the lender explicitly chose.
Do not store group display names or use their prefixes as identities.

- New lend: require explicit selection. From a group's own page, that group
  may be visibly preselected, but only the person's lending action commits it.
- Empty set: paused to groups; show that plainly, retaining configuration.
- Legacy missing field: needs selection, never implicitly all groups.
- An explicit "all current groups" action, if offered, stores today's IDs.
  It is a snapshot, never "all groups I might join later".
- Existing carried app: its audience remains its own group, derived from
  `Source::Carried`; do not add a second configurable owner.

Parse the file boundary into a small closed audience type distinguishing
legacy/unchosen from selected groups; derive the wire and UI forms from it.
Validate new selections with held seed IDs and the lender's standing, reusing
existing namespace/seed readers. Leave/removal makes a selected group confer
no access; renaming or duplicate display names must not retarget a grant.

### One policy, an allowed membership witness

Access requires a selected group in which both the lender and verified
asker stand. Resolve the person's identity from such a group, then pass that
same witness through authorization and identity injection. A valid pass
additionally needs its existing signature/holder/lifetime checks.

`PersonKey::resolved` currently picks the first group anywhere
(`door.rs:950-956`, runtime `register.rs:320-339`). Picking that group and
then checking the audience would wrongly refuse a person also in a later
allowed group. Picking any group and keeping its identity could attribute an
allowed request to an excluded group's person. Search only eligible groups;
if eligible groups map one device to different person keys, refuse the
ambiguity rather than silently merge accounts. Preserve deterministic
selection when their person identity agrees, and test it.

Owner administration and direct access to the underlying localhost program
remain the owner's. Preserve the authenticated `Who::Own` path deliberately;
do not manufacture membership or confuse owner access with a ring grant.
Guests additionally need the existing live visit and explicitly named lend,
and that visit's group must remain in the lend's audience.

### Overlapping membership is a union of grants

If a person belongs to allowed House and excluded Trip, they can still reach
the one Jellyfin lend through House. Trip's list should not advertise it.
There is no reliable restriction on "which tab they thought they were in":
the label and reach pass name a lend, not a selected group
(`crates/svrngs-core/src/pass.rs`: `Label`, `Pass::sign`, `message`).

This recommended first build grants people through selected groups. Strict
per-request group context or separate Jellyfin accounts/data per ring would
be a larger signed-context/origin design, not an extra checkbox.

## Enforcement and disclosure checklist

1. **HTTP and OIDC front channel:** check before `lend_for`'s authorize
   branch (`door.rs:1301-1336`), before secret injection or upstream contact.
   Direct bookmarks, native clients and old phone passes cannot bypass it.
2. **Raw TCP/UDP:** check before acknowledging the lend, on new uses and in
   the existing one-second sweep. Reload the live declaration; the captured
   `Lend` in `udp::serve`/`tcp::serve` otherwise outlives an audience change.
3. **Discovery:** apply the same group predicate in `/lends`,
   `Asking::lent`, `/people/lends`, `/v1/lends?namespace=...`, and app reads.
   Transitive listing through another member must not re-advertise an
   excluded lend. Return only audience IDs the verified recipient already
   shares; do not disclose other selected groups.
4. **Do not repurpose `Offered.group`:** it marks carried apps, not general
   access. `web/page/hand.js:132,171-172` uses it to select carried routes and
   signing, and `home/page/lent-read.js:46` hides it from owner editing.
   Add distinct audience metadata and keep the existing carried marker.
5. **App context:** filter `x-svrngs-pages` (`door.rs:1525-1542`) and
   response-driven append groups (`origin.rs:433-518`) to the selected,
   shared groups. Check `/svrngs-reads` and the phone's current-group handoff
   too. Do not advertise an excluded group as a context the app can write to.
6. **OIDC back channel:** scope-check authorization, then retain group/device
   provenance in the internal one-minute `Code` and recheck at redemption.
   `provider.rs:154-160,331-388` currently retains only subject/client/lend
   provenance. Review `/userinfo`'s current-membership behavior too. Preserve
   issuer, client ID and secret on audience-only changes; no plugin
   reconfiguration or signing-key rotation just to remove a group.
7. **WiFi/Play:** these already carry requests as the local person through
   the door (`room/owner.rs:162-166`, `play.rs`). Keep the lender-side check
   authoritative. A person's separate "Open to this WiFi" choice remains
   an additional network delegation and must be described as such.
8. **Give:** `apps/give/src/lend.rs:74-82` currently calls unscoped `share`.
   Its keeper already knows the group (`keep.rs:539-547`); pass that ID.
   Update its two keeper calls (`keep.rs:539,683`) and adapter definition,
   the harnesses and documented scripts.

### Existing streams and app sessions

New requests should lose access after the saved audience changes. Already
authorized requests may have reached the app; a policy change cannot undo
those effects. Raw connections can be cut at the current one-second sweep.
HTTP response bodies and WebSocket splices currently run to completion
(`door.rs`: `forward`, `splice`).

Recommend bounded closure of active lend streams as a separate, measured
part of the implementation. Reuse the existing ownership/cancellation
primitives and active-use owner where practical. Cancel the affected body
or splice, not the whole multiplexed iroh connection, which also carries
other lends and group traffic. Prototype this part before promising a bound.
The cheaper milestone denies new HTTP requests but lets existing streams
finish; it must say exactly that and must not claim immediate withdrawal.

The fresh pass's scratch experiment is now recorded in
`../demo/runs/economize/2026-11-04-lend-scope/PROTOTYPE.md`: existing
Tokio/Hyper/drop primitives closed a backpressured upstream HTTP body,
an upgraded-copy primitive and owned raw TCP/UDP tasks within a 1,250 ms
test ceiling with a one-second sweep. Each transport's disabled guard failed
first. This is not a WebSocket-handshake or real-iroh withdrawal proof.
Use an independently scheduled guard around the request/pump/upgrade lifetime,
not a timer polled only by downstream body demand; own the upstream driver
and raw children. The production row must prove the proposed two-second
bound with real nodes and unrelated traffic still working.

Jellyfin accounts, its own sessions and reach through an independently
exposed Jellyfin address remain Jellyfin's. Removing a svrngs audience is not
remote account deletion or universal token revocation. Protect subsequent
traffic through the door regardless of those app credentials.

## Updates, migration and mixed versions

Use one audience-only mutation helper for UI/CLI, preserving the rest of the
declaration. Atomic rename prevents torn files, not lost updates: today's
`lend::limit` and `home::sign_in` read then rewrite a full lend. A stale edit
must not restore an audience another request narrowed. Serialize the whole
read/modify/write using a stable per-lend lock and the existing standard
library `File::try_lock` primitive; return a visible busy/conflict on
contention. All limit/sign-in/audience writers must participate. Locking only
the final write is insufficient. Keep this at the I/O edge.

Recommended legacy transition: pause group access until selection. Before
the upgraded node advertises lends, stamp legacy declarations with an empty
explicit audience, preserving secrets and other fields; surface "Choose
which groups can open this" locally. A failed migration must not serve the
old broad grant. This deliberate break needs operator approval and release
instructions. Automatic snapshot-to-current-groups is a less disruptive
alternative, but preserves existing unintended exposure and infers consent;
it is not the recommendation.

Put the scope field inside the original declaration: old binaries use
`deny_unknown_fields`, so they reject a scoped file rather than treating a
sidecar as absent and sharing globally on downgrade. Do not strip the field
to make an old version run. Rollback must keep declarations paused or use
an explicitly chosen pre-change backup with its known broad semantics.

Discovery needs a mixed-version fence. Merely increasing VERSION is not
enough: `version::older` only rejects lower numbers, so an old v4 consumer
accepts v5 and ignores an unfamiliar audience field. New list requests must
declare scope awareness using a small version/capability marker; new lenders
refuse unmarked old discovery requests. New consumers reject unscoped old
provider answers through the existing version decider. Test both directions
with the old parser, plus phone caches; do not silently treat missing scope
as public. The marker is compatibility data, not authorization: a client
claiming it still passes the lender's actual audience/membership checks.
Existing pass signatures and group journals need not change.

## User interface and commands

- Creation: "Who can open Jellyfin?" with explicit group selections, then
  "Lend to House and Friends". Existing naming/dates distinguish duplicate
  group names. Joining another group shows existing audiences unchanged.
- Owner row: "Lent to House, Friends" plus edit and stop. Capacity is
  separately "up to N people at once", not "everyone in your groups".
- Proposed CLI: repeat `--group <name-or-id>` on `share <port>` and allow
  `unshare <lend> --group <name-or-id>` to remove just those groups. Keep
  bare `unshare <lend>` as total withdrawal. Reuse `group_named`/ambiguity
  handling. A creation without an audience refuses with the exact next
  command; a non-audience edit preserves the existing audience.
- Audience-only edits retain credentials and choices. Grant reach must still
  be checked before any previously stored app-choice yes is honored. Decide
  explicitly whether remove/re-add should clear those yeses; do not let a
  yes in one group override excluded reach.

## Economy and estimate

Planning ranges from the inspected extension sites, not measured code or
permission to spend their upper bounds. Product counts use the existing lean rules; JSDoc/tests/apps
are not banked as product savings. A behavior-preserving extraction remains
separate from policy changes. No credible deletion was established by this
read-only survey; no semantic-clone instrument was run.

| Slice | Non-JS added | JS added |
|---|---:|---:|
| Audience parsing/storage, serialized mutations | 80–125 | — |
| Allowed-group witness, HTTP/guest/choice/header checks | 55–95 | — |
| Discovery, app context, version fence | 45–80 | 15–30 |
| CLI and home endpoint changes | 50–90 | — |
| Legacy pause/migration and local explanation | 30–55 | 20–35 |
| OIDC provenance and redemption checks | 40–70 | — |
| Raw admission and current-scope sweep | 20–35 | — |
| Existing creation/edit UI selectors | — | 100–170 |
| **New-request milestone plus raw cut** | **320–550** | **135–235** |
| Bounded HTTP/body/WebSocket withdrawal, prototype first | 80–150 | — |
| **Recommended complete scope/withdrawal build** | **400–700** | **135–235** |

Thus about **535–935 counted product lines** for the complete build, midpoint
about 735, with **zero planned svrngs-core lines**. Separately allow roughly
10–25 Give product lines (apps are excluded from the platform cap), plus
700–1,300 test lines for meaningful cross-boundary and migration checks.
No estimate here funds a new ACL framework, per-ring endpoints/origins,
signed group-context passes or app account isolation.

The fresh economizing pass at `7ad807f1`, after the preceding fixes, reports
core **3,486/3,494**, non-JS **19,789/19,833**, JS **8,066/8,090** and Give
Rust source separately **2,669**. Its current-source/window inventory found
no supported behavior-preserving cut; semantic clones are could-not-judge,
not zero. Evidence and exact caller/mutation/witness/version/migration sites
are in `../demo/runs/economize/2026-11-04-lend-scope/SURVEY.md`.
With exclusive caps this permits **43 non-JS and 23 JS** additional lines,
not the arithmetic differences 44/24. The unchanged estimate does not fit
that headroom. The scratch result retains **+400–700 non-JS / +135–235 JS**,
zero core and Give separately +10–25. No product cut or cap edit lands in
the pass. Audience edits preserve existing app-choice yeses; the live scope
gate remains authoritative before any yes is honored.
The operator has preapproved only necessary actual additions, with each
required cap adjustment landed incrementally to one past the measured
count. The 700 non-JS / 235 JS upper estimates are review guardrails, not
up-front allocations; exceeding them, adding core lines or expanding scope
needs a new decision. Existing-stack economy is credible; fitting under
today's caps is not.

Reconciliation at `c3fa0ef7` checkpointed an unrelated, unfinished grouped-home
prototype: core **3,486/3,494**, non-JS **19,790/19,833**, JS **8,174/8,090**.
Relative to the survey this was **0 core / +1 non-JS / +108 JS**, none spent
by selected-ring lending. Do not charge that prototype to lending's
conditional approval. Its owner reconciled it in
`onboarding-grouped-home-reconcile` (`b3cfadfe`, closed `b484b2b3`), and the
operator separately approved its measured JS remainder (cap 8,090 to 8,188 at
8,187 counted lines; no core or non-JS increase), recorded in
`../demo/runs/economize/2026-11-04-grouped-home/BUDGET.md`.

The resumed pass rechecked the landed baseline (HEAD `b484b2b3`) with
`measure.py resume`, whose `resume-fast.txt` holds the fresh commands and
exits: lean **core 3,486/3,494, non-JS 19,795/19,833, JS 8,187/8,188**, exit 0
(the +5 non-JS over the handback is `dbfa37a8`'s budget-layout lines, no cap
value changed); principles, trust floor, advisories, vocabulary, rustfmt,
oxlint, oxfmt, types, typed-lint, the 141 page-law tests and
`cargo check --workspace --offline` all exit 0 under
`lockf -k /tmp/svrngs-gate.lock`. The same run's `seam-diff` shows the only
change to any surveyed lending surface since `7ad807f1` is one served-module
line in `home.rs:140` (`"/page/" "home/page/" ["grouped"]`), which the
grouped-home reconciliation owns; every other seam the survey cites
(`lend.rs`, `door.rs`, `door/`, `version.rs`, `node.rs`, `oidc/`, `origin.rs`,
`surface.rs`, `cli.rs`, `register.rs`, `apps/give/src/`) is byte-unchanged. The
seams therefore still hold as surveyed; the estimate and guardrails above are
unchanged.

That leaves **38 non-JS** and **1 JS** line of exclusive-cap headroom, well
under the +400–700 non-JS / +135–235 JS estimate, so implementation will need
incremental cap adjustments to `measured count + 1` as each landing occurs.
No product cut, cap edit or code change lands in this pass; the resume records
evidence only. The scratch prototype and its closing evidence remain in
`PROTOTYPE.md` and `CLOSE.md`. A full `scripts/check.sh` is not run here: the
operator's demo-deadline policy (`SCOPE_PARALLEL.md`) assigns the one
integrated locked full gate to the final audit after the backend/UI merge.

## Proof before landing

Use the existing node/door, native command, browser, raw-port and scene
harnesses. No separate test framework. Each new guard fails on its named
planted bypass before passing.

1. House and Trip share the lender; Jellyfin is selected only for House.
   House-only member succeeds; Trip-only member cannot list or reach it.
   Try a copied label/direct URL, old pass, forged group/identity headers,
   OIDC authorize/code redemption, TCP and UDP. Assert no upstream request.
2. A device in both groups gets authorized only through an eligible witness;
   unrelated group ordering cannot alter the verdict or silently change the
   app account. Cover conflicting person keys and duplicate group names.
3. Join a third group: declaration bytes/audience stay unchanged and its
   members get no service. New legitimate members of a selected group do.
4. Narrow the audience during HTTP streaming, WebSocket and raw traffic;
   prove the advertised withdrawal semantics and bound. Unrelated lends and
   sync remain connected. Already-issued app credentials do not bypass the
   next door request.
5. Race audience narrowing against limit/sign-in edits; verify no restored
   grant, credential loss or torn file. Restart preserves selected IDs.
6. Legacy/missing/malformed/unreadable scope never becomes everyone.
   Test migration failure and an actual old binary/parser against the new
   declaration and discovery message in both upgrade directions.
7. Recheck carried apps, owner access, visits, app-read/append contexts,
   WiFi/Play and Give's group-kept app. Existing independent app accounts
   remain explicit rather than implied revoked.
8. Browser: selection, audience names, changing/removing one group, visible
   save failures and no automatic expansion on join. Close with the locked
   full gate and a two-group real-node scene before a people claim.
