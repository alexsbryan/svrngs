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

## The operator's decisions, 2026-11-03

- **Budget:** the upper current-base estimate is approved: ceilings 3,494
  core, 19,833 `.`, and 8,090 `js` (`scripts/lean_check.py`). Each landing
  raises only to where it lands.
- **Resolver plan:** Upstash Redis Free, with no paid overage authorized.
  Deployment is not authorized now. Before an adapter, prove the free-tier
  cost controls, atomic reservation/closure operations and a self-hosting
  path in `onboarding-store-proof`.
- **Phrase space and request limits:** still paired; choose them after the
  storage behavior is understood.

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

## Stop lines

- The economizing pass is complete; its survey at `dece85b1` predates the
  completed JavaScript queue and later join/update work, so only the
  current-base survey and estimate govern the approved budget.
- Keep behavior-preserving moves separate from behavior changes.
- Do not build the adapter until `onboarding-store-proof` verifies the chosen
  free-tier plan's cost boundary, atomic operations and self-hosting path.
- No deployment or paid usage in this queue without a new operator decision.
- The people scene is not judged by a simulated run. Record the laptop scene
  from `docs/ONBOARDING.md` §6; one run is not a rate.
- The operator owns publish and push.
