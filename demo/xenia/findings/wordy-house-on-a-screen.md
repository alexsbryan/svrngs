# Too many words before acting: House on a screen

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: hard (the instrument's): `wordy`; never both

## Where

House on a screen. What was read there first: House / (yours, since 26 Oct) / Since this screen opened / Kim / Ana / 2 people · 1 on now · 0 things lent

## Who, and how often

- `together`: 3 of 6 accounts (Kim run 1, Kim run 2, Kim run 3)

## What happened

- `together` run 1, Kim: failed, ran out of patience. wordy {"words":58,"budget":30}; their last acts: tap “House” · tap “Open Money in reads and writes who paid for what” · scroll · tap “Show House on a screen” · scroll · back
- `together` run 2, Kim: failed, stopped before their goal. wordy {"words":60,"budget":30}; their last acts: look · tap “Open Money in reads and writes who paid for what” · scroll · tap “Show House on a screen” · scroll · tell
- `together` run 3, Kim: failed, ran out of patience. wordy {"words":58,"budget":30}; their last acts: look · tap “House” · scroll · tap “Show House on a screen” · scroll · back

## What the people said

- Kim, `together` run 1: “almost left: Yes, when I couldn't find the Money screen after several tries and was scrolling back and forth between the same pages.”
- Kim, `together` run 2: “almost left: Yes. The screens were hard to read and I couldn't find how to add an expense, so I stopped there instead of guessing.”
- Kim, `together` run 3: “almost left: Not really. I was unsure about tapping "Bring the linked ones to House" without knowing what it would do. I tapped it anyway, and the session then ended.”

## The product source

- `crates/svrngs-node/src/home/home.js:287` holds “Since this screen opened”
- `crates/svrngs-node/tests/browser/main.rs:3090` holds “Since this screen opened”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
