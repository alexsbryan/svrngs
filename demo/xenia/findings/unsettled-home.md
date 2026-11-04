# A page never settled: Home

- status: open, not seen since the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T21-23-37-479Z`, 3 run(s) on sonnet, xenia `f86739a`, the product `95895434`
- kind: hard (the instrument's): `unsettled`; never both

## Where

Home. What was read there first: Open Money. It reads and writes who paid for what, and who owes whom. / Copy its address for a program / Open it to this WiFi / Your groups / House / (yours, since

## Who, and how often

- `together`: 2 of 6 accounts (Kim run 1, Kim run 3)

## What happened

- `together` run 1, Kim: failed, ran out of patience. unsettled {}; their last acts: scroll · tap “Money” · tap “Bring the linked ones to House” · scroll · tap “Bring the listed ones to House” · scroll
- `together` run 1, Kim: failed, ran out of patience. unsettled {}; their last acts: tap “Money” · tap “Bring the linked ones to House” · scroll · tap “Bring the listed ones to House” · scroll · scroll
- `together` run 3, Kim: failed, ran out of patience. unsettled {}; their last acts: scroll · back · scroll · scroll · tap “Money” · tap “Bring the linked ones to House”

## What the people said

- Kim, `together` run 1: “almost left: Yes, when I couldn't find the Money screen after several tries and was scrolling back and forth between the same pages.”
- Kim, `together` run 1: “almost left: Yes, when I couldn't find the Money screen after several tries and was scrolling back and forth between the same pages.”
- Kim, `together` run 3: “almost left: Not really. I was unsure about tapping "Bring the linked ones to House" without knowing what it would do. I tapped it anyway, and the session then ended.”

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
