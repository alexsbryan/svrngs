# Too much: Home

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: soft (the person's): what they said when asked "too much"; never both

## Where

Home. What was read there first: Open Money. It reads and writes who paid for what, and who owes whom. / Copy its address for a program / Open it to this WiFi / Your groups / House / (yours, since

## Who, and how often

- `kim`: 1 of 3 accounts (Kim run 1)
- `newcomer`: 2 of 3 accounts (June run 1, June run 3)

## What happened

- `kim` run 1, Kim: failed, ran out of patience. their last acts: tap “Open this folder” · tell · tell · tell · tell · tell
- `newcomer` run 1, June: passed, reached their goal. their last acts: tap “Join House as June” · tell · tell · tell · tell · tell
- `newcomer` run 3, June: passed, reached their goal. their last acts: tap “Join House as June” · tell · tell · tell · tell · tell

## What the people said

- Kim, `kim` run 1: “Yes, the WiFi tap. Opening Money to the WiFi wasn't part of what Sam asked, and I did it without knowing what it would do. I shouldn't have tapped it.”
- June, `newcomer` run 1: “The "Copy its address for a program" button and the idea of programs reading and writing things were confusing. I'd want to ask Kim before doing anything with them.”
- June, `newcomer` run 3: “The page was a bit much. It had lots of unfamiliar wording, like "lend", "programs running here" and "copy address for a program". I didn't know what those meant, so I left them alone.”

## The product source

- `crates/svrngs-node/tests/browser/carried.rs:194` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `crates/svrngs-node/tests/browser/opens.rs:147` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `demo/xenia/findings/declared-home-the-scene-s-fault-4.md:10` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `crates/svrngs-node/src/home/home.js:168` holds “Copy its address for a program”
- `crates/svrngs-node/src/home/index.html:18` holds “Copy its address for a program”
- `crates/svrngs-node/tests/browser/main.rs:2736` holds “Copy its address for a program”
- `ARCHITECTURE.md:538` holds “Open it to this WiFi”
- `apps/jellyfin/ROOM.md:10` holds “Open it to this WiFi”
- `crates/svrngs-node/src/home/home.js:76` holds “Open it to this WiFi”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
