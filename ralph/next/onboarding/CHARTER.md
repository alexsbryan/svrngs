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

## Decisions that remain with the operator

- A budget ceiling, after the current-base economizing pass and estimate.
- Resolver storage/provider, after confirming operating cost, atomic
  operations and a self-hosting path. No adapter or hosting purchase before
  that choice.
- The phrase space and request limits, chosen together after the resolver's
  storage behavior is understood.

Starting this queue approves none of those decisions, a deployment, or a
release. Do not edit `scripts/lean_check.py`'s budgets. A budget ask comes
only after the economizing pass, names its cuts and explains why no further
ones were found.

## Reuse

The feature survey in `docs/ONBOARDING.md` §4 checked the repository and
commonwealth ring surfaces at `e1a43bcd2`. Recheck the current tree before
each row. Existing pieces include `Host::link`/`MintInvite`, the shared
`svrngs_node::http::client()` (`467d5d16`), owner-only `write_secret`, the
home page's invite and name forms, `web/install.sh`, and folder/open
surfaces. Extend them; do not make a parallel invitation, HTTPS client,
installer or folder owner.

## Stop lines

- The first row is a current-base survey and behavior-preserving cuts only.
  Its source survey at `dece85b1` predates the completed JavaScript queue and
  later join/update work; do not treat its estimates as the current budget.
- Keep behavior-preserving moves separate from behavior changes.
- Stop after the estimate for the operator's ceiling and resolver decision.
- The people scene is not judged by a simulated run. Record the laptop scene
  from `docs/ONBOARDING.md` §6; one run is not a rate.
- The operator owns publish and push.
