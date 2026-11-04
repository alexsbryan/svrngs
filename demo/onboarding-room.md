# Welcoming a room — laptop run-of-show

Contract: `docs/ONBOARDING.md` §6. Predictions:
`runs/onboarding/2026-11-04-scene/PREDICTIONS.md`. This is preparation, not a
recorded run. The operator supplies the people bar and staging decision before
the scene. No deployment, publish or push is authorized by this document.

## Before anyone starts

1. Arrange a host and at least two other real Mac/Linux laptops, with a
   fresh installation on at least one. Record OS, browser, existing-install
   status, participant aliases and the precise code/build each uses.
2. Obtain the operator's people bar, including any time bound and how help is
   scored. Do not invent a bound to fill `succeeded_in_time` in the demo record.
3. Resolve staging: the resolver is not deployed (`resolver/README.md:1-8`),
   and current distribution artifacts are not evidence of this checkout's
   arrival behavior. The operator must choose how the typed address, canonical
   page and matching Mac/Linux downloads will be reachable. Record their
   origins and build provenance; never put live invitation tokens or credentials
   in the committed record. Do not implement a deployment as scene setup
   without new authorization.
4. Commit predictions before collecting any data. At the chosen scene commit,
   run `lockf -k /tmp/svrngs-gate.lock scripts/check.sh` to completion, saving
   stdout/stderr and the exit code. Run in the foreground with a 600000 ms tool
   timeout, or hand off through `ralph-result await` if it needs longer. Start
   no scene until the full gate exits 0. A previous commit's green is not a
   substitute. Rerun the named isolated known flake before classifying it.
5. On the host, prepare the group's existing root folder with `WELCOME.txt`
   using ONBOARDING §2's optional prompt and an example text file. Record the
   actual folder path. Capture the starting membership. On an attendee with
   an existing installation, preserve a same-named unrelated folder to observe
   collision handling; do not replace their data.
6. Agree on screen recording and a timestamped observer log for each laptop.
   Record assistance verbatim. Protect recordings containing live capabilities;
   commit only redacted evidence and hashes of original recordings if needed.

## The scene

1. The host presses **Invite people here**, copies the address, then reloads
   the page and confirms its controls recover. Restart the host's svrngs and
   confirm recovery again before attendees start. Record the observed expiry
   and reachability guidance.
2. Show the address. Each attendee types it, recognizes the invitation, gives
   their chosen name and follows the installation instructions. Observe without
   verbal directions. Record address corrections, name prompts, reading time,
   download/start time, and any assistance or refusal.
3. For one attendee, interrupt the join after installation has stopped an
   existing service. Record the interruption point, service recovery and the
   retry using the same command. Preserve the identity and compare membership
   before/after; count actual new admissions, not just rows visible on screen.
4. Each attendee finds the correct local folder from their authenticated group
   page and opens it. Record the path the node reports and the folder actually
   opened, including any collision suffix. Each creates a text file of their
   choosing beside `WELCOME.txt`; a haiku is optional.
5. On every laptop record each contribution's first appearance, filename and
   content hash. Keep the source creation timestamp and each destination's
   observation timestamp; record clock differences before deriving transit
   times. Do not report timing from unsynchronized clocks as exact latency.
6. Before closure, retain an ordinary invitation privately for the closure
   probe. The host presses **Close joining**. Only after the UI reports closed,
   try a new identity via the short address and the retained ordinary invitation.
   Record both refusals and unchanged membership. Do not erase an attendee's
   identity to make this probe: use a separate disposable profile/data directory.
7. An existing member adds another text file. Record its appearance and matching
   hash on the other joined laptops after closure. Capture final membership.

## The record

Create the scene README only from the observations, linking the raw redacted
logs, recordings and pre-scene full-gate output. Record the predictions commit,
scene commit, build hashes, staging choice and operator's people bar. Every
number must point to a log event or inventory. Use the `demo/README.md` record
shape where applicable; do not fill missing timing or judgment with zero.

Keep technical checks, observed human behavior and the operator's judgment
explicit. A missing recording is missing evidence; an unattempted step is
never-ran. Retain failures and assisted attempts. Ask the operator to judge
the people bar from the record. A simulated run cannot close this scene.
