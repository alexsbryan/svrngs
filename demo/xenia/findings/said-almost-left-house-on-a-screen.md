# Almost left: House on a screen

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: soft (the person's): what they said when asked "almost left"; never both

## Where

House on a screen. What was read there first: House / (yours, since 26 Oct) / Since this screen opened / Kim / Ana / 2 people · 1 on now · 0 things lent

## Who, and how often

- `together`: 1 of 6 accounts (Kim run 2)

## What happened

- `together` run 2, Kim: failed, stopped before their goal. their last acts: tell · tell · tell · tell · tell · tell

## What the people said

- Kim, `together` run 2: “Yes. The screens were hard to read and I couldn't find how to add an expense, so I stopped there instead of guessing.”

## The product source

- `crates/svrngs-node/src/home/home.js:287` holds “Since this screen opened”
- `crates/svrngs-node/tests/browser/main.rs:3090` holds “Since this screen opened”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
