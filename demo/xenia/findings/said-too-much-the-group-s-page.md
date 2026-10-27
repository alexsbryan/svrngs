# Too much: the group's page

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T03-11-14-983Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `2a10d4d`, the product `ccd48cb4`
- kind: soft (the person's): what they said when asked "too much"; never both

## Where

the group's page. What was read there first: Money / Bring more to House / What acts as you / You’re in House (Kim’s, since 27 Oct), as Ana / What / Whose

## Who, and how often

- `together`: 3 of 6 accounts (Ana run 1, Ana run 2, Ana run 3)

## What happened

- `together` run 1, Ana: failed, stopped before their goal. their last acts: scroll · tell · tell · tell · tell · tell
- `together` run 2, Ana: failed, stopped before their goal. their last acts: scroll · tell · tell · tell · tell · tell
- `together` run 3, Ana: failed, stopped before their goal. their last acts: scroll · tell · tell · tell · tell · tell

## What the people said

- Ana, `together` run 1: “Yes, one thing. Money can read and write who owes whom on my behalf, and I never agreed to that. I'd like Kim or Sam to explain it first.”
- Ana, `together` run 2: “The page is small and dense. I couldn't see all of it at once, and the "What acts as you" part was unclear to me.”
- Ana, `together` run 3: “Nothing was too much. The Money section can read and write what people owe each other, so I'd check with the house before using it.”

## The product source

- `demo/xenia/findings/findings.json:8` holds “Bring more to House”
- `demo/xenia/findings/findings.json:39` holds “Bring more to House”
- `demo/xenia/findings/findings.json:40` holds “Bring more to House”
- `RELEASES.md:40` holds “What acts as you”
- `apps/jellyfin/ROOM.md:17` holds “What acts as you”
- `crates/svrngs/tests/join_page.rs:203` holds “What acts as you”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
