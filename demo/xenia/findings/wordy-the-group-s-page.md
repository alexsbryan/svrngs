# Too many words before acting: the group's page

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: hard (the instrument's): `wordy`; never both

## Where

the group's page. What was read there first: Open Money. It reads and writes who paid for what, and who owes whom. / Bring more to House / What acts

## Who, and how often

- `ana`: 3 of 3 accounts (Ana run 1, Ana run 2, Ana run 3)
- `together`: 3 of 6 accounts (Ana run 1, Ana run 2, Ana run 3)

## What happened

- `ana` run 1, Ana: passed, reached their goal. wordy {"words":60,"budget":20}; their last acts: tap “http://localhost:37249/#AoJ2dnIogmoaxzRqdOQ4hF5ADN_o5BtiV8hDv_wwA9NzZnq8gepfqlqz2p18ydNBkWh0dHA6Ly8xMjcuMC4wLjE6MzQ3MjEv” · type “Ana” into “What’s your name?” · tap “Join” · look · scroll · tap “Money”
- `together` run 1, Ana: failed, stopped before their goal. wordy {"words":60,"budget":20}; their last acts: tap “http://localhost:36313/#AsY0UjeWXj2UVbjmRbPNrsKAriRaN4wbEUXPOOZUDT1sf52vJ4hZMoYvOQi88O3Scmh0dHA6Ly8xMjcuMC4wLjE6NDA1Nzkv” · type “Ana” into “What's your name?” · tap “Join” · look · scroll · tell
- `ana` run 2, Ana: failed, stopped before their goal. wordy {"words":60,"budget":20}; their last acts: tap “http://localhost:44167/#ApwnsxcGSTCkVMrFHCIRPdeT4VmwNWIw79e5dkoqG3eM1hMOrgo2nh-sLSFyXgLedWh0dHA6Ly8xMjcuMC4wLjE6Mzg0ODUv” · type “Ana” into “What’s your name?” · tap “Join” · look · scroll · tap “Open Money”
- `together` run 2, Ana: failed, stopped before their goal. wordy {"words":60,"budget":20}; their last acts: tap “http://localhost:35107/#AsEbtGQ9Y_KF7HhMQqKlP1bqBD8AHmc9im-SHuAtRR0ebtD14iZw9H_8WebrEhCw0mh0dHA6Ly8xMjcuMC4wLjE6MzU5NDcv” · type “Ana” into “What’s your name?” · tap “Join” · look · scroll · tell
- `ana` run 3, Ana: failed, stopped before their goal. wordy {"words":60,"budget":20}; their last acts: tap “http://localhost:45665/#ArAdF3q53mAmc64bxjFG_5ZXZgC89I64s3WhtIEN7QBJgbz6qOZZ16ZamYIEqpUshmh0dHA6Ly8xMjcuMC4wLjE6NDA1MDkv” · type “Ana” into “What’s your name?” · tap “Join” · look · scroll · tell
- `together` run 3, Ana: failed, stopped before their goal. wordy {"words":60,"budget":20}; their last acts: tap “http://localhost:41655/#ApdPvRuFuNXAlWZREk9gSnzYkztJvWIo4Jpw0Ube9-gKN70rGqtxsKRm3RVWrlltW2h0dHA6Ly8xMjcuMC4wLjE6MzgzNTEv” · type “Ana” into “What’s your name?” · tap “Join” · look · scroll · tell

## What the people said

- Ana, `ana` run 1: “too much: Joining seemed to give something access to read and write the house's money, and it said "Bring more to House" with a list I couldn't read in full. That felt like more than I needed to see my $18 share.”
- Ana, `ana` run 1: “almost left: Not really. I hesitated at the "Say you paid Kim $18.00" button, since I hadn't actually paid. I didn't want to leave, but I stopped there.”
- Ana, `ana` run 2: “too much: Just wrapping up: answering the last questions about how it went.”
- Ana, `together` run 2: “too much: Yes. Money being able to write who owes whom, acting as me, is a lot of access to get just from joining a chat link. I'd want to check with the house before I opened it.”
- Ana, `together` run 2: “almost left: Not really. I was briefly stuck because the page was tiny and cut off, and scrolling showed nothing new. I didn't want to leave, though. I wanted to ask you what to do.”
- Ana, `ana` run 3: “too much: Yes. The page gives Money, which can write as well as read, access to what acts as me. That's more than I need just to see my half of a pizza. I stopped before approving it.”
- Ana, `ana` run 3: “almost left: Yes, at the access page. It wasn't clear why seeing my share needed write access, and the page was small and hard to read fully. I stopped there to ask Kim or Sam.”
- Ana, `together` run 3: “too much: Yes. Money being able to read and write who owes whom seems like a lot for a first join, and the page didn't say who set it up or what exactly it can do.”
- Ana, `together` run 3: “almost left: Not really. I just stopped when the page showed Money access I hadn't expected, and I wanted to check with the others first.”

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
