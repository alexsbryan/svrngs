# Welcoming people into a group

Status: native publication/recovery (`crates/svrngs-node/src/publication.rs`),
the home page's host controls (`home/page/group-link.js`, `group-links.js`),
the authenticated Upstash resolver (`resolver/`), and native attendee arrival
(`web/page/join.js`, `web/install.sh`) are built. The resolver
has local failure-injection and live isolated-fixture proofs, and is deployed
at `https://join.svrnme.sh/`. The canonical page and downloads are deployed at
`https://group.svrnme.sh/`. A native lifecycle through those public endpoints
passed with two processes on one Mac. After a scene startup-order correction,
the operator confirmed Fedora's public installer completed and opened the
focused native welcome and folder opener, reusing its existing membership.
The new contribution and full three-laptop people scene remain unperformed
(`../demo/runs/relay-meet/2026-11-04/README.md`).
Asked for after the JavaScript queue closes,
then moved ahead of the phone-write row by the operator on 2026-11-03 while
publication of the recent multi-ring join-flow fixes is deferred. `PLAN.md`
§1 holds the order of work. This records the feature, its estimate and the
operator's budget/storage decisions; it authorizes no hosting purchase. `DESIGN.md`
describes what exists today.

## 1. The outcome

A member welcomes a room of people through a public, temporary address
that is easy to type, such as `join.svrnme.sh/cedar-otter-moon`. Each person
gives their name once, installs svrngs, joins as their computer's own
identity, and finds the group's shared folder. They can contribute at once.

Omotenashi is the criterion: prepare the welcome, anticipate the next
question, and let the host and software carry the awkwardness. Someone
should know what to do next even when a step fails. Explain installation,
running at login, and which folder is shared before they proceed.

The first Mac–Fedora attempt exposed an arrival failure: the operator reached
a home page with three groups and found it overwhelming, without a welcome
or a clear invitation to contribute (`demo/runs/onboarding/2026-11-04-scene/
FEDORA.md`). Installation plus an anchor to a group does not meet this bar.
The `onboarding-welcome` correction gives native arrival a focused welcome
to that group and its real folder, with a way back to all groups. The haiku
invitation remains optional host-written content in `WELCOME.txt`.
HOME now reads the existing `#group-<id>` destination once: the welcome hides
other groups and management controls, explains sharing and offers the existing
folder opener. **All your groups** navigates to `/`; refreshing that page or
rereading groups does not return to the welcome. Missing/unread targets and
unknown/absent folders give explicit recovery guidance. The local browser
proof and matching-build retry procedure are in
`../demo/runs/onboarding/2026-11-04-welcome/README.md`.

This is a general, self-service feature. Making the next invitation needs
no deployment, hosting credentials, configuration edit, or developer.
An operator-provisioned address per demo was considered and rejected.

The first build assumes working internet and the existing Mac/Linux
one-command installer. Native Windows support and graphical installers
are separate work; browser-only joining does not meet this demo's bar.

## 2. The host and attendee paths

**Host:** on a group's home page, press **Invite people here**. The result
shows a large, copyable short address, when it ends, and that this computer
must remain reachable. **Close joining** ends admission through that
invitation. The address and controls survive reloading and restarting.

**Attendee:** type the address, see who invited them and which group it
is, give their name, and copy one installation command. The command
carries the ordinary invitation and chosen name through installation.
It opens their authenticated local group page, with the actual local
folder path under **Folder location** and **Open shared files in Finder**
easy to find. An existing installation
is reused; retrying preserves identity and does not duplicate membership.

The host prepares an ordinary `WELCOME.txt` and an example text file in
the group's existing root folder. Suggested welcome text:

> Add a small text file beside this one, named after you: sam.txt, for
> example. A hello, a thought, or something you brought. Need an idea? Try
> a short haiku about your journey here. Everyone in this group receives
> what you add.

The prompt is optional group content, not a platform schema or an entry
requirement. The existing folder opener points to the contribution's exact
destination; no special file-creation affordance is planned. The host can
use the existing group screen and file browser to receive contributions.

## 3. The smallest general implementation

### Short addresses over existing invitations

A dedicated, replaceable resolver maps an unlisted random phrase to an
ordinary invitation and its expiry. The example's word count is not a
security decision: choose the phrase space and request limits together.
Use cryptographic sampling and a reviewed wordlist, not a display-name
generator. No public directory of invitations is offered.

Publication and withdrawal are authenticated with the host's existing
key and domain-separated app signatures. Admission remains the issuing
computer's, under the existing `Invite` and redemption rules. Initially
reuse the existing one-day lifetime rather than add duration controls.

Closing must invalidate the underlying invitation, including a copied
long link, before reporting admission closed. Existing participants stay.
A stale close must not invalidate a newer invitation. The current rule of
one open invitation per issuer and group remains: making another ordinary
or device invitation supersedes it.

