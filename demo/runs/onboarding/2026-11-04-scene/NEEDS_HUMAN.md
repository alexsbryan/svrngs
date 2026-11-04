# Onboarding scene — operator staging and people bar

## Question

How should the real-laptop scene be staged under the current no-deployment /
no-publish restriction, and who will supply the two attendees and judge the
people bar? Please choose the reachable resolver/canonical-page/download
setup (with matching scene-build provenance), arrange the host plus two other
laptops including a fresh install, and set the people bar before data.
Any new deployment or publication needs explicit operator authorization.

## Committed progress

`896fcd60` commits `PREDICTIONS.md` before any scene observations, together
with `demo/onboarding-room.md`. The run-of-show preserves all of the assigned
row's steps, including interrupted joining, closure through a copied ordinary
invitation, and continued file exchange by existing members. No scene data
has been collected and no simulated welcome is offered as a people result.

## Why the worker stops here

- `ralph/next/onboarding/STATE.md:247-253` requires real laptops and explicitly
  stops for the operator's people bar. None has been supplied for this scene.
- `ralph/next/onboarding/CHARTER.md:54-58,88-91` forbids deployment/release now,
  rejects a simulated people scene, and reserves publishing to the operator.
- `resolver/README.md:1-8` describes an undeployed Fetch handler, no listener
  and no deployment configuration. `docs/ONBOARDING.md:115-118` requires a
  separate canonical page/distribution origin; existing published downloads
  cannot be silently assumed to contain this checkout's native-arrival work.
- `docs/ONBOARDING.md:299-305` requires the host plus at least two other
  laptops and a fresh installation, with no verbal directions. This shell
  session supplies neither real attendees nor their observations.

The worker inspected the existing run evidence, wrote predictions and a
staging/run procedure, and stopped before provisioning or substituting a
single-host test. This is an operator fork, not a request to weaken the check.

## Commands and actual output

Startup inspection:

```text
$ git status --short
?? demo/runs/v1.6/2026-11-03/publish.json
?? demo/runs/v1.6/2026-11-03/publish.log

$ git log --oneline -10
abc171af ralph: onboarding-native-arrival done
c0b58cd1 onboarding: record native arrival closing gate
3f29ba16 onboarding: bring attendees straight to their native group
4e8caa6f ralph: onboarding-host-controls done
23f25810 onboarding: record host controls closing gate
96f40310 onboarding: add recoverable host invitation controls
72cd9f05 ralph: onboarding-resolver done
e900f4ea onboarding: record resolver closing gate
9ab84b4f onboarding: preserve exact request bytes through UTF-8 decoding
61ea3533 onboarding: build replay-safe short invitation resolver

$ git diff --cached --check
(no output; exit 0, before predictions commit)

$ git commit -m "onboarding: commit scene predictions before people data" [body omitted]
[main 896fcd60] onboarding: commit scene predictions before people data
 2 files changed, 127 insertions(+)
 create mode 100644 demo/onboarding-room.md
 create mode 100644 demo/runs/onboarding/2026-11-04-scene/PREDICTIONS.md
```

The protected prior-session `publish.json:2-5` names source `75f9fd3b`, a
different release worktree and `state: running`; `publish.log:1352` ends in
compilation, not a success verdict. They were read, not changed or staged.
Those artifacts do not establish a deployed onboarding scene.

## Verdicts and resumption

- Predictions-before-data: **passed**, committed at `896fcd60`.
- Whitespace check for that commit: **passed**, exit 0.
- This session's `scripts/check.sh`: **never-ran**. No scene has begun; the
  required pre-scene gate is still owed, not inherited from the arrival row.
- Real-laptop scene, interrupted join, closure and human welcome: **never-ran**.
- Operator's people-bar judgment: **could-not-judge**, bar and observations
  are not yet supplied.

After the operator answers, use `demo/onboarding-room.md`. Run
`lockf -k /tmp/svrngs-gate.lock scripts/check.sh` on the chosen scene commit,
retain its complete output and exit status, and require exit 0 before people
start. Use `ralph-result await` for a gate longer than the foreground timeout.
Keep the committed predictions unchanged. Record actual observations and
only then write the run README and seek the operator's judgment. Keep the
phone-write sync row after onboarding and leave queue status markers to the loop.
