# Missing: Home

- status: open
- found by: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- last seen: the set `/Users/alexsbryan/dev/xenia/runs/2026-11-08T00-24-18-480Z`, 1 run(s) by deepseek-flash, through deepinfra (https://api.deepinfra.com/v1/openai; reasoning effort max), xenia `df729c3`, the product `5c0305ce`
- kind: soft (the person's): what they said when asked "missing"; never both

## Where

Home. What was read there first: All your groups / This computer / Seed swap (Sam’s, since 7 Nov) / Join a group / Make a group / Open a link

## Who, and how often

- `benefit`: 3 of 3 accounts (Sam run 1, Jo run 1, Ren run 1)

## What happened

- `benefit` run 1, Sam: failed, ran out of patience. their last acts: look · back · tell · tell · tell · tell
- `benefit` run 1, Jo: failed, ran out of patience. their last acts: look · say “Thanks Mira — I've seen the notebook: Saturday 15 Nov, outside number 41, bring spare envelopes. All clear from my side.” · tell · tell · tell · tell
- `benefit` run 1, Ren: failed, ran out of patience. their last acts: tap “Sam’s notebook” · look · tell · tell · tell · tell

## What the people said

- Sam, `benefit` run 1: “Yes — I wanted to open the Neighbourhood notebook in Seed swap to check the seed swap's date and place, and then reply to Mira in the street chat to say it's clear. The group page didn't let me get there: my tap on Sam's notebook ended the session instead of opening the notebook.”
- Jo, `benefit` run 1: “Yes — I wanted to open Sam's notebook folder properly: "Open this folder" didn't visibly open anything, so I ended up tapping the notebook's name instead. And I wanted to answer Mira in the street chat that the details were clear, but the session had already gone to "You've given up" and wouldn't let me say it.”
- Ren, `benefit` run 1: “Yes — I wanted to read the Neighbourhood notebook to learn when and where the seed swap is, then say in the street chat if anything was unclear. The pages didn't let me read it: after I tapped "Sam's notebook" it just said I'd given up, so I couldn't see the details.”

## The product source

- `crates/svrngs/tests/pages/members.rs:385` holds “Open this folder”
- `crates/svrngs/tests/pages/members.rs:419` holds “Open this folder”
- `crates/svrngs/tests/pages/members.rs:426` holds “Open this folder”
- `demo/xenia/findings/instrument/money-opened-as-patience-ran-out.md:17` holds “You've given up”
- `demo/xenia/findings/instrument/money-opened-as-patience-ran-out.md:21` holds “You've given up”
- `crates/svrngs/tests/xenia/neighbourhood.rs:266` holds “Sam's notebook”

## To reproduce

```sh
cd /Users/alexsbryan/dev/xenia && node /Users/alexsbryan/dev/xenia/src/run.ts run /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture --runs 1 --model deepseek-flash --reasoning-effort max --findings /Users/alexsbryan/dev/svrngs/demo/xenia/permaculture/findings --source /Users/alexsbryan/dev/svrngs --debug
```