Persist the publication receipt owner-only on the host. Include recovery
from a lost response, failed publication, restart, and failed withdrawal.
Keep network requests outside the native invitation lock, and recheck the
invitation before treating a late publication response as current.

The resolver needs atomic, replay-safe creation and closure. Retrying a
publication must not reopen a closed address. An old phrase must not later
lead to another group's invitation. Discard expired invitation secrets
while retaining the small reservations needed for these properties.
Expired and closed addresses explain that the invitation ended and how
someone already joined can reopen svrngs.

The resolver holds invitation capability and is part of its trust boundary.
It holds no group record or shared files. Its unavailability affects new
short-address arrivals, not an installed member's continued participation.

**A limit or provider failure is said as itself.** Commands/day/month,
bandwidth, storage, request size, rate limits, timeouts and provider 5xx/429
all produce one person's answer, "joining is temporarily unavailable", with
retry guidance. Existing members keep working, the host's pending receipt and
the underlying invitation stay recoverable, and no partial publication or
close is written. A limit is never shown as an ended or missing invitation,
never triggers a paid upgrade, and never causes a tight retry loop. The policy
handles any non-success, so it does not depend on undocumented provider codes.

### Extend the native arrival

Reuse the existing name form, copy control and installation command. A
desktop attendee follows the native path without first joining the browser
as a second identity. Quote their chosen name literally in the command.

Keep the canonical group-page/distribution origin separate from the short
address. The latter must not accidentally become the installer's download
source or the group's saved page origin. Redirecting to the existing group
page in native-install mode can reuse its UI and browser policy.

The native home page handles publication through its existing authenticated
boundary. It does not gain cross-origin browser access to the resolver or
expose the record API's bearer token. Reuse `/v1/open`'s local destination
support to focus the group, and obtain the folder from the node's answer,
not a guessed `~/svrngs/<name>` path.

Installer recovery restarts an existing svrngs after a failed or interrupted
join, retaining the refusal status. The EXIT/signal cleanup in `web/install.sh`
remains armed from a successful service stop until startup succeeds. Native
commands carry `--at '/#group-<id>'` before `--as`; the installer passes this
destination through `svrngs open` to `/v1/open {at}`. The browser reads only
the issuer's native welcome, never redeems as another identity. The actual local
folder answer and the recovery/name-literal proofs are recorded in
`../demo/runs/onboarding/2026-11-04-arrival/README.md`.

## 4. Reuse and estimate

The original survey was read-only at `dece85b1`. The current-base survey
at `75f9fd3b` is committed in `194aa994`, with candidates, dispositions,
instrument output and reused-file counts in
`demo/runs/economize/2026-11-03-onboarding/SURVEY.md`. It selected no
behavior-preserving product cuts; no saving is subtracted. Counts use
`scripts/lean_check.py`: nonblank, noncomment product lines, with tests,
apps and recorded-run drivers excluded. JavaScript has its own counter; the
core is a subset of the non-JavaScript total.

Existing pieces to extend:

- `Host::link`, `MintInvite` and redemption: multi-person invitations,
  expiry and admission; no new membership act or rule.
- Native publication extends those in `publication.rs`, retaining a 0600
  `publication.json` beside the journal. `RingRail::current_invite` asks the
  core for this issuer's last counting invitation; publication never mistakes
  the last invitation for an old link key for the current one. The native
  HTTP contract and fake-resolver red/green runs are in
  `../demo/runs/onboarding/2026-11-04-native/README.md`.
- `svrngs_core::app::App`: domain-separated publication signatures.
- `svrngs_node::http::client()`: one TLS client is already shared by
  `checked_page` and the node's updater (`crates/svrngs-node/src/http.rs`,
  `crates/svrngs/src/cli.rs:1096`, `crates/svrngs-node/src/update.rs:86`,
  landed in `467d5d16`). Its shared `http::builder()` now also supplies
  publication's client, with redirects and automatic retries disabled for
  signed mutations; TLS construction stays in one place.
- `declared::write_secret`: owner-only, atomic local receipt storage.
- The home page's invitation flow, the group's `NameForm`, and `Copy`.
- `web/install.sh`: platform selection, checksum verification, per-user
  installation, joining, startup and opening the home page.
- The folder listing and `/folder`, `/v1/open {at}`, and group screen.

The original survey found no hosted short-address resolver in this repository or the
checked commonwealth ring surfaces at `e1a43bcd2`. Upstream's current
`clients/desktop/src-tauri/src/friendly_names.rs` has 64 adjectives and 34
nouns, only 2,176 combinations: display names, not bearer credentials.
OIDC `Codes` is also unsuitable: in-memory, single-use and minute-long.

