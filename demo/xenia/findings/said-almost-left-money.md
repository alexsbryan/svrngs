# Almost left: Money

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- kind: soft (the person's): what they said when asked "almost left"; never both

## Where

Money. What was read there first: Say you were in on it / Money / Who pays whom / What people paid for / Add what you paid / You’re square

## Who, and how often

- `ana`: 2 of 3 accounts (Ana run 1, Ana run 2)
- `kim`: 2 of 3 accounts (Kim run 1, Kim run 3)
- `together`: 2 of 6 accounts (Kim run 1, Kim run 2)

## What happened

- `ana` run 1, Ana: failed, stopped before their goal. their last acts: look · tell · tell · tell · tell · tell
- `kim` run 1, Kim: failed, ran out of patience. their last acts: tap “Open Money” · tell · tell · tell · tell · tell
- `together` run 1, Kim: failed, ran out of patience. their last acts: tap “Open Money” · tell · tell · tell · tell · tell
- `ana` run 2, Ana: passed, reached their goal. their last acts: tell · tell · say “Kim, I see my half is $18. How would you like me to pay you: cash or a transfer?” · tell · tell · tell
- `together` run 2, Kim: failed, ran out of patience. their last acts: tap “Open Money” · tell · tell · tell · tell · tell
- `kim` run 3, Kim: failed, ran out of patience. their last acts: tap “Open Money. It reads and writes who paid for what, and who owes whom.” · tell · tell · tell · tell · tell

## What the people said

- Ana, `ana` run 1: “No, I didn't almost leave. The tiny text and the "You're square" message made me hesitate, but I kept going.”
- Kim, `kim` run 1: “Yes. I nearly left when the Money link wouldn't open and the page was a long list of buttons with no clear way to the expense form.”
- Kim, `together` run 1: “Yes, when "Open Money" did nothing visible. I couldn't tell whether it had worked or whether I was missing a step, such as bringing Money to House first. Then the session ended.”
- Ana, `ana` run 2: “No, I didn't want to leave at any point.”
- Kim, `together` run 2: “Yes. The page was long and confusing, with lots of sharing and permission options, and I couldn't tell which button opened Money. That's where it stopped.”
- Kim, `kim` run 3: “Yes. I got stuck looking for how to add an expense, and then the session ended after I tapped Open Money.”

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
