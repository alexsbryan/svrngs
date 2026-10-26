# Too many words before acting: Money

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: hard (the instrument's): `wordy`; never both

## Where

Money. What was read there first: Say you were in on it / Money / Who pays whom / What people paid for / Add what you paid / You’re square

## Who, and how often

- `ana`: 1 of 3 accounts (Ana run 2)

## What happened

- `ana` run 2, Ana: failed, stopped before their goal. wordy {"words":40,"budget":20}; their last acts: tap “Join” · look · scroll · tap “Open Money” · look · tell

## What the people said

- Ana, `ana` run 2: “too much: Just wrapping up: answering the last questions about how it went.”

## The product source

- `apps/expenses/index.js:63` holds “Say you were in on it”
- `crates/svrngs/tests/scenes/money.rs:142` holds “Say you were in on it”
- `demo/xenia/findings/findings.json:44` holds “Say you were in on it”
- `apps/expenses/expenses.js:303` holds “Who pays whom”
- `apps/expenses/index.js:86` holds “Who pays whom”
- `demo/xenia/findings/findings.json:44` holds “Who pays whom”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
