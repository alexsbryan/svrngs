# A page never settled: Money

- status: open, not seen since the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: hard (the instrument's): `unsettled`; never both

## Where

Money. What was read there first: nothing

## Who, and how often

- `together`: 1 of 6 accounts (Kim run 1)
- `kim`: 2 of 3 accounts (Kim run 2, Kim run 3)

## What happened

- `together` run 1, Kim: failed, ran out of patience. unsettled {}; their last acts: tap “Bring the linked ones to House” · scroll · tap “Bring the listed ones to House” · scroll · scroll · tap “Open Money”
- `kim` run 2, Kim: failed, ran out of patience. unsettled {}; their last acts: scroll · scroll · look · scroll · scroll · tap “Open Money”
- `kim` run 3, Kim: failed, ran out of patience. unsettled {}; their last acts: tap “Open Money in House” · scroll · scroll · scroll · scroll · tap “Open Money”

## What the people said

- Kim, `together` run 1: “almost left: Yes, when I couldn't find the Money screen after several tries and was scrolling back and forth between the same pages.”
- Kim, `kim` run 2: “almost left: Not really. The session ended right after I tapped "Open Money", so I didn't get to decide anything.”
- Kim, `kim` run 3: “almost left: Yes. I almost left when I couldn't find how to open Money for the House group. The page was long, and the "Open Money" link wasn't easy to tell apart from the other things on it.”

## The product source

None found by its exact text.

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
