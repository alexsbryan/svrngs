# Open the app; stop keeping it without losing the work

Implementation brief, 7 November 2026. The operator accepted these two
improvements after the neighbourhood rehearsal. This describes work to
build, not shipped behaviour. `DESIGN.md` remains the current product
design; update its affected paragraphs with each implementation landing.
This brief neither changes PLAN's order nor approves a size-budget raise.

## 1. The outcome and the cut

A neighbour can find and open the shared thing without mistaking its
files for the app. Someone keeping an app can stop providing it, preserve
its state, and let the group explicitly start it again from that state.

Two independently deliverable units:

- **A — access:** clearer opening actions on the computer's home page and
  an opening action on Give's existing app card, including after a move.
- **B — stop with a copy:** an intentional stopped state owned by Give,
  with truthful progress/failure, durable retention, and explicit restart.

The first build targets computer browsers, the surface this scene used.
Both supported home layouts get A's shared components. Keep phone access,
raw TCP/UDP Play, existing consent, lending scope and membership checks
working; a new phone-specific management flow is not part of this unit.
There is no new dashboard, general app registry, archive browser, automatic
host selection, stable-routing redesign, or change to handover consent.

## 2. A — opening is the obvious action

**A1. An HTTP app's primary link says `Open <displayed name>`.** It goes
through the same opener and any existing consent as the current name link,
opens in a separate tab, and leaves the group workspace available. A lend
that requires a yes still requires it. Do not add a second opening path.
Keep the existing name/owner rules; this does not add title metadata or
rename people's services. Raw ports retain their existing Play action.

**A2. Files are a distinct, secondary destination.** The group workspace
has **Shared apps** followed by **Shared files**, rather than one heading
"Shared apps and folder". The files action says **Open shared files in
Finder** on macOS, **Open shared files in your file manager** elsewhere.
For multiple folders, identify each separately; retain the existing folder
selection and names. The full local path is available under **Folder
location**, collapsed initially. Retain the existing opening/failure
feedback; opening a native window need not change the browser's screen.

**A3. Connection explanation follows the apps.** Move the current backup-
route explanation into a collapsed **Connection details** after the normal
use controls. The actual route remains inspectable. An inability to reach
the group/app, or a decision the person must make, remains visible where
it prevents their action. This is presentation, not a fallback-policy change.

**A4. Give's running HTTP-app card has `Open <card title>`.** It navigates
to that app as the person asking, on the computer the current keeping
names. It does not start, adopt, move, or change the app. After a move it
opens the new keeper's lend, not a URL cached from the former keeper.
Opening must not require leaving Give to rediscover the app on Home.

- Resolve by group id, current keeper device key and card kind, never by a
  person's display name, guessed port, or remembered URL.
- A keeping recorded is not proof the service is ready. If it is not yet
  offered, say **Starting on Jo's computer…** (or **Moving…** while a move
  is pending); when the computer cannot be reached, say **Jo's computer
  did not answer**. Offer **Try again**, not a successful-looking link to
  the old keeper. A stopped app has no Open action.
- Non-HTTP cards retain their current client/Play instructions; do not
  manufacture a web page for a game port.

## 3. B — the user-visible contract

**B1. Only the current keeper's person can request the stop.** Put
**Stop running and keep a copy** on their Give app card. Use the verified
person key, as Give's move permission does, including that person's other
devices. Other members can read its status but cannot stop their host's
program. No group vote or additional permission screen is introduced.

Beside the action, say: **Stops this app for the group. Its data stays in
the group's shared folder.** There is no deletion in this flow.

**B2. Stop is a persistent intention, not unshare.** It prevents this
keeping from restarting or being taken over automatically, including after
the keeper program or computer restarts. Closing the browser does not
cancel it. The existing **Stop lending** action remains access control
only; neither its semantics nor ordinary self-hosted apps' lifetimes change.

**B3. A completed stop means all three facts are true:**

1. The app is no longer lent to its group.
2. Its runtime is stopped; a pod removed by the runtime counts as stopped,
   but a failed inspection does not.
3. A complete final copy of its declared state and the exact card version
   is retained in this computer's group folder, identified by path and sum
   in Give's record.

Say **Stopped. A copy is saved in the group's folder on Sam's computer.**
Also say **Other computers receive it when they are online.** Success does
not assert another replica has received the bytes. With peers offline the
stop may complete locally; do not wait for a replication quorum.

**B4. Failure is not completion.** The card distinguishes:

