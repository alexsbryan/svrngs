# Almost left: Home

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- kind: soft (the person's): what they said when asked "almost left"; never both

## Where

Home. What was read there first: What acts as you / Open it / Your groups / Knows who you are / Open a link from your group / This page is yours, and only this computer opens it: nobody in your groups sees it. Here you see your groups and

## Who, and how often

- `newcomer`: 3 of 3 accounts (June run 1, June run 2, June run 3)
- `kim`: 1 of 3 accounts (Kim run 2)
- `together`: 1 of 6 accounts (Kim run 3)

## What happened

- `newcomer` run 1, June: passed, reached their goal. their last acts: tap “Join House as June” · tell · tell · tell · tell · tell
- `kim` run 2, Kim: failed, ran out of patience. their last acts: tap “Open Money in House” · tell · tell · tell · tell · tell
- `newcomer` run 2, June: failed, stopped before their goal. their last acts: look · tell · tell · tell · tell · tell
- `newcomer` run 3, June: passed, reached their goal. their last acts: tap “Join House as June” · tell · tell · tell · tell · tell
- `together` run 3, Kim: failed, ran out of patience. their last acts: tap “Bring the ticked ones to House” · tell · tell · tell · tell · tell

## What the people said

- June, `newcomer` run 1: “Not really. I did hesitate before typing the link in, because I wasn't sure about it. It came from Kim in the house chat, and Kim said they'd set this page up, so I went ahead. If it had asked me for anything I didn't understand, I would have stopped and asked someone.”
- Kim, `kim` run 2: “Yes, when scrolling kept returning the same screens and tapping "Money" did nothing. After that I tried the link on the first page, and the session ended.”
- June, `newcomer` run 2: “Yes. I left when the page said the link wasn't something sent in my groups. I didn't know why, and I'd rather ask Kim than guess.”
- June, `newcomer` run 3: “Yes, a little. After I pasted the link in the top box and pressed "Open it", nothing seemed to happen. It said something about this not being a link to something sent in my groups. I nearly stopped there. I kept going because Kim is someone I trust and the link was from her. Then I found the right form lower down.”
- Kim, `together` run 3: “Yes. I nearly stopped when I couldn't tap the Money link on the first page. After that I scrolled around the House page and found nothing for adding an expense, so I gave up.”

## The product source

- `RELEASES.md:32` holds “What acts as you”
- `apps/jellyfin/ROOM.md:17` holds “What acts as you”
- `crates/svrngs/tests/join_page.rs:202` holds “What acts as you”
- `ARCHITECTURE.md:538` holds “Open it”
- `README.md:34` holds “Open it”
- `RELEASES.md:142` holds “Open it”
- `crates/svrngs-node/src/home/index.html:8` holds “Your groups”
- `crates/svrngs-node/src/home/index.html:14` holds “Your groups”
- `demo/xenia/findings/first-set/wordy-june-home-page.md:9` holds “Your groups”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
