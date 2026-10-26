# A page never settled: Home

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- kind: hard (the instrument's): `unsettled`; never both

## Where

Home. What was read there first: Open Money. It reads and writes who paid for what, and who owes whom. / Copy its address for a program / Open it to this WiFi / Your groups / House / (yours, since

## Who, and how often

- `kim`: 2 of 3 accounts (Kim run 2, Kim run 3)
- `newcomer`: 1 of 3 accounts (June run 3)
- `together`: 1 of 6 accounts (Kim run 3)

## What happened

- `kim` run 2, Kim: failed, ran out of patience. unsettled {}; their last acts: scroll · scroll · scroll · scroll · tap “Money” · scroll
- `kim` run 2, Kim: failed, ran out of patience. unsettled {}; their last acts: scroll · scroll · scroll · tap “Money” · scroll · tap “Open Money in House”
- `kim` run 3, Kim: failed, ran out of patience. unsettled {}; their last acts: tap “Show House on a screen” · back · scroll · scroll · tap “Bring things to the table” · scroll
- `newcomer` run 3, June: passed, reached their goal. unsettled {}; their last acts: look · scroll · type “http://localhost:45495/#AjGRFGxGUmTxcsL2CEnWLjkkRNGkh-NGfQZDZTu6FHYRz6NY451KuB2Y0P2vaUygq2h0dHA6Ly8xMjcuMC4wLjE6MzgwNDEv” into “The link someone sent you:” · look · type “June” into “Your name in it: (under Join a group from a link)” · tap “Join House as June”
- `together` run 3, Kim: failed, ran out of patience. unsettled {}; their last acts: tap “Open money in rooms and writes” · scroll · say “Hi both, Kim here. I'm opening the house money page now to add the pizza. Ana, nothing needed from you for the moment.” · scroll · scroll · scroll
- `together` run 3, Kim: failed, ran out of patience. unsettled {}; their last acts: scroll · say “Hi both, Kim here. I'm opening the house money page now to add the pizza. Ana, nothing needed from you for the moment.” · scroll · scroll · scroll · tap “Bring the ticked ones to House”

## What the people said

- Kim, `kim` run 2: “almost left: Yes, when scrolling kept returning the same screens and tapping "Money" did nothing. After that I tried the link on the first page, and the session ended.”
- Kim, `kim` run 2: “almost left: Yes, when scrolling kept returning the same screens and tapping "Money" did nothing. After that I tried the link on the first page, and the session ended.”
- Kim, `kim` run 3: “almost left: Yes. I got stuck looking for how to add an expense, and then the session ended after I tapped Open Money.”
- June, `newcomer` run 3: “too much: The page was a bit confusing at first. Pasting the link into the top box didn't join me, and it took me a moment to find the separate "Join a group from a link" form further down. The page also has two boxes labelled "Your name in it", which was a bit much.”
- June, `newcomer` run 3: “almost left: Yes, a little. After I pasted the link in the top box and pressed "Open it", nothing seemed to happen. It said something about this not being a link to something sent in my groups. I nearly stopped there. I kept going because Kim is someone I trust and the link was from her. Then I found the right form lower down.”
- Kim, `together` run 3: “too much: Nothing was too much. I should have stopped before pressing "Bring the ticked ones to House", though, since I wasn't sure what it would do.”
- Kim, `together` run 3: “almost left: Yes. I nearly stopped when I couldn't tap the Money link on the first page. After that I scrolled around the House page and found nothing for adding an expense, so I gave up.”
- Kim, `together` run 3: “too much: Nothing was too much. I should have stopped before pressing "Bring the ticked ones to House", though, since I wasn't sure what it would do.”
- Kim, `together` run 3: “almost left: Yes. I nearly stopped when I couldn't tap the Money link on the first page. After that I scrolled around the House page and found nothing for adding an expense, so I gave up.”

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
