# A tap changed nothing: Home, “Open this folder”

- status: open
- found by: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- last seen: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- kind: hard (the instrument's): `inert`; never both

## Where

Home, the element “Open this folder”. What was read there first: give / Sam’s notebook / Open this folder / People / Add an app / Manage this group

## Who, and how often

- `handover`: 3 of 3 accounts (Sam run 1, Jo run 1, Ren run 1)

## What happened

- `handover` run 1, Sam: could-not-judge, the scene ran out of time while they were acting. inert {"tap":"Open this folder"}; their last acts: scroll · tap “Move it to…” · tap “Jo’s computer” · wait · back · tap “Open this folder”
- `handover` run 1, Jo: could-not-judge, the scene ran out of time while they were acting. inert {"tap":"Open this folder"}; their last acts: tap “What neighbours know” · tap “Seed swap: Saturday at 11, outside number 14. Bring spare envelopes.” · look · back · tap “Seed swap” · tap “Open this folder”
- `handover` run 1, Jo: could-not-judge, the scene ran out of time while they were acting. inert {"tap":"Open this folder"}; their last acts: tap “Seed swap” · tap “Open this folder” · tap “Sam’s notebook” · tap “Seed swap” · back · tap “Open this folder”
- `handover` run 1, Ren: failed, ran out of patience. inert {"tap":"Open this folder"}; their last acts: tap “People” · tap “Sam’s notebook” · scroll · scroll · back · tap “Open this folder”

## What the people said

- Ren, `handover` run 1: “missing: I wanted to get back to the notebook to check my note again after Sam and Jo's handover talk, and to say in the street chat that my note was still readable and I hadn't lost it. From the group settings I couldn't find my way back to the notebook page — tapping around only showed the folder and group details. The pages didn't let me return to the notebook or the chat from there.”

## The product source

- `crates/svrngs/tests/pages/members.rs:385` holds “Open this folder”
- `crates/svrngs/tests/pages/members.rs:419` holds “Open this folder”
- `crates/svrngs/tests/pages/members.rs:426` holds “Open this folder”

## To reproduce

```sh
cd /Users/alexsbryan/dev/xenia && node /Users/alexsbryan/dev/xenia/src/run.ts run /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture --runs 1 --model deepseek-flash --reasoning-effort max --findings /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture/findings --source /Users/alexsbryan/dev/svrngs --debug
```
