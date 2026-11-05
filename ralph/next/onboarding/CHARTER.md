# svrngs onboarding — welcoming a room

`docs/ONBOARDING.md` is the feature contract and acceptance scene. Read it
whole before changing code. `docs/DESIGN.md`, `NORTH_STAR.md` and
`docs/principles/` govern this queue.

## The operator's direction, 2026-11-03

The operator deferred publishing the recent multi-ring join-flow fixes and
said: "Instead of releasing that right now I want to start the work in
docs/ONBOARDING.md". `docs/PLAN.md` §1 now puts onboarding before the still
owed phone-write sync row; that row remains queued after onboarding.

## What this queue builds

A member creates and closes a temporary, easy-to-type address from the
group's page. Attendees give a name once, install svrngs on a Mac or Linux
computer, arrive at the actual shared folder and can contribute a text file.
The optional haiku prompt is group content, not a platform type or a join
requirement. The welcome must explain what to do next when a step fails.

The resolver holds invitation capability and belongs to the trust boundary.
It holds no group record or shared files. The phrase space and rate limits
must be chosen together; publication and closure must be replay-safe, and a
closed phrase must never name a later invitation. Admission remains the
issuing computer's existing `Invite` and redemption rules.

## The operator's decisions, 2026-11-03 and 2026-11-04

- **Budget:** the upper current-base estimate is approved: ceilings 3,494
  core, 19,833 `.`, and 8,090 `js` (`scripts/lean_check.py`). Each landing
  raises only to where it lands.
- **Resolver plan:** Upstash Redis Free, with no paid overage authorized.
  Deployment is not authorized now. The operator chose option A on
  2026-11-04, then provided the live Free database
  `https://true-pheasant-225442.upstash.io` with its key in Keychain (service
  `svrngs-upstash`, account `alexsbryan`) and asked to move forward. The live
  atomic proof is in `demo/runs/onboarding/2026-11-04-store/live.txt`. The
  written-answer gate is superseded by a permanent generation key: if the
  store looks reset, the resolver refuses short-address service rather than
  reuse the URL namespace.
