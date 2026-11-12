# A tap changed nothing: Home, “Seed swap”

- status: open
- found by: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- last seen: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- kind: hard (the instrument's): `inert`; never both

## Where

Home, the element “Seed swap”. What was read there first: Its name: / What it is: / Seed swap (Sam’s, since 7 Nov) (not ticked) / Lend it / Its name: / What it is:

## Who, and how often

- `end`: 1 of 3 accounts (Sam run 1)

## What happened

- `end` run 1, Sam: failed, ran out of patience. inert {"tap":"Seed swap"}; their last acts: tap “Open this folder” · scroll · look · back · tap “Manage this group” · tap “Seed swap”

## What the people said

None.

## The product source

- `crates/svrngs/tests/xenia/neighbourhood-smoke.mjs:71` holds “Seed swap”
- `crates/svrngs/tests/xenia/neighbourhood-smoke.mjs:107` holds “Seed swap”
- `crates/svrngs/tests/xenia/neighbourhood.rs:18` holds “Seed swap”
- `crates/svrngs-node/src/home/page/found.js:21` holds “Its name:”
- `docs/guide/lend.md:18` holds “Its name:”
- `docs/guide/models.md:17` holds “Its name:”
- `crates/svrngs-node/src/home/page/found.js:22` holds “What it is:”
- `docs/guide/lend.md:18` holds “What it is:”
- `docs/guide/models.md:17` holds “What it is:”

## To reproduce

```sh
cd /Users/alexsbryan/dev/xenia && node /Users/alexsbryan/dev/xenia/src/run.ts run /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture --runs 1 --model deepseek-flash --reasoning-effort max --findings /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture/findings --source /Users/alexsbryan/dev/svrngs --debug
```
