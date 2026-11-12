# A tap changed nothing: Home, “Open this folder”

- status: open
- found by: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- last seen: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- kind: hard (the instrument's): `inert`; never both

## Where

Home, the element “Open this folder”. What was read there first: All your groups / This computer / Seed swap (Sam’s, since 7 Nov) / Join a group / Make a group / Open a link

## Who, and how often

- `benefit`: 1 of 3 accounts (Jo run 1)

## What happened

- `benefit` run 1, Jo: failed, ran out of patience. inert {"tap":"Open this folder"}; their last acts: look · tap “Seed swap (Sam’s, since 7 Nov)” · tap “Open this folder” · scroll · tap “Open this folder”

## What the people said

- Jo, `benefit` run 1: “stuck: I got stuck twice. First, "Open this folder" didn't seem to change the screen — I couldn't tell whether Sam's notebook had actually opened, and the page text was too small to read comfortably. Second, after I tapped "Sam's notebook" and saw the notebook, the session switched to "You've given up", so I couldn't post in the street chat. Somebody would need to confirm the notebook is the right place and let me back into the chat to reply to Mira.”
- Jo, `benefit` run 1: “missing: Yes — I wanted to open Sam's notebook folder properly: "Open this folder" didn't visibly open anything, so I ended up tapping the notebook's name instead. And I wanted to answer Mira in the street chat that the details were clear, but the session had already gone to "You've given up" and wouldn't let me say it.”

## The product source

- `crates/svrngs/tests/pages/members.rs:385` holds “Open this folder”
- `crates/svrngs/tests/pages/members.rs:419` holds “Open this folder”
- `crates/svrngs/tests/pages/members.rs:426` holds “Open this folder”
- `crates/svrngs-node/src/home/page/grouped.js:62` holds “All your groups”
- `crates/svrngs-node/src/home/page/grouped.js:81` holds “All your groups”
- `crates/svrngs-node/src/home/page/grouped.js:106` holds “All your groups”
- `RELEASES.md:35` holds “This computer”
- `apps/canon/index.js:104` holds “This computer”
- `apps/events/server.mjs:189` holds “This computer”

## To reproduce

```sh
cd /Users/alexsbryan/dev/xenia && node /Users/alexsbryan/dev/xenia/src/run.ts run /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture --runs 1 --model deepseek-flash --reasoning-effort max --findings /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture/findings --source /Users/alexsbryan/dev/svrngs --debug
```
