# Too many words before acting: Money

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- kind: hard (the instrument's): `wordy`; never both

## Where

Money. What was read there first: Say you were in on it / Money / Who pays whom / What people paid for / Add what you paid / You’re square

## Who, and how often

- `ana`: 3 of 3 accounts (Ana run 1, Ana run 2, Ana run 3)

## What happened

- `ana` run 1, Ana: failed, stopped before their goal. wordy {"words":40,"budget":20}; their last acts: look · scroll · say “Hi! I joined the house link and I'm looking for my half of the pizza now.” · tap “Open Money” · look · tell
- `ana` run 2, Ana: passed, reached their goal. wordy {"words":40,"budget":20}; their last acts: look · scroll · say “Hi! Thanks Sam & Kim. I've joined the house via the link and I'm looking for my half of the pizza now.” · tap “Open Money” · look · tap “Say you were in on it”
- `ana` run 3, Ana: failed, stopped before their goal. wordy {"words":40,"budget":20}; their last acts: look · scroll · say “Hi! I joined the house page. Looking for my half of Friday's pizza now.” · tap “Money” · look · tell

## What the people said

- Ana, `ana` run 1: “too much: Not really. The join page's "What acts as you" and "Bring more to House" sections were more than I needed, so I skipped them.”
- Ana, `ana` run 1: “almost left: No, I didn't almost leave. The tiny text and the "You're square" message made me hesitate, but I kept going.”
- Ana, `ana` run 2: “too much: The Money app had access to read and write the house's records of who paid for what. I only used it to add myself to the pizza. I didn't check what else that access covers, so that might be more than was needed.”
- Ana, `ana` run 2: “almost left: No, I didn't want to leave at any point.”

## The product source

- `apps/expenses/index.js:63` holds “Say you were in on it”
- `crates/svrngs/tests/scenes/money.rs:142` holds “Say you were in on it”
- `demo/xenia/findings/first-set/split-leaves-out-ana.md:31` holds “Say you were in on it”
- `apps/expenses/expenses.js:303` holds “Who pays whom”
- `apps/expenses/index.js:86` holds “Who pays whom”
- `demo/xenia/findings/first-set/wordy-money.md:9` holds “Who pays whom”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
