# Onboarding's budget and resolver decision

Status: **answered by the operator on 2026-11-03.** The economizing pass is
complete; survey, revised estimate and passing full closing gate are recorded.
The unchanged retry passed in 1,141 seconds, exit 0
(`demo/runs/economize/2026-11-03-onboarding/data/check.txt`). The first
failure remains retained in `4555a32e`.

## The operator's answers

The operator: "Budget approved and we can plan on a free tier upstash deploy
(but again we don't deploy now)".

1. **Budget ceiling: approved at the upper estimate.** The caps are now
   3,494 core, 19,833 `.` and 8,090 `js`, landed in `scripts/lean_check.py`
   from the operator seat. Each future landing still raises only to where it
   lands.
2. **Resolver plan: Upstash Redis Free.** No paid overage is authorized, and
   nothing is deployed now. Before an adapter, `onboarding-store-proof`
   verifies that the free tier's limits and account controls prevent paid
   use, that reservation/closure are durable and atomic, and that a
   self-hosting path exists. The adapter is not built until that row passes.

Phrase-space size and request limits remain a paired decision after storage
behavior is understood (CHARTER:37-38); this answer does not choose them.

## The questions as asked

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

### Closing gate — first failure retained, unchanged full retry passed

The first `measure.py close` completed in 602 seconds, exit 1. Its complete
output is `demo/runs/economize/2026-11-03-onboarding/data/check-first.txt`
(committed in `4555a32e`), including:

```text
failed     tests (5 s)
failed     browser (161 s)
never-ran  scene give (chromium): Error: unable to connect to Podman socket: failed to connect: dial tcp 127.0.0.1:62636: connect: connection refused
never-ran  scene give_gone (chromium): Error: unable to connect to Podman socket: failed to connect: dial tcp 127.0.0.1:62636: connect: connection refused
never-ran  scene give_move (chromium): Error: unable to connect to Podman socket: failed to connect: dial tcp 127.0.0.1:62636: connect: connection refused
note       took 602 s in all
exit: 1
```

Seven Give tests refused to run without Podman (:51-100). Chromium failed
`the_home_page_lends_what_runs_here` at `tests/browser/main.rs:4574`, checking
the final chores lending redraw (:891-892). Its captured trace ended while
the final LENDING read was still in flight (:237-244); this suggests a
snapshot timing issue, but does not establish its cause.

Prerequisite recovery and focused retry, without product or test edits:

```text
$ podman machine start
Starting machine "podman-machine-default"
Machine "podman-machine-default" started successfully
$ podman info --format '{{.Host.Security.Rootless}}'
true
$ cargo test --quiet --workspace --test browser the_home_page_lends_what_runs_here -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 8.22s
```

All three commands exited 0. The isolated retry's trace completed with
`LENDING: answered (2 lent)` and the chores row present. One passing retry
does not resolve the initial failure or replace the full gate.

The full retry ran through the queue's await facility:

```sh
python3 -I demo/runs/economize/2026-11-03-onboarding/measure.py close
```

The driver ran `lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh`,
preserving its complete output and exit code in `data/check.txt`, and
recorded the unchanged final recount in `data/lean-after.txt` (exit 0).
The loop reported the driver exited 0 after 19m01s; its session log is
`ctl/log/sessions/onboarding-economize-2.await.log`. Actual gate output
(`data/check.txt:4-40,71-74`, selected verdicts):

```text
passed     principles pinned (0 s)
passed     lean and RUST_ARCH (1 s)
passed     trust floor (0 s)
passed     advisories (1 s)
passed     tests (493 s)
passed     ceiling canary (4 s)
passed     browser (156 s)
passed     types (2 s)
passed     typed lint (1 s)
passed     joining in a browser (43 s)
passed     pages (61 s)
passed     scenes (234 s)
passed     scene give (chromium)
passed     scene give_move (chromium)
passed     scene give_gone (chromium)
passed     conformance canary (91 s)
passed     app laws (41 s)
note       took 1141 s in all

exit: 0
```

Every gate passed, including all Give scenes. No product or test edits were
made between the failed gate and this retry. This closes the economizing
pass; it does not establish the cause of the first browser failure or judge
the future real-laptop onboarding scene. The two operator decisions above
are answered; storage proof remains the adapter's stop line.

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
