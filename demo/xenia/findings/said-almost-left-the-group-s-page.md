# Almost left: the group's page

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: soft (the person's): what they said when asked "almost left"; never both

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

- Ana, `together` run 2: “Not really. I was briefly stuck because the page was tiny and cut off, and scrolling showed nothing new. I didn't want to leave, though. I wanted to ask you what to do.”
- Ana, `ana` run 3: “Yes, at the access page. It wasn't clear why seeing my share needed write access, and the page was small and hard to read fully. I stopped there to ask Kim or Sam.”
- Ana, `together` run 3: “Not really. I just stopped when the page showed Money access I hadn't expected, and I wanted to check with the others first.”

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
