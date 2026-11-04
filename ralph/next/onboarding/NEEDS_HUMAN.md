# Onboarding's budget and resolver decision

Status: survey and revised estimate recorded; **full closing gate still
never-ran**. This package is prepared for the operator, but is not yet a
claim that `onboarding-economize` passed. Commit the gate's actual output
and exit code here before returning the unit's needs-human result.

## Questions, in order

1. **What budget ceiling may onboarding spend?** The current-base survey
   (`194aa994`, `demo/runs/economize/2026-11-03-onboarding/SURVEY.md`)
   selected no product cut. `docs/ONBOARDING.md` §4 now estimates
   **+230–391 non-JS**, **+345–590 JS**, with **+0–16 core included in
   non-JS**. If the operator funds the upper estimate, the one-past ceilings
   would be **19,833 `.`**, **8,090 `js`**, **3,494 core** (increases of
   391, 590 and 16 over current caps). These are proposed ceilings, not edits
   or approval. Each future landing would raise only to where it lands.
2. **Which resolver storage/provider and operating-cost limit should we
   use?** ONBOARDING §5's provisional candidate is Vercel plus Upstash
   Redis; no database is provisioned or authorized by this pass. Choose it
   only after confirming the account's cost cap, durable atomic reservation/
   closure operations and self-hosting path. Alternatively name the chosen
   self-hosted transactional store and where the resolver will run. No
   provider abstraction or adapter has been built.

Phrase-space size and request limits remain a paired decision after storage
behavior is understood (CHARTER:28-35); this package does not choose them.

## What was tried and why it did not pay for the feature

- Surveyed current reused invitation, signature, HTTPS-client, secret-file,
  link/name/Copy, installer and folder/open surfaces. Exact sites and
  per-file counted lines are in the committed survey and `data/sites.txt`.
- The latest applicable compiler demotion, test-cfg, line-window and page
  name/structural instruments ran. Native and wasm scratch checks passed.
  Public test helpers are still used; moving inline tests would only alter
  accounting; similar room/Play and nonce/redirect code has distinct policy.
  No behavior-preserving product deletion was established. All existing
  tests and all caps remain unchanged.
- Semantic clone analysis **could-not-judge**: installed binaries say the
  developer-only command is not available. This is not a manual clone report
  or evidence of zero clones (`data/clone-availability.txt:2-20`).
- The shared HTTPS client extraction already landed (`http.rs:10-23`,
  `467d5d16`); no further extraction saving is available. Native status,
  persistent receipt, authenticated publication and replay-safe closure do
  not exist yet (`invite.rs:151-159`, `home.rs:597-628`). The current lookup
  is newest for a key (`svrngs-runtime/src/lib.rs:227-247`), so stale close
  protection cannot be assumed from it.
- Installer recovery is still actual work: stop at `web/install.sh:113-116`,
  join/refusal at :154-158, restart at :197. Changing that path is behavior
  work, not a saving claimed by this pass.

## Commands and actual output

`python3 scripts/lean_check.py` before survey (`75f9fd3b`) and again after
survey (`194aa994`), both exit 0:

```text
crates/svrngs-core     3477 / 3478 lines
.                     19441 / 19442 lines
js                     7499 / 7500 lines
swift                     0 / 1 lines
```

`cargo check --workspace --offline`, exit 0
(`demo/runs/economize/2026-11-03-onboarding/data/check-workspace.txt`):

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
```

The scratch's successful final commands were `cargo check --workspace
--offline --message-format=json` (native, round 24) and `cargo check
--target wasm32-unknown-unknown --offline --message-format=json` (web,
round 5). Both warning summaries end:

```text
build-finished success: True

exit: 0
```

The original count and all run files are under
`demo/runs/economize/2026-11-03-onboarding/data/`. The survey driver reuses
the earlier instruments unchanged; no new gate was added.

### Closing gate — owed

Run once through the queue's await facility (do not background it):

```sh
python3 -I demo/runs/economize/2026-11-03-onboarding/measure.py close
```

The driver runs `lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh`,
preserving its output and exit code in `data/check.txt`, and records the
final recount in `data/lean-after.txt`. After exit 0, paste the verdicts
here, commit the logs and this package, and report needs-human for the two
operator decisions above. If it fails, preserve its actual failure and
resolve that before reporting a passing pass.

## Storage evidence and remaining confirmation

The inherited price check in `194aa994:docs/ONBOARDING.md:179-185` cites
<https://upstash.com/pricing/redis>: one free database, 250 MB and 500,000
commands/month; usage commands $0.20/100,000, with Vercel execution and
bandwidth separate. Those are published-plan figures, not this account's
limits or a measured bill. In this session a web-reader recheck of that URL
and <https://upstash.com/docs/redis/features/restapi> both timed out; current
cost and atomic REST operations therefore remain **could-not-judge** here.

The official Redis install page was readable:
<https://redis.io/docs/latest/operate/oss_and_stack/install/install-stack/>
documents Docker, Linux and macOS installation. This establishes a possible
self-hosted database, not an already-compatible self-hosted resolver. A
resolver transport, persistence policy, operational owner and equivalent
atomic/replay behavior still need confirming before choosing the adapter.

Required store behavior is `docs/ONBOARDING.md:78-86`: recover lost replies,
create and close atomically, retain non-reusable reservations after erasing
expired invitation secrets, and never reopen on replay. A free allowance
does not prove these semantics; an in-memory code store cannot supply them.
