# Too many words before acting: Home

- status: open
- found by: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- last seen: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- kind: hard (the instrument's): `wordy`; never both

## Where

Home. What was read there first: give / Sam’s notebook / Open this folder / People / Add an app / Manage this group

## Who, and how often

- `handover`: 1 of 3 accounts (Ren run 1)

## What happened

- `handover` run 1, Ren: failed, ran out of patience. wordy {"words":56,"budget":20}; their last acts: look · scroll · back · scroll · scroll · back

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
