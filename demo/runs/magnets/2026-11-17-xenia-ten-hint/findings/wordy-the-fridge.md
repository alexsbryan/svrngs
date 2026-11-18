# Too many words before acting: the fridge

- status: open
- found by: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-18T06-56-03-365Z`, 1 run(s) by claude-sonnet-5-5, as claude -p, xenia `df729c3`, the product `bd06f75c`
- last seen: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-18T06-56-03-365Z`, 1 run(s) by claude-sonnet-5-5, as claude -p, xenia `df729c3`, the product `bd06f75c`
- kind: hard (the instrument's): `wordy`; never both

## Where

the fridge. What was read there first: goose / invoice / moon / group chat / landlord / lasagna

## Who, and how often

- `first-move`: 1 of 10 accounts (Player run 1)

## What happened

- `first-move` run 1, Player: passed, reached their goal. wordy {"words":12,"budget":10}; their last acts: look · scroll · tap “entire”

## What the people said

None.

## The product source

- `apps/magnets/pack.json:5` holds “invoice”
- `apps/magnets/tests/page.test.mjs:72` holds “invoice”
- `apps/magnets/tests/page.test.mjs:76` holds “invoice”

## To reproduce

```sh
cd /Users/alexsbryan/dev/xenia && node /Users/alexsbryan/dev/xenia/src/run.ts run /private/tmp/claude-502/-Users-alexsbryan-dev-svrngs/d16ba5c5-78d9-48fe-a2f4-42f1ed4dec3e/scratchpad/first-move-only --runs 1 --model claude-sonnet-5-5 --people /Users/alexsbryan/dev/svrngs/apps/magnets/tests/xenia/people --findings /Users/alexsbryan/dev/svrngs/demo/runs/magnets/2026-11-17-xenia-ten-hint/findings --source /Users/alexsbryan/dev/svrngs
```
