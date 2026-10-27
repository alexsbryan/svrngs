# Too much: Home

- status: open
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T03-11-14-983Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `2a10d4d`, the product `ccd48cb4`
- kind: soft (the person's): what they said when asked "too much"; never both

## Where

Home. What was read there first: What acts as you / The link: / Open it / Your groups / Open a link from your group / Make a group

## Who, and how often

- `newcomer`: 2 of 3 accounts (June run 2, June run 3)

## What happened

- `newcomer` run 2, June: failed, ran out of patience. their last acts: type “June” into “Your name in it” · tell · tell · tell · tell · tell
- `newcomer` run 3, June: failed, ran out of patience. their last acts: look · tell · tell · tell · tell · tell

## What the people said

- June, `newcomer` run 2: “The page has a lot of wording about lending programs and copying addresses for programs. I didn't understand it, and it made me nervous.”
- June, `newcomer` run 3: “The page was confusing. There were two link boxes that look alike, and I picked the wrong one first. I also don't really know what "lending" my computer means. I'd want Kim to check with me before I go further.”

## The product source

- `RELEASES.md:40` holds “What acts as you”
- `apps/jellyfin/ROOM.md:17` holds “What acts as you”
- `crates/svrngs/tests/join_page.rs:203` holds “What acts as you”
- `crates/svrngs-node/src/home/home.js:388` holds “The link:”
- `demo/events.md:342` holds “The link:”
- `demo/xenia/findings/first-set/two-link-boxes.md:9` holds “The link:”
- `ARCHITECTURE.md:538` holds “Open it”
- `README.md:9` holds “Open it”
- `README.md:48` holds “Open it”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
