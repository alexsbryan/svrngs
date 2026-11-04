# Welcoming people into a group

Status: planned, not launched. Asked for after the JavaScript queue closes,
then moved ahead of the phone-write row by the operator on 2026-11-03 while
publication of the recent multi-ring join-flow fixes is deferred. `PLAN.md`
§1 holds the order of work. This records the feature and its estimate; it
approves neither a budget increase nor a hosting purchase. `DESIGN.md`
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
folder path and **Open this folder** easy to find. An existing installation
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

Installer recovery includes restarting an existing svrngs after a failed
join; the current installer stops it before joining and can exit before
the restart (`web/install.sh:113-116,154-158,197` at `75f9fd3b`).

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
- `svrngs_core::app::App`: domain-separated publication signatures.
- `svrngs_node::http::client()`: one TLS client is already shared by
  `checked_page` and the node's updater (`crates/svrngs-node/src/http.rs`,
  `crates/svrngs/src/cli.rs:1096`, `crates/svrngs-node/src/update.rs:86`,
  landed in `467d5d16`). Reuse it; no extraction or second HTTPS client is
  needed.
- `declared::write_secret`: owner-only, atomic local receipt storage.
- The home page's invitation flow, the group's `NameForm`, and `Copy`.
- `web/install.sh`: platform selection, checksum verification, per-user
  installation, joining, startup and opening the home page.
- The folder listing and `/folder`, `/v1/open {at}`, and group screen.

No hosted short-address resolver was found in this repository or the
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
besides. Resolver storage semantics can revise the provisional resolver
range; choosing that adapter is still the operator's decision.

Remeasured after the survey at `194aa994`: core **3,477/3,478**, `.`
**19,441/19,442**, `js` **7,499/7,500**, unchanged from its base. No
supported product cut was found: already-shared primitives cannot be saved
twice, test relocation removes no implementation, and the remaining similar
flows have different behavior. The clone instrument is could-not-judge,
not a zero-clone result. The unchanged full closing gate passed in 1,141
seconds, exit 0 (`demo/runs/economize/2026-11-03-onboarding/data/check.txt`).
The retained first failure, recovery and budget/storage questions are in
`ralph/next/onboarding/NEEDS_HUMAN.md`. A budget increase still requires the
operator's explicit approval.

## 5. Hosting remains a decision

The estimate assumes existing Vercel hosting plus one durable transactional
key-value store. The provider is not chosen. Confirm its operational cost,
atomic operations and self-hosting path before committing to its adapter;
do not build a provider framework. Persistent storage is a new operational
dependency, even if its first bill is zero.

Pricing checked on 2026-11-03: [Upstash Redis](https://upstash.com/pricing/redis)
offers one free database, 250 MB and 500,000 commands/month. Usage-based
commands are $0.20 per 100,000; function execution and bandwidth are separate
Vercel usage. These are published prices, not a measurement or a promise
about this account's bill. [Redis on Vercel](https://vercel.com/docs/redis)
is an external integration, not an already provisioned store here.
Use a free allowance or an explicit spending cap if that option is chosen.

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
