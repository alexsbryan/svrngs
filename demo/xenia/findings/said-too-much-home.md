# Too much: Home

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T22-40-27-374Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `b76397f`, the product `6d50d7d9`
- kind: soft (the person's): what they said when asked "too much"; never both

## Where

Home. What was read there first: What acts as you / Open it / Your groups / Open a link from your group / Make a group / This page is yours, and only this computer opens it: nobody in your groups sees it. Here you see your groups and who

## Who, and how often

- `newcomer`: 2 of 3 accounts (June run 2, June run 3)

## What happened

- `newcomer` run 2, June: passed, reached their goal. their last acts: tap “Join House as June” · tell · tell · tell · tell · tell
- `newcomer` run 3, June: failed, stopped before their goal. their last acts: say “Hi Kim, thank you. I pasted your link into the page and pressed Open it, but it says "That is not a link to something lent in your group." Could you check it and send it again, or show me what to do? I'd rather not guess.” · tell · tell · tell · tell · tell

## What the people said

- June, `newcomer` run 2: “The page had a lot of unfamiliar wording, like "What you lend", "Look for programs running on this computer" and "Copy its address for a program". I left all of that alone.”
- June, `newcomer` run 3: “The page is wordy and I don't fully understand it (lending, groups, programs). That made me nervous, so I stopped rather than try anything else.”

## The product source

- `RELEASES.md:40` holds “What acts as you”
- `apps/jellyfin/ROOM.md:17` holds “What acts as you”
- `crates/svrngs/tests/join_page.rs:203` holds “What acts as you”
- `ARCHITECTURE.md:538` holds “Open it”
- `README.md:34` holds “Open it”
- `RELEASES.md:150` holds “Open it”
- `crates/svrngs-node/src/home/index.html:10` holds “Your groups”
- `crates/svrngs-node/src/home/index.html:16` holds “Your groups”
- `demo/xenia/findings/declared-home-the-scene-s-fault-4.md:10` holds “Your groups”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
