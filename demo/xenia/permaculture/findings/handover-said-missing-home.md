# Missing: Home

- status: open
- found by: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- last seen: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- kind: soft (the person's): what they said when asked "missing"; never both

## Where

Home. What was read there first: give / Sam’s notebook / Open this folder / People / Add an app / Manage this group

## Who, and how often

- `handover`: 1 of 3 accounts (Ren run 1)

## What happened

- `handover` run 1, Ren: failed, ran out of patience. their last acts: tap “Open this folder” · look · tell · tell · tell · tell

## What the people said

- Ren, `handover` run 1: “I wanted to get back to the notebook to check my note again after Sam and Jo's handover talk, and to say in the street chat that my note was still readable and I hadn't lost it. From the group settings I couldn't find my way back to the notebook page — tapping around only showed the folder and group details. The pages didn't let me return to the notebook or the chat from there.”

## The product source

None found by its exact text.

## To reproduce

```sh
cd /Users/alexsbryan/dev/xenia && node /Users/alexsbryan/dev/xenia/src/run.ts run /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture --runs 1 --model deepseek-flash --reasoning-effort max --findings /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture/findings --source /Users/alexsbryan/dev/svrngs --debug
```