| State | What the page says and allows |
|---|---|
| Request recorded, keeper not yet responding | **Waiting for Sam's computer to stop it.** No Open or restart. |
| Work in progress | **Stopping and keeping a copy…** No conflicting action. |
| Withdrawal or runtime stop failed | Name the failed step; say the app may still be running. Keep the stop intention. |
| Runtime stopped, final copy failed | **Stopped here, but the final copy could not be saved: <reason>.** Do not claim the group has its latest state. |
| Stopped and copy recorded | B3's completion, retained-copy information, and B6's restart. |

Retry the same pending operation using Give's existing retry interval,
without another person's yes. Do not auto-restart after a failure. Keep
local volumes and previous copies intact. If a record write itself fails,
log the precise local failure; other pages keep reporting an unacknowledged
request, never infer success. A person may always stop their own process
outside this flow, but that is not a completed stop-with-copy.

**B5. Retention is structural.** Pin the final copy and the exact card
bytes (including image digests), not only a version hash or whatever card
is currently on the shelf. Include those bytes in the verified checkpoint
before its atomic publication; parse them through the normal card rules on
restore. Retaining a card does not promise its image is available offline.
Pin this checkpoint while this app is intentionally stopped. Ordinary
rolling-copy pruning must not remove it. Keep it through restart until a later successful
keeping has made a replacement recoverable copy. This unit exposes no
delete/purge control and makes no promise against loss of every disk that
holds the bytes.

**B6. Starting again is a person's explicit act.** Any current member on
a computer that can keep apps may choose **Start again on this computer**,
with the existing card's asks. This preserves Give's existing rule that
any capable member may volunteer; the former keeper is not indispensable.
Start from the pinned final copy, not fresh volumes or a guessed older
copy. Verify the named sum and card version before restoring. Missing
bytes mean waiting; a missing/incompatible card or corrupt copy is a
named refusal. Never substitute an empty app. Start and lend only after
the existing health check succeeds. A failed restart preserves the stopped
copy and remains unavailable, with the failure shown.

## 4. Minimal design and ownership

**svrngs owns reach, identity, scoped lending and folders. Give owns
keeping, stopping and restarting.** No `stopped` fact or app-specific word
goes into `svrngs-core`, the home page's model, or the member door.

### A's seams

- Reuse `home/page/lend-name.js`'s opener and consent machinery for A1;
  `grouped.js`, `group-folder.js` and `folder.js` for A2; `group.js` for A3.
- Give already has the card kind and current keeper from its fold, but its
  carried `/svrngs/acts` response has no general opening address. Inventory
  `/to`, the origin/session constructors and `/svrngs-reads` before adding
  code. `/svrngs-reads` grants declared cross-app reads; it is not a shortcut
  to silently grant Give access to the services it manages.
- A4 needs navigation to **one known lend**, not a registry or an iframe.
  Reuse the existing top-level opening machinery, with the smallest generic
  carried-origin adapter if necessary. It accepts the known lender/key and
  lend name, confines it to the carried app's group and the asker's current
  lending audience, and preserves all origin proofs and opening consent.
  The node knows no Give lifecycle. Give receives no home token, another
  app's session key, wildcard read permission or cross-origin fetch access.
  A scripted fetch must not obtain navigation-only opening credentials.

### B's state and effects

Extend `apps/give/fold/`, the single fold compiled for both program and
page. Represent **running**, **stop pending** (including its latest
failure), and **stopped with copy** explicitly. Absence is still "never
kept", not "intentionally stopped". Refusals and transitions trace at debug.
An explicit restart uses the existing start/restore progress and failure
path, retaining its stopped checkpoint until a new recoverable copy exists;
"restarting" is not evidence the service is ready to open.

Use the existing `over` keeping-act id for stale-action protection. The
minimum record additions are a person's **stop request**, the keeper
computer's **stop completion** naming that request plus `card/from/sum`,
and a keeper-reported **stop failure** naming its step. These are distinct
from the existing `failed` meaning "a restored app did not start". Wire
names can follow surrounding style; authority and reference checks are
requirements: only the named keeper device may acknowledge effects, even
when its person's other device requested them. An existing `taken` can
express an explicit restart if the fold requires the stopped state's exact
copy and normal card consent.

On the keeper computer, reuse `Keeper::plan`, `snapshot`, `said`, `Held`,
the runtime's stop/restore and the existing sum verification. Execute:

