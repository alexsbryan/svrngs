# Almost left: Money

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: soft (the person's): what they said when asked "almost left"; never both

## Where

Money. What was read there first: Say you were in on it / Money / Who pays whom / What people paid for / Add what you paid / You’re square

## Who, and how often

- `ana`: 1 of 3 accounts (Ana run 1)
- `together`: 1 of 6 accounts (Kim run 1)
- `kim`: 2 of 3 accounts (Kim run 2, Kim run 3)

## What happened

- `ana` run 1, Ana: passed, reached their goal. their last acts: tap “Say you were in on it” · tell · tell · tell · tell · tell
- `together` run 1, Kim: failed, ran out of patience. their last acts: tap “Open Money” · tell · tell · tell · tell · tell
- `kim` run 2, Kim: failed, ran out of patience. their last acts: tap “Open Money” · tell · tell · tell · tell · tell
- `kim` run 3, Kim: failed, ran out of patience. their last acts: tap “Open Money” · tell · tell · tell · tell · tell

## What the people said

- Ana, `ana` run 1: “Not really. I hesitated at the "Say you paid Kim $18.00" button, since I hadn't actually paid. I didn't want to leave, but I stopped there.”
- Kim, `together` run 1: “Yes, when I couldn't find the Money screen after several tries and was scrolling back and forth between the same pages.”
- Kim, `kim` run 2: “Not really. The session ended right after I tapped "Open Money", so I didn't get to decide anything.”
- Kim, `kim` run 3: “Yes. I almost left when I couldn't find how to open Money for the House group. The page was long, and the "Open Money" link wasn't easy to tell apart from the other things on it.”

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
