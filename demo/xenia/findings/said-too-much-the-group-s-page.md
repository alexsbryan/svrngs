# Too much: the group's page

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: soft (the person's): what they said when asked "too much"; never both

## Where

the group's page. What was read there first: Open Money. It reads and writes who paid for what, and who owes whom. / Bring more to House / What acts

## Who, and how often

- `together`: 2 of 6 accounts (Ana run 2, Ana run 3)
- `ana`: 1 of 3 accounts (Ana run 3)

## What happened

- `together` run 2, Ana: failed, stopped before their goal. their last acts: scroll · tell · tell · tell · tell · tell
- `ana` run 3, Ana: failed, stopped before their goal. their last acts: scroll · tell · tell · tell · tell · tell
- `together` run 3, Ana: failed, stopped before their goal. their last acts: scroll · tell · tell · tell · tell · tell

## What the people said

- Ana, `together` run 2: “Yes. Money being able to write who owes whom, acting as me, is a lot of access to get just from joining a chat link. I'd want to check with the house before I opened it.”
- Ana, `ana` run 3: “Yes. The page gives Money, which can write as well as read, access to what acts as me. That's more than I need just to see my half of a pizza. I stopped before approving it.”
- Ana, `together` run 3: “Yes. Money being able to read and write who owes whom seems like a lot for a first join, and the page didn't say who set it up or what exactly it can do.”

## The product source

- `crates/svrngs-node/tests/browser/carried.rs:194` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `crates/svrngs-node/tests/browser/opens.rs:147` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `demo/xenia/findings/declared-home-the-scene-s-fault-4.md:10` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `demo/xenia/findings/findings.json:8` holds “Bring more to House”
- `demo/xenia/findings/findings.json:18` holds “Bring more to House”
- `demo/xenia/findings/findings.json:54` holds “Bring more to House”
- `RELEASES.md:32` holds “What acts”
- `apps/jellyfin/ROOM.md:17` holds “What acts”
- `crates/svrngs/tests/join_page.rs:196` holds “What acts”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
