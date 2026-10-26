# Too many words before acting: Money

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T22-40-27-374Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `b76397f`, the product `6d50d7d9`
- kind: hard (the instrument's): `wordy`; never both

## Where

Money. What was read there first: Add it, paid by you / on / Money / Who pays whom / What people paid for / Add what you paid

## Who, and how often

- `kim`: 2 of 3 accounts (Kim run 2, Kim run 3)
- `together`: 2 of 6 accounts (Kim run 2, Kim run 3)

## What happened

- `kim` run 2, Kim: passed, reached their goal. wordy {"words":44,"budget":30}; their last acts: scroll · tap “Money” · type “Pizza (Friday)” into “What it was for” · type “36” into “How much” · scroll · tap “Ana”
- `together` run 2, Kim: failed, stopped before their goal. wordy {"words":44,"budget":30}; their last acts: back · tap “Money” · type “Friday's pizza” into “What it was for” · type “36” into “How much” · look · tap “Ana”
- `together` run 2, Kim: failed, stopped before their goal. wordy {"words":90,"budget":30}; their last acts: look · tap “Ana” · tap “Add it, paid by you” · look · scroll · tap “You paid $36.00 for Friday's pizza, split between Kim.”
- `kim` run 3, Kim: passed, reached their goal. wordy {"words":44,"budget":30}; their last acts: scroll · tap “Money” · type “Friday's pizza” into “What it was for” · type “36” into “How much” · scroll · tap “Ana”
- `together` run 3, Kim: failed, stopped before their goal. wordy {"words":39,"budget":30}; their last acts: look · tap “Money” · type “Friday's pizza (Kim and Ana)” into “What it was for” · type “36” into “How much” · scroll · tap “Add it, paid by you”
- `together` run 3, Kim: failed, stopped before their goal. wordy {"words":60,"budget":30}; their last acts: type “36” into “How much” · scroll · tap “Add it, paid by you” · look · say “Hi Ana, Sam asked me to log Friday's pizza. I paid $36 for you and me, and I've added it to the house money. It only shows as split with me, though, because you aren't in the house group there yet. Once you're in, your half is $18. Can you send it to me, or let me know once you've joined?” · tell

## What the people said

- Kim, `kim` run 2: “almost left: Yes, when the screen got stuck on "Tell a friend how it went" and I couldn't get back to fix the entry.”
- Kim, `together` run 2: “almost left: Yes, when the Ana checkbox stopped responding and I couldn't undo the first entry. I stopped there instead of adding a duplicate.”
- Kim, `together` run 2: “almost left: Yes, when the Ana checkbox stopped responding and I couldn't undo the first entry. I stopped there instead of adding a duplicate.”

## The product source

- `apps/expenses/index.js:71` holds “Add it, paid by you”
- `crates/svrngs/tests/xenia/main.rs:146` holds “Add it, paid by you”
- `demo/xenia/findings/first-set/scene-kim-voice-and-opening.md:17` holds “Add it, paid by you”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
