# Too many words before acting: Money

- status: open, not seen since the set `/home/alexbryan/dev/xenia/runs/2026-10-27T03-11-14-983Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `2a10d4d`, the product `ccd48cb4`
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T03-11-14-983Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `2a10d4d`, the product `ccd48cb4`
- kind: hard (the instrument's): `wordy`; never both

## Where

Money. What was read there first: Say you were in on it / What it was for: / How much: / Money / Who pays whom / What people paid for

## Who, and how often

- `ana`: 2 of 3 accounts (Ana run 1, Ana run 2)

## What happened

- `ana` run 1, Ana: passed, reached their goal. wordy {"words":80,"budget":20}; their last acts: scroll · tap “Money” · scroll · scroll · look · tap “Say you were in on it”
- `ana` run 2, Ana: passed, reached their goal. wordy {"words":60,"budget":20}; their last acts: look · scroll · tap “Money” · scroll · scroll · tap “Say you were in on it”

## What the people said

None.

## The product source

- `apps/expenses/index.js:63` holds “Say you were in on it”
- `crates/svrngs/tests/scenes/money.rs:142` holds “Say you were in on it”
- `demo/xenia/findings/findings.json:89` holds “Say you were in on it”
- `apps/expenses/index.js:71` holds “What it was for:”
- `demo/xenia/findings/first-set/split-leaves-out-ana.md:32` holds “What it was for:”
- `apps/expenses/index.js:71` holds “How much:”
- `demo/xenia/findings/first-set/split-leaves-out-ana.md:32` holds “How much:”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