1. Record the authorized stop intention before local effects.
2. Withdraw lending, then gracefully stop the runtime.
3. Snapshot the **stopped** volumes, so a write between a live snapshot and
   shutdown cannot be lost. Reuse the runtime's atomic snapshot publication.
4. Verify/pin the complete copy and record the stop completion naming it.

Persist enough local progress to resume after a crash at any boundary.
Replaying effects is idempotent: an already withdrawn lend or explicitly
absent pod is acceptable; permission errors and connection failures are
not absence. A complete copy awaiting its record write is reused rather
than deleted. Until completion, `plan` must not treat this app as wanted
running, abandoned, or eligible for ordinary takeover.

For competing actions, use record order and the current `over`: an already
accepted move makes stop refuse **Already moving**. A stop request accepted
first invalidates an unanswered offer and refuses later move/accept/take
actions for that keeping. A stale former keeper cannot stop a successor.
Duplicate requests/acks do not create a new copy or restart. A stop failure
does not remove the pending intention. Concurrent restarts from a completed
stop use the existing one-successor/stale-action rule.

Ship the updated carried Give page, its fold build and keeping program
together, preserving the per-person yes for new carried code. This contract
requires the updated keeping program. Its latest existing `ready` entry
must advertise support for stop-with-copy. With no such report from the
current keeper, show **Update Give on Sam's computer to stop it here**
instead of an action an old program cannot execute. Restart also requires
the receiving computer's reported support. Old programs ignore unknown
Give acts and may restart an old keeping. Document the minimum version and
do not advertise mixed-version or downgrade-safe stopping without a separate
compatibility proof. Old page actions are still judged by the updated fold.

## 5. Acceptance — facts before model accounts

Each new check first fails on its named planted input. These are required
checks, not claims that the original rehearsal proved them.

| Check | Required observation / bad input to plant |
|---|---|
| Open versus files | Keyboard and mouse open the actual app through the labelled Open action; the files action opens the native folder. Do not assert a native action is inert because the browser stayed still. |
| Consent/scope preserved | A consent-requiring lend still asks; another group's or removed member's target refuses. A top-level Open cannot become a cross-app read/signing capability. |
| Open after move | Give's Open reaches the new keeper and reads the existing note; plant the former keeper's cached address. |
| Stop ownership | Another person and a stale former keeper cannot stop it; the current keeper's person can. |
| Program support | A legacy `ready` without stop-with-copy support shows the update instruction, never a working-looking stop/restart action. |
| Final-copy ordering | A last write before stop survives restore; plant live-copy-then-later-write-before-stop. |
| Stop means stopped | Lending withdrawn, runtime observed stopped/absent, final copy whole and recorded. Plant unshare-only while the runtime remains running. |
| Failure truth | Inject withdrawal, runtime, copy and record failures separately; no false success, no discarded volumes, no auto-restart. |
| Recovery | Restart the keeping program after each effect boundary; it finishes the same stop without restarting the app. |
| Retention and restart | Leave the app stopped through pruning/restarts; another capable member explicitly restores its pinned note. Missing/corrupt bytes or wrong card never start fresh. |
| Races | Stop versus pending/accepted move; repeated stop; late ack from an old keeping; two restarters. One consistent result from both fold builds. |

Then one short observation with a real newcomer: **find the swap time,
leave the envelope note, and reopen it after a prepared handover**. Record
the first wrong action, any help, and what actually opened. Do not use a
model's patience cutoff as a human result. No automatic full Xenia rerun.

## 6. First work and evidence

Land A's shared home components first, then Give's Open, then B's
fold/runtime/page together. Keep behaviour-preserving extraction separate
from behaviour changes. Before new types/modules, check existing code here
and the commonwealth ring inventory. Measure the product-line delta and
economize before asking for any cap change. No budget is granted by this
brief. Each landing updates `DESIGN.md`, Give's README where relevant, and
runs the checks the change reaches; the full gate precedes any push.

The motivating run is `demo/runs/permaculture/2026-11-07/README.md` and
Xenia's `runs/2026-11-08T00-24-18-480Z/`. Its world attachment demonstrates
note retention and an actual consensual handover; the scripted proof also
demonstrates unshare leaving the app running. It did **not** validate a
people failure rate or fifteen distinct UX bugs. Native-folder visibility,
artificial give-up messages and the fixture's naming mismatch must not
become product requirements. Existing handover consent and state transfer
are retained, not rebuilt.