The revised ranges below use the current flow/component split, rather than
the pre-JavaScript-queue files. They are estimates of added counted lines,
not a completed implementation measurement. `home/` is
`crates/svrngs-node/src/home/`; Rust node paths are under
`crates/svrngs-node/src/` unless qualified.

| Work | Non-JS LoC added | JS LoC added | Current basis and what remains |
|---|---:|---:|---|
| Native publication, persistent receipt, status, guarded close and recovery | 180–280 | — | `invite.rs:101-159,190-247`, `declared.rs:136-161`, `home.rs:597-628`. Reuse minting and secret storage; add receipt lifecycle and recovery around network I/O outside the lock. +20 at both ends explicitly allows the late-response/current-invite guard and persisted pending/closed state absent from today's link endpoint. |
| Shared HTTPS client reuse (already landed) | 0 | — | `http.rs:10-23`, `update.rs:86`; the extraction in `467d5d16` is already counted. |
| Installer recovery and focused local opening | 35–65 | — | `web/install.sh:113-158,160-229`, `session.rs:650-665`. Preserve release-hash writes and restart a stopped installation on refusal; use the existing local `at` destination. Range unchanged. |
| Small authoritative current-invitation query, if needed | 0–16 | — | `svrngs-runtime/src/lib.rs:227-247` and `svrngs-core/src/project.rs:32-58`. `invite_for` is newest **for a key**, not a proof that it is the issuer's current open invitation. A small query may avoid a second fold in the adapter; no new membership rule. Range unchanged, core included here. |
| Hosted resolver, durable storage operations, authentication and limits | — | 220–360 | `svrngs-core/src/app.rs:30-53` supplies the signature contract, not the store. OIDC's `Codes` (`oidc/provider.rs:191-240`) is memory-only. The provider-dependent reservation/tombstone transaction is new; retain this range provisionally until provider semantics are chosen. |
| Host controls and short-address presentation | — | 65–110 | `home/page/group-link.js:58-94`, `group-links.js:23-70`, `home/ui/copy.js:47-72`. Extend the typed machine plus its view with persisted status/close/retry answers; today's last link is component state only. Up from 55–90 to cover that state/edge/view work. |
| Name-once native installation flow | — | 45–80 | `web/page/name-form.js:30-48`, `join.js:96-142`, `computer.js:39-72`. Reuse views; add native-mode messages/effects, literal shell quoting and separate distribution origin without redeeming a browser identity. Up from 35–65 for the current typed flow and its native-mode handoff. |
| Clear local folder arrival | — | 10–25 | `surface.rs:171-186`, `home/page/group-folder.js:15-27`, `folder.js:52-77`. Exact path and guarded opener already exist; only arrival focus/presentation remains. Down from 15–30; no second folder API. |
| Deployment, checking and type-check wiring | 15–30 | 5–15 | `scripts/check.sh:58-69,104-108`, `scripts/types.mjs`, `scripts/build-web.sh:115-143`, `web/vercel.json`. Existing gates cover static pages; a resolver handler needs explicit checking and eventual deployment wiring after approval. Ranges unchanged. |
| **Total** | **230–391** | **345–590** | Core's possible +0–16 is a subset of non-JS, not another addition. |

Combined estimate: **575–981 product lines**, midpoint about 780.
Allow roughly 450–800 test lines, markup/configuration and wordlist data
besides. The store's proof is recorded in §5. Its concrete adapter is now in
`resolver/`: one generation-checked EVAL, shared limits, pending reservations,
absolute secret expiry and final commit markers. Its failure/commit and live
adapter evidence is in `../demo/runs/onboarding/2026-11-04-resolver/README.md`.

Remeasured after the survey at `194aa994`: core **3,477/3,478**, `.`
**19,441/19,442**, `js` **7,499/7,500**, unchanged from its base. No
supported product cut was found: already-shared primitives cannot be saved
twice, test relocation removes no implementation, and the remaining similar
flows have different behavior. The clone instrument is could-not-judge,
not a zero-clone result. The unchanged full closing gate passed in 1,141
seconds, exit 0 (`demo/runs/economize/2026-11-03-onboarding/data/check.txt`).

On 2026-11-03 the operator approved the upper estimate: ceilings are **3,494
core**, **19,833 `.`** and **8,090 `js`**, recorded in
`scripts/lean_check.py` and `ralph/next/onboarding/NEEDS_HUMAN.md`. Each
landing raises only to where it lands. The operator chose Upstash Redis Free
for the resolver, with no paid overage and no deployment now. The queue's
`onboarding-store-proof` row records the free-tier cost boundary, atomic
reservation/closure and self-hosting path; §5 retains their limits.

## 5. Hosting plan and its proof

