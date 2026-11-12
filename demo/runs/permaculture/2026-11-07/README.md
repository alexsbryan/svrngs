# Neighbourhood rehearsal — launch, 7 November 2026

Simulated. The operator asked to fix max-effort support and launch one run,
with DeepInfra Flash for both the scene's people and its calibration plants.
There is no Sol or Claude actor in this run. People outcomes are pending.

## Before launching models

- Xenia's new fake-provider check failed on the original implementation:
  effort 100 was absent from both requests. It passed after the fix, for
  numeric and named effort, including route/transcript recording. Missing
  values and client routes refuse the setting instead of dropping it.
- Type checking, vocabulary, and all 121 local Xenia tests passed. A live
  provider rejection then exposed a vacuous sum: no replies reported zero
  usage and spend. An added no-reply control failed first, then passed with
  nulls. The subsequent 16 focused provider/model tests and types passed.
- Xenia source remains at its existing 1,344-line cap: redundant header
  comments were shortened to fund explicit effort transport. No cap raised.
- The scripted browser proof passed:
  `target/permaculture-smoke/2026-11-08T00-19-28-053Z/report.json`.
  Its controls first reject the empty notebook and the original provider;
  then Ren writes a note, Jo declines, Sam retains provision, Jo later
  agrees, and Jo's actual app serves the retained note while Sam's old pod
  is absent. **Stop lending removes access while Jo's pod remains Running.**
  Scratch cleanup is not counted as the group's ending.
- Earlier smoke attempts are retained under `target/permaculture-smoke/`:
  first the current groups overview required entering Seed swap's workspace;
  then the observer incorrectly required a stopped pod to remain present.
  Screenshots, page text and the runtime's stop/remove code explained both.

## Provider transport correction

Isaura resolves `deepseek-flash` to
`deepseek-ai/DeepSeek-V4.1-Flash-1017` at
`https://api.deepinfra.com/v1/openai`.

Despite the older benchmark describing maximum as numeric 100, the live
OpenAI endpoint rejects that value with HTTP 422, listing named efforts
including `max`. The first attempt was stopped (exit -15), not salvaged as
a behavioural result. Its plant transcripts are retained under
`~/dev/xenia/runs/2026-11-08T00-22-28-465Z/`.

A minimal `max` preflight returned HTTP 200 and the requested model, with
34 prompt tokens, 3 completion tokens and reported cost $0.000004214. That
proves acceptance of the request, not the provider's internal effort. Its
record is `target/permaculture-run/provider-preflight.json`.

## The launched set

```sh
node src/run.ts run /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture --runs 1 --model deepseek-flash --reasoning-effort max --findings /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture/findings --source /Users/alexsbryan/dev/svrngs --debug
```

Working directory: `~/dev/xenia`, Node 24.22.1. Source baselines:
svrngs `5c0305ce778d37e324ae6dafa2d40500f2154362`, Xenia
`df729c381eb9e03e167259e9c61b886c7a628363`, both with working-tree
changes captured as patches at launch. Other development is concurrent;
this is not claimed to be a clean-revision run. The launcher captures
source state again at exit.

- Supervisor PID at launch: **8204**; Xenia runner: **8208**.
- State and eventual exit: `target/permaculture-run/run.json`.
- Debug log: `target/permaculture-run/run.log`.
- Completed accounts and summary will be in Xenia's `runs/`.
- The previous attempt's state/log/patches were archived alongside these.
- The existing rootless Podman VM was started, without changing its
  configuration. `caffeinate` holds idle sleep off only for this run.
- Stop this run through `kill -TERM 8204`; the supervisor forwards it to
  Xenia, and the scene's stdin closing ends its scratch world.

Focused compilation/clippy/lint/format passed. The full repository gates
were not run. A lean-check sample during concurrent product work reported
20,846 non-JS lines against 20,835; none of this scene's test code changes
that rung, and no unrelated product code was changed to make it pass.

Plant verdicts, community-scene outcomes and full-run spend must be read
from the eventual artifacts, not inferred from a successful launch. Xenia's
multi-scene `passed` is a completed conversation, not proof of a successful
handover or shutdown; the world observations are separate evidence.
