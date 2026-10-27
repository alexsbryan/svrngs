# Too much: Money

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T06-30-27-895Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `4f12ba4`, the product `bbddad47`
- kind: soft (the person's): what they said when asked "too much"; never both

## Where

Money. What was read there first: Say you were in on pizza / What it was for: / How much: / Money / Who pays whom / What people paid for

## Who, and how often

- `ana`: 1 of 3 accounts (Ana run 2)

## What happened

- `ana` run 2, Ana: failed, stopped before their goal. their last acts: tap “Say you were in on pizza” · tell · tell · tell · tell · tell

## What the people said

- Ana, `ana` run 2: “Mildly, yes. The app said opening Money lets it read and write that as me. That's more access than I expected just to see my share. I'd have liked to be asked first.”

## The product source

- `apps/expenses/index.js:93` holds “What it was for:”
- `demo/xenia/findings/findings.json:48` holds “What it was for:”
- `demo/xenia/findings/findings.json:67` holds “What it was for:”
- `apps/expenses/index.js:93` holds “How much:”
- `demo/xenia/findings/findings.json:48` holds “How much:”
- `demo/xenia/findings/findings.json:69` holds “How much:”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
