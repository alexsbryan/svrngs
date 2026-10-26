# A fault the scene declared showed: Home, “the scene's fault 4”

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- kind: hard (the instrument's): `declared`; never both

## Where

Home, the element “the scene's fault 4”. What was read there first: Open Money. It reads and writes who paid for what, and who owes whom. / Copy its address for a program / Open it to this WiFi / Your groups / House / (yours, since

## Who, and how often

- `kim`: 2 of 3 accounts (Kim run 2, Kim run 3)

## What happened

- `kim` run 2, Kim: failed, ran out of patience. declared {"check":3}; their last acts: look · tap “House” · scroll · tap “Show House on a screen”
- `kim` run 2, Kim: failed, ran out of patience. declared {"check":3}; their last acts: look · tap “House” · scroll · tap “Show House on a screen” · look
- `kim` run 3, Kim: failed, ran out of patience. declared {"check":3}; their last acts: look · tap “House” · scroll · tap “Show House on a screen”

## What the people said

- Kim, `kim` run 2: “almost left: Yes, when scrolling kept returning the same screens and tapping "Money" did nothing. After that I tried the link on the first page, and the session ended.”
- Kim, `kim` run 2: “almost left: Yes, when scrolling kept returning the same screens and tapping "Money" did nothing. After that I tried the link on the first page, and the session ended.”
- Kim, `kim` run 3: “almost left: Yes. I got stuck looking for how to add an expense, and then the session ended after I tapped Open Money.”

## The product source

- `crates/svrngs-node/tests/browser/carried.rs:194` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `crates/svrngs-node/tests/browser/opens.rs:147` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
- `demo/xenia/findings/first-set/after-join-reads-as-permissions.md:9` holds “Open Money. It reads and writes who paid for what, and who owes whom.”
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
