# Didn't do what they expected: the fridge

- status: open
- found by: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-18T00-56-56-346Z`, 1 run(s) by claude-sonnet-5-5, as claude -p, xenia `df729c3`, the product `ba3e9cff`
- last seen: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-18T00-56-56-346Z`, 1 run(s) by claude-sonnet-5-5, as claude -p, xenia `df729c3`, the product `ba3e9cff`
- kind: soft (the person's): what they said when asked "not as expected"; never both

## Where

the fridge. What was read there first: goose / invoice / moon / group chat / landlord / lasagna

## Who, and how often

- `solo`: 1 of 3 accounts (Player run 1)

## What happened

- `solo` run 1, Player: passed, explored, and said they were done. their last acts: tap “situation” · tell · tell · tell · tell · tell
- `solo` run 1, Player: passed, explored, and said they were done. their last acts: tap “somehow” · tell · tell · tell · tell · tell
- `solo` run 1, Player: passed, explored, and said they were done. their last acts: scroll · tell · tell · tell · tell · tell

## What the people said

- Player, `solo` run 1: “Nothing failed outright. I expected tapped words to stay put in the list, but they moved up into the sentence and the list shifted. I didn't check whether the sentence can be saved or shared.”
- Player, `solo` run 1: “Nothing failed. I only tapped two words. I didn't try to remove one or send the sentence, so I don't know if that works.”
- Player, `solo` run 1: “Nothing went wrong. I only looked and scrolled once. Scrolling showed more than the first look did, but nothing broke.”

## The product source

None found by its exact text.

## To reproduce

```sh
cd /Users/alexsbryan/dev/xenia && node /Users/alexsbryan/dev/xenia/src/run.ts run /Users/alexsbryan/dev/svrngs/apps/magnets/tests/xenia --runs 1 --model claude-sonnet-5-5 --people /Users/alexsbryan/dev/svrngs/apps/magnets/tests/xenia/people --findings /Users/alexsbryan/dev/svrngs/demo/runs/magnets/2026-11-17-xenia/findings --source /Users/alexsbryan/dev/svrngs
```
