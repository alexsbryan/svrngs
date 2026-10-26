# Too much: the group's page

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T22-40-27-374Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `b76397f`, the product `6d50d7d9`
- kind: soft (the person's): what they said when asked "too much"; never both

## Where

the group's page. What was read there first: Money / Bring more to House / What acts as you / You’re in House (Kim’s, since 26 Oct), as Ana / What / Whose

## Who, and how often

- `ana`: 3 of 3 accounts (Ana run 1, Ana run 2, Ana run 3)
- `together`: 2 of 6 accounts (Ana run 1, Ana run 3)

## What happened

- `ana` run 1, Ana: failed, stopped before their goal. their last acts: scroll · tell · tell · tell · tell · tell
- `together` run 1, Ana: failed, stopped before their goal. their last acts: tell · tell · tell · tell · tell · tell
- `ana` run 2, Ana: failed, stopped before their goal. their last acts: scroll · tell · tell · tell · tell · tell
- `ana` run 3, Ana: failed, stopped before their goal. their last acts: tell · tell · tell · tell · tell · tell
- `together` run 3, Ana: failed, stopped before their goal. their last acts: tell · tell · tell · tell · say “Hi Kim! I've just joined the house through Sam's link, so I should be in the group now. Could you check that I show up and that the pizza is split to $18 each? I haven't sent money yet. I'll pay you my half once I see it on my side. How would you like it, cash or an app?” · tell

## What the people said

- Ana, `ana` run 1: “Possibly: the Money item can write who owes whom, not just read. To see my half I'd only need read access, so that's more than needed. I haven't confirmed the setting though.”
- Ana, `together` run 1: “Nothing was too much. I'd like to check that "Money" can read and write the house's payments under my name. I didn't expect that when I joined, so I'd ask the others about it before using it.”
- Ana, `ana` run 2: “The Money item gives access to read and write who owes whom, which seemed broad just for seeing my pizza share. I didn't change anything.”
- Ana, `ana` run 3: “Possibly: Money being able to read and write everything about who owes whom seems broader than just showing me my half of the pizza. I didn't change anything.”
- Ana, `together` run 3: “Maybe one thing: Money can write as me, not only read. I'd want to check with the others before using it.”

## The product source

- `demo/xenia/findings/findings.json:8` holds “Bring more to House”
- `demo/xenia/findings/findings.json:22` holds “Bring more to House”
- `demo/xenia/findings/findings.json:42` holds “Bring more to House”
- `RELEASES.md:40` holds “What acts as you”
- `apps/jellyfin/ROOM.md:17` holds “What acts as you”
- `crates/svrngs/tests/join_page.rs:203` holds “What acts as you”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
