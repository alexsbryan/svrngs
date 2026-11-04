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
