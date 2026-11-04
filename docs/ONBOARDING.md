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
the restart (`web/install.sh:114,149-151,191` at `dece85b1`).

## 4. Reuse and estimate

The original survey was read-only at `dece85b1`; no cuts were landed or
subtracted. Its current-base follow-up is open in
`demo/runs/economize/2026-11-03-onboarding/SURVEY.md`. Counts use
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

| Work | Non-JS LoC added | JS LoC added |
|---|---:|---:|
| Native publication, persistent receipt, status, guarded close and recovery | 160–260 | — |
| Shared HTTPS client reuse (already landed) | 0 | — |
| Installer recovery and focused local opening | 35–65 | — |
| Small authoritative current-invitation query, if needed | 0–16 | — |
| Hosted resolver, durable storage operations, authentication and limits | — | 220–360 |
| Host controls and short-address presentation | — | 55–90 |
| Name-once native installation flow | — | 35–65 |
| Clear local folder arrival | — | 15–30 |
| Deployment, checking and type-check wiring | 15–30 | 5–15 |
| **Total** | **210–371** | **330–560** |

Combined estimate: **about 540–930 product lines**, midpoint about 735.
The possible 0–16 core lines are included above, not added again. Allow
roughly 450–800 test lines, markup/configuration and wordlist data besides.
The JavaScript queue has closed, but the current-base economizing pass is
still open; remeasure after its cuts before asking for any budget.

No credible large deletion was found in this scope. Correcting the stale
sovereign grant instructions at `scripts/build-web.sh:149-159` could remove
about eight counted lines, but changes terminal output and is a separate
corrective change, not a banked saving. A cap increase follows a verified
economizing pass and the operator's approval.

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