The operator chose Upstash Redis Free for the planned resolver and authorized
no paid overage or public deployment now. On 2026-11-04 the operator provided
the live Free database `https://true-pheasant-225442.upstash.io` with its key
in Keychain (service `svrngs-upstash`, account `alexsbryan`) and asked to move
forward. The live smoke test in
`demo/runs/onboarding/2026-11-04-store/live.txt` passed: one owner from a
concurrent create-if-absent, one close from a compare-and-close, a refused
retry, a permanent reservation, and an error the client can map to
"temporarily unavailable". Vercel remains the hosting assumption for the
handler, not a deployment authorization. The written-answer gate is
superseded by a permanent generation key: if the store looks reset, the
resolver refuses short-address service rather than reuse the URL namespace.
The permanent live generation and local/live refusal proofs are recorded in
`../demo/runs/onboarding/2026-11-04-store/RESET.md`. These are experiments,
not a deployed resolver. Do not build a provider framework.

Pricing checked on 2026-11-03: [Upstash Redis](https://upstash.com/pricing/redis)
offers one free database, 250 MB and 500,000 commands/month. Usage-based
commands are $0.20 per 100,000; function execution and bandwidth are separate
Vercel usage. These are published prices, not a measurement or a promise
about this account's bill. [Redis on Vercel](https://vercel.com/docs/redis)
is an external integration, not an already provisioned store here.
The free-tier-only plan is not proof of a zero bill. The provider-proof row
must establish how use stays within the allowance before any deployment.

The store survey on **2026-11-04 UTC / 2026-11-03 local** is recorded in
`demo/runs/onboarding/2026-11-04-store/README.md`, with official URLs, access
failures and local red/green transcripts in `ATOMIC.md`. Current official
billing docs say **250 MB, 500K commands/month and 10 GB/month**; some older
tables say 256 MB. They require no card for Free and say adding a payment
method upgrades all free resources to paid. Monthly command exhaustion
returns an error; eviction must remain off to preserve reservations. EVAL
and each inner Redis call are metered separately.

The local EVAL proof passes creation races, compare-and-close, replay,
actual secret expiry and permanent reservations after SIGKILL/AOF recovery.
The live proof adds the same atomic create/close races and a permanent
reservation on the actual Free database. Neither proves hosted failover
semantics: official EVAL atomic-lock docs and eventual-consistency/replication
docs leave acknowledged-write recovery on Free unsettled, and the exact Free
bandwidth-exhaustion response is undocumented in the pages read. The resolver
therefore keeps a permanent generation key and refuses all short-address
service if it is missing, rather than reuse the URL namespace. The
self-hosting path is documented, including the handler transport change
required by ordinary Redis. The guard runs within every operation's EVAL,
not as a separate preflight; missing or mismatched generation refuses lookup,
publication, status and close without writes or automatic reseeding. A
sentinel cannot detect selective reservation loss or an old snapshot retaining
it: known/suspected rollback still requires operator reconciliation or URL
namespace retirement. The precise error/retry and uncertain-mutation contract
is in `../demo/runs/onboarding/2026-11-04-store/FAILURES.md`. The adapter's
`web/tests/resolver.test.mjs` injects errors before and after every lifecycle
write and a lost commit reply. Pending capabilities stay invisible; interrupted
close is unavailable until reconciled. Script isolation is not rollback of
Redis writes after runtime errors. The hosting edge must supply its trusted
ingress IP; caller-supplied forwarding headers are never a fallback. No handler
is publicly mounted. `resolver/README.md` names its Fetch-API boundary.
The paired five-word phrase and abuse-limit recommendation was approved by
the operator on 2026-11-04 ("The rest is reasonable too"), together with the
graceful-failure requirement ("We should fail gracefully in the case we hit
any of those limits"). The adapter uses those values and the reset guard; no
public deployment or paid usage is authorized.

## 6. Order and proof

After the JavaScript queue closes, work follows `PLAN.md` §1. Keep any
behavior-preserving extractions separate from the feature changes.

1. Self-service short invitations: native publication/closure and durable
   resolver, with expiry, reload, restart and failure recovery.
2. Native arrival: name once, installation and recovery, correct group and
   folder on the local page. Keep documentation current with each landing.
3. Record the scene on real laptops, after the full gate passes.

New checks fail on their named planted input before passing. Reuse the
installer, page-law and browser harnesses. Cover literal shell names,
matching Rust/server signature bytes, atomic publication retries, closure
replay, stale close versus newer invitations, original copied links after
expiry/closure, existing participants staying, and folder-name collisions.

The scene uses the host plus at least two other laptops, including a fresh
installation. Starting from the typed address, each person joins under
their chosen name, finds the correct folder without verbal directions,
adds a text file, and sees others' files arrive. Exercise an interrupted
join and closing admission too. `scripts/check.sh` must exit 0; a simulated
run alone does not judge the human welcome. No deployment or hosting
purchase is authorized by recording this plan.
