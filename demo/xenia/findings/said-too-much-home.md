# Too much: Home

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- kind: soft (the person's): what they said when asked "too much"; never both

## Where

Home. What was read there first: What acts as you / Open it / Your groups / Knows who you are / Open a link from your group / This page is yours, and only this computer opens it: nobody in your groups sees it. Here you see your groups and

## Who, and how often

- `newcomer`: 3 of 3 accounts (June run 1, June run 2, June run 3)
- `together`: 1 of 6 accounts (Kim run 3)

## What happened

- `newcomer` run 1, June: passed, reached their goal. their last acts: tap “Join House as June” · tell · tell · tell · tell · tell
- `newcomer` run 2, June: failed, stopped before their goal. their last acts: look · tell · tell · tell · tell · tell
- `newcomer` run 3, June: passed, reached their goal. their last acts: tap “Join House as June” · tell · tell · tell · tell · tell
- `together` run 3, Kim: failed, ran out of patience. their last acts: tap “Bring the ticked ones to House” · tell · tell · tell · tell · tell

## What the people said

- June, `newcomer` run 1: “The page has a lot of small writing, and the "Look for programs running on this computer" and "Copy its address for a program" options made no sense to me. I left them alone.”
- June, `newcomer` run 2: “Nothing was too much. The page was a bit confusing, though. It didn't say clearly what to do when a link doesn't work.”
- June, `newcomer` run 3: “The page was a bit confusing at first. Pasting the link into the top box didn't join me, and it took me a moment to find the separate "Join a group from a link" form further down. The page also has two boxes labelled "Your name in it", which was a bit much.”
- Kim, `together` run 3: “Nothing was too much. I should have stopped before pressing "Bring the ticked ones to House", though, since I wasn't sure what it would do.”

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
