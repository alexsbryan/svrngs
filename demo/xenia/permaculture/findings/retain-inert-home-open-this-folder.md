# A tap changed nothing: Home, “Open this folder”

- status: open
- found by: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- last seen: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- kind: hard (the instrument's): `inert`; never both

## Where

Home, the element “Open this folder”. What was read there first: give / Sam’s notebook / Open this folder / People / Add an app / Manage this group

## Who, and how often

- `retain`: 2 of 3 accounts (Sam run 1, Jo run 1)

## What happened

- `retain` run 1, Sam: could-not-judge, the scene ran out of time while they were acting. inert {"tap":"Open this folder"}; their last acts: look · scroll · scroll · tap “What neighbours know” · back · tap “Open this folder”
- `retain` run 1, Jo: could-not-judge, the scene ran out of time while they were acting. inert {"tap":"Open this folder"}; their last acts: wait · scroll · back · tap “Seed swap” · tap “Seed swap (Sam's, since 7 Nov)” · tap “Open this folder”

## What the people said

None.

## The product source

- `crates/svrngs/tests/pages/members.rs:385` holds “Open this folder”
- `crates/svrngs/tests/pages/members.rs:419` holds “Open this folder”
- `crates/svrngs/tests/pages/members.rs:426` holds “Open this folder”

## To reproduce

```sh
cd /Users/alexsbryan/dev/xenia && node /Users/alexsbryan/dev/xenia/src/run.ts run /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture --runs 1 --model deepseek-flash --reasoning-effort max --findings /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture/findings --source /Users/alexsbryan/dev/svrngs --debug
```