- **Phrase space and request limits:** approved as proposed ("The rest is
  reasonable too"): EFF Short Wordlist 1, five words, 51.69 bits; 30/min and
  300/day per IP, 20k/month global; 100 active secrets; 100k reservations or
  100 MB; 8 KiB bodies; one-day invitations. Shorter phrases or higher limits
  require the probability recalculated together.
- **Limits fail gracefully** (2026-11-04: "We should fail gracefully in the
  case we hit any of those limits"): every provider limit or error is shown
  as "joining is temporarily unavailable" with retry guidance, never as an
  ended invitation; existing members are unaffected; no partial publication
  or close, no paid upgrade, no tight retry loop, and the host's pending
  receipt and underlying invitation stay recoverable.

No Upstash/Vercel deployment, purchase or release is authorized now. The
economizing pass found no supported product cut; its candidates, instruments
and could-not-judge clone result are recorded in
`demo/runs/economize/2026-11-03-onboarding/SURVEY.md` and
`ralph/next/onboarding/NEEDS_HUMAN.md`.

## Reuse

The feature survey in `docs/ONBOARDING.md` §4 checked the repository and
commonwealth ring surfaces at `e1a43bcd2`. Recheck the current tree before
each row. Existing pieces include `Host::link`/`MintInvite`, the shared
`svrngs_node::http::client()` (`467d5d16`), owner-only `write_secret`, the
home page's invite and name forms, `web/install.sh`, and folder/open
surfaces. Extend them; do not make a parallel invitation, HTTPS client,
installer or folder owner.

## Selected-ring lending — preapproval, 2026-11-04

After the source survey in `docs/LENDING_SCOPE.md`, the operator said:
"Ok I approve the minimum amount of lines as pre approval. Let's add the
rows to the queue (as few as necessary)". Two rows follow the current
welcome/portal/OIDC fixes: `onboarding-scope-economize`, then
`onboarding-scoped-lending`. The people scene remains after them.

- Reuse the current stack: an explicit stable-group-ID audience on the
  lender's existing declaration, shared lender-side enforcement, no automatic
  audience expansion when joining a group, and legacy lends paused until
  explicit selection. Preserve owner access, other selected groups,
  credentials and unrelated traffic. Implement the recommended bounded
  HTTP/WebSocket/raw withdrawal as well as denial of new requests.
- **Zero additional core product lines.** This is node-local lending policy,
  not a new membership/grant rule. No new dependency, hosted service, policy
  database, ACL framework, per-ring endpoint or signed reach-pass format.
- The fresh economizing pass comes first. Each supported cut is its own
  behavior-preserving commit; lower caps to where cuts land and prove the
  ratchet on its plant. The pass records the minimal remaining work and
  revised measurements; it does not park for the budget already preapproved.
- **Only necessary actual growth is preapproved.** Use available headroom;
  when a cap adjustment is required, apply only the increment needed to put
  the measured landing one line below its cap. Do not raise all caps to the
  survey's upper estimate in advance. Each adjustment cites this decision,
  the fresh pass, its cuts (or none established), why the remainder is new
  work, and its failing budget plant before the passing recount.
- The survey's +700 non-JS / +235 JS upper bounds over the pass's baseline
  are review guardrails, not a spending target. More than those, any core
  growth, a new dependency or a wider design requires another operator
  decision. Give's app-only growth is separately reported, not hidden in
  platform savings. If the harness refuses an authorized cap edit, provide
  the supervisor its ready diff rather than bypass permissions.
- The implementation updates DESIGN in place when behavior changes and
  proves the House-versus-Trip scene with real node processes. Automated
  proof does not substitute for the remaining people welcome.

## Director

### Demo-deadline override, 2026-11-04 local

The operator explicitly requests independent parallel workers on DeepInfra's
DeepSeek Flash and fewer check.sh runs for a demo in a couple of hours.
`SCOPE_PARALLEL.md` is the shared API/ownership/check contract. Use this native
backend queue plus a separate Node-only UI worktree/queue; the two-Rust-lane
memory guard currently cannot admit a pool of two. This splits the same
product work without adding feature scope; UI starts against the frozen
contract while reconciliation/economy finish. All roles use the
live-probed `deepinfra/deepseek-ai/DeepSeek-V4.1-Flash`, medium.

Intermediate rows close with their focused affected checks, counts and honest
integration handoffs. The existing final audit may merge the completed clean
UI branch after its queue is done, integrates both workstreams and owns
one unchanged locked full gate before demo readiness. This supersedes earlier
per-row full-gate requirements in STATE/director reports; no check is deleted
from the final gate, and actual failures/product changes justify reruns.
Keep total necessary-growth approval across the lanes, not two budgets.

Completion sequencing (supervisor, 2026-11-04, after the operator's request
to finish): land grouped-home reconciliation, close the existing lending
economizing pass, implement scoped lending, then complete the existing
strict audit auto-1 across the full implementation range before the people
scene. Auto-2 stays as completed history. Automatic cadence generation is
off; the explicit final audit preserves every outstanding requirement. Do
not add features or redundant audit/preparation rows to this completion path.

Grouped-home B's measured remainder was separately approved by the operator
on 2026-11-04: JS cap **8,090 to 8,188 (+98)** at 8,187 counted lines after
the preserving-economy review; no core or non-JS increase. This is not paid
from lending's preapproval. The owner released claim
`ring-32fb4a66f5b1a57a`; reconciliation may land the scoped product and its
evidence, remeasure, and run a fresh locked gate. A higher remainder needs a
new decision; do not re-ask for the approved 98.

The director may split or re-scope a row that cannot execute, keeping every
check and the row's demo. It may apply only the incremental scoped-lending
budget adjustments explicitly preapproved above; other raises remain the
operator's. It may not weaken a check or authorize a deployment, purchase,
publish or push. A row that leaves tracked changes uncommitted is
checkpointed as `WIP: <row>` before any later row runs;
never discard it.

## Stop lines

- The economizing pass is complete; its survey at `dece85b1` predates the
  completed JavaScript queue and later join/update work, so only the
  current-base survey and estimate govern the approved budget.
- Keep behavior-preserving moves separate from behavior changes.
- Do not build the resolver adapter until the live Free database, the reset
  guard and the graceful-failure tests pass. The operator provided the live
  database and asked to move forward; the written-answer gate is superseded.
  Native publication and the host controls may proceed independently.
- No deployment or paid usage in this queue without a new operator decision.
- The people scene is not judged by a simulated run. Record the laptop scene
  from `docs/ONBOARDING.md` §6; one run is not a rate.
- The operator owns publish and push.
